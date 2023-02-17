use crate::hdr::*;
use crate::task::*;
use crate::Config;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

pub struct Bench {
    config: Rc<Config>,
    times_map: HashMap<String, Vec<Duration>>,
}

impl Bench {
    pub fn new(c: Rc<Config>) -> Self {
        Bench {
            config: c,
            times_map: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut t = task_factory(self.config.clone());
        t.init();
        for _ in 0..self.config.requests {
            let r = t.run();
            // print!("{r:?},");
            let t = self.times_map.entry(r.0.clone()).or_default();
            t.push(r.1);
        }
    }

    pub fn result(&self) -> HashMap<&String, Rc<Result>> {
        self.times_map
            .iter()
            .map(|(op, times)| {
                (
                    op,
                    Rc::new(Result::new(times, self.config.data_string.len())),
                )
            })
            .collect()
    }
}

#[derive(Default, Clone)]
pub struct Result {
    pub ops: usize,
    pub total: Duration,
    pub opsps: f64,
    pub p99: Duration,
    pub kbps: f64,
    pub gbps: f64,
    pub histogram: HDR,
}

impl Result {
    fn new(times: &Vec<Duration>, d: usize) -> Self {
        let mut r = Result::default();

        r.ops = times.len();
        for t in times {
            r.total += *t;
            r.histogram += t.as_micros() as u64;
        }
        r.opsps = r.ops as f64 / r.total.as_secs_f64();
        r.kbps = d as f64 * r.opsps / 1000 as f64;
        r.gbps = (d * 8) as f64 * r.opsps / 1000000000 as f64;
        r.p99 = Duration::from_micros(r.histogram.p99());

        r
    }
}

impl fmt::Display for Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ops {}; total {:?}; ops/sec {:.2}; p99: {:?}, KBps {:.2}; Gbps {:.2}",
            self.ops, self.total, self.opsps, self.p99, self.kbps, self.gbps
        )
    }
}
