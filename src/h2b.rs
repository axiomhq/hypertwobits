pub(super) mod sketch;
#[cfg(test)]
mod tests;

use std::hash::BuildHasher;

pub use sketch::{Sketch, M1024, M128, M2048, M256, M4096, M512, M64};

use crate::AHasherDefaultBuilder;

/// `HyperTwoBits` implementation that is fully stack allocated and generic to avoid branches for
/// different numbers of sub streams.
///
/// Both the hasher and the sub stream size siaz can be customized, by default it uses `AHasherBuilder` and `M256`
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct HyperTwoBits<SKETCH: Sketch = M256, HASH: BuildHasher = AHasherDefaultBuilder> {
    hash: HASH,
    sketch: SKETCH,
    count: u32,
    t: u32,
}

impl<SKETCH: Sketch> Default for HyperTwoBits<SKETCH> {
    fn default() -> Self {
        Self {
            hash: AHasherDefaultBuilder::default(),
            sketch: SKETCH::default(),
            count: 0,
            t: 1,
        }
    }
}

impl<HASH: BuildHasher + Default, BITS: Sketch> HyperTwoBits<BITS, HASH> {
    const ALPHA: f64 = 0.988;
    #[must_use]
    /// Creates a new `HyperTwoBits` counter with specified hasher and bitset,
    /// use `HyperTwoBits::default()` for default values.
    pub fn new() -> Self {
        Self {
            hash: HASH::default(),
            sketch: BITS::default(),
            count: 0,
            t: 1,
        }
    }

    /// Merges another `HyperTwoBits` counter into this one
    /// # Panics
    /// If hasheres are seeded as that prevents merging
    pub fn merge(&mut self, mut other: Self) {
        assert_eq!(
            self.hash.hash_one(42),
            other.hash.hash_one(42),
            "Hashers must be the same, can not merge"
        );
        // The paper asks for actions if the sketch is "nearly full", this is a very loose definition
        // we will assume 99% if substreams set is "nearly full"
        #[allow(
            clippy::cast_lossless,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        let threshold = const { (0.99 * (BITS::STREAMS as f64)) as u32 };
        // for simplicity we ensure that `self` is always the larger sketch
        if other.t > self.t {
            std::mem::swap(self, &mut other);
        }

        // If the values of T differ by 8 or more, use the larger value and its sketches.
        if self.t - other.t > 8 {
            return;
        }
        // we pre-compute if self.t == other.t so we can do the decrement below before handling
        // the other cases
        let same = self.t == other.t;
        // We now only have the first and third case left, so we can handle the decrement
        if self.count >= threshold {
            self.count = self.sketch.decrement();
            self.t += 4;
        }

        if same {
            // Merg sketches
            self.sketch.merge(&other.sketch);
        } else {
            // merge the high bits of other into the low bits of self
            self.sketch.merge_high_into_lo(&other.sketch);
        }
        // update the count
        self.count = self.sketch.count();
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    /// Inserts a value into the counter
    pub fn insert<V: std::hash::Hash + ?Sized>(&mut self, value: &V) {
        let hash = self.hash.hash_one(value);
        self.insert_hash(hash);
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    /// Inserts a value into the counter
    pub fn insert_hash(&mut self, hash: u64) {
        let threshold: u32 = const { (Self::ALPHA * BITS::STREAMS as f64) as u32 };
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        if self.count >= threshold {
            self.count = self.sketch.decrement();
            self.t += 4;
        }
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    /// Inserts 2 elements into the counter for micro batching purposes, note this will delay
    /// the count update to the end
    pub fn insert2<V: std::hash::Hash>(&mut self, v1: &V, v2: &V) {
        let threshold: u32 = const { (Self::ALPHA * BITS::STREAMS as f64) as u32 };

        let hash = self.hash.hash_one(v1);
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        let hash = self.hash.hash_one(v2);
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        if self.count >= threshold {
            self.count = self.sketch.decrement();
            self.t += 4;
        }
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    /// Inserts 4 elements into the counter for micro batching purposes, note this will delay
    /// the count update to the end
    pub fn insert4<V: std::hash::Hash>(&mut self, v1: &V, v2: &V, v3: &V, v4: &V) {
        let threshold: u32 = const { (Self::ALPHA * BITS::STREAMS as f64) as u32 };

        let hash = self.hash.hash_one(v1);
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        let hash = self.hash.hash_one(v2);
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        let hash = self.hash.hash_one(v3);
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        let hash = self.hash.hash_one(v4);
        // use most significant bits for k the rest for x
        let stream: u32 = (hash >> BITS::IDX_SHIFT) as u32;
        let hash: u64 = hash & BITS::HASH_MASK;

        if hash.trailing_ones() >= self.t && self.sketch.val(stream) < 1 {
            self.count += 1;
            self.sketch.set(stream, 1);
        }
        // 2^4
        if hash.trailing_ones() >= self.t + 4 && self.sketch.val(stream) < 2 {
            self.sketch.set(stream, 2);
        }

        // 2^8
        if hash.trailing_ones() >= self.t + 8 && self.sketch.val(stream) < 3 {
            self.sketch.set(stream, 3);
        }

        if self.count >= threshold {
            self.count = self.sketch.decrement();
            self.t += 4;
        }
    }

    /// returns the estimated count. This function is non destructive
    /// and can be called multiple times without changing the state of the counter
    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn count(&self) -> u64 {
        let beta = 1.0 - f64::from(self.count) / f64::from(BITS::STREAMS);
        let bias: f64 = (1.0 / beta).ln();
        ((2.0_f64.powf(f64::from(self.t))) * f64::from(BITS::STREAMS) * bias) as u64
    }
}
