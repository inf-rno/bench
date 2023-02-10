use crate::task::*;
use std::rc::Rc;

mod task;

pub struct Config {
    runs: i64,
    requests: i64,
    data: i64,
    data_string: String,
    data_bytes: Vec<u8>,
    ratio: f64,
    key: String,
    client_type: ClientType,
}

struct Bench {
    config: Rc<Config>,
}

impl Bench {
    fn new(c: Rc<Config>) -> Self {
        Bench { config: c }
    }

    fn run(&self) {
        let mut t = task_factory(self.config.clone());
        let res = (0..self.config.requests)
            .map(|_| t.run())
            .collect::<Vec<_>>();
        println!("{res:?}");
    }
}

fn main() {
    let mut c = Config {
        runs: 1,
        requests: 10,
        data: 32,
        data_string: String::new(),
        data_bytes: Vec::new(),
        ratio: 0.1,
        key: String::from("lol"),
        client_type: ClientType::LOCAL,
    };
    c.data_string = std::iter::repeat("x")
        .take(c.data as usize)
        .collect::<Vec<_>>()
        .join("");
    c.data_bytes = c.data_string.bytes().collect::<Vec<_>>();
    let c = Rc::new(c);

    (0..c.runs).for_each(|_| {
        Bench::new(c.clone()).run();
    });
}
