use hyperloglogplus::HyperLogLog as _;
use hypertwobits::prelude::*;
use std::{
    collections::{hash_map::RandomState, HashSet},
    error::Error,
    fs::File,
    hash::BuildHasher,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
struct Resultset {
    algorithm: String,
    results_100: Vec<u64>,
    results_1_000: Vec<u64>,
    results_10_000: Vec<u64>,
    results_100_000: Vec<u64>,
    results_all: Vec<u64>,
    total: u64,
}

fn mean(data: &[u64]) -> f64 {
    data.iter().sum::<u64>() as f64 / data.len() as f64
}
fn median(data: &[u64]) -> f64 {
    let n = data.len();
    if n % 2 == 0 {
        (data[n / 2 - 1] + data[n / 2]) as f64 / 2.0
    } else {
        data[n / 2] as f64
    }
}
fn stddev(data: &[u64]) -> f64 {
    let m = mean(data);
    let sum = data.iter().map(|x| (*x as f64 - m).powi(2)).sum::<f64>();
    (sum / data.len() as f64).sqrt()
}
fn min(data: &[u64]) -> f64 {
    data[0] as f64
}
fn max(data: &[u64]) -> f64 {
    data[data.len() - 1] as f64
}

impl Resultset {
    fn new<S: ToString>(algorithm: S, n: usize) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            results_100: Vec::with_capacity(n),
            results_1_000: Vec::with_capacity(n),
            results_10_000: Vec::with_capacity(n),
            results_100_000: Vec::with_capacity(n),
            results_all: Vec::with_capacity(n),
            total: 0,
        }
    }
    fn finalize(&mut self) {
        self.results_100.sort();
        self.results_1_000.sort();
        self.results_10_000.sort();
        self.results_100_000.sort();
        self.results_all.sort();
    }
}

fn hllp(results: &mut Vec<Resultset>, n: usize, data: &[String]) {
    let mut r = Resultset::new("HyperLogLogPlus", n);
    for _ in 0..n {
        let mut counter: hyperloglogplus::HyperLogLogPlus<String, RandomState> =
            hyperloglogplus::HyperLogLogPlus::new(16, RandomState::new()).unwrap();
        for (i, w) in data.iter().enumerate() {
            counter.insert(w);
            match i {
                100 => r.results_100.push(counter.count() as u64),
                1_000 => r.results_1_000.push(counter.count() as u64),
                10_000 => r.results_10_000.push(counter.count() as u64),
                100_000 => r.results_100_000.push(counter.count() as u64),
                _ => {}
            }
            r.total += 1;
        }
        r.results_all.push(counter.count() as u64);
    }
    r.finalize();
    results.push(r);
}
fn hll(results: &mut Vec<Resultset>, n: usize, data: &[String]) {
    let mut r = Resultset::new("HyperLogLog", n);
    for _ in 0..n {
        let mut counter: hyperloglog::HyperLogLog = hyperloglog::HyperLogLog::new(0.00408);
        for (i, w) in data.iter().enumerate() {
            counter.insert(w);
            match i {
                100 => r.results_100.push(counter.len() as u64),
                1_000 => r.results_1_000.push(counter.len() as u64),
                10_000 => r.results_10_000.push(counter.len() as u64),
                100_000 => r.results_100_000.push(counter.len() as u64),
                _ => {}
            }
            r.total += 1;
        }
        r.results_all.push(counter.len() as u64);
    }
    r.finalize();
    results.push(r);
}

