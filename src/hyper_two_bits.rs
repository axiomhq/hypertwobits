pub(super) mod sketch;
#[cfg(test)]
mod tests;

use std::hash::Hasher;

use sketch::Sketch;

use crate::M256;

/// `HyperTwoBits` implementation that is fully stack allocated and generic to avoid branches for
/// different numbers of sub streams.
///
/// Both the hasher and the sub stream size siaz can be customized, by default it uses `ahash` and `M256`
pub struct HyperTwoBits<SKETCH: Sketch = M256, HASH: Hasher + Default = ahash::AHasher> {
    _hash: std::marker::PhantomData<HASH>,
    sketch: SKETCH,
    count: u32,
    t: u32,
}

impl<SKETCH: Sketch> Default for HyperTwoBits<SKETCH> {
    fn default() -> Self {
        Self {
            _hash: std::marker::PhantomData,
            sketch: SKETCH::default(),
            count: 0,
            t: 1,
        }
    }
}

impl<HASH: Hasher + Default, BITS: Sketch> HyperTwoBits<BITS, HASH> {
    const ALPHA: f64 = 0.988;
    #[must_use]
    /// Creates a new `HyperTwoBits` counter with specified hasher and bitset,
    /// use `HyperTwoBits::default()` for default values.
    pub fn new() -> Self {
        Self {
            _hash: std::marker::PhantomData,
            sketch: BITS::default(),
            count: 0,
            t: 1,
        }
    }

    /// Merges another `HyperTwoBits` counter into this one
    pub fn merge(&mut self, mut other: Self) {
        // The paper asks for actions if the sketch is "nearly full", this is a very loose definition
        // we will assume 99% if substreams set is "nearly full"
        #[allow(
            clippy::cast_lossless,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        let threshold = const { (0.99 * (BITS::M as f64)) as u32 };
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
    pub fn insert<V: std::hash::Hash>(&mut self, v: &V) {
        let threshold: u32 = const { (Self::ALPHA * BITS::M as f64) as u32 };
        let mut x = HASH::default();
        v.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
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
        let threshold: u32 = const { (Self::ALPHA * BITS::M as f64) as u32 };

        let mut x = HASH::default();
        v1.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
        }

        let mut x = HASH::default();
        v2.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
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
        let threshold: u32 = const { (Self::ALPHA * BITS::M as f64) as u32 };

        let mut x = HASH::default();
        v1.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
        }

        let mut x = HASH::default();
        v2.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
        }

        let mut x = HASH::default();
        v3.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
        }

        let mut x = HASH::default();
        v4.hash(&mut x);
        let x = x.finish();
        // use most significant bits for k the rest for x
        let k: u32 = (x >> BITS::K_SHIFT) as u32;
        let x: u64 = x & BITS::X_MASK;

        if x.trailing_ones() >= self.t && self.sketch.val(k) < 1 {
            self.count += 1;
            self.sketch.set(k, 1);
        }
        // 2^4
        if x.trailing_ones() >= self.t + 4 && self.sketch.val(k) < 2 {
            self.sketch.set(k, 2);
        }

        // 2^8
        if x.trailing_ones() >= self.t + 8 && self.sketch.val(k) < 3 {
            self.sketch.set(k, 3);
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
        let beta = 1.0 - f64::from(self.count) / f64::from(BITS::M);
        let bias: f64 = (1.0 / beta).ln();
        ((2.0_f64.powf(f64::from(self.t))) * f64::from(BITS::M) * bias) as u64
    }
}
