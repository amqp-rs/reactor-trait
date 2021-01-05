//! A collection of traits to define a common interface across reactors

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use async_trait::async_trait;
use futures_core::Stream;
use futures_io::{AsyncRead, AsyncWrite};
use std::{
    fmt,
    io::{self, IoSlice, IoSliceMut, Read, Write},
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

/// A trait representing an asynchronous IO handle
pub trait AsyncIOHandle: AsyncRead + AsyncWrite {}
impl<IO: AsyncRead + AsyncWrite> AsyncIOHandle for IO {}

/// A common interface for registering IO handles in a reactor.
#[async_trait]
pub trait Reactor {
    /// Register a synchronous handle, returning an asynchronous one
    fn register(&self, socket: IOHandle) -> io::Result<Box<dyn AsyncIOHandle + Send>>;
    /// Sleep for the given duration
    async fn sleep(&self, dur: Duration);
    /// Stream that yields at every given interval
    fn interval(&self, dur: Duration) -> Box<dyn Stream<Item = Instant>>;
}

#[cfg(unix)]
mod sys {
    use crate::IOHandle;
    use std::{
        io::{Read, Write},
        os::unix::io::{AsRawFd, RawFd},
    };

    pub trait IO: Read + Write + AsRawFd {}
    impl<H: Read + Write + AsRawFd> IO for H {}

    impl AsRawFd for IOHandle {
        fn as_raw_fd(&self) -> RawFd {
            self.0.as_raw_fd()
        }
    }
}

#[cfg(windows)]
mod sys {
    use crate::IOHandle;
    use std::{
        io::{Read, Write},
        os::windows::io::{AsRawSocket, RawSocket},
    };

    pub trait IO: Read + Write + AsRawSocket {}
    impl<H: Read + Write + AsRawSocket> IO for H {}

    impl AsRawSocket for IOHandle {
        fn as_raw_socket(&self) -> RawSocket {
            self.0.as_raw_socket()
        }
    }
}
