[`HyperTwoBits`](https://www2.math.uu.se/~svantejs/papers/sj383-aofa.pdf) is a probabilistic data structure that estimates the number of distinct elements in a set.
 It has the same use case as `HyperLogLog`, but it uses less memory and is faster while achiving a roughly similar precision.
 similar accuracy.

 This implementation holds the entire sketch in the stack without heap allocations. It defaults to
 `ahash` for hashing, but you can use any hasher that implements `std::hash::Hasher`.


 ```rust
 use hypertwobits::{HyperTwoBits, M512};
 let mut htb = HyperTwoBits::<M512>::default();
 htb.insert(&"foo");
 assert_eq!(htb.count(), 1);
 htb.insert(&"bar");
 assert_eq!(htb.count(), 2);
 ```
