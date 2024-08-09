use crate::{HyperTwoBits, Sketch, M2048};

use std::{
    hash::RandomState,
    io::{BufRead, BufReader},
};

use hyperloglogplus::{HyperLogLog as _, HyperLogLogPlus};

#[test]
fn htb64_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M64>>(), 24);
}
#[test]
fn htb128_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M128>>(), 48);
}
#[test]
fn htb256_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M256>>(), 80);
}
#[test]
fn htb512_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M512>>(), 144);
}
#[test]
fn htb1024_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M1024>>(), 272);
}
#[test]
fn htb2048_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M2048>>(), 528);
}
#[test]
fn htb4096_size() {
    assert_eq!(std::mem::size_of::<HyperTwoBits<crate::M4096>>(), 1040);
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
// note: we have pre-computed actual to not have to use `set` in here and slow down the tests
fn run<S: Sketch>(f: &str, actual: usize, delta: f64, mut n: usize) -> std::io::Result<()> {
    let mut htb: HyperTwoBits<S> = HyperTwoBits::<S>::new();
    let mut htb_a: HyperTwoBits<S> = HyperTwoBits::<S>::new();
    let mut htb_b: HyperTwoBits<S> = HyperTwoBits::<S>::new();

    let mut hll: HyperLogLogPlus<[u8], RandomState> =
        HyperLogLogPlus::new(16, RandomState::new()).unwrap();
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
        hll.insert(s);
        // set.insert(line);
    }

    htb_a.merge(htb_b);

    let count = actual as f64;
    let delta_hll = (count - hll.count()).abs() / count;
    let delta_htb = (count - htb.count() as f64).abs() / count;
    let delta_htbm = (count - htb_a.count() as f64).abs() / count;

    let diff_htb = delta_htb - delta_hll;
    let diff_htbm = delta_htbm - delta_hll;

    // assert_eq!(actual, set.len());
    assert!(
        diff_htb < delta,
        "Delta between HLL+ and HTB is too high: {diff_htb}\nCount: {count}\nHLL+:  {}\nHTB:   {}",
        hll.count() as u64,
        htb.count()
    );
    // We know the merge is less precise we take that into account by multiplying the delta by 2
    assert!(
        diff_htbm < delta*2.0,
        "Delta between HLL+ and HTB(merged) is too high: {diff_htbm}\nCount: {count}\nHLL+:  {}\nHTB:   {}",
        hll.count() as u64,
        htb_a.count()
    );
    Ok(())
}

fn test_all(f: &str, actual: usize, delta: f64, n: usize) -> std::io::Result<()> {
    // we only test M2048 for now to sazve time when running tests
    // it's the medium tradeoff between space and precision, for HLL we use precision of 16
    // from a range 4 and 18
    run::<M2048>(f, actual, delta, n)?;
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
