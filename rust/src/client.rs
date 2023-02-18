use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::str;

enum Stream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Unix(s) => s.write(buf),
            Stream::Tcp(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Stream::Unix(s) => s.flush(),
            Stream::Tcp(s) => s.flush(),
        }
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Stream::Unix(s) => s.read(buf),
            Stream::Tcp(s) => s.read(buf),
        }
    }
}

pub struct Client {
    stream: Stream,
}

impl Client {
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = if addr.contains(".sock") {
            Stream::Unix(UnixStream::connect(addr)?)
        } else {
            Stream::Tcp(TcpStream::connect(addr)?)
        };
        Ok(Self { stream })
    }

    pub fn set(
        &mut self,
        key: &str,
        value: &[u8],
        flags: u32,
        expiration: u32,
    ) -> Result<bool, std::io::Error> {
        let cmd = format!("set {} {} {} {}\r\n", key, flags, expiration, value.len());
        self.stream.write_all(cmd.as_bytes())?;
        self.stream.write_all(value)?;
        self.stream.write_all(b"\r\n")?;
        self.stream.flush()?;

        let mut buf = [0; 128];
        let n = self.stream.read(&mut buf)?;
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

    pub fn get(&mut self, key: &str) -> io::Result<Option<Vec<u8>>> {
        write!(self.stream, "get {key}\r\n")?;
        self.stream.flush()?;

        let mut response = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = self.stream.read(&mut buf)?;
            response.extend_from_slice(&buf[..n]);
            if response.ends_with(b"\r\nEND\r\n") {
                break;
            }
        }

        let response_lines: Vec<&[u8]> = response.split(|b| *b == b'\r' || *b == b'\n').collect();
        if response_lines.len() < 3 {
            //better error handling here
            return Ok(None);
        }

        let header = std::str::from_utf8(response_lines[0]).unwrap();
        if header == "END" {
            //better error handling here
            return Ok(None);
        }

        let value = &response_lines[1..response_lines.len() - 2];
        Ok(Some(value.concat()))
    }
}
