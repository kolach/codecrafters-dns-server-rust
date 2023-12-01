use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("width must be between 1 and 8 (was {0})")]
    BitsWidth(u8),
    #[error("not enough space to write (offset {offset:?}, width {width:?})")]
    BitsWrite { offset: u8, width: u8 },
    #[error("not enough space to read (offset {offset:?}, width {width:?})")]
    BitsRead { offset: u8, width: u8 },
    #[error(
        "not enough bytes to read (offset {offset:?}, read_len {read_len:?}, buf_len {buf_len:?})"
    )]
    Read {
        offset: usize,
        read_len: usize,
        buf_len: usize,
    },
}

pub struct Encoder<'a> {
    offset: usize,
    buf: &'a mut Vec<u8>,
}

impl<'a> Encoder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self { offset: 0, buf }
    }

    pub fn write_slice(&mut self, b: &[u8]) {
        if self.buf.len() - self.offset > b.len() {
            self.buf.copy_from_slice(b);
            self.offset += b.len();
        } else {
            self.buf.extend_from_slice(b);
            self.offset = self.buf.len();
        }
    }

    pub fn write_u16(&mut self, v: u16) {
        self.write_slice(&v.to_be_bytes())
    }

    pub fn write_bits<F>(&mut self, mut func: F) -> Result<(), Error>
    where
        F: FnMut(&mut BitEncoder) -> Result<(), Error>,
    {
        let mut byte: u8 = 0;
        let mut bit_enc = BitEncoder::new(&mut byte);
        func(&mut bit_enc)?;
        self.write_slice(&[byte]);
        Ok(())
    }
}

pub struct BitEncoder<'a> {
    data: &'a mut u8,
    offset: u8,
}

impl<'a> BitEncoder<'a> {
    // Create a new BitEncoder
    pub fn new(data: &'a mut u8) -> Self {
        BitEncoder { data, offset: 0 }
    }

    // Method to emit bits into the byte
    pub fn write(&mut self, value: u8, width: u8) -> Result<(), Error> {
        if width == 0 || width > 8 {
            return Err(Error::BitsWidth(width));
        }
        if self.offset + width > 8 {
            return Err(Error::BitsWrite {
                offset: self.offset,
                width,
            });
        }
        // Shift the value to its correct position and mask out unnecessary bits
        let shifted_value = (value & ((1 << width) - 1)) << (8 - self.offset - width);
        // Combine the value with the current data
        *self.data |= shifted_value;

        // Update the offset
        self.offset += width;
        Ok(())
    }
}

pub struct BitDecoder<'a> {
    data: &'a u8,
    offset: u8,
}

impl<'a> BitDecoder<'a> {
    // Create a new BitDecoder
    pub fn new(data: &'a u8) -> Self {
        BitDecoder { data, offset: 0 }
    }

    // Method to read bits from the byte
    pub fn read(&mut self, width: u8) -> Result<u8, Error> {
        if width == 0 || width > 8 {
            return Err(Error::BitsWidth(width));
        }
        if self.offset + width > 8 {
            return Err(Error::BitsRead {
                offset: self.offset,
                width,
            });
        }
        // Calculate the mask for the required bits
        let mask = ((1 << width) - 1) << (8 - self.offset - width);
        // Extract and shift the bits to the rightmost positions
        let result = (*self.data & mask) >> (8 - self.offset - width);
        // Update the offset
        self.offset += width;
        Ok(result)
    }
}

pub struct Decoder<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], Error> {
        if self.offset + len > self.buf.len() {
            return Err(Error::Read {
                offset: self.offset,
                buf_len: self.buf.len(),
                read_len: len,
            });
        }
        let res = &self.buf[self.offset..self.offset + len];
        self.offset += len;
        Ok(res)
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        let b = self.read_slice(2)?;
        let res = u16::from_be_bytes([b[0], b[1]]);
        Ok(res)
    }

    pub fn read_bits<F>(&mut self, mut func: F) -> Result<(), Error>
    where
        F: FnMut(&mut BitDecoder) -> Result<(), Error>,
    {
        let b = self.read_slice(1)?;
        let mut bit_dec = BitDecoder::new(&b[0]);
        func(&mut bit_dec)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::encoder::{Encoder, Error};

    use super::{BitDecoder, BitEncoder, Decoder};

    #[derive(Debug, Default, PartialEq)]
    struct Header {
        id: u16,    // 2 bytes
        qr: u8,     // 1 bit
        opcode: u8, // 4 bits
        aa: u8,     // 1 bit
        tc: u8,     // 1 bit
        rd: u8,     // 1 bit
    }

    impl Header {
        fn from_bytes<'a>(buf: &'a [u8]) -> Result<Self, Error> {
            let mut res = Header::default();
            let mut dec = Decoder::new(&buf);

            res.id = dec.read_u16()?;
            dec.read_bits(|br| {
                res.qr = br.read(1)?;
                res.opcode = br.read(4)?;
                res.aa = br.read(1)?;
                res.tc = br.read(1)?;
                res.rd = br.read(1)?;
                Ok(())
            })?;
            Ok(res)
        }

        pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
            let mut buf = Vec::new();
            let mut enc = Encoder::new(&mut buf);

            enc.write_u16(self.id);
            enc.write_bits(|bw| {
                bw.write(self.qr, 1)?;
                bw.write(self.opcode, 4)?;
                bw.write(self.aa, 1)?;
                bw.write(self.tc, 1)?;
                bw.write(self.rd, 1)?;
                Ok(())
            })?;
            Ok(buf)
        }
    }

    #[test]
    fn test_encode_decode() {
        let header = Header {
            id: 100,
            qr: 1,
            opcode: 7,
            aa: 1,
            tc: 0,
            rd: 1,
        };
        let header_bytes = header.to_bytes();
        assert!(header_bytes.is_ok());
        let header_bytes = header_bytes.unwrap();
        let header_from_bytes = Header::from_bytes(&header_bytes);
        assert!(header_from_bytes.is_ok());
        let header_from_bytes = header_from_bytes.unwrap();
        assert_eq!(header, header_from_bytes);
    }

    #[test]
    fn test_bit_encoder_decoder() {
        let mut byte: u8 = 0;
        let mut enc = BitEncoder::new(&mut byte);
        assert_eq!(enc.write(1, 1), Ok(()));
        assert_eq!(enc.write(7, 4), Ok(()));
        assert_eq!(enc.write(1, 1), Ok(()));
        assert_eq!(enc.write(0, 1), Ok(()));
        assert_eq!(enc.write(1, 1), Ok(()));

        let mut dec = BitDecoder::new(&byte);
        assert_eq!(dec.read(1), Ok(1));
        assert_eq!(dec.read(4), Ok(7));
        assert_eq!(dec.read(1), Ok(1));
        assert_eq!(dec.read(1), Ok(0));
        assert_eq!(dec.read(1), Ok(1));
    }
}
