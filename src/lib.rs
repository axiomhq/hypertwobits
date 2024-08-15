//! `HyperTwoBits` is a probabilistic data structure that estimates the number of distinct elements in a set.
//! It has the same use case as `HyperLogLog`, but it uses less memory and is faster while achiving a roughly similar precision.
//! similar accuracy.
//!
//! This implementation holds the entire sketch in the stack without heap allocations. It defaults to
//! `ahash` for hashing, but you can use any hasher that implements `std::hash::Hasher`.
//!
//!
//! ```rust
//! use hypertwobits::{h2b::HyperTwoBits, h2b::M512};
//! let mut htb = HyperTwoBits::<M512>::default();
//! htb.insert(&"foo");
//! htb.insert(&"bar");
//! htb.count();
//! ```

#![deny(clippy::pedantic, missing_docs)]
/// `HyperTwoBits` implementation
pub mod h2b;
/// `HyperBitBit64` implementation
pub mod hbb64;

/// `HyperThreeBits` implementation
pub mod h3b;
/// Prelude for easy importing
pub mod prelude;

use std::hash::{BuildHasher, BuildHasherDefault, Hasher as _};

pub use prelude::*;

/// Random Seeded `AHasher` Builder that allows for seeded hashing per `HyperTwoBit` isnstance
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct AHasherBuilder {
    state: u64,
}

impl Default for AHasherBuilder {
    fn default() -> Self {
        Self {
            state: rand::random(),
        }
    }
}

impl BuildHasher for AHasherBuilder {
    type Hasher = ahash::AHasher;

    fn build_hasher(&self) -> Self::Hasher {
        let mut h = ahash::AHasher::default();
        h.write_u64(self.state);
        h
    }
}

/// Non seeded `AHasher` Builder that is fater but will create completely predictable results
pub type AHasherDefaultBuilder = BuildHasherDefault<ahash::AHasher>;

/// Random Seeded `SipHasher13` Builder
#[cfg(feature = "siphash")]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SipHasher13Builder {
    state: u64,
}
#[cfg(feature = "siphash")]
impl Default for SipHasher13Builder {
    fn default() -> Self {
        Self {
            state: rand::random(),
        }
    }
}
#[cfg(feature = "siphash")]
impl BuildHasher for SipHasher13Builder {
    type Hasher = siphasher::sip::SipHasher13;

    fn build_hasher(&self) -> Self::Hasher {
        let mut h = siphasher::sip::SipHasher13::default();
        h.write_u64(self.state);
        h
    }
}
#[cfg(feature = "siphash")]
/// Non seeded `SipHasher13` Builder that is fater but will create completely predictable results
pub type SipHasher13DefaultBuilder = BuildHasherDefault<siphasher::sip::SipHasher13>;
