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
    ) -> Result<(), std::io::Error> {
        let cmd = format!("set {} {} {} {}\r\n", key, flags, expiration, value.len());
        self.stream.write_all(cmd.as_bytes())?;
        self.stream.write_all(value)?;
        self.stream.write_all(b"\r\n")?;
        self.stream.flush()?;

        let mut response = [0; 1024];
        let _num_bytes = self.stream.read(&mut response)?;

        // println!("Response: {:?}", std::str::from_utf8(&response[..num_bytes]));

        Ok(())
    }

    pub fn get(&mut self, key: &str) -> io::Result<Option<String>> {
        write!(self.stream, "get {}\r\n", key)?;
        self.stream.flush()?;

        let mut buffer = vec![0u8; 1024];
        let num_bytes = self.stream.read(&mut buffer)?;

        // response is in the format: VALUE k <FLAGS> <EXP>\r\n<value>\r\nEND\r\n
        match String::from_utf8(buffer[..num_bytes].to_vec()) {
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
                // If the conversion fails, return an error
                Err(io::Error::new(io::ErrorKind::InvalidData, error))
            }
        }
    }
}