fn h2b<BITS: h2b::Sketch, HASH: BuildHasher + Default>(
    algo: &'static str,
    results: &mut Vec<Resultset>,
    n: usize,
    data: &[String],
) {
    let mut r = Resultset::new(algo, n);
    for _ in 0..n {
        let mut counter: h2b::HyperTwoBits<BITS, HASH> = h2b::HyperTwoBits::new();
        for (i, w) in data.iter().enumerate() {
            counter.insert(w);
            match i {
                100 => r.results_100.push(counter.count()),
                1_000 => r.results_1_000.push(counter.count()),
                10_000 => r.results_10_000.push(counter.count()),
                100_000 => r.results_100_000.push(counter.count()),
                _ => {}
            }
            r.total += 1;
        }
        r.results_all.push(counter.count());
    }
    r.finalize();
    results.push(r);
}
fn h3b<BITS: h3b::Sketch, HASH: BuildHasher + Default>(
    algo: &'static str,
    results: &mut Vec<Resultset>,
    n: usize,
    data: &[String],
) {
    let mut r = Resultset::new(algo, n);
    for _ in 0..n {
        let mut counter: h3b::HyperThreeBits<BITS, HASH> = h3b::HyperThreeBits::new();
        for (i, w) in data.iter().enumerate() {
            counter.insert(w);
            match i {
                100 => r.results_100.push(counter.count()),
                1_000 => r.results_1_000.push(counter.count()),
                10_000 => r.results_10_000.push(counter.count()),
                100_000 => r.results_100_000.push(counter.count()),
                _ => {}
            }
            r.total += 1;
        }
        r.results_all.push(counter.count());
    }
    r.finalize();
    results.push(r);
}

#[derive(serde::Serialize)]
struct JsonResults {
    r100: JsonResult,
    r1_000: JsonResult,
    r10_000: JsonResult,
    r100_000: JsonResult,
    rall: JsonResult,
}
#[derive(serde::Serialize, Clone)]
struct JsonResult {
    algorithm: String,
    mean: f64,
    stddev: f64,
    median: f64,
    min: f64,
    max: f64,
    counts: Vec<u64>,
}

impl JsonResult {
    fn from(name: &str, counts: Vec<u64>) -> Self {
        Self {
            algorithm: name.to_string(),
            mean: mean(&counts),
            stddev: stddev(&counts),
            median: median(&counts),
            min: min(&counts),
            max: max(&counts),
            counts,
        }
    }
}

impl From<Resultset> for JsonResults {
    fn from(r: Resultset) -> Self {
        let Resultset {
            algorithm,
            results_100,
            results_1_000,
            results_10_000,
            results_100_000,
            results_all,
            ..
        } = r;
        Self {
            r100: JsonResult::from(&algorithm, results_100),
            r1_000: JsonResult::from(&algorithm, results_1_000),
            r10_000: JsonResult::from(&algorithm, results_10_000),
            r100_000: JsonResult::from(&algorithm, results_100_000),
            rall: JsonResult::from(&algorithm, results_all),
        }
    }
}
#[derive(serde::Serialize)]
struct Output<'o> {
    results: Vec<&'o JsonResult>,
}

#[derive(Default)]
struct Inputs {
    ulysses: Vec<Resultset>,
    shakespeare: Vec<Resultset>,
    war_and_peace: Vec<Resultset>,
    combined: Vec<Resultset>,
}

