const MAX_VALUE: u8 = 7;
/// Sketch storage for `HyperTwoBits` the trait is used
/// to implement optimized storage structs for each value of M
/// this allows us let the compilare avoid know exaclty what M is
/// and avoid conditionals, loops and branches.
pub trait Sketch: Default {
    /// Number of substreams
    const STREAMS: u32;
    /// bitmask for x, the most significant bits n bits are used so that 2^n = M
    /// this is pre-compuated into a const to let the compiler do it's magic
    const HASH_MASK: u64;
    /// number of bits to shift to get k, turns the most significant n bits into k
    /// so that 2^n = M
    const IDX_SHIFT: u32;
    /// Fetches the value of the sketch at position k
    fn val(&self, index: u32) -> u8;
    /// Sets the value of the sketch at position k
    fn set(&mut self, index: u32, value: u8);
    /// Decrements the sketch and returns the new element count
    #[inline]
    fn decrement(&mut self) -> u32 {
        // FIXME: can we bitswiffel this?
        for stream_index in 0..Self::STREAMS {
            let value = self.val(stream_index);
            if value > 0 {
                self.set(stream_index, value - 1);
            }
        }
        self.count()
    }
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
    low_bits: u64,
    middle_bits: u64,
    high_bits: u64,
}

impl Sketch for M64 {
    const STREAMS: u32 = 64;
    const HASH_MASK: u64 =
        0b0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 58;
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn val(&self, stream_index: u32) -> u8 {
        debug_assert!(stream_index < Self::STREAMS);
        // get the bits at index
        let high_bit = (self.high_bits >> stream_index) as u8 & 1;
        let middle_bit = (self.middle_bits >> stream_index) as u8 & 1;
        let low_bit = (self.low_bits >> stream_index) as u8 & 1;
        // combine the bits into a single value
        high_bit << 2 | middle_bit << 1 | low_bit
    }
    #[inline]
    fn set(&mut self, stream_index: u32, value: u8) {
        debug_assert!(stream_index < Self::STREAMS);
        debug_assert!(value <= MAX_VALUE);
        // split value in it's respective bits
        let value = u64::from(value);
        let value_high_bit = (value >> 2) & 1;
        let value_middle_bit = (value >> 1) & 1;
        let value_low_bit = value & 1;
        // reset all bits at index
        self.high_bits &= !(1 << stream_index);
        self.middle_bits &= !(1 << stream_index);
        self.low_bits &= !(1 << stream_index);
        // set the bits at index to the value
        self.high_bits |= value_high_bit << stream_index;
        self.middle_bits |= value_middle_bit << stream_index;
        self.low_bits |= value_low_bit << stream_index;
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     // since we decrement our new count will equal the number of streams that currently are
    //     // set too two or three so we can count cheaply at this point by just looking at the number
    //     // of ones in the high bits;
    //     let count = self.hi.count_ones();
    //     // calculate the sketch where each value is decremented by 1

    //     self.lo = self.hi & !self.lo;
    //     self.hi &= !self.lo;

    //     count
    // }
    #[inline]
    fn count(&self) -> u32 {
        // count the number of sub channels that are active
        // buy looking which have either the high and/or the low bit set
        // and counting the ones in the value
        let set_streams = self.middle_bits | self.low_bits | self.high_bits;
        set_streams.count_ones()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.high_bits |= other.high_bits;
        self.middle_bits |= other.middle_bits;
        self.low_bits |= other.low_bits;
    }

    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.low_bits |= other.middle_bits;
        self.middle_bits |= other.high_bits;
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
    low_bits: u128,
    middle_bits: u128,
    high_bits: u128,
}

impl Sketch for M128 {
    const STREAMS: u32 = 128;
    const HASH_MASK: u64 =
        0b0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 57;

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn val(&self, stream_index: u32) -> u8 {
        debug_assert!(stream_index < Self::STREAMS);
        // get the bits at index
        let high_bit = (self.high_bits >> stream_index) as u8 & 1;
        let middle_bit = (self.middle_bits >> stream_index) as u8 & 1;
        let low_bit = (self.low_bits >> stream_index) as u8 & 1;
        // combine the bits into a single value
        high_bit << 2 | middle_bit << 1 | low_bit
    }

