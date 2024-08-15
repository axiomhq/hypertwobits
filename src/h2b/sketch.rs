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
    fn val(&self, stream: u32) -> u8;
    /// Sets the value of the sketch at position k
    fn set(&mut self, stream: u32, value: u8);
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
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Clone, Default)]
pub struct M64 {
    low: u64,
    high: u64,
}

impl Sketch for M64 {
    const STREAMS: u32 = 64;
    const HASH_MASK: u64 =
        0b0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 58;
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream 
          Self::STREAMS);
        let high_bit = (self.high >> stream) as u8 & 1;
        let low_bit = (self.low >> stream) as u8 & 1;
        high_bit << 1 | low_bit
    }
    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        let value = u64::from(value);
        let value_high_bit = (value >> 1) & 1;
        let value_low_bit = value & 1;
        // reset all bits at index
        self.high &= !(1 << stream);
        self.low &= !(1 << stream);
        // set the bits at index to the value
        self.high |= value_high_bit << stream;
        self.low |= value_low_bit << stream;
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        // since we decrement our new count will equal the number of streams that currently are
        // set too two or three so we can count cheaply at this point by just looking at the number
        // of ones in the high bits;
        let count = self.high.count_ones();
        // calculate the sketch where each value is decremented by 1
        self.low = self.high & !self.low;
        self.high &= !self.low;
        count
    }
    #[inline]
    fn count(&self) -> u32 {
        // count the number of sub channels that are active
        // buy looking which have either the high and/or the low bit set
        // and counting the ones in the value
        let used_streams = self.high | self.low;
        used_streams.count_ones()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.high |= other.high;
        self.low |= other.low;
    }

    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.low |= other.high;
    }
}

/// M = 128, using two 128 bit integers to store the sketch
/// we do this instead of a array of 64 bit integers to
/// take adavantage of modern architectures that offer good
/// instructions for 128 bit integers.
///
/// The implementation is similar to M64
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Clone, Default)]
pub struct M128 {
    low: u128,
    high: u128,
}

impl Sketch for M128 {
    const STREAMS: u32 = 128;
    const HASH_MASK: u64 =
        0b0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 57;

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        let high_bit = (self.high >> stream) as u8 & 1;
        let low_bit = (self.low >> stream) as u8 & 1;
        high_bit << 1 | low_bit
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        let value = value as u128;
        let value_high_bit = (value >> 1) & 1;
        let value_low_bit = value & 1;
        // reset all bits at index
        self.high &= !(1 << stream);
        self.low &= !(1 << stream);
        // set the bits at index to the value
        self.high |= value_high_bit << stream;
        self.low |= value_low_bit << stream;
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        let count = self.high.count_ones();
        self.low = self.high & !self.low;
        self.high &= !self.low;
        count
    }
    #[inline]
    fn count(&self) -> u32 {
        let used_streams = self.high | self.low;
        used_streams.count_ones()
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.high |= other.high;
        self.low |= other.low;
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        self.low |= other.high;
    }
}

/// We use a register to store hi and low bits together
/// to optimize for cache locallity when compiting inside
/// a vectored sketch
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct HiLoRegister {
    high: u128,
    low: u128,
}
/// Generic scatch using `REGISTERS` 128 bit `HiLoRegister`
/// so the total M for the sketch is `REGISTERS` * 128.
///
/// This is not meant to be used directly instead it serves as
/// a base for the other vectored sketches
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct M128Reg<const REGISTERS: usize> {
    registers: [HiLoRegister; REGISTERS],
}

impl<const REGISTERS: usize> Default for M128Reg<REGISTERS> {
    fn default() -> Self {
        Self {
            registers: [HiLoRegister { high: 0, low: 0 }; REGISTERS],
        }
    }
}

