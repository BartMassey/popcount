// Copyright (c) 2018 Bart Massey
// 
// [This program is licensed under the GPL version 3 or later.]
// Please see the file COPYING in the source
// distribution of this software for license terms.

extern crate rand;

use rand::Rng;
use std::env;
use std::time;

const BLOCKSIZE: usize = 1000;
const PREHEAT_BASE: u32 = 50000;

macro_rules! driver {
    ($drive:ident, $popcount:ident) => {
        fn $drive(n: u32, randoms: &[u32;BLOCKSIZE]) -> u32 {
            let mut result = 0;
            for _ in 0..n {
                for i in randoms.iter() {
                    result += $popcount(i ^ result)
                }
            }
            result
        }
    }
}

fn popcount_naive(mut n: u32) -> u32 {
    let mut c = 0;
    while n > 0 {
	c += n & 1;
	n >>= 1;
    }
    c
}
driver!(drive_naive, popcount_naive);

struct Driver {
    name: &'static str,
    f: Box<Fn (u32) -> u32>,
    blockf: Option<Box<Fn (u32, &[u32; BLOCKSIZE]) -> u32>>,
    divisor: u32
}

fn list_drivers() -> Vec<Driver> {
    vec![
        Driver {
            name: "popcount_naive",
            f: Box::new(popcount_naive),
            blockf: Some(Box::new(drive_naive)),
            divisor: 16
        }]
}

fn duration_secs(d: time::Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 / 1.0e9
}

fn test_drivers(drivers: &mut Vec<Driver>) {
    let testcases: Vec<(u32, u32)> = vec![
        (0x00000080, 1),
        (0x000000f0, 4),
        (0x00008000, 1),
        (0x0000f000, 4),
        (0x00800000, 1),
        (0x00f00000, 4),
        (0x80000000, 1),
        (0xf0000000, 4),
        (0xff000000, 8),
        (0x000000ff, 8),
        (0x01fe0000, 8),
        (0xea9031e8, 14),
        (0x2e8eb2b2, 16),
        (0x9b8be5b7, 20),
        (!0, 32),
        (0, 0) ];
    for driver in drivers {
        for (nt, &(input, expected)) in testcases.iter().enumerate() {
            let actual = (*driver.f)(input);
            if actual != expected {
                println!("{} failed case {}: {:#x} -> {} != {}: abandoning",
                         driver.name, nt, input, actual, expected);
                driver.blockf = None;
                return
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let n: u32 = args[1].parse().expect("invalid count");
    let mut rng = rand::thread_rng();
    let mut randoms = [0u32; BLOCKSIZE];
    for i in randoms.iter_mut() {
        *i = rng.gen()
    }
    let mut drivers = list_drivers();
    test_drivers(&mut drivers);
    for driver in drivers {
        if let Some(blockf) = driver.blockf {
            let mut csum = (*blockf)(PREHEAT_BASE / driver.divisor, &randoms);
            let now = time::Instant::now();
            csum += (*blockf)(n / driver.divisor, &randoms);
            let runtime = now.elapsed();
            println!("{}: {} {}", driver.name, duration_secs(runtime), csum);
        }
    }
}
