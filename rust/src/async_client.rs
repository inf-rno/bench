use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncWrite, AsyncRead, ReadBuf, AsyncWriteExt, AsyncReadExt, ErrorKind};
use tokio::net::{TcpStream, UnixStream};

enum AsyncStream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

impl AsyncWrite for AsyncStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            AsyncStream::Unix(s) => Pin::new(s).poll_write(cx, buf),
            AsyncStream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            AsyncStream::Unix(s) => Pin::new(s).poll_flush(cx),
            AsyncStream::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            AsyncStream::Unix(s) => Pin::new(s).poll_shutdown(cx),
            AsyncStream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl AsyncRead for AsyncStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            AsyncStream::Unix(s) => Pin::new(s).poll_read(cx, buf),
            AsyncStream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

pub struct AsyncClient {
    stream: Pin<Box<AsyncStream>>,
}

impl AsyncClient {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let stream = if addr.contains(".sock") {
            AsyncStream::Unix(UnixStream::connect(addr).await?)
        } else {
            AsyncStream::Tcp(TcpStream::connect(addr).await?)
        };
        Ok(Self { stream: Box::pin(stream) })
    }

    pub async fn set(
        &mut self,
        key: &str,
        value: &[u8],
        flags: u32,
        expiration: u32
    ) -> Result<bool, std::io::Error> {
        let cmd = format!("set {} {} {} {}\r\n", key, flags, expiration, value.len());
        self.stream.write_all(cmd.as_bytes()).await?;
        self.stream.write_all(value).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await?;

        let mut buf = [0; 128];
        let n = self.stream.read(&mut buf).await?;
        let response = std::str::from_utf8(&buf[..n]).unwrap();

        if response == "STORED\r\n" {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("memcached error: {}", response),
            ))
        }
    }

    pub async fn get(&mut self, key: &str) -> io::Result<Option<Vec<u8>>> {
        let cmd = format!("get {key}\r\n");
        self.stream.write_all(cmd.as_bytes()).await?;
        self.stream.flush().await?;

        let mut response = Vec::new();
        let mut buf = [0u8; 32];
        loop {
            let n = self.stream.read(&mut buf).await?;
            response.extend_from_slice(&buf[..n]);
            if response.ends_with(b"\r\nEND\r\n") {
                break;
            }
        }

        let response_lines: Vec<&[u8]> = response.split(|b| *b == b'\r' || *b == b'\n').collect();
        if response_lines.len() < 3 {
            return Err(ErrorKind::InvalidData.into());
        }

        let header = std::str::from_utf8(response_lines[0]).unwrap();
        if header == "END" {
            return Err(ErrorKind::InvalidData.into());
        }

        let value = &response_lines[1..response_lines.len() - 2];
        dbg!(value.concat());
        Ok(Some(value.concat()))
    }
}
