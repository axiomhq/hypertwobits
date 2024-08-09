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
//! htb.insert(&"bar");
//! htb.count();
//! ```

#![deny(clippy::pedantic, missing_docs)]
/// `HyperBitBit64` implementation
mod hbb64;
/// `HyperTwoBits` implementation
mod hyper_two_bits;
/// Prelude for easy importing
pub mod prelude;

pub use prelude::*;
