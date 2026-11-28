use crate::{
    catalog, config,
    gateway::{self, Gateway, History},
    memory::{distill::schedule, tidy},
    peer::{self, PeerServer},
};
use anyhow::Result;
use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    task::{JoinHandle, JoinSet},
    time::timeout,
};

mod http1;

pub(crate) async fn serve_connection(io: impl AsyncRead + AsyncWrite + Unpin, app: Arc<Gateway>) {
    let _ = timeout(Duration::from_secs(3600), http1::serve(io, &app)).await;
}

pub(crate) async fn serve_local(listener: TcpListener, app: Arc<Gateway>) -> std::io::Result<()> {
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
    memory: JoinHandle<()>,
    catalogs: JoinHandle<()>,
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
        let app = gateway::router(cfg.clone(), config::token(dir)?, history.clone(), dir)?;
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
        let http = tokio::spawn(serve_local(listener, app.clone()));
        let root = dir.to_owned();
        let catalogs = tokio::spawn(async move { catalog::maintain(&root, app, cfg).await });
        let root = dir.to_owned();
        let memory = tokio::spawn(async move {
            tokio::join!(
                schedule::hourly(&root),
                crate::project_instructions::publication::maintain(&root),
                tidy::maintain(&root),
            );
        });
        Ok(Self {
            addr,
            peer,
            history,
            http,
            memory,
            catalogs,
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
        self.memory.abort();
        self.catalogs.abort();
        if let Some(p) = self.peer.take() {
            p.task.abort();
            let _ = p.task.await;
        }
        let _ = (&mut self.http).await;
        let _ = (&mut self.memory).await;
        let _ = (&mut self.catalogs).await;
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.http.abort();
        self.memory.abort();
        self.catalogs.abort();
        if let Some(p) = &self.peer {
            p.task.abort();
        }
    }
}
