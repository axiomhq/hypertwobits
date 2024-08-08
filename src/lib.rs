//! `HyperTwoBits` is a probabilistic data structure that estimates the number of distinct elements in a set.
//! It has the same use case as `HyperLogLog`, but it uses less memory and is faster while achiving a roughly similar precision.
//! similar accuracy.
//!
//! This implementation holds the entire sketch in the stack without heap allocations. It defaults to
//! `ahash` for hashing, but you can use any hasher that implements `std::hash::Hasher`.
//!
//!
//! ```rust
//! use hypertwobits::{HyperTwoBits, M512};
//! let mut htb = HyperTwoBits::<M512>::default();
//! htb.insert(&"foo");
//! assert_eq!(htb.count(), 1);
//! htb.insert(&"bar");
//! assert_eq!(htb.count(), 2);
//! ```

#![deny(clippy::pedantic, missing_docs)]
/// `HyperBitBit64` implementation
mod hbb64;
/// `HyperTwoBits` implementation
mod htb;

pub use hbb64::HyperBitBit64;
pub use htb::sketch::{Sketch, M1024, M128, M2048, M256, M4096, M512, M64};
pub use htb::HyperTwoBits;

#[cfg(test)]
mod tests {

    use std::{
        collections::HashSet,
        hash::RandomState,
        io::{BufRead, BufReader},
    };

    use hyperloglogplus::{HyperLogLog as _, HyperLogLogPlus};

    use super::*;

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn run<S: Sketch>(f: &str, delta: f64, mut n: usize) -> std::io::Result<()> {
        let mut htb: HyperTwoBits<S> = HyperTwoBits::<S>::new();
        let mut hll: HyperLogLogPlus<[u8], RandomState> =
            HyperLogLogPlus::new(16, RandomState::new()).unwrap();
        let mut set = HashSet::new();

        let buf = BufReader::new(std::fs::File::open(f)?);
        let mut lines = buf.lines();
        while let Some(Ok(line)) = lines.next() {
            if n == 0 {
                break;
            }
            n -= 1;
            let s = line.as_bytes();
            htb.insert(&s);
            hll.insert(s);
            set.insert(line);
        }

        let count = set.len() as f64;
        let delta_hll = (count - hll.count()).abs() / count;
        let delta_htb = (count - htb.count() as f64).abs() / count;

        let diff_htb = delta_htb - delta_hll;

        assert!(
            diff_htb < delta,
            "Delta between HLL+ and HTB is too high: {diff_htb}\nCount: {count}\nHLL+:  {}\nHTB:   {}",
            hll.count() as u64,
            htb.count()
        );
        Ok(())
    }

    fn test_all(f: &str, delta: f64, n: usize) -> std::io::Result<()> {
        // we only test M2048 for now to sazve time when running tests
        // it's the medium tradeoff between space and precision, for HLL we use precision of 16
        // from a range 4 and 18
        run::<M2048>(f, delta, n)?;
        Ok(())
    }
    #[test]
    fn test_shakespear() -> std::io::Result<()> {
        test_all("data/Shakespeare.csv", 0.1, usize::MAX)
    }
    #[test]
    fn test_shakespear_100() -> std::io::Result<()> {
        test_all("data/Shakespeare.csv", 0.22, 100)
    }
    #[test]
    fn test_shakespear_1_000() -> std::io::Result<()> {
        test_all("data/Shakespeare.csv", 0.13, 1_000)
    }
    #[test]
    fn test_shakespear_10_000() -> std::io::Result<()> {
        test_all("data/Shakespeare.csv", 0.1, 10_000)
    }
    #[test]
    fn test_shakespear_100_000() -> std::io::Result<()> {
        test_all("data/Shakespeare.csv", 0.1, 100_000)
    }

    #[test]
    fn test_ulysses() -> std::io::Result<()> {
        test_all("data/Ulysses.csv", 0.1, usize::MAX)
    }
    #[test]
    fn test_ulysses_100() -> std::io::Result<()> {
        test_all("data/Ulysses.csv", 0.23, 100)
    }
    #[test]
    fn test_ulysses_1_000() -> std::io::Result<()> {
        test_all("data/Ulysses.csv", 0.14, 1_000)
    }
    #[test]
    fn test_ulysses_10_000() -> std::io::Result<()> {
        test_all("data/Ulysses.csv", 0.1, 10_000)
    }
    #[test]
    fn test_ulysses_100_000() -> std::io::Result<()> {
        test_all("data/Ulysses.csv", 0.12, 100_000)
    }

    #[test]
    fn test_war_and_peace() -> std::io::Result<()> {
        test_all("data/War_and_Peace.csv", 0.1, usize::MAX)
    }
    #[test]
    fn test_war_and_peace_100() -> std::io::Result<()> {
        test_all("data/War_and_Peace.csv", 0.20, 100)
    }
    #[test]
    fn test_war_and_peace_1_000() -> std::io::Result<()> {
        test_all("data/War_and_Peace.csv", 0.13, 1_000)
    }
    #[test]
    fn test_war_and_peace_10_000() -> std::io::Result<()> {
        test_all("data/War_and_Peace.csv", 0.1, 10_000)
    }
    #[test]
    fn test_war_and_peace_100_000() -> std::io::Result<()> {
        test_all("data/War_and_Peace.csv", 0.1, 100_000)
    }
}
