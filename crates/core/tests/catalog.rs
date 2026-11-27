use anyhow::Result;
use crabforge_core::{catalog::discover, config::Endpoints};
use crabforge_test_support::server::{Incoming, Json, Router, get};
use nextjson::json;
use tokio::net::TcpListener;

#[tokio::test]
async fn plans_without_live_catalogs_use_their_own_model_lists() -> Result<()> {
    use crabforge_core::catalog::{fixed_models, presets};
    let items: Vec<nextjson::Value> =
        nextjson::from_str(&presets()?).map_err(anyhow::Error::msg)?;
    let endpoints = |id: &str| -> Endpoints {
        let item = items
            .iter()
            .find(|p| p.get("id").and_then(nextjson::Value::as_str) == Some(id))
            .unwrap();
        nextjson::from_value(item["endpoints"].clone()).unwrap()
    };
    let personal = discover(&endpoints("baidu-qianfan-personal"), "UNUSED_KEY").await?;
    assert!(personal.iter().any(|m| m == "qianfan-code-latest"));
    assert!(!personal.iter().any(|m| m == "deepseek-v3.2"));
    let team = discover(&endpoints("baidu-qianfan-team"), "UNUSED_KEY").await?;
    assert!(team.iter().any(|m| m == "deepseek-v3.2"));
    for (region, count) in [("global", 23), ("eu", 9), ("us", 9), ("us-central1", 3)] {
        let mut vertex: Endpoints = nextjson::from_value(
            nextjson::json!({"vertex":{"project":"sample-project", "location":region}}),
        )
        .map_err(anyhow::Error::msg)?;
        assert_eq!(discover(&vertex, "").await?.len(), count);
        vertex.models_url = "https://relay.example/v1/models".to_owned();
        assert!(
            discover(&vertex, "")
                .await
                .unwrap_err()
                .to_string()
                .contains("project's Google endpoint")
        );
    }
    for (id, expected) in [
        ("bedrock", "global.anthropic.claude-opus-5-5"),
        ("voyage", "rerank-3"),
        ("google-vertex", "gemini-2.5-pro"),
        ("volcengine-agent", "ark-code-latest"),
        ("tencent-cloud-plan", "tc-code-latest"),
        ("tencent-cloud-enterprise-intl", "auto"),
    ] {
        let list = discover(&endpoints(id), "UNUSED_KEY").await?;
        assert!(list.iter().any(|m| m == expected));
    }
    for id in [
        "bedrock-mantle",
        "volcengine-api",
        "baidu-qianfan-api",
        "fireworks",
        "tencent-cloud-intl",
    ] {
        assert!(fixed_models(&endpoints(id)).is_none());
    }
    Ok(())
}

#[tokio::test]
async fn protocol_directories_merge_only_complete_lists_and_allow_an_override() -> Result<()> {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let root = format!("http://{}", listener.local_addr()?);
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let app = Router::new()
        .route(
            "/chat/models",
            get(move |_| {
                counter.fetch_add(1, Ordering::Relaxed);
                async { Json(json!({"data":[{"id":"shared"},{"id":"chat"}]})) }
            }),
        )
        .route(
            "/anthropic/v1/models",
            get(|Incoming { uri, .. }| async move {
                if uri.query().is_none() {
                    Json(json!({"data":[{"id":"partial"}],"has_more":true,"last_id":"next"}))
                } else {
                    Json(json!({"data":"broken"}))
                }
            }),
        )
        .route(
            "/gemini/models",
            get(|_| async {
                Json(json!({"models":[{"name":"models/shared"},{"name":"models/gemini"}]}))
            }),
        )
        .route(
            "/judge/v1/models",
            get(|Incoming { headers, .. }| async move {
                assert_eq!(headers["x-magpie-deciders"], "true");
                Json(json!({"data":[{"id":"jev","native_endpoints":["/v1/systemone"]}]}))
            }),
        )
        .route(
            "/account/catalog",
            get(|Incoming { uri, .. }| async move {
                assert_eq!(uri.query(), Some("edition=team"));
                Json(json!({"data":[{"id":"account-model"}]}))
            }),
        );
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    let endpoints = Endpoints {
        chat: format!("{root}/chat"),
        responses: format!("{root}/chat/"),
        anthropic: format!("{root}/anthropic"),
        gemini: format!("{root}/gemini"),
        systemone: format!("{root}/judge/v1/systemone/"),
        ..Default::default()
    };
    let models = crabforge_core::catalog::discover_catalog(&endpoints, "").await?;
    assert_eq!(models.models, ["shared", "chat", "gemini", "jev"]);
    assert!(
        models.model_protocols["jev"]
            .native_endpoints
            .as_ref()
            .unwrap()
            .contains(&crabforge_core::catalog::RelayEndpoint::SystemOne)
    );
    assert_eq!(hits.load(Ordering::Relaxed), 1);
    let endpoints = Endpoints {
        models_url: format!("{root}/account/catalog?edition=team"),
        ..endpoints
    };
    assert_eq!(discover(&endpoints, "").await?, ["account-model"]);
    assert_eq!(hits.load(Ordering::Relaxed), 1);
    let builtin_with_override = Endpoints {
        chat: "https://api.voyageai.com/v1".into(),
        ..endpoints
    };
    assert_eq!(
        discover(&builtin_with_override, "").await?,
        ["account-model"]
    );
    server.abort();
    Ok(())
}

