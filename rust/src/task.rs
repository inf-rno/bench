use std::ops::Deref;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

use memcached;
use memcached::proto::{Operation, ProtoType};

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
            self.client
                .set(
                    self.config.key.as_bytes(),
                    self.config.data_bytes.deref(),
                    0,
                    0,
                )
                .unwrap();
        }
    }
    fn run(&mut self) -> TaskResult {
        let r: f64 = self.rng.gen();
        let start = Instant::now();
        let op = if r < self.config.ratio {
            self.client
                .set(
                    self.config.key.as_bytes(),
                    self.config.data_bytes.deref(),
                    0,
                    0,
                )
                .unwrap();
            "SET"
        } else {
            self.client.get(self.config.key.as_bytes()).unwrap();
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
        let mut addr = format!("memcache://{}:{}?protocol=binary", c.server, c.port);
        if !c.socket.is_empty() {
            addr = format!("memcache://{}?protocol=binary", c.socket)
        }
        RSMem {
            config: c,
            client: memcache::connect(addr).unwrap(),
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
