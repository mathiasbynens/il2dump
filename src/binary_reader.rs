#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

pub struct BinaryReader<R> {
    inner: R,
    pub is_32bit: bool,
    pub endian: Endianness,
}

impl<R: Read + Seek> BinaryReader<R> {
    pub fn new(inner: R, is_32bit: bool, endian: Endianness) -> Self {
        Self {
            inner,
            is_32bit,
            endian,
        }
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }

    pub fn position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }

    pub fn seek(&mut self, pos: u64) -> io::Result<u64> {
        self.inner.seek(SeekFrom::Start(pos))
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.inner.read_u8()? != 0)
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.inner.read_u8()
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.inner.read_i8()
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        match self.endian {
            Endianness::Little => self.inner.read_u16::<LittleEndian>(),
            Endianness::Big => self.inner.read_u16::<BigEndian>(),
        }
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        match self.endian {
            Endianness::Little => self.inner.read_i16::<LittleEndian>(),
            Endianness::Big => self.inner.read_i16::<BigEndian>(),
        }
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        match self.endian {
            Endianness::Little => self.inner.read_u32::<LittleEndian>(),
            Endianness::Big => self.inner.read_u32::<BigEndian>(),
        }
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        match self.endian {
            Endianness::Little => self.inner.read_i32::<LittleEndian>(),
            Endianness::Big => self.inner.read_i32::<BigEndian>(),
        }
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        match self.endian {
            Endianness::Little => self.inner.read_u64::<LittleEndian>(),
            Endianness::Big => self.inner.read_u64::<BigEndian>(),
        }
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        match self.endian {
            Endianness::Little => self.inner.read_i64::<LittleEndian>(),
            Endianness::Big => self.inner.read_i64::<BigEndian>(),
        }
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        match self.endian {
            Endianness::Little => self.inner.read_f32::<LittleEndian>(),
            Endianness::Big => self.inner.read_f32::<BigEndian>(),
        }
    }

    pub fn read_f64(&mut self) -> io::Result<f64> {
        match self.endian {
            Endianness::Little => self.inner.read_f64::<LittleEndian>(),
            Endianness::Big => self.inner.read_f64::<BigEndian>(),
        }
    }

    pub fn read_ptr(&mut self) -> io::Result<u64> {
        if self.is_32bit {
            self.read_u32().map(|x| x as u64)
        } else {
            self.read_u64()
        }
    }

    pub fn read_iptr(&mut self) -> io::Result<i64> {
        if self.is_32bit {
            self.read_i32().map(|x| x as i64)
        } else {
            self.read_i64()
        }
    }

    pub fn read_string_to_null(&mut self, addr: u64) -> io::Result<String> {
        let saved_pos = self.position()?;
        self.seek(addr)?;
        let mut bytes = Vec::new();
        loop {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }
        self.seek(saved_pos)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn read_uleb128(&mut self) -> io::Result<u32> {
        let mut value = self.read_u8()? as u32;
        if value >= 0x80 {
            let mut bitshift = 0;
            value &= 0x7f;
            loop {
                let b = self.read_u8()?;
                bitshift += 7;
                value |= ((b & 0x7f) as u32) << bitshift;
                if b < 0x80 {
                    break;
                }
            }
        }
        Ok(value)
    }

    pub fn read_compressed_uint32(&mut self) -> io::Result<u32> {
        let read = self.read_u8()?;
        let val = if (read & 0x80) == 0 {
            read as u32
        } else if (read & 0xC0) == 0x80 {
            let next = self.read_u8()? as u32;
            (((read & !0x80) as u32) << 8) | next
        } else if (read & 0xE0) == 0xC0 {
            let b1 = self.read_u8()? as u32;
            let b2 = self.read_u8()? as u32;
            let b3 = self.read_u8()? as u32;
            (((read & !0xC0) as u32) << 24) | (b1 << 16) | (b2 << 8) | b3
        } else if read == 0xF0 {
            self.read_u32()?
        } else if read == 0xFE {
            u32::MAX - 1
        } else if read == 0xFF {
            u32::MAX
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid compressed integer format",
            ));
        };
        Ok(val)
    }

    pub fn read_compressed_int32(&mut self) -> io::Result<i32> {
        let encoded = self.read_compressed_uint32()?;
        if encoded == u32::MAX {
            return Ok(i32::MIN);
        }
        let is_negative = (encoded & 1) != 0;
        let mut shifted = encoded >> 1;
        if is_negative {
            Ok(-((shifted + 1) as i32))
        } else {
            Ok(shifted as i32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_primitives() {
        let bytes = vec![0x78, 0x56, 0x34, 0x12];
        let mut r = BinaryReader::new(Cursor::new(bytes), false, Endianness::Little);
        assert_eq!(r.read_u32().unwrap(), 0x12345678);

        let bytes = vec![0x12, 0x34, 0x56, 0x78];
        let mut r = BinaryReader::new(Cursor::new(bytes), false, Endianness::Big);
        assert_eq!(r.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_compressed_uint32() {
        // Less than 0x80 (encoded in 1 byte).
        let bytes = vec![0x3F];
        let mut r = BinaryReader::new(Cursor::new(bytes), false, Endianness::Little);
        assert_eq!(r.read_compressed_uint32().unwrap(), 0x3F);

        // Between 0x80 and 0xC0 (encoded in 2 bytes).
        let bytes = vec![0x80 | 0x12, 0x34];
        let mut r = BinaryReader::new(Cursor::new(bytes), false, Endianness::Little);
        assert_eq!(r.read_compressed_uint32().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_uleb128() {
        // ULEB128 of 624485 is [0xE5, 0x8E, 0x26].
        let bytes = vec![0xE5, 0x8E, 0x26];
        let mut r = BinaryReader::new(Cursor::new(bytes), false, Endianness::Little);
        assert_eq!(r.read_uleb128().unwrap(), 624485);
    }
}
