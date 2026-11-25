use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4,
};
use http_body_util::BodyExt;
use reqwest::{
    Request, RequestBuilder, Response, Url,
    header::{HeaderName, HeaderValue},
};
use std::{env, time::SystemTime};

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
pub(crate) async fn send(builder: RequestBuilder, key_env: &str) -> Result<Response> {
    let (client, request) = builder.build_split();
    let mut request = request?;
    if key_env.is_empty() && scope(request.url()).is_some() {
        let required = |name| {
            env::var(name)
                .ok()
                .filter(|v| !v.is_empty())
                .with_context(|| format!("Bedrock signing requires {name}"))
        };
        let credentials = Credentials::new(
            required("AWS_ACCESS_KEY_ID")?,
            required("AWS_SECRET_ACCESS_KEY")?,
            env::var("AWS_SESSION_TOKEN").ok().filter(|v| !v.is_empty()),
            None,
            "environment",
        );
        sign_request(&mut request, credentials, SystemTime::now()).await?;
    }
    Ok(client.execute(request).await?)
}

async fn sign_request(
    request: &mut Request,
    credentials: Credentials,
    now: SystemTime,
) -> Result<()> {
    // Rewritten JSON and multipart can be segmented bodies. Materialize only on the
    // signing path so the digest covers exactly the bytes the client transmits.
    if request.body().is_some_and(|body| body.as_bytes().is_none()) {
        let body = request.body_mut().take().expect("checked above");
        *request.body_mut() = Some(body.collect().await?.to_bytes().into());
    }
    let (region, service) = scope(request.url()).context("invalid Bedrock signing host")?;
    let identity = credentials.into();
    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(service)
        .time(now)
        .settings(SigningSettings::default())
        .build()?
        .into();
    let body = request
        .body()
        .and_then(|body| body.as_bytes())
        .unwrap_or_default();
    let signable = SignableRequest::new(
        request.method().as_str(),
        request.url().as_str(),
        std::iter::empty(),
        SignableBody::Bytes(body),
    )?;
    let (instructions, _) = sign(signable, &params)?.into_parts();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        convert::Infallible,
        time::{Duration, UNIX_EPOCH},
    };

    #[tokio::test]
    async fn rewritten_body_matches_botocore_reference() -> Result<()> {
        let url = "https://bedrock-runtime.eu-west-1.amazonaws.com/openai/v1/chat/completions";
        let chunks = [
            r#"{"model":"#,
            r#""openai.gpt-oss-120b-1:0""#,
            r#","stream":true}"#,
        ];
        let body = reqwest::Body::wrap_stream(futures_util::stream::iter(
            chunks.into_iter().map(Ok::<_, Infallible>),
        ));
        let mut req = reqwest::Client::new().post(url).body(body).build()?;
        let creds = Credentials::new(
            "AKIDEXAMPLE",
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            Some("FQoGZXIvYXdzEXAMPLETOKEN".into()),
            None,
            "fixture",
        );
        sign_request(
            &mut req,
            creds,
            UNIX_EPOCH + Duration::from_secs(1795552519),
        )
        .await?;
        assert_eq!(
            req.headers()["authorization"],
            "AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20261124/eu-west-1/bedrock/aws4_request, SignedHeaders=host;x-amz-date;x-amz-security-token, Signature=9e43ae46521e59b3521125dc5ed327d424d6cba8f6af769466dd4ff6da00e824"
        );
        assert_eq!(
            req.headers()["x-amz-security-token"],
            "FQoGZXIvYXdzEXAMPLETOKEN"
        );
        assert_eq!(
            req.body().unwrap().as_bytes().unwrap(),
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
