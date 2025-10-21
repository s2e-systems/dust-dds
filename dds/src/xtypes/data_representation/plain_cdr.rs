use crate::xtypes::{
    data_representation::{endianness::EndiannessRead, read_write::Read},
    error::{XTypesError, XTypesResult},
};

pub enum CdrVersion {
    Version1,
    Version2,
}

struct Reader<'a> {
    buffer: &'a [u8],
    pos: usize,
    max_align: usize,
}

impl<'a> Reader<'a> {
    fn new(buffer: &'a [u8], max_align: usize) -> Self {
        Self {
            buffer,
            pos: 0,
            max_align,
        }
    }

    fn read_all(&mut self, length: usize) -> XTypesResult<&'a [u8]> {
        if self.pos + length > self.buffer.len() {
            return Err(XTypesError::InvalidData);
        }
        let ret = &self.buffer[self.pos..self.pos + length];
        self.pos += length;
        Ok(ret)
    }

    fn seek(&mut self, v: usize) {
        self.pos += v
    }

    fn seek_padding(&mut self, alignment: usize) {
        let alignment = usize::min(alignment, self.max_align);
        let mask = alignment - 1;
        self.seek(((self.pos + mask) & !mask) - self.pos)
    }
}

impl<'a> Read for Reader<'a> {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]> {
        self.read_all(size)
    }
}

pub struct PlainCdrDecoder<'a, E: EndiannessRead> {
    reader: Reader<'a>,
    _endianness: E,
}

impl<'a, E: EndiannessRead> PlainCdrDecoder<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E, version: CdrVersion) -> Self {
        let max_align = match version {
            CdrVersion::Version1 => 8,
            CdrVersion::Version2 => 4,
        };
        Self {
            reader: Reader::new(buffer, max_align),
            _endianness: endianness,
        }
    }

    pub fn read_boolean(&mut self) -> XTypesResult<bool> {
        self.reader.seek_padding(1);
        E::read_bool(&mut self.reader)
    }

    pub fn read_i8(&mut self) -> XTypesResult<i8> {
        self.reader.seek_padding((i8::BITS / 8) as usize);
        E::read_i8(&mut self.reader)
    }

    pub fn read_u8(&mut self) -> XTypesResult<u8> {
        self.reader.seek_padding((u8::BITS / 8) as usize);
        E::read_u8(&mut self.reader)
    }

    pub fn read_i16(&mut self) -> XTypesResult<i16> {
        self.reader.seek_padding((i16::BITS / 8) as usize);
        E::read_i16(&mut self.reader)
    }

    pub fn read_u16(&mut self) -> XTypesResult<u16> {
        self.reader.seek_padding((u16::BITS / 8) as usize);
        E::read_u16(&mut self.reader)
    }

    pub fn read_i32(&mut self) -> XTypesResult<i32> {
        self.reader.seek_padding((i32::BITS / 8) as usize);
        E::read_i32(&mut self.reader)
    }

    pub fn read_u32(&mut self) -> XTypesResult<u32> {
        self.reader.seek_padding((u32::BITS / 8) as usize);
        E::read_u32(&mut self.reader)
    }

    pub fn read_i64(&mut self) -> XTypesResult<i64> {
        self.reader.seek_padding((i64::BITS / 8) as usize);
        E::read_i64(&mut self.reader)
    }

    pub fn read_u64(&mut self) -> XTypesResult<u64> {
        self.reader.seek_padding((u64::BITS / 8) as usize);
        E::read_u64(&mut self.reader)
    }

    pub fn read_f32(&mut self) -> XTypesResult<f32> {
        self.reader.seek_padding(4);
        E::read_f32(&mut self.reader)
    }

    pub fn read_f64(&mut self) -> XTypesResult<f64> {
        self.reader.seek_padding(8);
        E::read_f64(&mut self.reader)
    }
    
    pub fn read_char(&mut self) -> XTypesResult<char> {
        self.reader.seek_padding(1);
        E::read_char(&mut self.reader)
    }
}
