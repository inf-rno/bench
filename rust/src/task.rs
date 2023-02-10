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
pub struct TaskResult(String, Duration);

pub trait Task {
    fn run(&mut self) -> TaskResult;
}

#[allow(dead_code)]
pub enum ClientType {
    MEMRS,
    RSMEM,
    LOCAL
}

pub fn task_factory(c: Rc<Config>) -> Box<dyn Task> {
    match &c.client_type {
        ClientType::MEMRS => return Box::new(MemRS::new(c)),
        ClientType::RSMEM => return Box::new(RSMem::new(c)),
        ClientType::LOCAL => return Box::new(LOCAL::new(c)),
    }
}

struct MemRS {
    config: Rc<Config>,
    client: memcached::client::Client,
    rng: SmallRng,
}

impl MemRS {
    fn new(c: Rc<Config>) -> Self {
        MemRS {
            config: c,
            client: memcached::Client::connect(&[("tcp://127.0.0.1:11211", 1)], ProtoType::Binary)
                .unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for MemRS {
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
        RSMem {
            config: c,
            client: memcache::connect("memcache://127.0.0.1:11211?protocol=ascii").unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Task for RSMem {
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

struct LOCAL {
    config: Rc<Config>,
    client: client::Client,
    rng: SmallRng,
}

impl LOCAL {
    fn new(c: Rc<Config>) -> Self {
        LOCAL {
            config: c,
            client: Client::connect("/var/run/memcached/memcached.sock").unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}

// let mut client = Client::connect("/var/run/memcached/memcached.sock").unwrap();
// client.set("k", "val".as_bytes(), 0, 300).unwrap();
// match client.get(k) {
//     Ok(Some(value)) => println!("Value for key '{}': {:?}", k, value),
//     Ok(None) => println!("Key '{}' not found in cache", k),
//     Err(err) => println!("Error while retrieving key '{}': {}", k, err),
// }

impl Task for LOCAL {
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
