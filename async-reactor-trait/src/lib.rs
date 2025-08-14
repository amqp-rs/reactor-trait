use async_io::{Async, Timer};
use async_trait::async_trait;
use futures_core::Stream;
use reactor_trait::{AsyncIOHandle, IOHandle, Reactor, TcpReactor, TimeReactor};
use std::{
    io,
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
};

/// Dummy object implementing reactor-trait common interfaces on top of async-io
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsyncIo;

impl Reactor for AsyncIo {
    fn register(&self, socket: IOHandle) -> io::Result<Box<dyn AsyncIOHandle + Send>> {
        Ok(Box::new(Async::new(socket)?))
    }
}

#[async_trait]
impl TimeReactor for AsyncIo {
    async fn sleep(&self, dur: Duration) {
        Timer::after(dur).await;
    }

    fn interval(&self, dur: Duration) -> Box<dyn Stream<Item = Instant>> {
        Box::new(Timer::interval(dur))
    }
}

#[async_trait]
impl TcpReactor for AsyncIo {
    /// Create a TcpStream by connecting to a remove host
    async fn connect(&self, addr: SocketAddr) -> io::Result<Box<dyn AsyncIOHandle + Send>> {
        Ok(Box::new(Async::<TcpStream>::connect(addr).await?))
    }
}
