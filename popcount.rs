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

struct Driver {
    name: &'static str,
    f: &'static Fn (u32) -> u32,
    blockf: &'static Fn (u32, &[u32; BLOCKSIZE]) -> u32,
    divisor: u32
}

macro_rules! driver {
    ($drive:ident, $popcount:ident, $entry:ident, $name:expr, $div:expr) => {
        fn $drive(n: u32, randoms: &[u32;BLOCKSIZE]) -> u32 {
            let mut result = 0;
            for _ in 0..n {
                for i in randoms.iter() {
                    result += $popcount(i ^ result)
                }
            }
            result
        }
        const $entry: Driver = Driver {
            name: $name,
            f: &$popcount,
            blockf: &$drive,
            divisor: $div
        };
    }
}

// One bit at a time, with early termination.
#[inline(always)]
fn popcount_naive(mut n: u32) -> u32 {
    let mut c = 0;
    while n > 0 {
	c += n & 1;
	n >>= 1
    }
    c
}
driver!(drive_naive, popcount_naive, DRIVER_NAIVE, "popcount_naive", 16);

/* bit-parallelism */
#[inline(always)]
fn popcount_8(mut n: u32) -> u32 {
    let m = 0x01010101u32;
    let mut c = n & m;
    for _ in 0..7 {
	n >>= 1;
	c += n & m
    }
    c += c >> 8;
    c += c >> 16;
    c & 0x3f
}
driver!(drive_8, popcount_8, DRIVER_8, "popcount_8", 4);

/* more bit-parallelism */
#[inline(always)]
fn popcount_6(mut n: u32) -> u32 {
    let m = 0x41041041;
    let mut c = n & m;
    for _ in 0..5 {
	n >>= 1;
	c += n & m
    }
    c += c >> 6;
    c += c >> 12;
    c += c >> 24;
    c & 0x3f
}
driver!(drive_6, popcount_6, DRIVER_6, "popcount_6", 4);

const DRIVERS: &[Driver] = &[
    DRIVER_NAIVE,
    DRIVER_8,
    DRIVER_6 ];

fn test_drivers() -> Vec<&'static Driver> {
    let testcases: &[(u32, u32)] = &[
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
    let mut drivers = Vec::new();
    for driver in DRIVERS.iter() {
        let mut ok = true;
        for (nt, &(input, expected)) in testcases.iter().enumerate() {
            let actual = (*driver.f)(input);
            if actual != expected {
                println!("{} failed case {}: {:#x} -> {} != {}: abandoning",
                         driver.name, nt, input, actual, expected);
                ok = false;
                break
            }
        }
        if ok {
            drivers.push(driver)
        }
    }
    drivers
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let n: u32 = args[1].parse().expect("invalid count");
    let mut rng = rand::thread_rng();
    let mut randoms = [0u32; BLOCKSIZE];
    for i in randoms.iter_mut() {
        *i = rng.gen()
    }
    let drivers = test_drivers();
    for driver in drivers {
        let preheat = PREHEAT_BASE / driver.divisor;
        let mut csum = (*driver.blockf)(preheat, &randoms);
        let now = time::Instant::now();
        csum += (*driver.blockf)(n / driver.divisor, &randoms);
        let runtime = now.elapsed();
        println!("{}: {} {}", driver.name, duration_secs(runtime), csum);
    }
}
