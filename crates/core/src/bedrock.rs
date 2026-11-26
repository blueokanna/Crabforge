use anyhow::{Context, Result};
use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Request, header::AUTHORIZATION};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use ring::hmac;
use sha2::{Digest, Sha256};
use std::{env, time::SystemTime};
use time::{OffsetDateTime, macros::format_description};
use url::Url;
use zeroize::Zeroizing;

struct Credentials {
    access: String,
    secret: Zeroizing<String>,
    token: Option<Zeroizing<String>>,
}

pub(crate) fn scope(url: &Url) -> Option<(&str, &'static str)> {
    let (prefix, rest) = url.host_str()?.split_once('.')?;
    let (region, domain) = rest.split_once('.')?;
    let service = match (prefix, domain) {
        ("bedrock-runtime" | "bedrock-runtime-fips", "amazonaws.com" | "amazonaws.com.cn") => {
            "bedrock"
        }
        ("bedrock-mantle", "api.aws") => "bedrock-mantle",
        _ => return None,
    };
    (url.scheme() == "https" && !region.is_empty()).then_some((region, service))
}

/// A configured API key has priority. Only AWS Bedrock hosts receive IAM signatures.
pub(crate) fn authenticate(
    request: &mut Request<Vec<Bytes>>,
    url: &Url,
    key_env: &str,
) -> Result<()> {
    if key_env.is_empty() && scope(url).is_some() {
        sign_request(request, url, credentials()?, SystemTime::now())?;
    }
    Ok(())
}

fn credentials() -> Result<Credentials> {
    let required = |name| {
        env::var(name)
            .ok()
            .filter(|v| !v.is_empty())
            .with_context(|| format!("Bedrock signing requires {name}"))
    };
    Ok(Credentials {
        access: required("AWS_ACCESS_KEY_ID")?,
        secret: required("AWS_SECRET_ACCESS_KEY")?.into(),
        token: env::var("AWS_SESSION_TOKEN")
            .ok()
            .filter(|v| !v.is_empty())
            .map(Into::into),
    })
}

fn sign_request(
    request: &mut Request<Vec<Bytes>>,
    url: &Url,
    credentials: Credentials,
    now: SystemTime,
) -> Result<()> {
    let mut digest = Sha256::new();
    for segment in request.body() {
        digest.update(segment);
    }
    let headers = signature(
        request.method().as_str(),
        url,
        &hex::encode(digest.finalize()),
        credentials,
        now,
    )?;
    request.headers_mut().extend(headers);
    Ok(())
}

pub(crate) fn catalog_headers(url: &Url) -> Result<HeaderMap> {
    signature(
        "GET",
        url,
        &hex::encode(Sha256::digest([])),
        credentials()?,
        SystemTime::now(),
    )
}

// AWS SigV4 signs the normalized, double-encoded path and sorts query pairs
// after encoding. Url already removed dot segments when it parsed the request.
fn canonical_target(url: &Url) -> (String, String) {
    const ENCODE: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~');
    let mut path = String::from("/");
    let segments = url.path().split('/').filter(|s| !s.is_empty());
    for segment in segments {
        if path.len() > 1 {
            path.push('/');
        }
        path.extend(utf8_percent_encode(segment, ENCODE));
    }
    if path.len() > 1 && url.path().ends_with('/') {
        path.push('/');
    }
    let mut pairs: Vec<_> = url
        .query_pairs()
        .map(|(k, v)| {
            (
                utf8_percent_encode(&k, ENCODE).to_string(),
                utf8_percent_encode(&v, ENCODE).to_string(),
            )
        })
        .collect();
    pairs.sort_unstable();
    let query = pairs
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    (path, query)
}

