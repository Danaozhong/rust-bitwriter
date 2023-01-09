#![no_std]
cfg_if::cfg_if!{
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::io::Result;
        use std::io::{Error, ErrorKind};
        use std::vec::Vec;
    } else {
        use core::result;
        use core::fmt;
        use core::cmp::min;
    }
}

#[cfg(test)]
mod tests;

pub struct BitWriter {
    // the final output
    data: Vec<u8>,
    bit_count: u64,

    // stores unwritten bits
    cache: u8,

    // number of unwritten bits in cache
    bits: u8,

}

impl BitWriter{
    pub fn new() -> BitWriter {
        BitWriter {
            data: Vec::new(),
            bit_count: 0,
            cache: 0,
            bits: 0,
        }
    }
    /// Read at most 8 bits into a u8.
    pub fn write_u8(&mut self, v: u8, bit_count: u8) -> Result<()>  {
        self.write_unsigned_bits(v as u64, bit_count, 8)
    }

    pub fn write_u16(&mut self, v: u16, bit_count: u8) -> Result<()>  {
        self.write_unsigned_bits(v as u64, bit_count, 16)
    }

    pub fn write_u32(&mut self, v: u32, bit_count: u8) -> Result<()>  {
        self.write_unsigned_bits(v as u64, bit_count, 32)
    }

    pub fn write_u64(&mut self, v: u64, bit_count: u8) -> Result<()> {
        self.write_unsigned_bits(v, bit_count, 64)
    }

    pub fn write_i8(&mut self, v: i16, bit_count: u8) -> Result<()>  {
        self.write_signed_bits(v as i64, bit_count, 8)
    }

    pub fn write_i16(&mut self, v: i16, bit_count: u8) -> Result<()>  {
        self.write_signed_bits(v as i64, bit_count, 16)
    }

    pub fn write_i32(&mut self, v: i32, bit_count: u8) -> Result<()>  {
        self.write_signed_bits(v as i64, bit_count, 32)
    }

    pub fn write_i64(&mut self, v: i64, bit_count: u8) -> Result<()> {
        self.write_signed_bits(v, bit_count, 64)
    }

    pub fn write_bool(&mut self, v: bool) -> Result<()> {
        self.write_unsigned_bits(v as u64, 1, 1)
    }

    pub fn skip(&mut self, n: u64) -> Result<()> {
        // fill the current buffer
        for _ in 0..(n / 64) {
            self.write_unsigned_bits(0, 64, 64)?;
        }

        let leftover = (n % 64) as u8;
        if leftover != 0 {
            self.write_unsigned_bits(0, leftover, 64)?;
        }
        Ok(())
    }

    pub fn align(&mut self, alignment_bytes: u32) -> Result<()> {
        let alignment_bits = alignment_bytes as u64 * 8;
        let cur_alignment = self.bit_count % alignment_bits;
        let bits_to_skip = (alignment_bits - cur_alignment) % alignment_bits;
        self.skip(bits_to_skip)
    }

    pub fn write_signed_bits(&mut self,  mut v: i64, n: u8, maximum_count: u8) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        if v < 0 && n != 64 {
            // move the sign to the correct bit position (bit_count)
            v |= 1 << (n - 1);
        }
        // write as unsigned int
        self.write_unsigned_bits(v as u64, n, maximum_count)
    }

    pub fn write_unsigned_bits(&mut self, mut v: u64, mut n: u8, maximum_count: u8) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        if n > maximum_count || n > 64 {
            return Err(Error::new(ErrorKind::Unsupported, "too many bits to write"));
        }
        // mask all upper bits out to be 0
        v &= (1 << n) - 1;

        self.bit_count += n as u64;

        let new_bits = self.bits + n;
        if new_bits < 8 {
            // the new bits fit into the cache, no writing needed
            self.cache |= (v as u8) << (8 - new_bits);
            self.bits = new_bits;
            return Ok(());
        }
        
        if new_bits >= 8 {
            // write all bytes, by first taking the existing buffer, form a complete byte,
            // and write that first.
            let free_buffer = 8 - self.bits;
            let new_cache = (v >> (n - free_buffer)) as u8;
            self.data.push(self.cache | new_cache);
            n -= free_buffer;

            // Afterwards, data can be written in complete bytes
            while n >= 8 {
                n -= 8;
                self.data.push((v >> n) as u8);
            }
        }
        
        // Whatever is left is smaller than a byte, and will be put into the cache
        self.cache = 0;
        self.bits = n;
        if n > 0 {
            let mask = ((1<<n) as u8) - 1;
            self.cache = ((v as u8) & mask) << (8-n);
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        // align to the next byte boundary
        self.align(1)?;
        Ok(())
    }

    /// Returns the number of bits written so far.
    pub fn bit_count(&self) -> u64 {
        self.bit_count
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
  
}
