use std::mem::MaybeUninit;

pub struct Bitstream<'a> {
    pub output: &'a mut [MaybeUninit<u8>],
    pub out_idx: usize,
    pub bitbuf: u64,
    pub bitcount: u32,
}

impl<'a> Bitstream<'a> {
    pub fn new(output: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            output,
            out_idx: 0,
            bitbuf: 0,
            bitcount: 0,
        }
    }

    #[inline(always)]
    pub fn write_bits(&mut self, bits: u32, count: u32) -> bool {
        if count == 0 {
            return true;
        }
        let mask = ((1u64 << count) - 1) as u32;
        unsafe { self.write_bits_unchecked(bits & mask, count) }
    }

    /// Writes up to 32 bits without checking count or masking bits.
    ///
    /// * `count` must be > 0.
    /// * `bits` must not have any bits set above `count`.
    #[inline(always)]
    pub unsafe fn write_bits_upto_32(&mut self, bits: u32, count: u32) -> bool {
        self.write_bits_unchecked(bits, count)
    }

    /// Writes bits assuming sufficient buffer space (at least 8 bytes at current `out_idx`).
    ///
    /// * `count` must be > 0.
    /// * `bits` must not have any bits set above `count`.
    /// * `self.out_idx + 8 <= self.output.len()`.
    #[inline(always)]
    pub unsafe fn write_bits_unchecked_fast(&mut self, bits: u32, count: u32) {
        debug_assert!(count > 0);
        debug_assert!(count <= 32);
        debug_assert!(self.out_idx + 8 <= self.output.len());

        let bitcount = self.bitcount;
        let new_bitcount = bitcount + count;

        if new_bitcount >= 32 {
            let bitbuf = self.bitbuf | ((bits as u64) << bitcount);
            std::ptr::write_unaligned(
                self.output.as_mut_ptr().add(self.out_idx) as *mut u64,
                bitbuf.to_le(),
            );
            self.out_idx += 4;
            self.bitbuf = bitbuf >> 32;
            self.bitcount = new_bitcount - 32;
        } else {
            self.bitbuf |= (bits as u64) << bitcount;
            self.bitcount = new_bitcount;
        }
    }

    /// Writes up to 60 bits assuming sufficient buffer space (at least 8 bytes at current `out_idx`).
    ///
    /// * `count` must be > 0 and <= 60.
    /// * `bits` must not have any bits set above `count`.
    /// * `self.out_idx + 8 <= self.output.len()`.
    #[inline(always)]
    pub unsafe fn write_bits_unchecked_fast_64(&mut self, bits: u64, count: u32) {
        debug_assert!(count > 0);
        debug_assert!(count <= 60);
        debug_assert!(self.out_idx + 8 <= self.output.len());

        let bitcount = self.bitcount;
        let new_bitcount = bitcount + count;

        if new_bitcount >= 64 {
            let bitbuf_low = self.bitbuf | (bits << bitcount);
            let bitbuf_high = bits >> (64 - bitcount);

            std::ptr::write_unaligned(
                self.output.as_mut_ptr().add(self.out_idx) as *mut u64,
                bitbuf_low.to_le(),
            );
            self.out_idx += 8;
            self.bitbuf = bitbuf_high;
            self.bitcount = new_bitcount - 64;
        } else {
            let bitbuf = self.bitbuf | (bits << bitcount);
            if new_bitcount >= 32 {
                std::ptr::write_unaligned(
                    self.output.as_mut_ptr().add(self.out_idx) as *mut u64,
                    bitbuf.to_le(),
                );
                self.out_idx += 4;
                self.bitbuf = bitbuf >> 32;
                self.bitcount = new_bitcount - 32;
            } else {
                self.bitbuf = bitbuf;
                self.bitcount = new_bitcount;
            }
        }
    }

    /// Writes bits without checking count or masking bits.
    ///
    /// * `count` must be > 0.
    /// * `bits` must not have any bits set above `count` (i.e., `bits & !((1 << count) - 1) == 0`).
    #[inline(always)]
    pub unsafe fn write_bits_unchecked(&mut self, bits: u32, count: u32) -> bool {
        debug_assert!(count > 0);
        debug_assert!(count <= 32);

        let bitcount = self.bitcount;
        let new_bitcount = bitcount + count;

        if new_bitcount >= 32 {
            let bitbuf = self.bitbuf | ((bits as u64) << bitcount);

            if self.out_idx + 8 <= self.output.len() {
                unsafe {
                    std::ptr::write_unaligned(
                        self.output.as_mut_ptr().add(self.out_idx) as *mut u64,
                        bitbuf.to_le(),
                    );
                }
                self.out_idx += 4;
                self.bitbuf = bitbuf >> 32;
                self.bitcount = new_bitcount - 32;
                return true;
            }

            if self.out_idx + 4 <= self.output.len() {
                unsafe {
                    std::ptr::write_unaligned(
                        self.output.as_mut_ptr().add(self.out_idx) as *mut u32,
                        (bitbuf as u32).to_le(),
                    );
                }
                self.out_idx += 4;
                self.bitbuf = bitbuf >> 32;
                self.bitcount = new_bitcount - 32;
                return true;
            }

            self.bitbuf = bitbuf;
            self.bitcount = new_bitcount;

            while self.bitcount >= 8 {
                if self.out_idx >= self.output.len() {
                    return false;
                }
                unsafe {
                    self.output
                        .get_unchecked_mut(self.out_idx)
                        .write((self.bitbuf & 0xFF) as u8);
                }
                self.out_idx += 1;
                self.bitbuf >>= 8;
                self.bitcount -= 8;
            }
        } else {
            self.bitbuf |= (bits as u64) << bitcount;
            self.bitcount = new_bitcount;
        }
        true
    }

    pub fn flush(&mut self) -> (bool, u32) {
        while self.bitcount >= 8 {
            if self.out_idx >= self.output.len() {
                return (false, 0);
            }
            unsafe {
                self.output
                    .get_unchecked_mut(self.out_idx)
                    .write((self.bitbuf & 0xFF) as u8);
            }
            self.out_idx += 1;
            self.bitbuf >>= 8;
            self.bitcount -= 8;
        }

        let mut valid_bits = 0;
        if self.bitcount > 0 {
            if self.out_idx >= self.output.len() {
                return (false, 0);
            }

            self.output[self.out_idx].write((self.bitbuf & 0xFF) as u8);
            self.out_idx += 1;
            valid_bits = self.bitcount;
            self.bitbuf = 0;
            self.bitcount = 0;
        }
        (true, valid_bits)
    }

    pub fn flush_align(&mut self) -> (bool, u32) {
        while self.bitcount >= 8 {
            if self.out_idx >= self.output.len() {
                return (false, 0);
            }
            unsafe {
                self.output
                    .get_unchecked_mut(self.out_idx)
                    .write((self.bitbuf & 0xFF) as u8);
            }
            self.out_idx += 1;
            self.bitbuf >>= 8;
            self.bitcount -= 8;
        }

        if self.bitcount > 0 {
            if self.out_idx >= self.output.len() {
                return (false, 0);
            }

            self.output[self.out_idx].write((self.bitbuf & 0xFF) as u8);
            self.out_idx += 1;
            self.bitbuf = 0;
            self.bitcount = 0;
        }
        (true, 0)
    }
}
