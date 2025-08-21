//! A collection of traits to define a common interface across reactors

// We want to forbid unsafe code but need to unsafe impl for async-io...
#![allow(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use async_trait::async_trait;
use std::{
    io,
    net::SocketAddr,
};

pub use reactor_trait_3::{AsyncIOHandle, IOHandle, Reactor, TimeReactor, AsyncToSocketAddrs};

/// A common interface for registering TCP handles in a reactor.
#[async_trait]
pub trait TcpReactor {
    /// Create a TcpStream by connecting to a remove host
    async fn connect<A: Into<SocketAddr> + Send>(
        // FIXME: missing &self
        addr: A,
    ) -> io::Result<Box<dyn AsyncIOHandle + Send>>;
}
