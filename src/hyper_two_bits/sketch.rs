/// Sketch storage for `HyperTwoBits` the trait is used
/// to implement optimized storage structs for each value of M
/// this allows us let the compilare avoid know exaclty what M is
/// and avoid conditionals, loops and branches.
pub trait Sketch: Default {
    /// Number of substreams
    const M: u32;
    /// bitmask for x, the most significant bits n bits are used so that 2^n = M
    /// this is pre-compuated into a const to let the compiler do it's magic
    const X_MASK: u64;
    /// number of bits to shift to get k, turns the most significant n bits into k
    /// so that 2^n = M
    const K_SHIFT: u32;
    /// Fetches the value of the sketch at position k
    fn val(&self, k: u32) -> u8;
    /// Sets the value of the sketch at position k
    fn set(&mut self, k: u32, v: u8);
    /// Decrements the sketch and returns the new element count
    fn decrement(&mut self) -> u32;
    /// Returns the number of active sub streams in the sketch
    fn count(&self) -> u32;
    /// Merges the sketch with another sketch by oring the values
    fn merge(&mut self, other: &Self);
    /// Merges sketches that differ in T by the following rules:
    /// - self.lo = self.lo | other.hi
    /// - self.hi remains unchanged
    fn merge_high_into_lo(&mut self, other: &Self);
}

/// M = 64, using two 64 bit integers to store the sketch
#[derive(Default)]
pub struct M64 {
    lo: u64,
    hi: u64,
}

impl Sketch for M64 {
    const M: u32 = 64;
    const X_MASK: u64 =
        0b0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 58;
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        let hi = (self.hi >> k) as u8 & 1;
        let lo = (self.lo >> k) as u8 & 1;
        hi << 1 | lo
    }
    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        let v = u64::from(v);
        self.hi = (self.hi & !(1 << k)) | (((v / 2) & 1) << k);
        self.lo = (self.lo & !(1 << k)) | ((v & 1) << k);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        // since we decrement our new count will equal the number of streams that currently are
        // set too two or three so we can count cheaply at this point by just looking at the number
        // of ones in the high bits;
        let c = self.hi.count_ones();
        // calculate the sketch where each value is decremented by 1
        self.lo = self.hi & !self.lo;
        self.hi &= !self.lo;
        c
    }
    #[inline]
    fn count(&self) -> u32 {
        // count the number of sub channels that are active
        // buy looking which have either the high and/or the low bit set
        // and counting the ones in the value
        let d = self.hi | self.lo;
        d.count_ones()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.hi |= other.hi;
        self.lo |= other.lo;
    }

    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.lo |= other.hi;
    }
}

/// M = 128, using two 128 bit integers to store the sketch
/// we do this instead of a array of 64 bit integers to
/// take adavantage of modern architectures that offer good
/// instructions for 128 bit integers.
///
/// The implementation is similar to M64
#[derive(Default)]
pub struct M128 {
    lo: u128,
    hi: u128,
}

impl Sketch for M128 {
    const M: u32 = 128;
    const X_MASK: u64 =
        0b0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 57;

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        let hi = (self.hi >> k) as u8 & 1;
        let lo = (self.lo >> k) as u8 & 1;
        hi << 1 | lo
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        let v = v as u128;
        self.hi = (self.hi & !(1 << k)) | (((v / 2) & 1) << k);
        self.lo = (self.lo & !(1 << k)) | ((v & 1) << k);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        let count = self.hi.count_ones();
        self.lo = self.hi & !self.lo;
        self.hi &= !self.lo;
        count
    }
    #[inline]
    fn count(&self) -> u32 {
        let d = self.hi | self.lo;
        d.count_ones()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.hi |= other.hi;
        self.lo |= other.lo;
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.lo |= other.hi;
    }
}

/// We use a register to store hi and low bits together
/// to optimize for cache locallity when compiting inside
/// a vectored sketch
#[derive(Default, Clone, Copy, Debug)]
struct HiLoRegister {
    hi: u128,
    lo: u128,
}
/// Generic scatch using `REGISTERS` 128 bit `HiLoRegister`
/// so the total M for the sketch is `REGISTERS` * 128.
///
/// This is not meant to be used directly instead it serves as
/// a base for the other vectored sketches
pub struct M128Reg<const REGISTERS: usize> {
    s: [HiLoRegister; REGISTERS],
}

impl<const REGISTERS: usize> Default for M128Reg<REGISTERS> {
    fn default() -> Self {
        Self {
            s: [HiLoRegister { hi: 0, lo: 0 }; REGISTERS],
        }
    }
}

