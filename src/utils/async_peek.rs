use std::io::Error;
use std::pin::Pin;
use std::task::Poll;
use std::task::Context;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::io::poll_read_buf;
use tokio_util::io::poll_write_buf;

pub trait AsyncPeek: AsyncRead + AsyncWrite + Unpin {
    fn peek(self: Pin<&mut Self>, cx: &mut Context<'_>, size: usize, buf: &[u8]) -> Poll<std::io::Result<()>>;
}

pub struct Peek<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    peek: Vec<u8>,
    inner: RW
}

impl<RW> Peek<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    pub fn new(rw: RW) -> Self {
        return Self {
            inner: rw,
            peek: Vec::new(),
        }
    }
}

impl<RW> AsyncPeek for Peek<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    fn peek(self: Pin<&mut Self>, cx: &mut Context<'_>, size: usize, buf: &[u8]) -> Poll<std::io::Result<()>> {
        todo!()
    }
}

use std::pin::pin;

impl<RW> AsyncRead for Peek<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    #[allow(unused)]
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        poll_read_buf(pin!(&mut self.inner), cx, buf);

        return Poll::Ready(Ok(()))
    }
}

impl<RW> AsyncWrite for Peek<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, mut buf: &[u8]) -> Poll<Result<usize, Error>> {
        return poll_write_buf(pin!(&mut self.inner), cx, &mut buf);
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }
}

pub fn poll_peek_buf<T: AsyncPeek + ?Sized>(io: Pin<&mut T>, cx: &mut Context<'_>, size: usize, buf: &[u8]) -> Poll<std::io::Result<()>>{
    return io.peek(cx, size, buf) 
}