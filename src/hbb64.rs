use std::hash::Hasher;

/// `HyperBitBit` cardinality counter with 64 substreams
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct HyperBitBit64<HASH: Hasher + Default = ahash::AHasher> {
    _hash: std::marker::PhantomData<HASH>,
    sketch1: u64,
    sketch2: u64,
    count1: u64,
    count2: u64,
    u: u64, // Power-of-two estimate of substream cardinality
}
impl<HASH: Hasher + Default> HyperBitBit64<HASH> {
    #[must_use]
    /// Creates a new `HyperBitBit64` counter
    pub fn new() -> Self {
        Self {
            _hash: std::marker::PhantomData,
            sketch1: 0,
            sketch2: 0,
            count1: 0,
            count2: 0,
            u: 1,
        }
    }
}

impl Default for HyperBitBit64 {
    fn default() -> Self {
        Self {
            _hash: std::marker::PhantomData,
            sketch1: 0,
            sketch2: 0,
            count1: 0,
            count2: 0,
            u: 1,
        }
    }
}

impl<HASH: Hasher + Default> HyperBitBit64<HASH> {
    /// The number of substreams
    const M: u64 = 64;
    #[inline]
    /// Inserts a value into the counter
    pub fn insert<V: std::hash::Hash>(&mut self, v: V) {
        let mut x = HASH::default();
        v.hash(&mut x);
        let x = x.finish();
        let k: u32 = (x >> 58) as u32 % 64;
        let x: u64 = x & 0x03FF_FFFF_FFFF_FFFF;

        if x & (self.u - 1) == self.u - 1 {
            if (self.sketch1 & (1_u64 << k)) == 0 {
                self.count1 += 1;
            };
            self.sketch1 |= 1 << k;
        }

        if x & (4 * self.u - 1) == 4 * self.u - 1 {
            if (self.sketch2 & (1_u64 << k)) == 0 {
                self.count2 += 1;
            };
            self.sketch2 |= 1 << k;
        }

        if self.count1 >= 62 {
            self.u *= 4;
            self.sketch1 = self.sketch2;
            self.count1 = self.count2;
            self.sketch2 = 0;
            self.count2 = 0;
        }
    }
    #[inline]
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    /// Returns the cardinality estimate
    pub fn count(&self) -> u64 {
        let beta = 1.0 - (self.count1 as f64 / Self::M as f64);
        let bias: f64 = 1.1 * (1.0 / beta).ln();
        ((self.u as f64) * (Self::M as f64) * bias) as u64
    }
}
