use std::os::unix::net::UnixStream;
use std::io::{self, Write, Read};
use std::str;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub fn connect(socket_addr: &str) -> std::io::Result<Self> {
        let stream = UnixStream::connect(socket_addr)?;
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

    pub fn get(&mut self, key: &str) -> io::Result<Option<String>> {
        write!(
            self.stream,
            "get {}\r\n",
            key
        )?;
        self.stream.flush()?;

        let mut buf = vec![0u8; 1024];
        let n = self.stream.read(&mut buf)?;

        // response is in the format: VALUE k <FLAGS> <EXP>\r\n<value>\r\nEND\r\n
        match String::from_utf8(buf[..n].to_vec()) {
            Ok(response) => {
                let response_lines: Vec<&str> = response.split("\r\n").collect();
                if response_lines.len() < 2 {
                    return Ok(None);
                }

                let header = response_lines[0];
                if header == "END" {
                    return Ok(None);
                }

                let value = response_lines[1];
                Ok(Some(value.to_string()))
            },
            Err(error) => {
                Err(io::Error::new(io::ErrorKind::InvalidData, error))
            }
        }
    }
}
