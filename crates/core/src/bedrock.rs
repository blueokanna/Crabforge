use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningInstructions, SigningSettings, sign},
    sign::v4,
};
use axum::{
    body::Bytes,
    http::{
        Request,
        header::{HeaderName, HeaderValue},
    },
};
use sha2::{Digest, Sha256};
use std::{env, time::SystemTime};
use url::Url;

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

pub(crate) fn credentials() -> Result<Credentials> {
    let required = |name| {
        env::var(name)
            .ok()
            .filter(|v| !v.is_empty())
            .with_context(|| format!("Bedrock signing requires {name}"))
    };
    Ok(Credentials::new(
        required("AWS_ACCESS_KEY_ID")?,
        required("AWS_SECRET_ACCESS_KEY")?,
        env::var("AWS_SESSION_TOKEN").ok().filter(|v| !v.is_empty()),
        None,
        "environment",
    ))
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
    let instructions = signature(
        request.method().as_str(),
        url,
        SignableBody::Precomputed(hex::encode(digest.finalize())),
        credentials,
        now,
    )?;
    for (name, value) in instructions.headers() {
        let mut value = HeaderValue::from_str(value)?;
        value.set_sensitive(
            name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("x-amz-security-token"),
        );
        request
            .headers_mut()
            .insert(HeaderName::from_bytes(name.as_bytes())?, value);
    }
    Ok(())
}

pub(crate) fn signature(
    method: &str,
    url: &Url,
    body: SignableBody<'_>,
    credentials: Credentials,
    now: SystemTime,
) -> Result<SigningInstructions> {
    let (region, service) = scope(url).context("invalid Bedrock signing host")?;
    let identity = credentials.into();
    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(service)
        .time(now)
        .settings(SigningSettings::default())
        .build()?
        .into();
    let signable = SignableRequest::new(method, url.as_str(), std::iter::empty(), body)?;
    let (instructions, _) = sign(signable, &params)?.into_parts();
    Ok(instructions)
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
        let creds = Credentials::new(
            "AKIDEXAMPLE",
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            Some("FQoGZXIvYXdzEXAMPLETOKEN".into()),
            None,
            "fixture",
        );
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
