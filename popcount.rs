// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#[macro_use]
extern crate lazy_static;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::asm;
use std::env;
use std::time;

const BLOCKSIZE: usize = 1000;
const PREHEAT_BASE: u32 = 5000;

mod prng {
    /// Boring and probably bad linear congruential
    /// pseudo-random number generator to deterministically
    /// generate a block of "random" bits in a
    /// cross-platform fashion.

    // Pierre L’Ecuyer
    // Tables Of Linear Congruential Generators
    // Of Different Sizes and Good Lattice Structure
    // Mathematics of Computation
    // 68(225) Jan 1999 pp. 249-260
    const M: u64 = 85876534675u64;
    const A: u64 = 116895888786u64;

    pub struct Prng {
        state: u64,
    }

    impl Prng {
        pub fn new() -> Prng {
            Prng {
                state: A.wrapping_mul(0x123456789abcdef0u64 % M) % M,
            }
        }

        pub fn next(&mut self) -> u32 {
            self.state = A.wrapping_mul(self.state) % M;
            (self.state & (!0u32 as u64)) as u32
        }
    }
}

struct Driver {
    name: &'static str,
    f: &'static dyn Fn(u32) -> u32,
    blockf: &'static dyn Fn(u32, &[u32; BLOCKSIZE]) -> u32,
    divisor: u32,
}

macro_rules! driver {
    ($drive:ident, $popcount:ident, $entry:ident, $div:expr) => {
        fn $drive(n: u32, randoms: &[u32; BLOCKSIZE]) -> u32 {
            let mut result = 0u32;
            for _ in 0..n {
                for i in randoms.iter() {
                    result += $popcount(i ^ result)
                }
            }
            result
        }
        const $entry: Driver = Driver {
            name: stringify!($popcount),
            f: &$popcount,
            blockf: &$drive,
            divisor: $div,
        };
    };
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
driver!(drive_naive, popcount_naive, DRIVER_NAIVE, 16);

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
driver!(drive_8, popcount_8, DRIVER_8, 4);

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
driver!(drive_6, popcount_6, DRIVER_6, 4);

// HAKMEM 169
#[inline(always)]
fn popcount_hakmem(n: u32) -> u32 {
    let y = (n >> 1) & 0o33333333333;
    let z = n - y - ((y >> 1) & 0o33333333333);
    ((z + (z >> 3)) & 0o30707070707) % 63
}
driver!(drive_hakmem, popcount_hakmem, DRIVER_HAKMEM, 4);

// Joe Keane, sci.math.num-analysis, 9 July 1995,
// as given in Hacker's Delight (2nd ed) Figure 10-39.
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
    let z = n - y - ((y >> 1) & 0o33333333333);
    remu63((z + (z >> 3)) & 0o30707070707)
}
driver!(drive_keane, popcount_keane, DRIVER_KEANE, 4);

// 64-bit HAKMEM variant by Sean Anderson.
// http://graphics.stanford.edu/~seander/bithacks.html#CountBitsSet64
#[inline(always)]
fn popcount_anderson(n: u32) -> u32 {
    let p = 0x1001001001001u64;
    let m = 0x84210842108421u64;
    let mut c = (((n & 0xfffu32) as u64 * p) & m) % 0x1fu64;
    c += ((((n & 0xfff000u32) >> 12) as u64 * p) & m) % 0x1fu64;
    c += (((n >> 24) as u64 * p) & m) % 0x1fu64;
    c as u32
}
driver!(drive_anderson, popcount_anderson, DRIVER_ANDERSON, 6);

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
driver!(drive_3, popcount_3, DRIVER_3, 4);

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
driver!(drive_4, popcount_4, DRIVER_4, 4);

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
    n += n >> 8;
    (n + (n >> 16)) & 0x3f
}
driver!(drive_2, popcount_2, DRIVER_2, 4);

// Popcount using multiply.
// This is popcount_3() from
// http://en.wikipedia.org/wiki/Hamming_weight
#[inline(always)]
fn popcount_mult(mut n: u32) -> u32 {
    let m1 = 0x55555555;
    let m2 = 0x33333333;
    let m4 = 0x0f0f0f0f;
    let h01 = 0x01010101;
    // put count of each 2 bits into those 2 bits
    n -= (n >> 1) & m1;
    // put count of each 4 bits in
    n = (n & m2) + ((n >> 2) & m2);
    // put count of each 8 bits in
    n = (n + (n >> 4)) & m4;

    let m = std::hint::black_box(n * h01);
    let result = (m >> 24) & 0xff;
    result as u32
}
driver!(drive_mult, popcount_mult, DRIVER_MULT, 4);

