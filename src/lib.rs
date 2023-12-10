#![no_std]
cfg_if::cfg_if! {
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

impl BitWriter {
    pub fn new() -> BitWriter {
        BitWriter {
            data: Vec::new(),
            bit_count: 0,
            cache: 0,
            bits: 0,
        }
    }

    /// Writes at most 8 bits from an u8 type.
    pub fn write_u8(&mut self, v: u8, bit_count: u8) -> Result<()> {
        self.write_unsigned_bits(v as u64, bit_count, 8)
    }

    /// Writes at most 16 bits from an u16 type.
    pub fn write_u16(&mut self, v: u16, bit_count: u8) -> Result<()> {
        self.write_unsigned_bits(v as u64, bit_count, 16)
    }

    /// Writes at most 32 bits from an u32 type.
    pub fn write_u32(&mut self, v: u32, bit_count: u8) -> Result<()> {
        self.write_unsigned_bits(v as u64, bit_count, 32)
    }

    /// Writes at most 64 bits from an u64 type.
    pub fn write_u64(&mut self, v: u64, bit_count: u8) -> Result<()> {
        self.write_unsigned_bits(v, bit_count, 64)
    }

    /// Writes at most 8 bits from an i8 type.
    pub fn write_i8(&mut self, v: i8, bit_count: u8) -> Result<()> {
        self.write_signed_bits(v as i64, bit_count, 8)
    }

    /// Writes at most 16 bits from an i16 type.
    pub fn write_i16(&mut self, v: i16, bit_count: u8) -> Result<()> {
        self.write_signed_bits(v as i64, bit_count, 16)
    }

    /// Writes at most 32 bits from an i32 type.
    pub fn write_i32(&mut self, v: i32, bit_count: u8) -> Result<()> {
        self.write_signed_bits(v as i64, bit_count, 32)
    }

    /// Writes at most 64 bits from an i64 type.
    pub fn write_i64(&mut self, v: i64, bit_count: u8) -> Result<()> {
        self.write_signed_bits(v, bit_count, 64)
    }

    /// Writes a boolean type (as one bit).
    pub fn write_bool(&mut self, v: bool) -> Result<()> {
        self.write_unsigned_bits(v as u64, 1, 1)
    }

    /// Skips a number of bits by writing 0.
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

    /// Aligns the bit stream to the x byte boundary.
    /// for alignment_bytes = 1, it will align to 8 bits,
    /// for alignment_bytes = 2, it will align to 16 bits, and so forth.
    pub fn align(&mut self, alignment_bytes: u32) -> Result<()> {
        if alignment_bytes == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "cannot align to 0 bytes",
            ));
        }
        let alignment_bits = alignment_bytes as u64 * 8;
        let cur_alignment = self.bit_count % alignment_bits;
        let bits_to_skip = (alignment_bits - cur_alignment) % alignment_bits;
        self.skip(bits_to_skip)
    }

    /// Writes a signed integer of any length.
    pub fn write_signed_bits(&mut self, mut v: i64, n: u8, maximum_count: u8) -> Result<()> {
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
        if n == 64 {
            // avoid bitshift overflow exception
            v &= u64::MAX;
        } else {
            v &= (1 << n) - 1;
        }

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
            let mask = ((1 << n) as u8) - 1;
            self.cache = ((v as u8) & mask) << (8 - n);
        }
        Ok(())
    }

    /// Writes a number of full bytes into the stream.
    /// This will give best performance if the data is aligned
    /// to a byte boundary. Otherwise, each data bytes will be
    /// spread across two bytes in the bitstream.
    /// Byte boundary can be ensured by using align().
    pub fn write(&mut self, data: &Vec<u8>) -> Result<()> {
        // If the data is byte-aligned, we can directly
        // copy the bytes without performance penalty.
        if self.bits == 0 {
            // We are writing full bytes, so there is no
            // need to update the bit count.
            self.bit_count += 8 * data.len() as u64;
            self.data.extend(data);
            return Ok(());
        }
        // Since the buffer is not aligned, we need to
        // bit-shift each byte.
        for byte in data {
            match self.write_u8(*byte, 8) {
                Ok(()) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Closes the bit stream, which will align to the next byte boundary
    /// by writing 0s.
    pub fn close(&mut self) -> Result<()> {
        // align to the next byte boundary
        self.align(1)?;
        Ok(())
    }

    /// Returns the number of bits written so far.
    pub fn bit_count(&self) -> u64 {
        self.bit_count
    }

    /// Returns the written data as a byte array. Ensure to call
    /// close() before retrieving the data, to ensure the last
    /// bits were written correctly.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}
