use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncWrite, AsyncRead, ReadBuf, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};

enum Stream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            Stream::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Stream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Stream::Unix(s) => Pin::new(s).poll_flush(cx),
            Stream::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Stream::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Stream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Stream::Unix(s) => Pin::new(s).poll_read(cx, buf),
            Stream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

pub struct Client {
    stream: Pin<Box<Stream>>,
}

impl Client {
    pub async fn connect(addr: &str, unix: bool) -> std::io::Result<Self> {
        let stream = if unix {
            Stream::Unix(UnixStream::connect(addr).await?)
        } else {
            Stream::Tcp(TcpStream::connect(addr).await?)
        };
        Ok(Self { stream: Box::pin(stream) })
    }

    pub async fn set(
        &mut self,
        // cx: &mut Context<'_>,
        key: &str,
        value: &[u8],
        flags: u32,
        expiration: u32
    ) -> std::io::Result<bool> {
        let cmd = format!("set {} {} {} {}\r\n", key, flags, expiration, value.len());
        self.stream.write_all(cmd.as_bytes()).await?;
        // self.stream.as_mut().poll_write(cx, cmd.as_bytes())?;
        // self.stream.as_mut().poll_write(cx, value)?;
        // self.stream.as_mut().poll_write(cx, b"\r\n")?;

        Ok(true)
    }

    pub async fn get(&mut self, _key: &str) -> std::io::Result<Option<Vec<u8>>> {
        unimplemented!()
    }
}