fn run(data: &[String], results: &mut Vec<Resultset>, n: usize) {
    let mut set = HashSet::new();
    let mut r = Resultset::new("baseline", 1);
    for (i, w) in data.iter().enumerate() {
        set.insert(w);
        match i {
            100 => r.results_100.push(set.len() as u64),
            1_000 => r.results_1_000.push(set.len() as u64),
            10_000 => r.results_10_000.push(set.len() as u64),
            100_000 => r.results_100_000.push(set.len() as u64),
            _ => {}
        }
        r.total += 1;
    }
    r.results_all.push(set.len() as u64);
    results.push(r);

    hllp(results, n, data);
    hll(results, n, data);
    h2b!(
        results, n, data;
        // AHasherBuilder, RandomState, SipHasher13Builder;
        AHasherBuilder;
        // M64, M128, M256, M512, M1024, M2048, M4096
        M4096
    );
    h3b!(
        results, n, data;
        // AHasherBuilder, RandomState, SipHasher13Builder;
        AHasherBuilder;
        // M64, M128, M256, M512, M1024, M2048, M4096
        M4096
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let n = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let ulysses = BufReader::new(File::open("data/Ulysses.csv")?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?;
    let shakespeare = BufReader::new(File::open("data/Shakespeare.csv")?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?;
    let war_and_peace = BufReader::new(File::open("data/War_and_Peace.csv")?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?;

    let combined = ulysses
        .iter()
        .chain(shakespeare.iter())
        .chain(war_and_peace.iter())
        .cloned()
        .collect::<Vec<_>>();

    let r_u = std::thread::spawn(move || {
        let mut results = Vec::new();
        run(&ulysses, &mut results, n);
        results
    });
    let r_s = std::thread::spawn(move || {
        let mut results = Vec::new();
        run(&shakespeare, &mut results, n);
        results
    });
    let r_w = std::thread::spawn(move || {
        let mut results = Vec::new();
        run(&war_and_peace, &mut results, n);
        results
    });
    let r_c = std::thread::spawn(move || {
        let mut results = Vec::new();
        run(&combined, &mut results, n);
        results
    });
    let results = Inputs {
        ulysses: r_u.join().unwrap(),
        shakespeare: r_s.join().unwrap(),
        war_and_peace: r_w.join().unwrap(),
        combined: r_c.join().unwrap(),
    };

    let outputs_ulysses = results
        .ulysses
        .into_iter()
        .map(JsonResults::from)
        .collect::<Vec<_>>();
    let outputs_shakespear = results
        .shakespeare
        .into_iter()
        .map(JsonResults::from)
        .collect::<Vec<_>>();
    let outputs_war_and_peace = results
        .war_and_peace
        .into_iter()
        .map(JsonResults::from)
        .collect::<Vec<_>>();
    let outputs_combined = results
        .combined
        .into_iter()
        .map(JsonResults::from)
        .collect::<Vec<_>>();

    write_json("ulysses", &outputs_ulysses)?;
    write_json("shakespeare", &outputs_shakespear)?;
    write_json("war_and_peace", &outputs_war_and_peace)?;
    write_json("combined", &outputs_combined)?;

    Ok(())
}

fn write_json(name: &str, outputs: &[JsonResults]) -> Result<(), Box<dyn Error>> {
    std::fs::write(
        format!("stats/{name}-100.json"),
        serde_json::to_vec(&Output {
            results: outputs.iter().map(|r| &r.r100).collect(),
        })?,
    )?;
    std::fs::write(
        format!("stats/{name}-1000.json"),
        serde_json::to_vec(&Output {
            results: outputs.iter().map(|r| &r.r1_000).collect(),
        })?,
    )?;
    std::fs::write(
        format!("stats/{name}-10000.json"),
        serde_json::to_vec(&Output {
            results: outputs.iter().map(|r| &r.r10_000).collect(),
        })?,
    )?;
    std::fs::write(
        format!("stats/{name}-100000.json"),
        serde_json::to_vec(&Output {
            results: outputs.iter().map(|r| &r.r100).collect(),
        })?,
    )?;
    std::fs::write(
        format!("stats/{name}-all.json"),
        serde_json::to_vec(&Output {
            results: outputs.iter().map(|r| &r.rall).collect(),
        })?,
    )?;
    Ok(())
}

#[macro_export]
macro_rules! h2b {
    ( $results:expr, $n:expr, $data:expr; $( $hash:tt ),*; $( $m:tt ),* ) => {
        h2b!(@call_hash  $results, $n, $data; $($hash),*; @ ($($m),*))
    };
    (@call_hash $results:expr, $n:expr, $data:expr; $( $hash:tt ),*; @ $ms:tt) => {
        $(h2b!(@call $results, $n, $data; $hash; $ms));*
    };
    (@call $results:expr, $n:expr, $data:expr; $hash:tt; ($($m:tt),*)) => {
        $(
        h2b::<h2b::$m, $hash>(concat!("HyperTwoBits<", stringify!($m), " + ", stringify!($hash),">"), $results, $n, $data)
        );*
    };
}

#[macro_export]
macro_rules! h3b {
    ( $results:expr, $n:expr, $data:expr; $( $hash:tt ),*; $( $m:tt ),* ) => {
        h3b!(@call_hash  $results, $n, $data; $($hash),*; @ ($($m),*))
    };
    (@call_hash $results:expr, $n:expr, $data:expr; $( $hash:tt ),*; @ $ms:tt) => {
        $(h3b!(@call $results, $n, $data; $hash; $ms));*
    };
    (@call $results:expr, $n:expr, $data:expr; $hash:tt; ($($m:tt),*)) => {
        $(
        h3b::<h3b::$m, $hash>(concat!("HyperThreeBits<", stringify!($m), " + ", stringify!($hash),">"), $results, $n, $data)
        );*
    };
}
