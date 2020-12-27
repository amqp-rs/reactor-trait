use async_io::{Async, Timer};
use async_trait::async_trait;
use reactor_trait::{Reactor, IOHandle, AsyncIOHandle};
use std::{io, time::Duration};

/// Dummy object implementing reactor-trait common interfaces on top of async-io
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsyncIo;

#[async_trait]
impl Reactor for AsyncIo {
    fn register(&self, socket: IOHandle) -> io::Result<Box<dyn AsyncIOHandle + Send>> {
        Ok(Box::new(Async::new(socket)?))
    }

    async fn sleep(&self, dur: Duration) {
        Timer::after(dur).await;
    }
}
