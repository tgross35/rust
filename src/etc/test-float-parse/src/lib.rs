use std::any::type_name;
use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::mem::transmute;
use std::ops;
use std::thread;
use std::time;

mod gen {
    // pub mod long_fractions;
    // pub mod short_decimals;
    pub mod subnorm;
    // pub mod u64_pow2;
}

// Nothing up my sleeve: Just (PI - 3) in base 16.
#[allow(dead_code)]
pub const SEED: [u32; 3] = [0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e];

pub fn validate(text: &str) {
    let mut out = io::stdout();
    let x: f64 = text.parse().unwrap();
    let f64_bytes: u64 = unsafe { transmute(x) };
    let x: f32 = text.parse().unwrap();
    let f32_bytes: u32 = unsafe { transmute(x) };
    writeln!(&mut out, "{:016x} {:08x} {}", f64_bytes, f32_bytes, text).unwrap();
}

pub fn run_all() {
    run_all_f::<f32>();
    run_all_f::<f64>();
}

fn run_all_f<F: Float>() {
    let v = [spawn_gen::<F, gen::subnorm::SubnormEdge<F>>()];

    for th in v {
        th.join();
    }
}

fn spawn_gen<F: Float, G: Generator<F>>() -> thread::JoinHandle<()> {
    std::thread::spawn(|| {
        println!("testing {} {}", type_name::<F>(), G::NAME);
    })
}

/// Open a file with a reasonable name that we can dump data to
pub fn log_file() -> fs::File {
    let now = chrono::Utc::now();
    let name = format!("parse-float-{}.txt", now.format("%Y-%m-%dT%H_%M_%S_%3fZ"));
    fs::OpenOptions::new().write(true).create_new(true).open(name).unwrap()
}

trait Int:
    Copy
    + fmt::Debug
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::AddAssign
    + ops::Shl<u32, Output = Self>
    + PartialOrd
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
}

macro_rules! impl_int {
    ($($ty:ty),+) => {
        $(
            impl Int for $ty {
                const ZERO: Self = 0;
                const ONE: Self = 0;
            }
        )+
    }
}

impl_int!(u32, u64);

trait Float: Copy + fmt::LowerExp {
    /// Unsigned integer of same width
    type Int: Int;

    /// Total bits
    const BITS: u32;

    /// (Stored) bits in the mantissa)
    const MAN_BITS: u32;

    /// Bits in the exponent
    const EXP_BITS: u32 = Self::BITS - Self::MAN_BITS - 1;

    // const MAN_MASK: Self::Int = (Self::Int::ONE << Self::MAN_BITS) - Self::Int::ONE;
    const MAN_MASK: Self::Int;

    fn from_bits(i: Self::Int) -> Self;
    fn to_bits(self) -> Self::Int;
}

macro_rules! impl_float {
    ($($ty:ty, $ity:ty, $bits:literal);+) => {
        $(
            impl Float for $ty {
                type Int = $ity;
                const BITS: u32 = $bits;
                const MAN_BITS: u32 = Self::MANTISSA_DIGITS - 1;
                const MAN_MASK: Self::Int = (Self::Int::ONE << Self::MAN_BITS) - Self::Int::ONE;
                fn from_bits(i: Self::Int) -> Self { Self::from_bits(i) }
                fn to_bits(self) -> Self::Int { self.to_bits() }
            }
        )+
    }
}

impl_float!(f32, u32, 32; f64, u64, 64);

/// Implement this on
trait Generator<F: Float> {
    const NAME: &'static str;

    /// Approximate number of tests that will be run
    fn estimated_tests() -> u64;

    fn new() -> Self;

    /// Return the next number in this generator
    fn next<'a>(&'a mut self) -> Option<&'a str>;
}

const fn const_min(a: u32, b: u32) -> u32 {
    if a <= b { a } else { b }
}
