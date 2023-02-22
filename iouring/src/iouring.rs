use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_uring::net::{TcpStream, UnixStream};
use std::io::{ErrorKind};

enum IOuringStream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

pub struct IOuringClient {
    stream: Pin<Box<IOuringStream>>,
}

impl IOuringClient {
    pub async fn connect(addr: &str) -> io::Result<Self> {
        let stream = if addr.contains(".sock") {
            IOuringStream::Unix(UnixStream::connect(addr).await?)
        } else {
            IOuringStream::Tcp(TcpStream::connect(addr).await?)
        };
        Ok(Self { stream: Box::pin(stream) })
    }

    // pub async fn set(
    //     &mut self,
    //     key: &str,
    //     value: &[u8],
    //     flags: u32,
    //     expiration: u32
    // ) -> Result<bool, std::io::Error> {
    //     let cmd = format!("set {} {} {} {}\r\n", key, flags, expiration, value.len());
    //     self.stream.write_all(cmd.as_bytes()).await?;
    //     self.stream.write_all(value).await?;
    //     self.stream.write_all(b"\r\n").await?;
    //     self.stream.flush().await?;

    //     let mut buf = [0; 32];
    //     let n = self.stream.read(&mut buf).await?;
    //     let response = std::str::from_utf8(&buf[..n]).unwrap();
    //     if response == "STORED\r\n" {
    //         Ok(true)
    //     } else {
    //         Err(std::io::Error::new(
    //             std::io::ErrorKind::Other,
    //             format!("memcached error: {}", response),
    //         ))
    //     }
    // }

    // pub async fn get(&mut self, key: &str) -> io::Result<Option<Vec<u8>>> {
    //     let cmd = format!("get {key}\r\n");
    //     self.stream.write_all(cmd.as_bytes()).await?;
    //     self.stream.flush().await?;

    //     let mut response = Vec::new();
    //     let mut buf = [0u8; 32];
    //     loop {
    //         let n = self.stream.read(&mut buf).await?;
    //         response.extend_from_slice(&buf[..n]);
    //         if response.ends_with(b"\r\nEND\r\n") {
    //             break;
    //         }
    //     }

    //     let response_lines: Vec<&[u8]> = response.split(|b| *b == b'\r' || *b == b'\n').collect();
    //     if response_lines.len() < 3 {
    //         return Err(ErrorKind::InvalidData.into());
    //     }

    //     let header = std::str::from_utf8(response_lines[0]).unwrap();
    //     if header == "END" {
    //         return Err(ErrorKind::InvalidData.into());
    //     }

    //     Ok(Some(response_lines[2].to_vec()))
    // }
}
