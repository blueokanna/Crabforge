use std::{io, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    runtime::Handle,
};
use tokio_util::sync::CancellationToken;
use ureq::{
    Agent, Error,
    config::Config,
    unversioned::{
        resolver::Resolver,
        transport::{
            Buffers, ConnectProxyConnector, ConnectionDetails, Connector, Either, LazyBuffers,
            NextTimeout, RustlsConnector, ShutdownHandle, Transport, time::Duration,
        },
    },
};

pub(super) fn agent(config: Config, resolver: impl Resolver) -> Agent {
    let chain = ConnectProxyConnector::default()
        .chain(SocketConnector)
        .chain(RustlsConnector::default());
    Agent::with_parts(config, chain, resolver)
}

#[derive(Debug, Default)]
struct Cancel(CancellationToken);

impl ShutdownHandle for Cancel {
    fn shutdown(&self) {
        self.0.cancel();
    }
}

impl Cancel {
    // ureq drives the codec on a blocking worker. A cancelled Tokio socket operation
    // wakes that worker even if the remote server never writes again or observes EOF.
    fn wait<T>(
        &self,
        timeout: NextTimeout,
        operation: impl Future<Output = io::Result<T>>,
    ) -> Result<T, Error> {
        Handle::current().block_on(async {
            tokio::select! {
                biased;
                _ = self.0.cancelled() => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "request cancelled").into()),
                result = async {
                    match timeout.after {
                        Duration::Exact(left) => tokio::time::timeout(left, operation).await.map_err(|_| Error::Timeout(timeout.reason))?,
                        Duration::NotHappening => operation.await,
                    }.map_err(Error::from)
                } => result,
            }
        })
    }
}

#[derive(Debug)]
struct SocketConnector;

impl<In: Transport> Connector<In> for SocketConnector {
    type Out = Either<In, Socket>;

    fn connect(
        &self,
        details: &ConnectionDetails,
        chained: Option<In>,
    ) -> Result<Option<Self::Out>, Error> {
        if let Some(tunnel) = chained {
            return Ok(Some(Either::A(tunnel)));
        }
        if details.config.proxy().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "expected an HTTP or HTTPS CONNECT proxy",
            )
            .into());
        }
        let cancel = Arc::new(Cancel::default());
        (details.register_shutdown)(Arc::downgrade(&cancel) as _)?;
        let tcp = cancel.wait(details.timeout, TcpStream::connect(&details.addrs[..]))?;
        tcp.set_nodelay(details.config.no_delay())?;
        let buffers = LazyBuffers::new(
            details.config.input_buffer_size(),
            details.config.output_buffer_size(),
        );
        Ok(Some(Either::B(Socket {
            tcp,
            cancel,
            buffers,
        })))
    }
}

#[derive(Debug)]
struct Socket {
    tcp: TcpStream,
    cancel: Arc<Cancel>,
    buffers: LazyBuffers,
}

impl Transport for Socket {
    fn buffers(&mut self) -> &mut dyn Buffers {
        &mut self.buffers
    }

    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), Error> {
        self.cancel.wait(
            timeout.for_write(),
            self.tcp.write_all(&self.buffers.output()[..amount]),
        )
    }

    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, Error> {
        let size = self.cancel.wait(
            timeout.for_read(),
            self.tcp.read(self.buffers.input_append_buf()),
        )?;
        self.buffers.input_appended(size);
        Ok(size > 0)
    }

    fn is_open(&mut self) -> bool {
        matches!(self.tcp.try_read(&mut [0]), Err(e) if e.kind() == io::ErrorKind::WouldBlock)
    }
}
