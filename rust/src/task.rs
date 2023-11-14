use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

use memcached;
use memcached::proto::{MultiOperation, Operation, ProtoType};

use memcache;

use client::Client;

use crate::*;

#[derive(Debug)]
pub struct TaskResult(pub String, pub Duration);

pub trait Task {
    fn init(&mut self);
    fn run(&mut self) -> TaskResult;
}

#[allow(dead_code)]
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ClientType {
    MEMRS,
    RSMEM,
    BASIC,
}

pub fn task_factory(c: Rc<Config>) -> Box<dyn Task> {
    match &c.client_type {
        ClientType::MEMRS => return Box::new(MemRS::new(c)),
        ClientType::RSMEM => return Box::new(RSMem::new(c)),
        ClientType::BASIC => return Box::new(Basic::new(c)),
    }
}

struct MemRS {
    config: Rc<Config>,
    client: memcached::client::Client,
    rng: SmallRng,
}

impl MemRS {
    fn new(c: Rc<Config>) -> Self {
        dbg!("MEMRS");
        let mut addr = format!("tcp://{}:{}", c.server, c.port);
        if !c.socket.is_empty() {
            addr = format!("unix://{}", c.socket)
        }
        MemRS {
            config: c,
            client: memcached::Client::connect(&[(addr, 1)], ProtoType::Binary).unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for MemRS {
    fn init(&mut self) {
        if self.config.data_bytes.len() != 0 {
            if let Some(chunk_size) = self.config.chunk_size {
                let chunks = self.config.data_bytes.chunks(chunk_size as usize);
                self.client
                    .set(
                        self.config.key.as_bytes(),
                        &(chunks.len() as u8).to_be_bytes(),
                        0,
                        0,
                    )
                    .unwrap();
                let keys: Vec<_> = (0..chunks.len())
                    .map(|i| {
                        format!("{}.{}", self.config.key, i)
                            .into_boxed_str()
                            .into_boxed_bytes()
                    })
                    .collect();
                let kv: BTreeMap<&[u8], (&[u8], u32, u32)> = keys
                    .iter()
                    .map(|k| k.as_ref())
                    .zip(chunks.clone().map(|v| (v, 0, 0)))
                    .collect();
                self.client.set_multi(kv).unwrap();

                let v = self.client.get(self.config.key.as_bytes()).unwrap();
                assert_eq!(u8::from_be_bytes([(*v.0)[0]]), chunks.len() as u8);
                let v = self
                    .client
                    .get_multi(&keys.iter().map(|k| k.as_ref()).collect::<Vec<_>>())
                    .unwrap();
                assert_eq!(
                    keys.iter().map(|k| v.get(k.as_ref())).fold(
                        Vec::new(),
                        |mut acc: Vec<u8>, v| {
                            match v {
                                Some(v) => {
                                    acc.extend(v.0.iter());
                                }
                                None => (),
                            }
                            acc
                        },
                    ),
                    chunks.fold(Vec::new(), |mut acc, v| {
                        acc.extend(v.iter());
                        acc
                    })
                );
            } else {
                self.client
                    .set(
                        self.config.key.as_bytes(),
                        self.config.data_bytes.deref(),
                        0,
                        0,
                    )
                    .unwrap();

                let v = self.client.get(self.config.key.as_bytes()).unwrap();
                assert_eq!(v.0, self.config.data_bytes);
            }
        }
    }
    fn run(&mut self) -> TaskResult {
        let r: f64 = self.rng.gen();
        let start = Instant::now();
        let op = if r < self.config.ratio {
            if let Some(chunk_size) = self.config.chunk_size {
                let chunks = self.config.data_bytes.chunks(chunk_size as usize);
                self.client
                    .set(
                        self.config.key.as_bytes(),
                        &(chunks.len() as u8).to_be_bytes(),
                        0,
                        0,
                    )
                    .unwrap();
                let keys: Vec<_> = (0..chunks.len())
                    .map(|i| {
                        format!("{}.{}", self.config.key, i)
                            .into_boxed_str()
                            .into_boxed_bytes()
                    })
                    .collect();
                let kv: BTreeMap<&[u8], (&[u8], u32, u32)> = keys
                    .iter()
                    .map(|k| k.as_ref())
                    .zip(chunks.map(|v| (v, 0, 0)))
                    .collect();
                self.client.set_multi(kv).unwrap();
            } else {
                self.client
                    .set(
                        self.config.key.as_bytes(),
                        self.config.data_bytes.deref(),
                        0,
                        0,
                    )
                    .unwrap();
            }
            "SET"
        } else {
            if let Some(_) = self.config.chunk_size {
                let v = self.client.get(self.config.key.as_bytes()).unwrap();
                let chunk_count = u8::from_be_bytes([(*v.0)[0]]);
                let keys: Vec<_> = (0..chunk_count)
                    .map(|i| {
                        format!("{}.{}", self.config.key, i)
                            .into_boxed_str()
                            .into_boxed_bytes()
                    })
                    .collect();
                let v = self
                    .client
                    .get_multi(&keys.iter().map(|k| k.as_ref()).collect::<Vec<_>>())
                    .unwrap();
                //merge results even if its not used to test perf properly
                let _ = keys.iter().map(|k| v.get(k.as_ref())).fold(
                    Vec::new(),
                    |mut acc: Vec<u8>, v| {
                        match v {
                            Some(v) => {
                                acc.extend(v.0.iter());
                            }
                            None => (),
                        }
                        acc
                    },
                );
            } else {
                self.client.get(self.config.key.as_bytes()).unwrap();
            }
            "GET"
        };
        TaskResult(op.into(), start.elapsed())
    }
}

struct RSMem {
    config: Rc<Config>,
    client: memcache::Client,
    rng: SmallRng,
}

impl RSMem {
    fn new(c: Rc<Config>) -> Self {
        dbg!("RSMEM");
        let mut addr = format!("memcache+tcp://{}:{}", c.server, c.port);
        if c.udp_port != 0 {
            addr = format!("memcache+udp://{}:{}", c.server, c.udp_port)
        }
        if !c.socket.is_empty() {
            addr = format!("memcache://{}", c.socket)
        }
        addr = format!(
            "{}?protocol=binary&connect_timeout=1&tcp_nodelay=true",
            addr
        );
        let client = memcache::connect(addr).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        client
            .set_write_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        RSMem {
            config: c,
            client: client,
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for RSMem {
    fn init(&mut self) {
        if self.config.data_string.len() != 0 {
            self.client
                .set(&self.config.key, self.config.data_string.deref(), 0)
                .unwrap();

            let v: String = self.client.get(&self.config.key).unwrap().unwrap();
            assert_eq!(v, self.config.data_string);
        }
    }
    fn run(&mut self) -> TaskResult {
        let r: f64 = self.rng.gen();
        let start = Instant::now();
        let op = if r < self.config.ratio {
            self.client
                .set(&self.config.key, self.config.data_string.deref(), 0)
                .unwrap();
            "SET"
        } else {
            self.client.get::<Vec<u8>>(&self.config.key).unwrap();
            "GET"
        };
        TaskResult(op.into(), start.elapsed())
    }
}

struct Basic {
    config: Rc<Config>,
    client: client::Client,
    rng: SmallRng,
}

impl Basic {
    fn new(c: Rc<Config>) -> Self {
        dbg!("Basic");
        let mut addr = format!("{}:{}", c.server, c.port);
        if !c.socket.is_empty() {
            addr = c.socket.clone()
        }
        Basic {
            config: c,
            client: Client::connect(&addr).unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for Basic {
    fn init(&mut self) {
        if self.config.data_string.len() != 0 {
            self.client
                .set(
                    self.config.key.as_str(),
                    self.config.data_bytes.deref(),
                    0,
                    0,
                )
                .unwrap();
            let v: Vec<u8> = self.client.get(self.config.key.as_str()).unwrap().unwrap();
            assert!(v == self.config.data_bytes)
        }
    }
    fn run(&mut self) -> TaskResult {
        let r: f64 = self.rng.gen();
        let start = Instant::now();
        let op = if r < self.config.ratio {
            self.client
                .set(
                    self.config.key.as_str(),
                    self.config.data_bytes.deref(),
                    0,
                    0,
                )
                .unwrap();
            "SET"
        } else {
            self.client.get(self.config.key.as_str()).unwrap();
            "GET"
        };
        TaskResult(op.into(), start.elapsed())
    }
}
