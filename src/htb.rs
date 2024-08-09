pub(super) mod sketch;

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

#[cfg(test)]
mod tests {

    use super::*;
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
}
