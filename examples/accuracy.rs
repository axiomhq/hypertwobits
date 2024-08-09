use std::{
    collections::HashSet,
    fs::File,
    hash::{DefaultHasher, Hasher},
    io::{BufRead, BufReader},
};

use ahash::AHasher;
use hyperloglogplus::HyperLogLog as _;
use hypertwobits::prelude::*;
use itertools::izip;
use siphasher::sip::SipHasher13;

struct Resultset {
    algorithm: String,
    input: &'static str,
    results_100: Vec<u64>,
    results_1_000: Vec<u64>,
    results_10_000: Vec<u64>,
    results_100_000: Vec<u64>,
    results_all: Vec<u64>,
}
impl Resultset {
    fn write_csv_header<W>(w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writeln!(w, "algorithm,input,100,1_000,10_000,100_000,all")
    }
    fn new<S: ToString>(algorithm: S, input: &'static str, n: usize) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            input,
            results_100: Vec::with_capacity(n),
            results_1_000: Vec::with_capacity(n),
            results_10_000: Vec::with_capacity(n),
            results_100_000: Vec::with_capacity(n),
            results_all: Vec::with_capacity(n),
        }
    }
    fn write_csv<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        for (a, b, c, d, e) in izip!(
            self.results_100.iter(),
            self.results_1_000.iter(),
            self.results_10_000.iter(),
            self.results_100_000.iter(),
            self.results_all.iter()
        ) {
            writeln!(
                w,
                "{},{},{},{},{},{},{}",
                self.algorithm, self.input, a, b, c, d, e
            )?;
        }
        Ok(())
    }
}

fn hll(name: &'static str, results: &mut Vec<Resultset>, n: usize, data: &[String]) {
    let mut r = Resultset::new("HyperLogLogPlus", name, n);
    for _ in 0..n {
        let mut counter: hyperloglogplus::HyperLogLogPlus<
            String,
            std::collections::hash_map::RandomState,
        > = hyperloglogplus::HyperLogLogPlus::new(
            16,
            std::collections::hash_map::RandomState::new(),
        )
        .unwrap();
        for (i, w) in data.iter().enumerate() {
            counter.insert(w);
            match i {
                100 => r.results_100.push(counter.count() as u64),
                1_000 => r.results_1_000.push(counter.count() as u64),
                10_000 => r.results_10_000.push(counter.count() as u64),
                100_000 => r.results_100_000.push(counter.count() as u64),
                _ => {}
            }
        }
        r.results_all.push(counter.count() as u64);
    }
    results.push(r);
}

fn htb<BITS: Sketch, HASH: Hasher + Default>(
    algo: &'static str,
    name: &'static str,
    results: &mut Vec<Resultset>,
    n: usize,
    data: &[String],
) {
    let mut r = Resultset::new(algo, name, n);
    for _ in 0..n {
        let mut counter: HyperTwoBits<BITS, HASH> = HyperTwoBits::new();
        for (i, w) in data.iter().enumerate() {
            counter.insert(w);
            match i {
                100 => r.results_100.push(counter.count() as u64),
                1_000 => r.results_1_000.push(counter.count() as u64),
                10_000 => r.results_10_000.push(counter.count() as u64),
                100_000 => r.results_100_000.push(counter.count() as u64),
                _ => {}
            }
        }
        r.results_all.push(counter.count() as u64);
    }
    results.push(r);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let ulysses = BufReader::new(File::open("data/Ulysses.csv")?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?;
    let shakespear = BufReader::new(File::open("data/Shakespeare.csv")?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?;
    let war_and_peace = BufReader::new(File::open("data/War_and_Peace.csv")?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?;

    let mut set = HashSet::new();

    let n = 10;
    for (name, data) in [
        ("Ulysses", &ulysses),
        ("Shakespeare", &shakespear),
        ("War and Peace", &war_and_peace),
    ] {
        let mut r = Resultset::new("baseline", name, 1);
        for (i, w) in data.iter().enumerate() {
            set.insert(w);
            match i {
                100 => r.results_100.push(set.len() as u64),
                1_000 => r.results_1_000.push(set.len() as u64),
                10_000 => r.results_10_000.push(set.len() as u64),
                100_000 => r.results_100_000.push(set.len() as u64),
                _ => {}
            }
        }
        r.results_all.push(set.len() as u64);
        results.push(r);

        hll(name, &mut results, n, data);
        htb!(
            name, &mut results, n, data;
            AHasher,DefaultHasher,SipHasher13;
            M64, M128, M256, M512, M1024, M2048, M4096
        );
    }

    let mut w = std::io::stdout();
    Resultset::write_csv_header(&mut w)?;
    for r in results.iter() {
        r.write_csv(&mut w)?;
    }

    Ok(())
}

#[macro_export]
macro_rules! htb {
    ( $name:expr, $results:expr, $n:expr, $data:expr; $( $hash:tt ),*; $( $m:tt ),* ) => {
        htb!(@call_hash $name, $results, $n, $data; $($hash),*; @ ($($m),*))
    };
    (@call_hash $name:expr, $results:expr, $n:expr, $data:expr; $( $hash:tt ),*; @ $ms:tt) => {
        $(htb!(@call $name, $results, $n, $data; $hash; $ms));*
    };
    (@call $name:expr, $results:expr, $n:expr, $data:expr; $hash:tt; ($($m:tt),*)) => {
        $(
        htb::<$m, $hash>($name, concat!("HyperTwoBits<", stringify!($m), " + ", stringify!($hash),">"), $results, $n, $data)
        );*
    };
}