    #[inline]
    fn set(&mut self, stream_index: u32, value: u8) {
        debug_assert!(stream_index < Self::STREAMS);
        debug_assert!(value <= MAX_VALUE);
        // split value in it's respective bits
        let value = u128::from(value);
        let value_high_bit = (value >> 2) & 1;
        let value_middle_bit = (value >> 1) & 1;
        let value_low_bit = value & 1;
        // reset all bits at index
        self.high_bits &= !(1 << stream_index);
        self.middle_bits &= !(1 << stream_index);
        self.low_bits &= !(1 << stream_index);
        // set the bits at index to the value
        self.high_bits |= value_high_bit << stream_index;
        self.middle_bits |= value_middle_bit << stream_index;
        self.low_bits |= value_low_bit << stream_index;
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     let count = self.hi.count_ones();
    //     self.lo = self.hi & !self.lo;
    //     self.hi &= !self.lo;
    //     count
    // }
    #[inline]
    fn count(&self) -> u32 {
        let d = self.middle_bits | self.low_bits | self.high_bits;
        d.count_ones()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.high_bits |= other.high_bits;
        self.middle_bits |= other.middle_bits;
        self.low_bits |= other.low_bits;
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.low_bits |= other.middle_bits;
        self.middle_bits |= other.high_bits;
    }
}

/// We use a register to store hi and low bits together
/// to optimize for cache locallity when compiting inside
/// a vectored sketch
#[derive(Default, Clone, Copy, Debug)]
struct BitRegister {
    high: u128,
    middle: u128,
    low: u128,
}
/// Generic scatch using `REGISTERS` 128 bit `HiLoRegister`
/// so the total M for the sketch is `REGISTERS` * 128.
///
/// This is not meant to be used directly instead it serves as
/// a base for the other vectored sketches
pub struct M128Reg<const REGISTERS: usize> {
    registers: [BitRegister; REGISTERS],
}

impl<const REGISTERS: usize> Default for M128Reg<REGISTERS> {
    fn default() -> Self {
        Self {
            registers: [BitRegister {
                high: 0,
                middle: 0,
                low: 0,
            }; REGISTERS],
        }
    }
}

impl<const REGISTERS: usize> M128Reg<REGISTERS> {
    const REG_SIZE: usize = 128;
    #[inline]
    fn val(&self, k: u32) -> u8 {
        // Calculate the index in the sketch vector
        let register_index = k as usize / Self::REG_SIZE;
        // calculate the left over index into the sketc
        let bit_index = k as usize % Self::REG_SIZE;
        let hx = ((self.registers[register_index].high >> bit_index) & 1) as u8;
        // Calculate the high bit
        let hi = ((self.registers[register_index].middle >> bit_index) & 1) as u8;
        // Calculate the low bit
        let lo = ((self.registers[register_index].low >> bit_index) & 1) as u8;
        (hx << 2) | (hi << 1) | lo
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(v <= MAX_VALUE);
        let v: u128 = v as u128;
        // Calculate the index in the sketch vector
        let i = k as usize / Self::REG_SIZE;
        // calculate the left over index into the sketc
        let k = k as usize % Self::REG_SIZE;
        // set the high bit by first clearing the bit in the sketch and then setting it
        // to the value in v
        self.registers[i].high = (self.registers[i].high & !(1 << k)) | (((v >> 2) & 1) << k);
        self.registers[i].middle = (self.registers[i].middle & !(1 << k)) | (((v >> 1) & 1) << k);
        // set the low bit analogously
        self.registers[i].low = (self.registers[i].low & !(1 << k)) | ((v & 1) << k);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     let mut count = 0;
    //     // Decrement by decrementing each register
    //     for s in &mut self.s {
    //         count += s.hi.count_ones();
    //         s.lo = s.hi & !s.lo;
    //         s.hi &= !s.lo;
    //     }
    //     count
    // }
    #[inline]
    fn count(&self) -> u32 {
        let mut r = 0;
        // Count the number of active substreams by counting them for each register
        // and summing them up
        for s in self.registers {
            r += (s.middle | s.low | s.high).count_ones();
        }
        r
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        // Merge by merging each register
        for (a, b) in self.registers.iter_mut().zip(other.registers.iter()) {
            a.high |= b.high;
            a.middle |= b.middle;
            a.low |= b.low;
        }
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        // Merge by merging each register
        for (a, b) in self.registers.iter_mut().zip(other.registers.iter()) {
            a.low |= b.middle;
            a.middle |= b.high;
        }
    }
}

/// M = 256 Sketch Implementation
pub type M256 = M128Reg<2>;

impl Sketch for M256 {
    const STREAMS: u32 = 256;
    const HASH_MASK: u64 =
        0b0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 56;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::STREAMS);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::STREAMS);
        debug_assert!(v <= MAX_VALUE);
        self.set(k, v);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     self.decrement()
    // }
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
    const STREAMS: u32 = 512;
    const HASH_MASK: u64 =
        0b0000_0000_0111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 55;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::STREAMS);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::STREAMS);
        debug_assert!(v <= MAX_VALUE);
        self.set(k, v);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     self.decrement()
    // }
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
    const STREAMS: u32 = 1024;
    const HASH_MASK: u64 =
        0b0000_0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 54;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::STREAMS);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::STREAMS);
        debug_assert!(v <= MAX_VALUE);
        self.set(k, v);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     self.decrement()
    // }
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
    const STREAMS: u32 = 2048;
    const HASH_MASK: u64 =
        0b0000_0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 53;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::STREAMS);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::STREAMS);
        debug_assert!(v <= MAX_VALUE);
        self.set(k, v);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     self.decrement()
    // }
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
    const STREAMS: u32 = 4096;
    const HASH_MASK: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 52;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::STREAMS);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::STREAMS);
        debug_assert!(v <= MAX_VALUE);
        self.set(k, v);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     self.decrement()
    // }
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
    const STREAMS: u32 = 4096;
    const HASH_MASK: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 52;

    #[inline]
    fn val(&self, k: u32) -> u8 {
        debug_assert!(k < Self::STREAMS);
        self.val(k)
    }

    #[inline]
    fn set(&mut self, k: u32, v: u8) {
        debug_assert!(k < Self::STREAMS);
        debug_assert!(v <= MAX_VALUE);
        self.set(k, v);
    }
    // #[inline]
    // fn decrement(&mut self) -> u32 {
    //     self.decrement()
    // }
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
        for i in 0..S::STREAMS {
            assert_eq!(s.val(i), 0);

            for r in 1..=7 {
                s.set(i, r);
                assert_eq!(s.val(i), r);
            }
            for j in 0..S::STREAMS {
                if j == i {
                    assert_eq!(s.val(j), 7);
                } else {
                    assert_eq!(s.val(j), 0);
                }
            }
            s.set(i, 0);
            assert_eq!(s.val(i), 0);
        }
        for i in 0..S::STREAMS {
            s.set(i, 7);
            assert_eq!(s.val(i), 7);
        }
        for r in (0..=6).rev() {
            s.decrement();
            for i in 0..S::STREAMS {
                assert_eq!(s.val(i), r);
            }
        }
        s.decrement();
        for i in 0..S::STREAMS {
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