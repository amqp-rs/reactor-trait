/// Dummy object implementing reactor-trait common interfaces on top of tokio
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tokio;

#[cfg(unix)]
mod unix {
    use crate::Tokio;
    use async_trait::async_trait;
    use futures_io::{AsyncRead, AsyncWrite};
    use reactor_trait::{AsyncIOHandle, IOHandle, Reactor};
    use std::{
        io::{self, IoSlice, IoSliceMut, Read, Write},
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };
    use tokio::{io::unix::AsyncFd, runtime::Handle};

    #[derive(Debug)]
    pub(super) struct TokioReactor(pub(super) Handle);

    #[async_trait]
    impl Reactor for Tokio {
        fn register(&self, socket: IOHandle) -> io::Result<Box<dyn AsyncIOHandle + Send>> {
            Ok(Box::new(AsyncFdWrapper(AsyncFd::new(socket)?)))
        }

        async fn sleep(&self, dur: Duration) {
            tokio::time::sleep(dur).await;
        }
    }

    struct AsyncFdWrapper(AsyncFd<IOHandle>);

    impl AsyncFdWrapper {
        fn read<F: FnOnce(&mut AsyncFd<IOHandle>) -> futures_io::Result<usize>>(
            &mut self,
            cx: &mut Context<'_>,
            f: F,
        ) -> Option<Poll<futures_io::Result<usize>>> {
            Some(match self.0.poll_read_ready_mut(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(mut guard)) => match guard.try_io(f) {
                    Ok(res) => Poll::Ready(res),
                    Err(_) => return None,
                },
            })
        }

        fn write<R, F: FnOnce(&mut AsyncFd<IOHandle>) -> futures_io::Result<R>>(
            &mut self,
            cx: &mut Context<'_>,
            f: F,
        ) -> Option<Poll<futures_io::Result<R>>> {
            Some(match self.0.poll_write_ready_mut(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(mut guard)) => match guard.try_io(f) {
                    Ok(res) => Poll::Ready(res),
                    Err(_) => return None,
                },
            })
        }
    }

    impl AsyncRead for AsyncFdWrapper {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<futures_io::Result<usize>> {
            loop {
                if let Some(res) = self.read(cx, |socket| socket.get_mut().read(buf)) {
                    return res;
                }
            }
        }

        fn poll_read_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &mut [IoSliceMut<'_>],
        ) -> Poll<futures_io::Result<usize>> {
            loop {
                if let Some(res) = self.read(cx, |socket| socket.get_mut().read_vectored(bufs)) {
                    return res;
                }
            }
        }
    }

    impl AsyncWrite for AsyncFdWrapper {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<futures_io::Result<usize>> {
            loop {
                if let Some(res) = self.write(cx, |socket| socket.get_mut().write(buf)) {
                    return res;
                }
            }
        }

        fn poll_write_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[IoSlice<'_>],
        ) -> Poll<futures_io::Result<usize>> {
            loop {
                if let Some(res) = self.write(cx, |socket| socket.get_mut().write_vectored(bufs)) {
                    return res;
                }
            }
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<futures_io::Result<()>> {
            loop {
                if let Some(res) = self.write(cx, |socket| socket.get_mut().flush()) {
                    return res;
                }
            }
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<futures_io::Result<()>> {
            self.poll_flush(cx)
        }
    }
}
