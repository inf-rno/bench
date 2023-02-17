use crate::bench::*;
use crate::task::*;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
use std::{thread, time};

mod bench;
mod hdr;
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
    out_dir: String,
}

fn main() -> std::io::Result<()> {
    let mut c = Config {
        runs: 3,
        requests: 10000,
        data: 1000000,
        data_string: String::new(),
        data_bytes: Vec::new(),
        ratio: 0.1,
        key: String::from("lol"),
        client_type: ClientType::MEMRS,
        out_dir: String::from("../results"),
    };
    c.data_string = std::iter::repeat("x")
        .take(c.data as usize)
        .collect::<Vec<_>>()
        .join("");
    c.data_bytes = c.data_string.bytes().collect::<Vec<_>>();
    let c = Rc::new(c);

    let (mut min_map, mut max_map) = (
        HashMap::<String, Rc<Result>>::new(),
        HashMap::<String, Rc<Result>>::new(),
    );

    for i in 0..c.runs {
        println!("RUN: {i}");
        let mut b = Bench::new(c.clone());
        b.run();
        for (op, r) in b.result() {
            min_map
                .entry(op.clone())
                .and_modify(|e| {
                    if r.p99 < e.p99 {
                        *e = r.clone();
                    }
                })
                .or_insert(r.clone());

            max_map
                .entry(op.clone())
                .and_modify(|e| {
                    if r.p99 > e.p99 {
                        *e = r.clone();
                    }
                })
                .or_insert(r.clone());
        }
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("~~~~~~~~~~~~~~~~~~~RESULTS~~~~~~~~~~~~~~~~");
    println!("\nWORST RESULT:");
    for (op, r) in max_map {
        println!("OP: {op} \n {r}");
    }

    println!("\nBEST RESULT:");
    for (op, r) in min_map {
        println!("OP: {op} \n {r}");
        if !c.out_dir.is_empty() {
            let file = File::create(c.out_dir.clone() + "/rs_" + &op)?;
            r.histogram.percentiles(file)?;
        }
    }
    Ok(())
}