#[tokio::test]
async fn catalog_follows_vendor_cursors_and_rejects_incomplete_pages() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let root = format!("http://{}", listener.local_addr()?);
    let app = Router::new()
        .route(
            "/v1beta/models",
            get(|Incoming {uri, ..}| async move {
                Json(match uri.query() {
                    Some("pageSize=1000") => json!({"models":[{"name":"models/first"},{"name":"models/embedding","supportedGenerationMethods":["embedContent"]}],"nextPageToken":"next+1"}),
                    Some("pageSize=1000&pageToken=next%2B1") => {
                        json!({"models":[{"name":"models/first"},{"name":"models/second", "supportedGenerationMethods":["countTokens","generateContent"]},{"name":"models/video","supportedGenerationMethods":["predictLongRunning"]}]})
                    }
                    other => panic!("unexpected cursor {other:?}"),
                })
            }),
        )
        .route(
            "/v1/messages-api/v1/models",
            get(
                |Incoming {headers, uri, ..}| async move {
                    assert_eq!(headers["anthropic-version"], "2023-06-01");
                    Json(match uri.query() {
                        None => json!({"data":[{"id":"first"}],"has_more":true,"last_id":"abc"}),
                        Some("after_id=abc") => json!({"data":[{"id":"second"}],"has_more":false}),
                        other => panic!("unexpected cursor {other:?}"),
                    })
                },
            ),
        )
        .route(
            "/broken/v1/models",
            get(|_| async { Json(json!({"data":[{"id":"partial"}],"has_more":true,"last_id":""})) }),
        );
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    for endpoints in [
        Endpoints {
            gemini: format!("{root}/v1beta/models/vendor/demo:streamGenerateContent?alt=sse"),
            ..Default::default()
        },
        Endpoints {
            gemini: root.clone(),
            ..Default::default()
        },
        Endpoints {
            gemini: format!("{root}/models"),
            ..Default::default()
        },
        Endpoints {
            anthropic: format!("{root}/v1/messages-api"),
            ..Default::default()
        },
    ] {
        assert_eq!(discover(&endpoints, "").await?, ["first", "second"]);
    }
    let incomplete = Endpoints {
        anthropic: format!("{root}/broken"),
        ..Default::default()
    };
    assert!(
        discover(&incomplete, "")
            .await
            .unwrap_err()
            .to_string()
            .contains("cursor")
    );
    server.abort();
    let _ = server.await;
    Ok(())
}

#[tokio::test]
async fn redirects_stay_at_the_source_and_pages_share_a_byte_limit() -> Result<()> {
    use crabforge_test_support::server::StatusCode;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let root = format!("http://{}", listener.local_addr()?);
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let app = Router::new()
        .route(
            "/redirect",
            get(|_| async { (StatusCode::TEMPORARY_REDIRECT, [("location", "/target")]) }),
        )
        .route(
            "/target",
            get(move |_| {
                counter.fetch_add(1, Ordering::Relaxed);
                async { Json(json!({"data":[{"id":"redirected"}]})) }
            }),
        )
        .route(
            "/pages",
            get(|Incoming { uri, .. }| async move {
                let mut text = if uri.query().is_none() {
                    r#"{"data":[{"id":"partial"}],"has_more":true,"last_id":"p"}"#
                } else {
                    r#"{"data":[{"id":"end"}]}"#
                }
                .to_owned();
                text.extend(std::iter::repeat_n(' ', 2 * 1024 * 1024));
                text
            }),
        );
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    for (path, error) in [("redirect", "HTTP 307"), ("pages", "exceeds 4 MiB")] {
        let source = Endpoints {
            anthropic: root.clone(),
            models_url: format!("{root}/{path}"),
            ..Default::default()
        };
        let message = discover(&source, "").await.unwrap_err().to_string();
        assert!(message.contains(error), "{message}");
    }
    assert_eq!(hits.load(Ordering::Relaxed), 0);
    server.abort();
    Ok(())
}
