use super::*;
use axum::{Form, Json, Router, extract::OriginalUri, http::HeaderMap, routing::post};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::net::TcpListener;

#[tokio::test]
async fn adc_exchanges_once_for_concurrent_requests_and_reloads_changed_credentials() -> Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let root = format!("http://{}", listener.local_addr()?);
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let app = Router::new().route("/token",post(move |Form(form):Form<HashMap<String,String>>| {
        counter.fetch_add(1,Ordering::SeqCst);
        async move {
            assert_eq!(form["grant_type"],"refresh_token");
            assert_eq!(form["client_id"],"test-client");
            Json(json!({"access_token":format!("source-{}", form["refresh_token"]),"expires_in":3600}))
        }
    })).route("/iam/{name}",post(|OriginalUri(uri):OriginalUri, headers:HeaderMap, Json(body):Json<Value>| async move {
        assert_eq!(body["scope"], json!([SCOPE]));
        let token = if uri.path().contains("first@") {
            assert_eq!(headers["authorization"], "Bearer source-b");
            assert_eq!(body["delegates"], json!(["projects/-/serviceAccounts/delegate@example.com"]));
            "first"
        } else {
            assert_eq!(uri.path(), "/iam/final@example.com:generateAccessToken");
            assert_eq!(headers["authorization"], "Bearer first");
            "final"
        };
        Json(json!({"accessToken":token,"expireTime":(OffsetDateTime::now_utc()+time::Duration::hours(1)).format(&Rfc3339).unwrap()}))
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("adc.json");
    let mut source = json!({"type":"authorized_user","client_id":"test-client","client_secret":"s","refresh_token":"a","token_uri":"https://unused.invalid/token"});
    fs::write(&path, source.to_string())?;
    let mut v = Vertex {
        project: "sample-project".into(),
        credentials: path.display().to_string(),
        ..Default::default()
    };
    let cache = TokenCache::default();
    let oauth = format!("{root}/token");
    let iam = format!("{root}/iam/");
    let calls = (0..8).map(|_| cache.exchange(&v, &oauth, &iam));
    for result in futures_util::future::join_all(calls).await {
        assert_eq!(result?, "source-a");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    source["refresh_token"] = json!("b");
    fs::write(&path, json!({"type":"impersonated_service_account","service_account_impersonation_url":"https://unused.invalid/serviceAccounts/first%40example.com:generateAccessToken","delegates":["delegate@example.com"],"source_credentials":source}).to_string())?;
    v.impersonate = "final@example.com".into();
    assert_eq!(cache.exchange(&v, &oauth, &iam).await?, "final");
    assert_eq!(hits.load(Ordering::SeqCst), 2);
    source["universe_domain"] = json!("example.org");
    fs::write(path, source.to_string())?;
    assert!(cache.exchange(&v, &oauth, &iam).await.is_err());
    assert_eq!(hits.load(Ordering::SeqCst), 2);
    server.abort();
    Ok(())
}

#[tokio::test]
async fn service_account_assertion_has_a_verifiable_signature_and_google_audience() -> Result<()> {
    use aws_lc_rs::{
        encoding::{AsDer, Pkcs8V1Der},
        rsa::{KeyPair, KeySize},
    };
    use ring::signature::{KeyPair as _, RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey};
    let key = KeyPair::generate(KeySize::Rsa2048)?;
    let der = AsDer::<Pkcs8V1Der>::as_der(&key)?;
    let body = base64::engine::general_purpose::STANDARD.encode(der.as_ref());
    let credentials = Credentials {
        kind: "service_account".into(),
        client_email: "service@example.com".into(),
        private_key_id: "example".into(),
        private_key: format!("-----BEGIN PRIVATE KEY-----\n{body}\n-----END PRIVATE KEY-----\n"),
        ..Default::default()
    };
    let jwt = credentials.assertion(OAUTH)?;
    let (unsigned, signature) = jwt.rsplit_once('.').unwrap();
    let (header, claims) = unsigned.split_once('.').unwrap();
    fn decode(part: &str) -> Result<Value> {
        Ok(serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part)?)?)
    }
    assert_eq!(
        decode(header)?,
        json!({"alg":"RS256","typ":"JWT","kid":"example"})
    );
    let claims = decode(claims)?;
    assert_eq!(claims["aud"], OAUTH);
    assert_eq!(claims["scope"], SCOPE);
    assert_eq!(claims["iss"], "service@example.com");
    assert_eq!(
        claims["exp"].as_i64().unwrap() - claims["iat"].as_i64().unwrap(),
        3600
    );
    let public = Arc::new(RsaKeyPair::from_pkcs8(der.as_ref()).unwrap());
    UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public.public_key().as_ref())
        .verify(unsigned.as_bytes(), &URL_SAFE_NO_PAD.decode(signature)?)
        .expect("valid RSA signature");
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let oauth = format!("http://{}/token", listener.local_addr()?);
    let expected_audience = oauth.clone();
    let app = Router::new().route(
        "/token",
        post(move |Form(form): Form<HashMap<String, String>>| {
            assert_eq!(
                form["grant_type"],
                "urn:ietf:params:oauth:grant-type:jwt-bearer"
            );
            let (signed, sig) = form["assertion"].rsplit_once('.').unwrap();
            UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public.public_key().as_ref())
                .verify(signed.as_bytes(), &URL_SAFE_NO_PAD.decode(sig).unwrap())
                .unwrap();
            assert_eq!(
                decode(signed.split_once('.').unwrap().1).unwrap()["aud"],
                expected_audience
            );
            async { Json(json!({"access_token":"account-token","expires_in":3600})) }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    assert_eq!(credentials.grant(&oauth).await?.value, "account-token");
    server.abort();
    Ok(())
}

#[tokio::test]
async fn refreshes_near_expiry_before_reusing_the_token() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let oauth = format!("http://{}/token", listener.local_addr()?);
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let app = Router::new().route(
        "/token",
        post(move || {
            let n = count.fetch_add(1, Ordering::SeqCst);
            async move {
                Json(json!({"access_token":n.to_string(),"expires_in":if n==0 {30} else {3600}}))
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let root = tempfile::tempdir()?;
    let file = root.path().join("adc.json");
    fs::write(
        &file,
        br#"{"type":"authorized_user","client_id":"test","refresh_token":"r"}"#,
    )?;
    let vertex = Vertex {
        project: "sample-project".into(),
        credentials: file.display().to_string(),
        ..Default::default()
    };
    let cache = TokenCache::default();
    for expected in ["0", "1", "1"] {
        assert_eq!(cache.exchange(&vertex, &oauth, IAM).await?, expected);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    server.abort();
    Ok(())
}
