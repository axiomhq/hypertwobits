use super::{HyperThreeBits, Sketch, M4096};

use std::io::{BufRead, BufReader};

use hyperloglog::HyperLogLog;

#[test]
fn htb64_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M64>>(), 32);
}
#[test]
fn htb128_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M128>>(), 64);
}
#[test]
fn htb256_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M256>>(), 112);
}
#[test]
fn htb512_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M512>>(), 208);
}
#[test]
fn htb1024_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M1024>>(), 400);
}
#[test]
fn htb2048_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M2048>>(), 784);
}
#[test]
fn htb4096_size() {
    assert_eq!(std::mem::size_of::<HyperThreeBits<super::M4096>>(), 1552);
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
// note: we have pre-computed actual to not have to use `set` in here and slow down the tests
fn run<S: Sketch>(f: &str, actual: usize, delta: f64, mut n: usize) -> std::io::Result<()> {
    let mut htb: HyperThreeBits<S> = HyperThreeBits::<S>::new();
    let mut htb_a: HyperThreeBits<S> = HyperThreeBits::<S>::new();
    let mut htb_b: HyperThreeBits<S> = HyperThreeBits::<S>::new();

    let mut hll: HyperLogLog = HyperLogLog::new(0.00408);
    // let mut set = std::collections::HashSet::new();

    let buf = BufReader::new(std::fs::File::open(f)?);
    let mut lines = buf.lines();
    while let Some(Ok(line)) = lines.next() {
        if n == 0 {
            break;
        }
        n -= 1;
        if n % 2 == 0 {
            htb_a.insert(&line.as_bytes());
        } else {
            htb_b.insert(&line.as_bytes());
        }
        let s = line.as_bytes();
        htb.insert(&s);
        hll.insert(&s);
        // set.insert(line);
    }

    htb_a.merge(htb_b);

    let count = actual as f64;
    let delta_hll = (count - hll.len()).abs() / count;
    let delta_htb = (count - htb.count() as f64).abs() / count;
    // let delta_htbm = (count - htb_a.count() as f64).abs() / count;

    let diff_htb = delta_htb - delta_hll;
    // let diff_htbm = delta_htbm - delta_hll;

    // assert_eq!(actual, set.len());
    assert!(
        delta_htb < delta,
        "Delta between HLL and H3B is too high: {diff_htb}\nCount: {count}\nHLL:  {}\nH3B:   {}",
        hll.len() as u64,
        htb.count()
    );
    // FIXME: todo
    // // We know the merge is less precise we take that into account by multiplying the delta by 2
    // assert!(
    //     delta_htbm < delta*2.0,
    //     "Delta between HLL and H3B(merged) is too high: {diff_htbm}\nCount: {count}\nHLL:  {}\nH3B:   {}",
    //     hll.len() as u64,
    //     htb_a.count()
    // );
    Ok(())
}

fn test_all(f: &str, actual: usize, delta: f64, n: usize) -> std::io::Result<()> {
    // we only test M4096 for now to sazve time when running tests
    // it's the medium tradeoff between space and precision, for HLL we use error rate of 0.00408
    run::<M4096>(f, actual, delta, n)?;
    Ok(())
}
#[test]
fn test_shakespear() -> std::io::Result<()> {
    test_all("data/Shakespeare.csv", 35594, 0.1, usize::MAX)
}
#[test]
fn test_shakespear_100() -> std::io::Result<()> {
    test_all("data/Shakespeare.csv", 75, 0.22, 100)
}
#[test]
fn test_shakespear_1_000() -> std::io::Result<()> {
    test_all("data/Shakespeare.csv", 462, 0.13, 1_000)
}
#[test]
fn test_shakespear_10_000() -> std::io::Result<()> {
    test_all("data/Shakespeare.csv", 2501, 0.1, 10_000)
}
#[test]
fn test_shakespear_100_000() -> std::io::Result<()> {
    test_all("data/Shakespeare.csv", 9519, 0.1, 100_000)
}

#[test]
fn test_ulysses() -> std::io::Result<()> {
    test_all("data/Ulysses.csv", 35343, 0.1, usize::MAX)
}
#[test]
fn test_ulysses_100() -> std::io::Result<()> {
    test_all("data/Ulysses.csv", 74, 0.23, 100)
}
#[test]
fn test_ulysses_1_000() -> std::io::Result<()> {
    test_all("data/Ulysses.csv", 471, 0.14, 1_000)
}
#[test]
fn test_ulysses_10_000() -> std::io::Result<()> {
    test_all("data/Ulysses.csv", 2510, 0.1, 10_000)
}
#[test]
fn test_ulysses_100_000() -> std::io::Result<()> {
    test_all("data/Ulysses.csv", 14869, 0.12, 100_000)
}

#[test]
fn test_war_and_peace() -> std::io::Result<()> {
    test_all("data/War_and_Peace.csv", 22668, 0.1, usize::MAX)
}
#[test]
fn test_war_and_peace_100() -> std::io::Result<()> {
    test_all("data/War_and_Peace.csv", 70, 0.20, 100)
}
#[test]
fn test_war_and_peace_1_000() -> std::io::Result<()> {
    test_all("data/War_and_Peace.csv", 200, 0.13, 1_000)
}
#[test]
fn test_war_and_peace_10_000() -> std::io::Result<()> {
    test_all("data/War_and_Peace.csv", 2030, 0.1, 10_000)
}
#[test]
fn test_war_and_peace_100_000() -> std::io::Result<()> {
    test_all("data/War_and_Peace.csv", 8248, 0.1, 100_000)
}