fn signature(
    method: &str,
    url: &Url,
    payload_hash: &str,
    credentials: Credentials,
    now: SystemTime,
) -> Result<HeaderMap> {
    let (region, service) = scope(url).context("invalid Bedrock signing host")?;
    let timestamp = OffsetDateTime::from(now).format(format_description!(
        "[year][month][day]T[hour][minute][second]Z"
    ))?;
    let date = &timestamp[..8];
    let scope = format!("{date}/{region}/{service}/aws4_request");
    let host = &url[url::Position::BeforeHost..url::Position::AfterPort];
    let mut fields = format!("host:{host}\nx-amz-date:{timestamp}\n");
    let mut names = String::from("host;x-amz-date");
    let mut headers = HeaderMap::new();
    headers.insert("x-amz-date", HeaderValue::from_str(&timestamp)?);
    if let Some(token) = credentials.token {
        let mut value = HeaderValue::from_str(&token)?;
        value.set_sensitive(true);
        headers.insert("x-amz-security-token", value);
        fields.push_str("x-amz-security-token:");
        fields.push_str(&token.split_ascii_whitespace().collect::<Vec<_>>().join(" "));
        fields.push('\n');
        names.push_str(";x-amz-security-token");
    }
    let (path, query) = canonical_target(url);
    let canonical = format!("{method}\n{path}\n{query}\n{fields}\n{names}\n{payload_hash}");
    let hash = hex::encode(Sha256::digest(canonical.as_bytes()));
    let to_sign = format!("AWS4-HMAC-SHA256\n{timestamp}\n{scope}\n{hash}");
    let secret = Zeroizing::new(format!("AWS4{}", credentials.secret.as_str()));
    let mut key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    for component in [date, region, service, "aws4_request"] {
        key = hmac::Key::new(
            hmac::HMAC_SHA256,
            hmac::sign(&key, component.as_bytes()).as_ref(),
        );
    }
    let mac = hex::encode(hmac::sign(&key, to_sign.as_bytes()).as_ref());
    let mut auth = HeaderValue::from_str(&format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={names}, Signature={mac}",
        credentials.access
    ))?;
    auth.set_sensitive(true);
    headers.insert(AUTHORIZATION, auth);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn rewritten_body_matches_botocore_reference() -> Result<()> {
        let url = "https://bedrock-runtime.eu-west-1.amazonaws.com/openai/v1/chat/completions";
        let chunks = [
            r#"{"model":"#,
            r#""openai.gpt-oss-120b-1:0""#,
            r#","stream":true}"#,
        ];
        let body = chunks.into_iter().map(Bytes::from).collect();
        let mut req = Request::post(url).body(body)?;
        let creds = Credentials {
            access: "AKIDEXAMPLE".into(),
            secret: String::from("wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY").into(),
            token: Some(String::from("FQoGZXIvYXdzEXAMPLETOKEN").into()),
        };
        sign_request(
            &mut req,
            &Url::parse(url)?,
            creds,
            UNIX_EPOCH + Duration::from_secs(1795552519),
        )?;
        assert_eq!(
            req.headers()["authorization"],
            "AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20261124/eu-west-1/bedrock/aws4_request, SignedHeaders=host;x-amz-date;x-amz-security-token, Signature=9e43ae46521e59b3521125dc5ed327d424d6cba8f6af769466dd4ff6da00e824"
        );
        assert_eq!(
            req.headers()["x-amz-security-token"],
            "FQoGZXIvYXdzEXAMPLETOKEN"
        );
        assert_eq!(
            req.body().concat(),
            br#"{"model":"openai.gpt-oss-120b-1:0","stream":true}"#
        );
        Ok(())
    }

    #[test]
    fn catalog_query_matches_aws_sdk_reference() -> Result<()> {
        let url = Url::parse(
            "https://bedrock-mantle.us-east-1.api.aws:8443/v1//models/%2F/?z=last&a=%E4%B8%AD&a=b+c&a=b%2Bc&empty",
        )?;
        let creds = Credentials {
            access: "KEY".into(),
            secret: String::from("SECRET").into(),
            token: None,
        };
        let mut request = Request::get(url.as_str()).body(Vec::new())?;
        sign_request(
            &mut request,
            &url,
            creds,
            UNIX_EPOCH + Duration::from_secs(1795552519),
        )?;
        assert_eq!(
            request.headers()["authorization"].to_str()?,
            "AWS4-HMAC-SHA256 Credential=KEY/20261124/us-east-1/bedrock-mantle/aws4_request, SignedHeaders=host;x-amz-date, Signature=e648f29c6809ee812827cdf63a22fe4fc9dacd2908562086a1e7e42abaae2603"
        );
        Ok(())
    }

    #[test]
    fn signing_scope_follows_aws_host() -> Result<()> {
        for (host, expected) in [
            (
                "bedrock-mantle.eu-west-1.api.aws",
                Some(("eu-west-1", "bedrock-mantle")),
            ),
            (
                "bedrock-runtime-fips.cn-north-1.amazonaws.com.cn",
                Some(("cn-north-1", "bedrock")),
            ),
            ("bedrock-runtime.eu-west-1.amazonaws.com.example.com", None),
        ] {
            assert_eq!(scope(&Url::parse(&format!("https://{host}/"))?), expected);
        }
        Ok(())
    }
}
