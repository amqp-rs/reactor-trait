//! A collection of traits to define a common interface across reactors

// We want to forbid unsafe code but need to unsafe impl for async-io...
#![allow(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use async_trait::async_trait;
use futures_core::Stream;
use futures_io::{AsyncRead, AsyncWrite};
use std::{
    fmt,
    io::{self, IoSlice, IoSliceMut, Read, Write},
    net::SocketAddr,
    time::{Duration, Instant},
};
use sys::IO;

/// A synchronous IO handle
pub struct IOHandle(Box<dyn IO + Send>);

impl IOHandle {
    /// Instantiate a new IO handle
    pub fn new<H: IO + Send + 'static>(io: H) -> Self {
        Self(Box::new(io))
    }
}

impl Read for IOHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.0.read_exact(buf)
    }
}

impl Write for IOHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

impl fmt::Debug for IOHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IOHandle").finish()
    }
}

#[cfg(feature = "async_io_safe")]
unsafe impl async_io::IoSafe for IOHandle {}

/// A trait representing an asynchronous IO handle
pub trait AsyncIOHandle: AsyncRead + AsyncWrite {}
impl<IO: AsyncRead + AsyncWrite> AsyncIOHandle for IO {}

/// A common interface for registering IO handles in a reactor.
pub trait Reactor {
    /// Register a synchronous handle, returning an asynchronous one
    fn register(&self, socket: IOHandle) -> io::Result<Box<dyn AsyncIOHandle + Send>>;
}

/// A common interface for registering timers in a reactor.
#[async_trait]
pub trait TimeReactor {
    /// Sleep for the given duration
    async fn sleep(&self, dur: Duration);
    /// Stream that yields at every given interval
    fn interval(&self, dur: Duration) -> Box<dyn Stream<Item = Instant>>;
}

/// A common interface for registering TCP handles in a reactor.
#[async_trait]
pub trait TcpReactor {
    /// Create a TcpStream by connecting to a remove host
    async fn connect<A: Into<SocketAddr> + Send>(
        // FIXME: missing &self
        addr: A,
    ) -> io::Result<Box<dyn AsyncIOHandle + Send>>;
}

/// A common interface for resolving domain name + port to `SocketAddr`
#[async_trait]
pub trait AsyncToSocketAddrs {
    /// Resolve the domain name through DNS and return an `Iterator` of `SocketAddr`
    async fn to_socket_addrs(&self) -> io::Result<Box<dyn Iterator<Item = SocketAddr>>>;
}

#[cfg(unix)]
mod sys {
    use crate::IOHandle;
    use std::{
        io::{Read, Write},
        os::unix::io::{AsFd, AsRawFd, BorrowedFd, RawFd},
    };

    pub trait IO: Read + Write + AsFd {}
    impl<H: Read + Write + AsFd> IO for H {}

    impl AsFd for IOHandle {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.0.as_fd()
        }
    }

    impl AsRawFd for IOHandle {
        fn as_raw_fd(&self) -> RawFd {
            self.as_fd().as_raw_fd()
        }
    }
}

#[cfg(windows)]
mod sys {
    use crate::IOHandle;
    use std::{
        io::{Read, Write},
        os::windows::io::{AsRawSocket, AsSocket, BorrowedSocket, RawSocket},
    };

    pub trait IO: Read + Write + AsSocket {}
    impl<H: Read + Write + AsSocket> IO for H {}

    impl AsSocket for IOHandle {
        fn as_socket(&self) -> BorrowedSocket<'_> {
            self.0.as_socket()
        }
    }

    impl AsRawSocket for IOHandle {
        fn as_raw_socket(&self) -> RawSocket {
            self.as_socket().as_raw_socket()
        }
    }
}
