[![docs.rs](https://docs.rs/hypertwobits/badge.svg)](https://docs.rs/hypertwobits/)
[![Rust](https://github.com/axiomhq/hypertwobits/actions/workflows/rust.yml/badge.svg)](https://github.com/axiomhq/hypertwobits/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/hypertwobits.svg)](https://crates.io/crates/hypertwobits)
[![License](https://img.shields.io/crates/l/hypertwobits)](LICENSE-MIT)


[`HyperTwoBits`](https://www2.math.uu.se/~svantejs/papers/sj383-aofa.pdf) is a probabilistic data structure
that estimates the number of distinct elements in a set. It has the same use case as `HyperLogLog`, 
but it uses less memory and is faster while achieving roughly similar accuracy. In numbers while
`HyperLogLog` uses six bits per substream `HyperTwoBits` uses, as the name suggests, two bits

To illustrate Tabe 4 from the linked paper page 14:

![HyperTwoBits P 14 Table 4](static/HyperTwoBitComp.png)

This implementation improves on the proposed algorithm in a few ways. 
- It holds the entire sketch in the stack without heap allocations.
- It defaults to `ahash` for hashing, but you can use any hasher that implements `std::hash::Hasher`.
- It uses traits to remove branches from the runtime execution.
- It moves as much of the computation as possible into constants or compile-time evaluation.
- It removes float comparisons in favor of integer operations.
It uses 128-bit integers to take advantage of wide registers where possible.
- It uses intrinsics to count ones instead of computing them over binary logic.
- It changes the register layout for high/low bits in the scatch to colocate the memory for each region.
- It adds micro batching functions that improve performance when two or four values can be provided at the same time.

Based on [benchmarks](https://github.com/axiomhq/hypertwobits/actions/workflows/criterion.yml) it is about 4 times faster than `HyperLogLogPlus` and 2.5x faster than `HyperBitBit`

 ```rust
 use hypertwobits::{HyperTwoBits, M512};
 let mut htb = HyperTwoBits::<M512>::default();
 htb.insert(&"foo");
 htb.insert(&"bar");
 htb.count();
 ```
