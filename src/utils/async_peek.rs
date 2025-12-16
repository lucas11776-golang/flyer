use std::io::Error;
use std::mem::take;
use std::pin::Pin;
use std::task::Poll;
use std::task::Context;

use async_std::task::block_on;
use bytes::BufMut;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::io::poll_read_buf;
use tokio_util::io::poll_write_buf;

pub trait AsyncPeek: AsyncRead + AsyncWrite + Unpin {
    fn peek<B: BufMut>(self: Pin<&mut Self>, size: usize, buf: &mut B) -> Poll<std::io::Result<()>>;
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
    fn peek<B: BufMut>(mut self: Pin<&mut Self>, size: usize, buf: &mut B) -> Poll<std::io::Result<()>> {
        if size >= self.peek.len() {
            let mut buffer: Vec<u8> = Vec::with_capacity(size - self.peek.len());

            block_on(self.inner.read_buf(&mut buffer))?;

            self.peek.put(take(&mut buffer).as_slice());
        }

        buf.put(&self.peek[0..size]);

        return Poll::Ready(Ok(()));
    }
}

use std::pin::pin;

impl<RW> AsyncRead for Peek<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    #[allow(unused)]
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, mut buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let mut buffer = Vec::new();

        poll_read_buf(pin!(&mut self.inner), cx, &mut buffer)?;

        buffer.splice(0..0, take(&mut self.peek));

        // println!("{}\r\n\r\n\r\n\r\n\r\n\r\n", String::from_utf8_lossy(&buffer));

        // buf.put_slice(&buffer);

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

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }
}

pub fn poll_peek_buf<T: AsyncPeek + ?Sized, B: BufMut>(io: Pin<&mut T>, size: usize, buf: &mut B) -> Poll<std::io::Result<()>>{
    return io.peek(size, buf) 
}