// 8-bit popcount table, filled at first use.
lazy_static! {
    static ref POPCOUNT_TABLE_8: Vec<u32> = (0..0x100).map(popcount_naive).collect();
}

// Table-driven popcount, with 8-bit tables
#[inline(always)]
fn popcount_tabular_8(n: u32) -> u32 {
    POPCOUNT_TABLE_8[n as u8 as usize]
        + POPCOUNT_TABLE_8[(n >> 8) as u8 as usize]
        + POPCOUNT_TABLE_8[(n >> 16) as u8 as usize]
        + POPCOUNT_TABLE_8[(n >> 24) as usize]
}
driver!(drive_tabular_8, popcount_tabular_8, DRIVER_TABULAR_8, 4);

// 16-bit popcount table, filled at first use.
lazy_static! {
    static ref POPCOUNT_TABLE_16: Vec<u32> = (0..0x10000).map(popcount_naive).collect();
}

// Table-driven popcount, with 16-bit tables
#[inline(always)]
fn popcount_tabular_16(n: u32) -> u32 {
    POPCOUNT_TABLE_16[n as u16 as usize] + POPCOUNT_TABLE_16[(n >> 16) as usize]
}
driver!(drive_tabular_16, popcount_tabular_16, DRIVER_TABULAR_16, 4);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn has_popcnt_x86() -> bool {
    let eax = 0x01u32;
    let mut ecx: u32;
    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            lateout("ecx") ecx,
            in("eax") eax,
            lateout("eax") _,
            lateout("edx") _,
            options(preserves_flags),
        );
    }
    ((ecx >> 23) & 1) == 1
}

// x86 popcount instruction
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
fn popcount_x86(n: u32) -> u32 {
    let mut result: u32;
    unsafe {
        asm!(
            "popcntl {0:e}, {1:e}",
            in(reg) n,
            lateout(reg) result,
            options(att_syntax),
        );
    }
    result
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
driver!(drive_x86, popcount_x86, DRIVER_X86, 1);

// Rust native: can get a popcnt insn, but only via -march=native
// See https://users.rust-lang.org/t/4923/3 and the Makefile.
#[inline(always)]
fn popcount_rs(n: u32) -> u32 {
    n.count_ones()
}
driver!(drive_rs, popcount_rs, DRIVER_RS, 1);

const DRIVERS: &[Driver] = &[
    DRIVER_NAIVE,
    DRIVER_8,
    DRIVER_6,
    DRIVER_HAKMEM,
    DRIVER_KEANE,
    DRIVER_ANDERSON,
    DRIVER_3,
    DRIVER_4,
    DRIVER_2,
    DRIVER_MULT,
    DRIVER_TABULAR_8,
    DRIVER_TABULAR_16,
    DRIVER_RS,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    DRIVER_X86,
];

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
        (0, 0),
    ];
    let mut drivers = Vec::new();
    for driver in DRIVERS.iter() {
        let mut ok = true;
        for (nt, &(input, expected)) in testcases.iter().enumerate() {
            let actual = (*driver.f)(input);
            if actual != expected {
                println!(
                    "{} failed case {}: {:#x} -> {} != {}: abandoning",
                    driver.name, nt, input, actual, expected
                );
                ok = false;
                break;
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
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    assert!(has_popcnt_x86());
    let args: Vec<String> = env::args().collect();
    let n: u32 = args[1].parse().expect("invalid count");
    let mut rng = prng::Prng::new();
    let mut randoms = [0u32; BLOCKSIZE];
    let mut csum = 0u64;
    for i in randoms.iter_mut() {
        *i = rng.next()
    }
    //for i in randoms.iter() {
    //    println!("{:08x}", i)
    //}
    let drivers = test_drivers();
    for driver in drivers {
        let preheat = PREHEAT_BASE / driver.divisor;
        csum += (*driver.blockf)(preheat, &randoms) as u64;
        let now = time::Instant::now();
        let nblocks = n / driver.divisor;
        csum += (*driver.blockf)(nblocks, &randoms) as u64;
        let runtime = total_time(now.elapsed());
        let size = nblocks as f64 * BLOCKSIZE as f64;
        println!(
            "{}: {:e} iters in {:.0} msecs for {:.2} nsecs/iter",
            driver.name,
            size,
            runtime * 1.0e3,
            (runtime / size) * 1.0e9
        );
    }
    println!("{}", csum);
}
