#![feature(async_fn_in_trait)]

use crate::bench::*;
use crate::task::*;
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
use std::{thread, time};

mod bench;
mod hdr;
mod task;
mod basic;

#[derive(Parser, Debug)]
pub struct Config {
    /// Number of full test iterations
    #[arg(short = 'x', long, default_value_t = 3)]
    runs: i64,
    /// Number of requests per thread
    #[arg(short = 'n', long, default_value_t = 10000)]
    requests: i64,
    /// Size of the data payload in bytes, specify 0 to not perform any writes
    #[arg(short = 'd', long, default_value_t = 100000)]
    data: i64,
    #[arg(skip)]
    data_string: String,
    #[arg(skip)]
    data_bytes: Vec<u8>,
    /// Ratio of ops (eg. 0.1 == 10% sets && 90% gets)
    #[arg(short = 'r', long, default_value_t = 0.1)]
    ratio: f64,
    /// Key/prefix to use
    #[arg(short = 'k', long, default_value = "lol")]
    key: String,
    /// Client to use
    #[arg(short = 't', long, value_enum, default_value_t = ClientType::Sync(SyncClientType::MEMRS))]
    client_type: ClientType,
    /// output prefix for hdrHistogram files
    #[arg(short = 'o', long, default_value = "")]
    out: String,
    /// Server address
    #[arg(short = 's', long, default_value = "127.0.0.1")]
    server: String,
    /// Server Port
    #[arg(short = 'p', long, default_value_t = 11211)]
    port: i64,
    /// UNIX domain socket name
    #[arg(short = 'S', long, default_value = "")]
    socket: String,
}

fn main() -> std::io::Result<()> {
    let mut c = Config::parse();
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
                    if r.opsps < e.opsps {
                        *e = r.clone();
                    }
                })
                .or_insert(r.clone());

            max_map
                .entry(op.clone())
                .and_modify(|e| {
                    if r.opsps > e.opsps {
                        *e = r.clone();
                    }
                })
                .or_insert(r.clone());
        }
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("~~~~~~~~~~~~~~~~~~~RESULTS~~~~~~~~~~~~~~~~");
    println!("\nWORST RESULT:");
    for (op, r) in min_map {
        println!("OP: {op} \n {r}");
    }

    println!("\nBEST RESULT:");
    for (op, r) in max_map {
        println!("OP: {op} \n {r}");
        if !c.out.is_empty() {
            let file = File::create(c.out.clone() + "_rs_" + &op)?;
            r.histogram.percentiles(file)?;
            // r.histogram.serialize(file);
        }
    }
    Ok(())
}
