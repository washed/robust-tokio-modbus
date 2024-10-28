use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_modbus::{prelude::*, Result as ModbusResult};
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::Retry;
use tracing::{error, info};

#[derive(Debug)]
pub struct RobustContext {
    pub host: String,
    pub slave: Slave,
    slave_sender: mpsc::Sender<Slave>,
    pub ctx: Arc<Mutex<io::Result<client::Context>>>,
}

impl RobustContext {
    pub async fn new(host: &str, slave: Slave) -> io::Result<RobustContext> {
        let ctx = Arc::new(Mutex::new(Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "not yet connected",
        ))));

        let (slave_sender, mut slave_receiver) = mpsc::channel(8);

        let ctx_slave_setter = ctx.clone();
        tokio::spawn(async move {
            tokio::select! {
                Some(slave) = slave_receiver.recv() => {
                    let _ = Self::set_slave(ctx_slave_setter, slave).await;
                }
            }
        });

        Ok(Self {
            host: host.to_string(),
            slave,
            slave_sender,
            ctx,
        })
    }

    fn resolve_host(host: &str) -> io::Result<SocketAddr> {
        host.to_socket_addrs()?.next().ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "cannot resolve hostname",
        ))
    }

    pub fn retry_strategy_connect() -> impl Iterator<Item = Duration> {
        FixedInterval::from_millis(10).map(jitter).take(3)
    }

    pub fn retry_strategy_command() -> impl Iterator<Item = Duration> {
        FixedInterval::from_millis(10).map(jitter).take(3)
    }

    async fn set_context(
        ctx: Arc<Mutex<io::Result<client::Context>>>,
        host: &str,
        slave: Slave,
    ) -> io::Result<()> {
        let socket_addr = RobustContext::resolve_host(host)?;

        let mut ctx_guard = ctx.lock().await;
        info!("trying to connect modbus: {:?}", ctx_guard);
        *ctx_guard = tcp::connect_slave(socket_addr, slave).await;

        match ctx_guard.as_ref() {
            Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    async fn set_slave(
        ctx: Arc<Mutex<io::Result<client::Context>>>,
        slave: Slave,
    ) -> io::Result<()> {
        let mut ctx_guard = ctx.lock().await;
        let ctx = ctx_guard
            .as_mut()
            .map_err(|e| io::Error::new(e.kind(), e.to_string()))?;
        ctx.set_slave(slave);

        Ok(())
    }

    pub async fn refresh_context(
        ctx: Arc<Mutex<io::Result<client::Context>>>,
        host: &str,
        slave: Slave,
    ) {
        let action = || RobustContext::set_context(ctx.clone(), host, slave);
        match Retry::spawn(RobustContext::retry_strategy_connect(), action).await {
            Ok(_) => info!("successfully reconnected modbus"),
            Err(_) => error!("could not reconnect modbus"),
        };
    }
}

impl SlaveContext for RobustContext {
    fn set_slave(&mut self, slave: Slave) {
        let _ = self.slave_sender.blocking_send(slave);
    }
}

impl Client for RobustContext {
    #[doc = " Invoke a _Modbus_ function"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn call<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        request: Request<'life1>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Response>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            let mut ctx_guard = self.ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| io::Error::new(e.kind(), e.to_string()))?;

            ctx.call(request).await
        })
    }

    #[doc = " Disconnects the client."]
    #[doc = ""]
    #[doc = " Permanently disconnects the client by shutting down the"]
    #[doc = " underlying stream in a graceful manner (`AsyncDrop`)."]
    #[doc = ""]
    #[doc = " Dropping the client without explicitly disconnecting it"]
    #[doc = " beforehand should also work and free all resources. The"]
    #[doc = " actual behavior might depend on the underlying transport"]
    #[doc = " protocol (RTU/TCP) that is used by the client."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn disconnect<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = io::Result<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            let mut ctx_guard = self.ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| io::Error::new(e.kind(), e.to_string()))?;

            ctx.disconnect().await
        })
    }
}
