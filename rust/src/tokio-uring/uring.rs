// use tokio_uring::net::UnixStream;
// use tokio::io::{Error, ErrorKind};

// pub struct UringClient {
//     stream: UnixStream,
// }

// impl UringClient {
//     pub async fn connect(addr: &str) -> Result<Self> {
//         Ok(Self { stream: UnixStream::connect(addr).await? })
//     }

//     pub async fn set(
//         &mut self,
//         key: &str,
//         value: &[u8],
//         flags: u32,
//         expiration: u32,
//     ) -> Result<bool, Error> {
//         let mut cmd: Vec<u8> = format!("set {} {} {} {}\r\n", key, flags, expiration, value.len()).into_bytes();
//         cmd.extend_from_slice(value);
//         cmd.extend_from_slice(b"\r\n");
//         let (res, buf) = self.stream.write(cmd).await;
//         res.unwrap();

//         let (res, cmd) = self.stream.read(buf).await;
//         let response = std::str::from_utf8(&cmd[..res.unwrap()]).unwrap();
//         if response == "STORED\r\n" {
//             Ok(true)
//         } else {
//             Err(Error::new(
//                 ErrorKind::Other,
//                 format!("memcached error: {}", response),
//             ))
//         }
//     }

//     pub async fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
//         let cmd: Vec<u8> = format!("get {key}\r\n").into_bytes();
//         let (res, _buf) = self.stream.write(cmd).await;
//         res.unwrap();

//         let mut response = Vec::new();
//         let mut buf = vec![0u8; 1024];
//         loop {
//             let (result, nbuf) = self.stream.read(buf).await;
//             buf = nbuf;
//             let read = result.unwrap();
//             if read == 0 {
//                 break;
//             }

//             response.extend_from_slice(&buf[..read]);
//             if response.ends_with(b"\r\nEND\r\n") {
//                 break;
//             }
//         }

//         let response_lines: Vec<&[u8]> = response.split(|b| *b == b'\r' || *b == b'\n').collect();
//         if response_lines.len() < 3 {
//             return Err(ErrorKind::InvalidData.into());
//         }

//         let header = std::str::from_utf8(response_lines[0]).unwrap();
//         if header == "END" {
//             return Err(ErrorKind::InvalidData.into());
//         }
//         Ok(Some(response_lines[2].to_vec()))
//     }
// }
