// Copyright (c) 2018 Bart Massey
// 
// [This program is licensed under the GPL version 3 or later.]
// Please see the file COPYING in the source
// distribution of this software for license terms.

extern crate rand;
#[macro_use]
extern crate lazy_static;

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

// bit-parallelism
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

// more bit-parallelism
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

// HAKMEM 169
#[inline(always)]
fn popcount_hakmem(n: u32) -> u32 {
    let y = (n >> 1) & 0o33333333333;
    let z = n - y - ((y >>1) & 0o33333333333);
    ((z + (z >> 3)) & 0o30707070707) % 63
}
driver!(drive_hakmem, popcount_hakmem, DRIVER_HAKMEM, "popcount_hakmem", 4);

// Joe Keane, sci.math.num-analysis, 9 July 1995,
// as cited by an addendum to Hacker's Delight.
// http://www.hackersdelight.org/divcMore.pdf
#[inline(always)]
fn remu63(n: u32) -> u32 {
    let t = (((n >> 12) + n) >> 10) + (n << 2);
    let u = ((t >> 6) + t + 3) & 0xff;
    (u - (u >> 6)) >> 2
}

// HAKMEM 169 with Keane modulus
#[inline(always)]
fn popcount_keane(n: u32) -> u32 {
    let y = (n >> 1) & 0o33333333333;
    let z = n - y - ((y >>1) & 0o33333333333);
    remu63((z + (z >> 3)) & 0o30707070707)
}
driver!(drive_keane, popcount_keane, DRIVER_KEANE, "popcount_keane", 4);

// Divide-and-conquer with a ternary stage to reduce masking
#[inline(always)]
fn popcount_3(mut n: u32) -> u32 {
    let m1 = 0x55555555;
    let m2 = 0xc30c30c3;
    n -= (n >> 1) & m1;
    n = (n & m2) + ((n >> 2) & m2) + ((n >> 4) & m2); 
    n += n >> 6;
    (n + (n >> 12) + (n >> 24)) & 0x3f
}
driver!(drive_3, popcount_3, DRIVER_3, "popcount_3", 4);

// Divide-and-conquer with a quaternary stage to reduce masking
// and provide mostly power-of-two shifts
#[inline(always)]
fn popcount_4(mut n: u32) -> u32 {
    let m1 = 0x55555555;
    let m2 = 0x03030303;
    n -= (n >> 1) & m1;
    n = (n & m2) + ((n >> 2) & m2) + ((n >> 4) & m2) + ((n >> 6) & m2);
    n += n >> 8;
    (n + (n >> 16)) & 0x3f
}
driver!(drive_4, popcount_4, DRIVER_4, "popcount_4", 4);

// Classic binary divide-and-conquer popcount.
// This is popcount_2() from
// http://en.wikipedia.org/wiki/Hamming_weight
#[inline(always)]
fn popcount_2(mut n: u32) -> u32 {
    let m1 = 0x55555555;
    let m2 = 0x33333333;
    let m4 = 0x0f0f0f0f;
    n -= (n >> 1) & m1;
    n = (n & m2) + ((n >> 2) & m2);
    n = (n + (n >> 4)) & m4;
    n += n >>  8;
    return (n + (n >> 16)) & 0x3f;
}
driver!(drive_2, popcount_2, DRIVER_2, "popcount_2", 4);

// Popcount using multiply.
// This is popcount_3() from
// http://en.wikipedia.org/wiki/Hamming_weight
#[inline(always)]
fn popcount_mult(mut n: u32) -> u32 {
    let m1 = 0x55555555;
    let m2 = 0x33333333;
    let m4 = 0x0f0f0f0f;
    let h01 = 0x01010101;
    n -= (n >> 1) & m1;   // put count of each 2 bits into those 2 bits
    n = (n & m2) + ((n >> 2) & m2);   // put count of each 4 bits in
    n = (n + (n >> 4)) & m4;  // put count of each 8 bits in
    (n * h01) >> 24  // returns left 8 bits of n + (n<<8) + ...
}
driver!(drive_mult, popcount_mult, DRIVER_MULT, "popcount_mult", 4);

// 8-bit popcount table, filled at first use.
lazy_static! {
    static ref POPCOUNT_TABLE_8: Vec<u32> =
        (0..0x100).map(|i| popcount_naive(i)).collect();
}

// Table-driven popcount, with 8-bit tables
#[inline(always)]
fn popcount_tabular_8(n: u32) -> u32 {
    POPCOUNT_TABLE_8[n as u8 as usize] +
    POPCOUNT_TABLE_8[(n >> 8) as u8 as usize] +
    POPCOUNT_TABLE_8[(n >> 16) as u8 as usize] +
    POPCOUNT_TABLE_8[(n >> 24) as usize]
}
driver!(drive_tabular_8, popcount_tabular_8, DRIVER_TABULAR_8,
        "popcount_tabular_8", 4);

// 16-bit popcount table, filled at first use.
lazy_static! {
    static ref POPCOUNT_TABLE_16: Vec<u32> =
        (0..0x10000).map(|i| popcount_naive(i)).collect();
}

// Table-driven popcount, with 16-bit tables
#[inline(always)]
fn popcount_tabular_16(n: u32) -> u32 {
    POPCOUNT_TABLE_16[n as u16 as usize] +
    POPCOUNT_TABLE_16[(n >> 16) as usize]
}
driver!(drive_tabular_16, popcount_tabular_16, DRIVER_TABULAR_16,
        "popcount_tabular_16", 4);

const DRIVERS: &[Driver] = &[
    DRIVER_NAIVE,
    DRIVER_8,
    DRIVER_6,
    DRIVER_HAKMEM,
    DRIVER_KEANE,
    DRIVER_3,
    DRIVER_4,
    DRIVER_2,
    DRIVER_MULT,
    DRIVER_TABULAR_8,
    DRIVER_TABULAR_16 ];

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

fn total_time(d: time::Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 / 1.0e9
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let n: u32 = args[1].parse().expect("invalid count");
    let mut rng = rand::thread_rng();
    let mut randoms = [0u32; BLOCKSIZE];
    let mut csum = 0;
    for i in randoms.iter_mut() {
        *i = rng.gen()
    }
    let drivers = test_drivers();
    for driver in drivers {
        let preheat = PREHEAT_BASE / driver.divisor;
        csum += (*driver.blockf)(preheat, &randoms);
        let now = time::Instant::now();
        let nblocks = n / driver.divisor;
        csum += (*driver.blockf)(nblocks, &randoms);
        let runtime = total_time(now.elapsed());
        let size = nblocks as f64 * BLOCKSIZE as f64;
        println!("{}: {:e} iters in {:.0} msecs for {:.2} nsecs/iter",
                 driver.name,
                 size,
                 runtime * 1.0e3,
                 (runtime / size) * 1.0e9);
    }
    println!("{}", csum);
}