impl<const REGISTERS: usize> M128Reg<REGISTERS> {
    const REG_SIZE: usize = 128;
    #[inline]
    fn val(&self, stream: u32) -> u8 {
        // Calculate the index in the sketch vector
        let register_index = stream as usize / Self::REG_SIZE;
        // calculate the left over index into the sketc
        let bit_index = stream as usize % Self::REG_SIZE;
        // Calculate the high bit
        let high_bit = ((self.registers[register_index].high >> bit_index) & 1) as u8;
        // Calculate the low bit
        let low_bit = ((self.registers[register_index].low >> bit_index) & 1) as u8;
        (high_bit << 1) | low_bit
    }
    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(value < 4);
        // Calculate the index in the sketch vector
        let register_index = stream as usize / Self::REG_SIZE;
        // calculate the left over index into the sketc
        let bit_index = stream as usize % Self::REG_SIZE;

        // split value in it's respective bits
        let value = u128::from(value);
        let value_high_bit = (value >> 1) & 1;
        let value_low_bit = value & 1;

        // reset all bits at index
        self.registers[register_index].high &= !(1 << bit_index);
        self.registers[register_index].low &= !(1 << bit_index);

        // set the bits at index to the value
        self.registers[register_index].high |= value_high_bit << bit_index;
        self.registers[register_index].low |= value_low_bit << bit_index;
    }
    #[inline]
    fn decrement(&mut self) -> u32 {
        let mut count = 0;
        // Decrement by decrementing each register
        for register in &mut self.registers {
            count += register.high.count_ones();
            register.low = register.high & !register.low;
            register.high &= !register.low;
        }
        count
    }
    #[inline]
    fn count(&self) -> u32 {
        let mut count = 0;
        // Count the number of active substreams by counting them for each register
        // and summing them up
        for registers in self.registers {
            count += (registers.high | registers.low).count_ones();
        }
        count
    }
    #[inline]
    fn merge(&mut self, other: &Self) {
        // Merge by merging each register
        for (self_register, other_register) in self.registers.iter_mut().zip(other.registers.iter())
        {
            self_register.high |= other_register.high;
            self_register.low |= other_register.low;
        }
    }
    #[inline]
    fn merge_high_into_lo(&mut self, other: &Self) {
        // Merge by merging each register
        for (self_register, other_register) in self.registers.iter_mut().zip(other.registers.iter())
        {
            self_register.low |= other_register.high;
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
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        self.val(stream)
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        self.set(stream, value);
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
    const STREAMS: u32 = 512;
    const HASH_MASK: u64 =
        0b0000_0000_0111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 55;

    #[inline]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        self.val(stream)
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        self.set(stream, value);
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
    const STREAMS: u32 = 1024;
    const HASH_MASK: u64 =
        0b0000_0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 54;

    #[inline]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        self.val(stream)
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        self.set(stream, value);
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
    const STREAMS: u32 = 2048;
    const HASH_MASK: u64 =
        0b0000_0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 53;

    #[inline]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        self.val(stream)
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        self.set(stream, value);
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
    const STREAMS: u32 = 4096;
    const HASH_MASK: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 52;

    #[inline]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        self.val(stream)
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        self.set(stream, value);
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
    const STREAMS: u32 = 4096;
    const HASH_MASK: u64 =
        0b0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
    const IDX_SHIFT: u32 = 52;

    #[inline]
    fn val(&self, stream: u32) -> u8 {
        debug_assert!(stream < Self::STREAMS);
        self.val(stream)
    }

    #[inline]
    fn set(&mut self, stream: u32, value: u8) {
        debug_assert!(stream < Self::STREAMS);
        debug_assert!(value < 4);
        self.set(stream, value);
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
        for i in 0..S::STREAMS {
            assert_eq!(s.val(i), 0);

            for r in 1..=3 {
                s.set(i, r);
                assert_eq!(s.val(i), r);
            }
            for j in 0..S::STREAMS {
                if j == i {
                    assert_eq!(s.val(j), 3);
                } else {
                    assert_eq!(s.val(j), 0);
                }
            }
            s.set(i, 0);
            assert_eq!(s.val(i), 0);
        }
        for i in 0..S::STREAMS {
            s.set(i, 3);
            assert_eq!(s.val(i), 3);
        }
        for r in (0..=2).rev() {
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
