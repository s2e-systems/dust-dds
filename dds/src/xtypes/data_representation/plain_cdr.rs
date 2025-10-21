use crate::xtypes::{
    data_representation::{
        deserialize::{PrimitiveTypeDeserialize, PrimitiveTypeDeserializer},
        endianness::EndiannessRead,
        read_write::Read,
    },
    error::{XTypesError, XTypesResult},
};

struct Reader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, pos: 0 }
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

    pub fn seek_padding(&mut self, alignment: usize) {
        let mask = alignment - 1;
        self.seek(((self.pos + mask) & !mask) - self.pos)
    }
}

impl<'a> Read for Reader<'a> {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]> {
        self.read_all(size)
    }
}

pub struct PlainCdr1Decoder<'a, E: EndiannessRead> {
    reader: Reader<'a>,
    _endianness: E,
}

impl<'a, E: EndiannessRead> PlainCdr1Decoder<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: Reader::new(buffer),
            _endianness: endianness,
        }
    }
}

impl<'a, E: EndiannessRead> PrimitiveTypeDeserializer for PlainCdr1Decoder<'a, E> {
    fn deserialize_u8(&mut self) -> XTypesResult<u8> {
        self.reader.seek_padding((u8::BITS / 8) as usize);
        E::read_u8(&mut self.reader)
    }

    fn deserialize_u16(&mut self) -> XTypesResult<u16> {
        self.reader.seek_padding((u16::BITS / 8) as usize);
        E::read_u16(&mut self.reader)
    }

    fn deserialize_u32(&mut self) -> XTypesResult<u32> {
        self.reader.seek_padding((u32::BITS / 8) as usize);
        E::read_u32(&mut self.reader)
    }

    fn deserialize_u64(&mut self) -> XTypesResult<u64> {
        self.reader.seek_padding((u64::BITS / 8) as usize);
        E::read_u64(&mut self.reader)
    }

    fn deserialize_i8(&mut self) -> XTypesResult<i8> {
        self.reader.seek_padding((i8::BITS / 8) as usize);
        E::read_i8(&mut self.reader)
    }

    fn deserialize_i16(&mut self) -> XTypesResult<i16> {
        self.reader.seek_padding((i16::BITS / 8) as usize);
        E::read_i16(&mut self.reader)
    }

    fn deserialize_i32(&mut self) -> XTypesResult<i32> {
        self.reader.seek_padding((i32::BITS / 8) as usize);
        E::read_i32(&mut self.reader)
    }

    fn deserialize_i64(&mut self) -> XTypesResult<i64> {
        self.reader.seek_padding((i64::BITS / 8) as usize);
        E::read_i64(&mut self.reader)
    }

    fn deserialize_f32(&mut self) -> XTypesResult<f32> {
        self.reader.seek_padding(4);
        E::read_f32(&mut self.reader)
    }

    fn deserialize_f64(&mut self) -> XTypesResult<f64> {
        self.reader.seek_padding(8);
        E::read_f64(&mut self.reader)
    }

    fn deserialize_boolean(&mut self) -> XTypesResult<bool> {
        self.reader.seek_padding(1);
        E::read_bool(&mut self.reader)
    }

    fn deserialize_char(&mut self) -> XTypesResult<char> {
        self.reader.seek_padding(1);
        E::read_char(&mut self.reader)
    }

    fn deserialize_primitive_type_sequence<T: PrimitiveTypeDeserialize>(
        &mut self,
    ) -> XTypesResult<Vec<T>> {
        let length = u32::deserialize(self)?;
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(T::deserialize(self)?);
        }
        Ok(values)
    }
}

pub struct PlainCdr2Deserializer<'a, E: EndiannessRead> {
    reader: Reader<'a>,
    _endianness: E,
}

impl<'a, E: EndiannessRead> PlainCdr2Deserializer<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: Reader::new(buffer),
            _endianness: endianness,
        }
    }
}

impl<'a, E: EndiannessRead> PrimitiveTypeDeserializer for PlainCdr2Deserializer<'a, E> {
    fn deserialize_u8(&mut self) -> XTypesResult<u8> {
        self.reader.seek_padding((u8::BITS / 8) as usize);
        E::read_u8(&mut self.reader)
    }

    fn deserialize_u16(&mut self) -> XTypesResult<u16> {
        self.reader.seek_padding((u16::BITS / 8) as usize);
        E::read_u16(&mut self.reader)
    }

    fn deserialize_u32(&mut self) -> XTypesResult<u32> {
        self.reader.seek_padding((u32::BITS / 8) as usize);
        E::read_u32(&mut self.reader)
    }

    fn deserialize_u64(&mut self) -> XTypesResult<u64> {
        self.reader.seek_padding(4);
        E::read_u64(&mut self.reader)
    }

    fn deserialize_i8(&mut self) -> XTypesResult<i8> {
        self.reader.seek_padding((i8::BITS / 8) as usize);
        E::read_i8(&mut self.reader)
    }

    fn deserialize_i16(&mut self) -> XTypesResult<i16> {
        self.reader.seek_padding((i16::BITS / 8) as usize);
        E::read_i16(&mut self.reader)
    }

    fn deserialize_i32(&mut self) -> XTypesResult<i32> {
        self.reader.seek_padding((i32::BITS / 8) as usize);
        E::read_i32(&mut self.reader)
    }

    fn deserialize_i64(&mut self) -> XTypesResult<i64> {
        self.reader.seek_padding(4);
        E::read_i64(&mut self.reader)
    }

    fn deserialize_f32(&mut self) -> XTypesResult<f32> {
        self.reader.seek_padding(4);
        E::read_f32(&mut self.reader)
    }

    fn deserialize_f64(&mut self) -> XTypesResult<f64> {
        self.reader.seek_padding(4);
        E::read_f64(&mut self.reader)
    }

    fn deserialize_boolean(&mut self) -> XTypesResult<bool> {
        self.reader.seek_padding(1);
        E::read_bool(&mut self.reader)
    }

    fn deserialize_char(&mut self) -> XTypesResult<char> {
        self.reader.seek_padding(1);
        E::read_char(&mut self.reader)
    }

    fn deserialize_primitive_type_sequence<T: PrimitiveTypeDeserialize>(
        &mut self,
    ) -> XTypesResult<Vec<T>> {
        let length = u32::deserialize(self)?;
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(T::deserialize(self)?);
        }
        Ok(values)
    }
}
