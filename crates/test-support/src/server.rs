pub use axum_core::{
    body::Body,
    response::{IntoResponse, Response},
};
pub use bytes::Bytes;
use futures_util::{TryStreamExt, future::BoxFuture};
pub use http::{HeaderMap, StatusCode};
use http::{Method, Uri};
use http_body::Body as _;
use nextjson::Value;
use std::{io, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    task::JoinSet,
};
use tokio_util::{compat::TokioAsyncReadCompatExt, io::StreamReader};
use trillium_http::{Conn, HttpContext};

pub struct Json(pub Value);

impl IntoResponse for Json {
    fn into_response(self) -> Response {
        ([("content-type", "application/json")], self.0.to_string()).into_response()
    }
}

pub struct Incoming {
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub param: String,
}

pub struct Route {
    method: Option<Method>,
    handler: Box<dyn Fn(Incoming) -> BoxFuture<'static, Response> + Send + Sync>,
}

pub fn any<F, Fut, R>(handler: F) -> Route
where
    F: Fn(Incoming) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    Route {
        method: None,
        handler: Box::new(move |request| {
            let result = handler(request);
            Box::pin(async move { result.await.into_response() })
        }),
    }
}

pub fn post<F, Fut, R>(handler: F) -> Route
where
    F: Fn(Incoming) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    Route {
        method: Some(Method::POST),
        ..any(handler)
    }
}

pub fn get<F, Fut, R>(handler: F) -> Route
where
    F: Fn(Incoming) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    Route {
        method: Some(Method::GET),
        ..any(handler)
    }
}

#[derive(Default)]
pub struct Router {
    routes: Vec<(&'static str, Route)>,
    fallback: Option<Route>,
    observer: Option<tokio::sync::mpsc::UnboundedSender<String>>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn route(mut self, path: &'static str, route: Route) -> Self {
        self.routes.push((path, route));
        self
    }
    pub fn fallback(mut self, route: Route) -> Self {
        self.fallback = Some(route);
        self
    }
    pub fn merge(mut self, other: Self) -> Self {
        self.routes.extend(other.routes);
        self
    }
    pub fn observe(mut self, sender: tokio::sync::mpsc::UnboundedSender<String>) -> Self {
        self.observer = Some(sender);
        self
    }
    async fn handle(
        &self,
        paths: &matchit::Router<usize>,
        method: Method,
        mut request: Incoming,
    ) -> Response {
        if let Some(sender) = &self.observer {
            let _ = sender.send(request.uri.path().into());
        }
        let route = match paths.at(request.uri.path()) {
            Ok(found) => {
                request.param = found
                    .params
                    .iter()
                    .next()
                    .map(|(_, v)| v.to_owned())
                    .unwrap_or_default();
                &self.routes[*found.value].1
            }
            Err(_) => match &self.fallback {
                Some(route) => route,
                None => return StatusCode::NOT_FOUND.into_response(),
            },
        };
        if route.method.as_ref().is_some_and(|m| m != method) {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }
        (route.handler)(request).await
    }
}

pub async fn serve(listener: TcpListener, app: Router) -> io::Result<()> {
    let app = Arc::new(app);
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {let (tcp, _) = accepted?; connections.spawn(serve_io(tcp, app.clone()));}
            Some(result) = connections.join_next(), if !connections.is_empty() => { result.unwrap(); }
        }
    }
}

pub async fn serve_io(
    io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    app: Arc<Router>,
) {
    let context = Arc::new(HttpContext::new());
    let mut paths = matchit::Router::new();
    for (index, (path, _)) in app.routes.iter().enumerate() {
        paths.insert(*path, index).unwrap();
    }
    let _ = context
        .run(io.compat(), |mut conn: Conn<_>| {
            let app = &app;
            let paths = &paths;
            async move {
                let method = Method::from_bytes(conn.method().as_ref().as_bytes()).unwrap();
                let request = Incoming {
                    uri: conn.path_and_query().parse().unwrap(),
                    headers: conn.request_headers().clone().try_into().unwrap(),
                    body: conn.request_body().read_bytes().await.unwrap().into(),
                    param: String::new(),
                };
                let (parts, body) = app.handle(paths, method, request).await.into_parts();
                *conn.response_headers_mut() = parts.headers.into();
                let len = body.size_hint().exact();
                let stream = StreamReader::new(body.into_data_stream().map_err(io::Error::other));
                conn.with_status(parts.status.as_u16())
                    .with_response_body(trillium_http::Body::new_streaming(stream.compat(), len))
            }
        })
        .await;
}
