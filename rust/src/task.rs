use std::ops::Deref;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

use memcached;
use memcached::proto::{Operation, ProtoType};

use memcache;

use client::Client;
use async_client::AsyncClient;
use uring::UringClient;

use crate::*;

#[derive(Debug)]
pub struct TaskResult(pub String, pub Duration);

pub trait Task {
    fn init(&mut self);
    fn run(&mut self) -> TaskResult;
}

pub trait AsyncTask {
    async fn init(&mut self);
    async fn run(&mut self) -> TaskResult;
}

#[allow(dead_code)]
#[derive(clap::Parser, Debug, Clone)]
pub enum ClientType {
    Sync(SyncClientType),
    Async(AsyncClientType),
}

#[allow(dead_code)]
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum SyncClientType {
    MEMRS,
    RSMEM,
    LOCAL,
    ASYNC,
    URING,
}

#[allow(dead_code)]
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum AsyncClientType {
    ASYNC,
    URING,
}

pub fn task_factory(c: Rc<Config>) -> Box<dyn Task> {
    match &c.client_type {
        ClientType::Sync(t) => return sync_task_factory(c, t),
        ClientType::Async(t) => return async_task_factory(c, t),
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
        let mut addr = format!("memcache://{}:{}?protocol=ascii", c.server, c.port);
        if !c.socket.is_empty() {
            addr = format!("memcache://{}?protocol=ascii", c.socket)
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

struct Local {
    config: Rc<Config>,
    client: Client,
    rng: SmallRng,
}

impl Local {
    fn new(c: Rc<Config>) -> Self {
        dbg!("LOCAL");
        let mut addr = format!("{}:{}", c.server, c.port);
        if !c.socket.is_empty() {
            addr = c.socket.clone()
        }
        Local {
            config: c,
            client: Client::connect(&addr).unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for Local {
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

struct Async {
    config: Rc<Config>,
    client: AsyncClient,
    rng: SmallRng,
}

impl Async {
    async fn new(c: Rc<Config>) -> Self {
        dbg!("ASYNC");
        let mut addr = format!("{}:{}", c.server, c.port);
        if !c.socket.is_empty() {
            addr = c.socket.clone()
        }
        Async {
            config: c,
            client: AsyncClient::connect(&addr).await.unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for Async {
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
