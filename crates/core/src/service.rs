use crate::{
    config,
    gateway::{self, History},
    peer::{self, PeerServer},
};
use anyhow::Result;
use axum::Router;
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use std::{net::SocketAddr, path::Path, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    task::{JoinHandle, JoinSet},
    time::timeout,
};

pub(crate) async fn serve_connection(io: impl AsyncRead + AsyncWrite + Unpin, app: Router) {
    let mut http = hyper::server::conn::http1::Builder::new();
    http.timer(TokioTimer::new())
        .header_read_timeout(Duration::from_secs(15))
        .max_buf_size(32768);
    let _ = timeout(
        Duration::from_secs(3600),
        http.serve_connection(TokioIo::new(io), TowerToHyperService::new(app)),
    )
    .await;
}

async fn serve_local(listener: TcpListener, app: Router) -> std::io::Result<()> {
    let mut workers = JoinSet::new();
    loop {
        tokio::select! {
            socket=listener.accept(), if workers.len()<64=>{
                let (socket,_)=socket?;
                workers.spawn(serve_connection(socket,app.clone()));
            }
            _=workers.join_next(), if !workers.is_empty()=>(),
        }
    }
}

pub struct Service {
    pub addr: SocketAddr,
    pub peer: Option<PeerServer>,
    pub history: History,
    http: JoinHandle<std::io::Result<()>>,
}

impl Service {
    pub async fn start(dir: &Path) -> Result<Self> {
        let cfg = config::load(dir)?;
        let history = History::default();
        let addr = cfg.listen;
        let peer_listen = cfg.peer_listen.clone();
        let allowed = cfg
            .allowed_peers
            .iter()
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;
        let app = gateway::router(cfg, config::token(dir)?, history.clone(), dir)?;
        let listener = TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        let peer = if peer_listen.is_empty() {
            None
        } else {
            Some(
                peer::serve(
                    peer::identity(dir)?,
                    peer::listen_address(&peer_listen)?,
                    allowed,
                    app.clone(),
                )
                .await?,
            )
        };
        let http = tokio::spawn(serve_local(listener, app));
        Ok(Self {
            addr,
            peer,
            history,
            http,
        })
    }

    pub fn running(&self) -> bool {
        !self.http.is_finished() && self.peer.as_ref().is_none_or(|p| !p.task.is_finished())
    }

    pub async fn wait(&mut self) -> Result<()> {
        if let Some(p) = &mut self.peer {
            tokio::select! {r=&mut p.task=>r??,r=&mut self.http=>r??}
        } else {
            (&mut self.http).await??;
        }
        Ok(())
    }

    pub async fn stop(mut self) {
        self.http.abort();
        if let Some(p) = self.peer.take() {
            p.task.abort();
            let _ = p.task.await;
        }
        let _ = (&mut self.http).await;
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.http.abort();
        if let Some(p) = &self.peer {
            p.task.abort();
        }
    }
}