impl<const REGISTERS: usize> M128Reg<REGISTERS> {
    const REG_SIZE: usize = 128;
    #[inline]
    fn val(&self, k: u32) -> u8 {
        // Calculate the index in the sketch vector
        let i = k as usize / Self::REG_SIZE;
        // calculate the left over index into the sketc
        let k = k as usize % Self::REG_SIZE;
        // Calculate the high bit
        let hi = ((self.s[i].hi >> k) & 1) as u8;
        // Calculate the low bit
        let lo = ((self.s[i].lo >> k) & 1) as u8;
        (hi << 1) | lo
    }
    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(v < 4);
        let v: u128 = v as u128;
        // Calculate the index in the sketch vector
        let i = k as usize / Self::REG_SIZE;
        // calculate the left over index into the sketc
        let k = k as usize % Self::REG_SIZE;
        // set the high bit by first clearing the bit in the sketch and then setting it
        // to the value in v
        self.s[i].hi = (self.s[i].hi & !(1 << k)) | (((v / 2) & 1) << k);
        // set the low bit analogously
        self.s[i].lo = (self.s[i].lo & !(1 << k)) | ((v & 1) << k);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        let mut count = 0;
        // Decrement by decrementing each register
        for s in &mut self.s {
            count += s.hi.count_ones();
            s.lo = s.hi & !s.lo;
            s.hi &= !s.lo;
        }
        count
    }
    #[inline]
    fn count(&self) -> u32 {
        let mut r = 0;
        // Count the number of active substreams by counting them for each register
        // and summing them up
        for s in self.s {
            r += (s.hi | s.lo).count_ones();
        }
        r
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        // Merge by merging each register
        for (a, b) in self.s.iter_mut().zip(other.s.iter()) {
            a.hi |= b.hi;
            a.lo |= b.lo;
        }
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        // Merge by merging each register
        for (a, b) in self.s.iter_mut().zip(other.s.iter()) {
            a.lo |= b.hi;
        }
    }
}

/// M = 256 Sketch Implementation
pub type M256 = M128Reg<2>;

impl Sketch for M256 {
    const M: u32 = 256;
    const X_MASK: u64 =
        0b0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 56;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        self.set(k, v);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        self.decrement()
    }
    #[inline]
    fn count(&self) -> u32 {
        self.count()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.merge(other);
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.merge_high_into_lo(other);
    }
}

/// M = 512 Sketch Implementation
pub type M512 = M128Reg<4>;

impl Sketch for M512 {
    const M: u32 = 512;
    const X_MASK: u64 =
        0b0000_0000_0111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 55;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        self.set(k, v);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        self.decrement()
    }
    #[inline]
    fn count(&self) -> u32 {
        self.count()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.merge(other);
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.merge_high_into_lo(other);
    }
}

/// M = 1024 Sketch Implementation
pub type M1024 = M128Reg<8>;

impl Sketch for M1024 {
    const M: u32 = 1024;
    const X_MASK: u64 =
        0b0000_0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 54;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        self.set(k, v);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        self.decrement()
    }
    #[inline]
    fn count(&self) -> u32 {
        self.count()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.merge(other);
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.merge_high_into_lo(other);
    }
}

/// M = 2048 Sketch Implementation
pub type M2048 = M128Reg<16>;

impl Sketch for M2048 {
    const M: u32 = 2048;
    const X_MASK: u64 =
        0b0000_0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 53;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        self.set(k, v);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        self.decrement()
    }
    #[inline]
    fn count(&self) -> u32 {
        self.count()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.merge(other);
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.merge_high_into_lo(other);
    }
}

/// M = 4096 Sketch Implementation
pub type M4096 = M128Reg<32>;

impl Sketch for M4096 {
    const M: u32 = 4096;
    const X_MASK: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 52;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        self.set(k, v);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        self.decrement()
    }
    #[inline]
    fn count(&self) -> u32 {
        self.count()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.merge(other);
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.merge_high_into_lo(other);
    }
}

/// M = 4096 Sketch Implementation
pub type M8192 = M128Reg<64>;

impl Sketch for M8192 {
    const M: u32 = 4096;
    const X_MASK: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const K_SHIFT: u32 = 52;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::M);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::M);
        debug_assert!(v < 4);
        self.set(k, v);
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        self.decrement()
    }
    #[inline]
    fn count(&self) -> u32 {
        self.count()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.merge(other);
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.merge_high_into_lo(other);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test<S: Sketch>() {
        let mut s = S::default();
        for i in 0..S::M {
            assert_eq!(s.val(i), 0);
            s.set(i, 1);
            assert_eq!(s.val(i), 1);
            s.set(i, 2);
            assert_eq!(s.val(i), 2);
            s.set(i, 3);
            assert_eq!(s.val(i), 3);
            for j in 0..S::M {
                if j == i {
                    assert_eq!(s.val(j), 3);
                } else {
                    assert_eq!(s.val(j), 0);
                }
            }
            s.set(i, 0);
            assert_eq!(s.val(i), 0);
        }
        for i in 0..S::M {
            s.set(i, 3);
            assert_eq!(s.val(i), 3);
        }
        s.decrement();
        for i in 0..S::M {
            assert_eq!(s.val(i), 2);
        }
        s.decrement();
        for i in 0..S::M {
            assert_eq!(s.val(i), 1);
        }
        s.decrement();
        for i in 0..S::M {
            assert_eq!(s.val(i), 0);
        }
        s.decrement();
        for i in 0..S::M {
            assert_eq!(s.val(i), 0);
        }
    }

    #[test]
    fn test_m64() {
        test::<M64>();
    }
    #[test]
    fn test_m128() {
        test::<M128>();
    }
    #[test]
    fn test_m265() {
        test::<M256>();
    }
    #[test]
    fn test_m512() {
        test::<M512>();
    }
    #[test]
    fn test_m1024() {
        test::<M1024>();
    }
    #[test]
    fn test_m2048() {
        test::<M2048>();
    }
    #[test]
    fn test_m4096() {
        test::<M4096>();
    }
}
