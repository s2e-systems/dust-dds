use core::str;
use crate::{
    deserialize::XTypesDeserialize,
    deserializer::{
        DeserializeAppendableStruct, DeserializeCollection, DeserializeFinalStruct,
        DeserializeMutableStruct, XTypesDeserializer,
    },
    error::XcdrError,
};

const PID_SENTINEL: u16 = 1;

pub struct Xcdr1BeDeserializer<'a> {
    reader: Reader<'a>,
}

impl<'a> Xcdr1BeDeserializer<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            reader: Reader::new(buffer),
        }
    }
}

pub struct Xcdr1LeDeserializer<'a> {
    reader: Reader<'a>,
}

impl<'a> Xcdr1LeDeserializer<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            reader: Reader::new(buffer),
        }
    }
}

pub struct Xcdr2BeDeserializer<'a> {
    reader: Reader<'a>,
}

impl<'a> Xcdr2BeDeserializer<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            reader: Reader::new(buffer),
        }
    }
}

pub struct Xcdr2LeDeserializer<'a> {
    reader: Reader<'a>,
}

impl<'a> Xcdr2LeDeserializer<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            reader: Reader::new(buffer),
        }
    }
}

struct Reader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, pos: 0 }
    }
    fn buffer(&self) -> &'a [u8] {
        &self.buffer[self.pos..]
    }
    fn read<const N: usize>(&mut self) -> Result<&'a [u8; N], XcdrError> {
        if self.pos + N > self.buffer.len() {
            return Err(XcdrError::OutOfMemory);
        }
        let r = core::convert::TryFrom::try_from(&self.buffer[self.pos..self.pos + N])
            .map_err(|_| XcdrError::InvalidData);
        self.pos += N;
        r
    }
    fn seek(&mut self, v: usize) {
        self.pos += v
    }

    fn seek_padding(&mut self, alignment: usize) {
        let mask = alignment - 1;
        self.seek((self.pos + mask & !mask) - self.pos)
    }
}

fn read_with_padding_v1<const N: usize>(reader: &mut Reader) -> Result<[u8; N], XcdrError> {
    reader.seek_padding(N);
    reader.read().cloned()
}

fn read_with_padding_v2<const N: usize>(reader: &mut Reader) -> Result<[u8; N], XcdrError> {
    reader.seek_padding(core::cmp::min(N, 4));
    reader.read().cloned()
}

fn seek_to_pid_be(reader: &mut Reader, pid: u16) -> Result<(), XcdrError> {
    loop {
        let current_pid = u16::from_be_bytes(*reader.read()?);
        let length = u16::from_be_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(());
        } else if current_pid == PID_SENTINEL {
            return Err(XcdrError::PidNotFound(pid));
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn seek_to_pid_le(reader: &mut Reader, pid: u16) -> Result<(), XcdrError> {
    loop {
        let current_pid = u16::from_le_bytes(*reader.read()?);
        let length = u16::from_le_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(());
        } else if current_pid == PID_SENTINEL {
            return Err(XcdrError::PidNotFound(pid));
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn seek_to_optional_pid_be(reader: &mut Reader, pid: u16) -> Result<bool, XcdrError> {
    loop {
        let current_pid = u16::from_be_bytes(*reader.read()?);
        let length = u16::from_be_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(true);
        } else if current_pid == PID_SENTINEL {
            return Ok(false);
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn seek_to_optional_pid_le(reader: &mut Reader, pid: u16) -> Result<bool, XcdrError> {
    loop {
        let current_pid = u16::from_le_bytes(*reader.read()?);
        let length = u16::from_le_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(true);
        } else if current_pid == PID_SENTINEL {
            return Ok(false);
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn into_bool(v: u8) -> Result<bool, XcdrError> {
    match v {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(XcdrError::InvalidData),
    }
}

struct DelimitedCdrDecoder<'a, D> {
    deserializer: &'a mut D,
}
impl<'de, D> DeserializeAppendableStruct<'de> for DelimitedCdrDecoder<'_, D>
where
    for<'a> &'a mut D: XTypesDeserializer<'de>,
{
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<T, XcdrError> {
        u32::deserialize(&mut *self.deserializer)?;
        T::deserialize(&mut *self.deserializer)
    }
}

struct PlCdrBeDecoder<'a> {
    buffer: &'a [u8],
}

impl<'de> DeserializeMutableStruct<'de> for PlCdrBeDecoder<'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<T, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        seek_to_pid_be(&mut reader, pid)?;
        T::deserialize(&mut Xcdr1BeDeserializer { reader })
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<Option<T>, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        Ok(if seek_to_optional_pid_be(&mut reader, pid)? {
            Some(T::deserialize(&mut Xcdr1BeDeserializer { reader })?)
        } else {
            None
        })
    }
}

struct PlCdrLeDecoder<'a> {
    buffer: &'a [u8],
}

impl<'de> DeserializeMutableStruct<'de> for PlCdrLeDecoder<'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<T, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        seek_to_pid_le(&mut reader, pid)?;
        T::deserialize(&mut Xcdr1LeDeserializer { reader })
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<Option<T>, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        Ok(if seek_to_optional_pid_le(&mut reader, pid)? {
            Some(T::deserialize(&mut Xcdr1BeDeserializer { reader })?)
        } else {
            None
        })
    }
}

struct PlCdr2BeDecoder<'a> {
    buffer: &'a [u8],
}

impl<'de> DeserializeMutableStruct<'de> for PlCdr2BeDecoder<'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<T, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        seek_to_pid_be(&mut reader, pid)?;
        T::deserialize(&mut Xcdr2BeDeserializer { reader })
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<Option<T>, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        Ok(if seek_to_optional_pid_be(&mut reader, pid)? {
            Some(T::deserialize(&mut Xcdr2BeDeserializer { reader })?)
        } else {
            None
        })
    }
}

struct PlCdr2LeDecoder<'a> {
    buffer: &'a [u8],
}

impl<'de> DeserializeMutableStruct<'de> for PlCdr2LeDecoder<'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u16,
        _name: &str,
    ) -> Result<T, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        seek_to_pid_le(&mut reader, pid)?;
        T::deserialize(&mut Xcdr2LeDeserializer { reader })
    }
    
    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(&mut self, pid: u16, _name: &str) -> Result<Option<T>, XcdrError> {
        let mut reader = Reader::new(self.buffer);
        Ok(if seek_to_optional_pid_le(&mut reader, pid)? {
            Some(T::deserialize(&mut Xcdr2BeDeserializer { reader })?)
        } else {
            None
        })
    }
}

struct SeqDeserializer<'a, D> {
    deserializer: &'a mut D,
}
impl<'de, D> DeserializeCollection<'de> for SeqDeserializer<'_, D>
where
    for<'a> &'a mut D: XTypesDeserializer<'de>,
{
    fn deserialize_element<T: XTypesDeserialize<'de>>(&mut self) -> Result<T, XcdrError> {
        T::deserialize(&mut *self.deserializer)
    }
}

struct PlainCdrBeDecoder<'a, 'de> {
    deserializer: &'a mut Xcdr1BeDeserializer<'de>,
}
impl<'de> DeserializeAppendableStruct<'de> for PlainCdrBeDecoder<'_, 'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<T, XcdrError> {
        T::deserialize(&mut *self.deserializer)
    }
}

impl<'de> DeserializeFinalStruct<'de> for PlainCdrBeDecoder<'_, 'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<T, XcdrError> {
        T::deserialize(&mut *self.deserializer)
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<Option<T>, XcdrError> {
        self.deserializer.reader.seek_padding(4);
        let _pid = u16::deserialize(&mut *self.deserializer)?;
        let length = u16::deserialize(&mut *self.deserializer)?;
        if length == 0 {
            Ok(None)
        } else {
            Ok(Some(T::deserialize(&mut *self.deserializer)?))
        }
    }
}

struct PlainCdrLeDecoder<'a, 'de> {
    deserializer: &'a mut Xcdr1LeDeserializer<'de>,
}
impl<'de> DeserializeAppendableStruct<'de> for PlainCdrLeDecoder<'_, 'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<T, XcdrError> {
        T::deserialize(&mut *self.deserializer)
    }
}

impl<'de> DeserializeFinalStruct<'de> for PlainCdrLeDecoder<'_, 'de> {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<T, XcdrError> {
        T::deserialize(&mut *self.deserializer)
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<Option<T>, XcdrError> {
        self.deserializer.reader.seek_padding(4);
        let _pid = u16::deserialize(&mut *self.deserializer)?;
        let length = u16::deserialize(&mut *self.deserializer)?;
        if length == 0 {
            Ok(None)
        } else {
            Ok(Some(T::deserialize(&mut *self.deserializer)?))
        }
    }
}

struct PlainCdr2Decoder<'a, D> {
    deserializer: &'a mut D,
}
impl<'de, D> DeserializeFinalStruct<'de> for PlainCdr2Decoder<'_, D>
where
    for<'a> &'a mut D: XTypesDeserializer<'de>,
{
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<T, XcdrError> {
        T::deserialize(&mut *self.deserializer)
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        _name: &str,
    ) -> Result<Option<T>, XcdrError> {
        if bool::deserialize(&mut *self.deserializer)? {
            Ok(Some(T::deserialize(&mut *self.deserializer)?))
        } else {
            Ok(None)
        }
    }
}

impl<'de> XTypesDeserializer<'de> for &mut Xcdr1BeDeserializer<'de> {
    fn deserialize_final_struct(self) -> Result<impl DeserializeFinalStruct<'de>, XcdrError> {
        Ok(PlainCdrBeDecoder { deserializer: self })
    }
    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, XcdrError> {
        Ok(PlainCdrBeDecoder { deserializer: self })
    }
    fn deserialize_mutable_struct(self) -> Result<impl DeserializeMutableStruct<'de>, XcdrError> {
        Ok(PlCdrBeDecoder {
            buffer: self.reader.buffer(),
        })
    }
    fn deserialize_array(self) -> Result<impl DeserializeCollection<'de>, XcdrError> {
        Ok(SeqDeserializer { deserializer: self })
    }

    fn deserialize_boolean(self) -> Result<bool, XcdrError> {
        into_bool(self.deserialize_uint8()?)
    }
    fn deserialize_uint8(self) -> Result<u8, XcdrError> {
        Ok(u8::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_uint16(self) -> Result<u16, XcdrError> {
        Ok(u16::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_uint32(self) -> Result<u32, XcdrError> {
        Ok(u32::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_uint64(self) -> Result<u64, XcdrError> {
        Ok(u64::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int8(self) -> Result<i8, XcdrError> {
        Ok(i8::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int16(self) -> Result<i16, XcdrError> {
        Ok(i16::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int32(self) -> Result<i32, XcdrError> {
        Ok(i32::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int64(self) -> Result<i64, XcdrError> {
        Ok(i64::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_float32(self) -> Result<f32, XcdrError> {
        Ok(f32::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_float64(self) -> Result<f64, XcdrError> {
        Ok(f64::from_be_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_char8(self) -> Result<char, XcdrError> {
        Ok(self.deserialize_uint8()? as char)
    }
    fn deserialize_string(self) -> Result<&'de str, XcdrError> {
        let length = u32::deserialize(&mut *self)? as usize;
        str::from_utf8(&self.reader.buffer()[..length - 1]).map_err(|_| XcdrError::InvalidData)
    }
    fn deserialize_byte_sequence(self) -> Result<&'de [u8], XcdrError> {
        let length = self.deserialize_uint32()? as usize;
        Ok(&self.reader.buffer()[..length])
    }
    fn deserialize_byte_array<const N: usize>(self) -> Result<&'de [u8; N], XcdrError> {
        self.reader.read()
    }
}

impl<'de> XTypesDeserializer<'de> for &mut Xcdr1LeDeserializer<'de> {
    fn deserialize_final_struct(self) -> Result<impl DeserializeFinalStruct<'de>, XcdrError> {
        Ok(PlainCdrLeDecoder { deserializer: self })
    }
    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, XcdrError> {
        Ok(PlainCdrLeDecoder { deserializer: self })
    }
    fn deserialize_mutable_struct(self) -> Result<impl DeserializeMutableStruct<'de>, XcdrError> {
        Ok(PlCdrLeDecoder {
            buffer: &self.reader.buffer(),
        })
    }
    fn deserialize_array(self) -> Result<impl DeserializeCollection<'de>, XcdrError> {
        Ok(SeqDeserializer { deserializer: self })
    }

    fn deserialize_boolean(self) -> Result<bool, XcdrError> {
        into_bool(self.deserialize_uint8()?)
    }
    fn deserialize_uint8(self) -> Result<u8, XcdrError> {
        Ok(u8::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_uint16(self) -> Result<u16, XcdrError> {
        Ok(u16::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_uint32(self) -> Result<u32, XcdrError> {
        Ok(u32::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_uint64(self) -> Result<u64, XcdrError> {
        Ok(u64::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int8(self) -> Result<i8, XcdrError> {
        Ok(i8::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int16(self) -> Result<i16, XcdrError> {
        Ok(i16::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int32(self) -> Result<i32, XcdrError> {
        Ok(i32::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_int64(self) -> Result<i64, XcdrError> {
        Ok(i64::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_float32(self) -> Result<f32, XcdrError> {
        Ok(f32::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_float64(self) -> Result<f64, XcdrError> {
        Ok(f64::from_le_bytes(read_with_padding_v1(&mut self.reader)?))
    }
    fn deserialize_char8(self) -> Result<char, XcdrError> {
        Ok(self.deserialize_uint8()? as char)
    }
    fn deserialize_string(self) -> Result<&'de str, XcdrError> {
        let length = u32::deserialize(&mut *self)? as usize;
        str::from_utf8(&self.reader.buffer()[..length - 1]).map_err(|_| XcdrError::InvalidData)
    }
    fn deserialize_byte_sequence(self) -> Result<&'de [u8], XcdrError> {
        let length = self.deserialize_uint32()? as usize;
        Ok(&self.reader.buffer()[..length])
    }
    fn deserialize_byte_array<const N: usize>(self) -> Result<&'de [u8; N], XcdrError> {
        self.reader.read()
    }
}

impl<'de> XTypesDeserializer<'de> for &mut Xcdr2BeDeserializer<'de> {
    fn deserialize_final_struct(self) -> Result<impl DeserializeFinalStruct<'de>, XcdrError> {
        Ok(PlainCdr2Decoder { deserializer: self })
    }
    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, XcdrError> {
        Ok(DelimitedCdrDecoder { deserializer: self })
    }
    fn deserialize_mutable_struct(self) -> Result<impl DeserializeMutableStruct<'de>, XcdrError> {
        Ok(PlCdr2BeDecoder {
            buffer: &self.reader.buffer(),
        })
    }
    fn deserialize_array(self) -> Result<impl DeserializeCollection<'de>, XcdrError> {
        Ok(SeqDeserializer { deserializer: self })
    }

    fn deserialize_boolean(self) -> Result<bool, XcdrError> {
        into_bool(self.deserialize_uint8()?)
    }
    fn deserialize_int8(self) -> Result<i8, XcdrError> {
        Ok(i8::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int16(self) -> Result<i16, XcdrError> {
        Ok(i16::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int32(self) -> Result<i32, XcdrError> {
        Ok(i32::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int64(self) -> Result<i64, XcdrError> {
        Ok(i64::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint8(self) -> Result<u8, XcdrError> {
        Ok(u8::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint16(self) -> Result<u16, XcdrError> {
        Ok(u16::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint32(self) -> Result<u32, XcdrError> {
        Ok(u32::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint64(self) -> Result<u64, XcdrError> {
        Ok(u64::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_float32(self) -> Result<f32, XcdrError> {
        Ok(f32::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_float64(self) -> Result<f64, XcdrError> {
        Ok(f64::from_be_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_char8(self) -> Result<char, XcdrError> {
        Ok(self.deserialize_uint8()? as char)
    }
    fn deserialize_string(self) -> Result<&'de str, XcdrError> {
        let length = u32::deserialize(&mut *self)? as usize;
        str::from_utf8(&self.reader.buffer()[..length - 1]).map_err(|_| XcdrError::InvalidData)
    }
    fn deserialize_byte_sequence(self) -> Result<&'de [u8], XcdrError> {
        let length = self.deserialize_uint32()? as usize;
        Ok(&self.reader.buffer()[..length])
    }
    fn deserialize_byte_array<const N: usize>(self) -> Result<&'de [u8; N], XcdrError> {
        self.reader.read()
    }
}

impl<'de> XTypesDeserializer<'de> for &mut Xcdr2LeDeserializer<'de> {
    fn deserialize_final_struct(self) -> Result<impl DeserializeFinalStruct<'de>, XcdrError> {
        Ok(PlainCdr2Decoder { deserializer: self })
    }
    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, XcdrError> {
        Ok(DelimitedCdrDecoder {
            deserializer: &mut *self,
        })
    }
    fn deserialize_mutable_struct(self) -> Result<impl DeserializeMutableStruct<'de>, XcdrError> {
        Ok(PlCdr2LeDecoder {
            buffer: &self.reader.buffer(),
        })
    }
    fn deserialize_array(self) -> Result<impl DeserializeCollection<'de>, XcdrError> {
        Ok(SeqDeserializer { deserializer: self })
    }

    fn deserialize_boolean(self) -> Result<bool, XcdrError> {
        into_bool(self.deserialize_uint8()?)
    }
    fn deserialize_uint8(self) -> Result<u8, XcdrError> {
        Ok(u8::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint16(self) -> Result<u16, XcdrError> {
        Ok(u16::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint32(self) -> Result<u32, XcdrError> {
        Ok(u32::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_uint64(self) -> Result<u64, XcdrError> {
        Ok(u64::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int8(self) -> Result<i8, XcdrError> {
        Ok(i8::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int16(self) -> Result<i16, XcdrError> {
        Ok(i16::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int32(self) -> Result<i32, XcdrError> {
        Ok(i32::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_int64(self) -> Result<i64, XcdrError> {
        Ok(i64::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_float32(self) -> Result<f32, XcdrError> {
        Ok(f32::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_float64(self) -> Result<f64, XcdrError> {
        Ok(f64::from_le_bytes(read_with_padding_v2(&mut self.reader)?))
    }
    fn deserialize_char8(self) -> Result<char, XcdrError> {
        Ok(self.deserialize_uint8()? as char)
    }
    fn deserialize_string(self) -> Result<&'de str, XcdrError> {
        let length = u32::deserialize(&mut *self)? as usize;
        str::from_utf8(&self.reader.buffer()[..length - 1]).map_err(|_| XcdrError::InvalidData)
    }
    fn deserialize_byte_sequence(self) -> Result<&'de [u8], XcdrError> {
        let length = self.deserialize_uint32()? as usize;
        Ok(&self.reader.buffer()[..length])
    }
    fn deserialize_byte_array<const N: usize>(self) -> Result<&'de [u8; N], XcdrError> {
        self.reader.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_reader() {
        let buffer = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut reader = Reader::new(buffer);
        reader.seek(1);
        reader.seek_padding(4);
        assert_eq!(reader.pos, 4);
        let mut reader = Reader::new(buffer);
        reader.seek(2);
        reader.seek_padding(4);
        assert_eq!(reader.pos, 4);
        let mut reader = Reader::new(buffer);
        reader.seek(3);
        reader.seek_padding(4);
        assert_eq!(reader.pos, 4);
        let mut reader = Reader::new(buffer);
        reader.seek(4);
        reader.seek_padding(4);
        assert_eq!(reader.pos, 4);
        let mut reader = Reader::new(buffer);
        reader.seek(5);
        reader.seek_padding(4);
        assert_eq!(reader.pos, 8);

        let mut reader = Reader::new(buffer);
        reader.seek(1);
        reader.seek_padding(8);
        assert_eq!(reader.pos, 8);
        let mut reader = Reader::new(buffer);
        reader.seek(2);
        reader.seek_padding(8);
        assert_eq!(reader.pos, 8);
        let mut reader = Reader::new(buffer);
        reader.seek(7);
        reader.seek_padding(8);
        assert_eq!(reader.pos, 8);
        let mut reader = Reader::new(buffer);
        reader.seek(8);
        reader.seek_padding(8);
        assert_eq!(reader.pos, 8);
        let mut reader = Reader::new(buffer);
        reader.seek(9);
        reader.seek_padding(8);
        assert_eq!(reader.pos, 16);
    }

    fn deserialize_v1_be<'de, T: XTypesDeserialize<'de>>(data: &'de [u8]) -> Result<T, XcdrError> {
        T::deserialize(&mut Xcdr1BeDeserializer {
            reader: Reader::new(data),
        })
    }
    fn deserialize_v1_le<'de, T: XTypesDeserialize<'de>>(data: &'de [u8]) -> Result<T, XcdrError> {
        T::deserialize(&mut Xcdr1LeDeserializer {
            reader: Reader::new(data),
        })
    }
    fn deserialize_v2_be<'de, T: XTypesDeserialize<'de>>(data: &'de [u8]) -> Result<T, XcdrError> {
        T::deserialize(&mut Xcdr2BeDeserializer {
            reader: Reader::new(data),
        })
    }
    fn deserialize_v2_le<'de, T: XTypesDeserialize<'de>>(data: &'de [u8]) -> Result<T, XcdrError> {
        T::deserialize(&mut Xcdr2LeDeserializer {
            reader: Reader::new(data),
        })
    }
    #[test]
    fn deserialize_bool() {
        let expected = Ok(true);
        assert_eq!(deserialize_v1_be(&[1]), expected);
        assert_eq!(deserialize_v1_le(&[1]), expected);
        assert_eq!(deserialize_v2_be(&[1]), expected);
        assert_eq!(deserialize_v2_le(&[1]), expected);
        let expected = Ok(false);
        assert_eq!(deserialize_v1_be(&[0]), expected);
        assert_eq!(deserialize_v1_le(&[0]), expected);
        assert_eq!(deserialize_v2_be(&[0]), expected);
        assert_eq!(deserialize_v2_le(&[0]), expected);
        let expected = Err(XcdrError::InvalidData);
        assert_eq!(deserialize_v1_be::<bool>(&[2]), expected);
        assert_eq!(deserialize_v1_le::<bool>(&[2]), expected);
        assert_eq!(deserialize_v2_be::<bool>(&[2]), expected);
        assert_eq!(deserialize_v2_le::<bool>(&[2]), expected);
    }
    #[test]
    fn deserialize_u8() {
        let expected = Ok(1_u8);
        assert_eq!(deserialize_v1_be(&[1]), expected);
        assert_eq!(deserialize_v1_le(&[1]), expected);
        assert_eq!(deserialize_v2_be(&[1]), expected);
        assert_eq!(deserialize_v2_le(&[1]), expected);
    }
    #[test]
    fn deserialize_u16() {
        let expected = Ok(1_u16);
        assert_eq!(deserialize_v1_be(&[0, 1]), expected);
        assert_eq!(deserialize_v1_le(&[1, 0]), expected);
        assert_eq!(deserialize_v2_be(&[0, 1]), expected);
        assert_eq!(deserialize_v2_le(&[1, 0]), expected);
    }
    #[test]
    fn deserialize_u32() {
        let expected = Ok(1_u32);
        assert_eq!(deserialize_v1_be(&[0, 0, 0, 1]), expected);
        assert_eq!(deserialize_v1_le(&[1, 0, 0, 0]), expected);
        assert_eq!(deserialize_v2_be(&[0, 0, 0, 1]), expected);
        assert_eq!(deserialize_v2_le(&[1, 0, 0, 0]), expected);
    }
    #[test]
    fn deserialize_u64() {
        let expected = Ok(7_u64);
        assert_eq!(deserialize_v1_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
        assert_eq!(deserialize_v1_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
        assert_eq!(deserialize_v2_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
        assert_eq!(deserialize_v2_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
    }
    #[test]
    fn deserialize_i8() {
        let expected = Ok(1_i8);
        assert_eq!(deserialize_v1_be(&[1]), expected);
        assert_eq!(deserialize_v1_le(&[1]), expected);
        assert_eq!(deserialize_v2_be(&[1]), expected);
        assert_eq!(deserialize_v2_le(&[1]), expected);
    }
    #[test]
    fn deserialize_i16() {
        let expected = Ok(1_i16);
        assert_eq!(deserialize_v1_be(&[0, 1]), expected);
        assert_eq!(deserialize_v1_le(&[1, 0]), expected);
        assert_eq!(deserialize_v2_be(&[0, 1]), expected);
        assert_eq!(deserialize_v2_le(&[1, 0]), expected);
    }
    #[test]
    fn deserialize_i32() {
        let expected = Ok(1_i32);
        assert_eq!(deserialize_v1_be(&[0, 0, 0, 1]), expected);
        assert_eq!(deserialize_v1_le(&[1, 0, 0, 0]), expected);
        assert_eq!(deserialize_v2_be(&[0, 0, 0, 1]), expected);
        assert_eq!(deserialize_v2_le(&[1, 0, 0, 0]), expected);
    }
    #[test]
    fn deserialize_i64() {
        let expected = Ok(7_i64);
        assert_eq!(deserialize_v1_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
        assert_eq!(deserialize_v1_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
        assert_eq!(deserialize_v2_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
        assert_eq!(deserialize_v2_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
    }
    #[test]
    fn deserialize_f32() {
        let expected = Ok(core::f32::MIN_POSITIVE);
        assert_eq!(deserialize_v1_be(&[0x00, 0x80, 0x00, 0x00]), expected);
        assert_eq!(deserialize_v1_le(&[0x00, 0x00, 0x80, 0x00]), expected);
        assert_eq!(deserialize_v2_be(&[0x00, 0x80, 0x00, 0x00]), expected);
        assert_eq!(deserialize_v2_le(&[0x00, 0x00, 0x80, 0x00]), expected);
    }
    #[test]
    fn deserialize_f64() {
        let expected = Ok(core::f64::MIN_POSITIVE);
        assert_eq!(deserialize_v1_be(&[0, 0x10, 0, 0, 0, 0, 0, 0]), expected);
        assert_eq!(deserialize_v1_le(&[0, 0, 0, 0, 0, 0, 0x10, 0]), expected);
        assert_eq!(deserialize_v2_be(&[0, 0x10, 0, 0, 0, 0, 0, 0]), expected);
        assert_eq!(deserialize_v2_le(&[0, 0, 0, 0, 0, 0, 0x10, 0]), expected);
    }
    #[test]
    fn deserialize_char() {
        let expected = Ok('A');
        assert_eq!(deserialize_v1_be(&[0x41]), expected);
        assert_eq!(deserialize_v1_le(&[0x41]), expected);
        assert_eq!(deserialize_v2_be(&[0x41]), expected);
        assert_eq!(deserialize_v2_le(&[0x41]), expected);
    }

    #[test]
    fn deserialize_string() {
        let expected = Ok("Hola");
        assert_eq!(
            deserialize_v1_be(&[
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]),
            expected
        );
        assert_eq!(
            deserialize_v1_le(&[
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_be(&[
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_le(&[
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]),
            expected
        );
    }

    #[test]
    fn deserialize_bytes() {
        let expected = Ok(&[1u8, 2, 3, 4, 5][..]);
        assert_eq!(
            deserialize_v1_be(&[
                0, 0, 0, 5, // length
                1, 2, 3, 4, 5 // data
            ]),
            expected
        );
        assert_eq!(
            deserialize_v1_le(&[
                5, 0, 0, 0, // length
                1, 2, 3, 4, 5 // data
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_be(&[
                0, 0, 0, 5, // length
                1, 2, 3, 4, 5 // data
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_le(&[
                5, 0, 0, 0, // length
                1, 2, 3, 4, 5 // data
            ]),
            expected
        );
    }

    #[test]
    fn deserialize_byte_array() {
        let expected = Ok(&[1u8, 2, 3, 4, 5]);
        assert_eq!(deserialize_v1_be(&[1, 2, 3, 4, 5, 77]), expected);
        assert_eq!(deserialize_v1_le(&[1, 2, 3, 4, 5, 77]), expected);
        assert_eq!(deserialize_v2_be(&[1, 2, 3, 4, 5, 77]), expected);
        assert_eq!(deserialize_v2_le(&[1, 2, 3, 4, 5, 77]), expected);
    }

    #[test]
    fn deserialize_sequence() {
        #[derive(Debug, PartialEq)]
        struct Atype(u8);
        impl<'de> XTypesDeserialize<'de> for Atype {
            fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
                Ok(Atype(deserializer.deserialize_uint8()?))
            }
        }

        let expected = Ok([Atype(1), Atype(2)]);
        assert_eq!(deserialize_v1_be(&[1, 2, 77]), expected);
    }

    #[derive(Debug, PartialEq)]
    //@extensibility(FINAL)
    struct FinalType {
        field_u16: u16,
        field_u64: u64,
    }
    impl<'de> XTypesDeserialize<'de> for FinalType {
        fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
            let mut deserializer = deserializer.deserialize_final_struct()?;
            Ok(FinalType {
                field_u16: deserializer.deserialize_field("field_u16")?,
                field_u64: deserializer.deserialize_field("field_u64")?,
            })
        }
    }

    #[test]
    fn deserialize_final_struct() {
        let expected = Ok(FinalType {
            field_u16: 7,
            field_u64: 9,
        });
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be::<FinalType>(&[
                0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]),
            expected
        );
        assert_eq!(
            deserialize_v1_le::<FinalType>(&[
                7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]),
            expected
        );
        // PLAIN_CDR2:
        assert_eq!(
            deserialize_v2_be::<FinalType>(&[
                0, 7, 0, 0, // field_u16 | padding (2 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_le::<FinalType>(&[
                7, 0, 0, 0, // field_u16 | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]),
            expected
        );
    }

    #[derive(Debug, PartialEq)]
    //@extensibility(FINAL)
    struct NestedFinalType {
        field_nested: FinalType,
        field_u8: u8,
    }
    impl<'de> XTypesDeserialize<'de> for NestedFinalType {
        fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
            let mut deserializer = deserializer.deserialize_final_struct()?;
            Ok(Self {
                field_nested: deserializer.deserialize_field("field_nested")?,
                field_u8: deserializer.deserialize_field("field_u8")?,
            })
        }
    }

    #[derive(Debug, PartialEq)]
    //@extensibility(FINAL)
    struct FinalOptionalType {
        field: u8,
        optional_field: Option<u16>,
    }

    impl<'de> XTypesDeserialize<'de> for FinalOptionalType {
        fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
            let mut d = deserializer.deserialize_final_struct()?;
            Ok(Self {
                field: d.deserialize_field("field")?,
                optional_field: d.deserialize_optional_field("optional_field")?,
            })
        }
    }
    #[test]
    fn deserialize_final_optional_struct() {
        let some = Ok(FinalOptionalType {
            field: 6,
            optional_field: Some(7),
        });
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be(&[
                6, 0, 0, 0, // u8 | padding
                0, 0, 0, 2, // HEADER (FLAGS+ID | length)
                0, 7 // optional_field value
            ]),
            some
        );
        assert_eq!(
            deserialize_v1_le(&[
                6, 0, 0, 0, // u8 | padding
                0, 0, 2, 0, // HEADER (FLAGS+ID | length)
                7, 0 // optional_field value
            ]),
            some
        );
        // PLAIN_CDR2:
        assert_eq!(
            deserialize_v2_be(&[
                6, 1, // u8 | boolean for option
                0, 7 // optional_field value
            ]),
            some
        );
        assert_eq!(
            deserialize_v2_le(&[
                6, 1, // u8 | boolean for option
                7, 0 // optional_field value
            ]),
            some
        );

        let none = Ok(FinalOptionalType {
            field: 6,
            optional_field: None,
        });
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be(&[
                6, 0, 0, 0, // u8 | padding
                0, 0, 0, 0, // HEADER (FLAGS+ID | length)
            ]),
            none
        );
        assert_eq!(
            deserialize_v1_le(&[
                6, 0, 0, 0, // u8 | padding
                0, 0, 0, 0, // HEADER (FLAGS+ID | length)
            ]),
            none
        );
        // PLAIN_CDR2:
        assert_eq!(
            deserialize_v2_be(&[
            6, 0, // u8 | boolean for option
        ]),
            none
        );
        assert_eq!(
            deserialize_v2_le(&[
            6, 0, // u8 | boolean for option
        ]),
            none
        );
    }

    #[test]
    fn deserialize_nested_final_struct() {
        let expected = Ok(NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        });
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be::<NestedFinalType>(&[
                0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                10, //u8
            ]),
            expected
        );
        assert_eq!(
            deserialize_v1_le::<NestedFinalType>(&[
                7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                10, //u8
            ]),
            expected
        );
        //PLAIN_CDR2:
        assert_eq!(
            deserialize_v2_le::<NestedFinalType>(&[
                7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                10, //u8
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_be::<NestedFinalType>(&[
                0, 7, 0, 0, // nested FinalType (u16) | padding
                0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                10, //u8
            ]),
            expected
        );
    }

    #[derive(Debug, PartialEq)]
    // @extensibility(APPENDABLE) @nested
    struct AppendableType {
        value: u16,
    }
    impl<'de> XTypesDeserialize<'de> for AppendableType {
        fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
            let mut deserializer = deserializer.deserialize_appendable_struct()?;
            Ok(Self {
                value: deserializer.deserialize_field("value")?,
            })
        }
    }

    #[test]
    fn deserialize_appendable_struct() {
        let expected = Ok(AppendableType { value: 7 });
        // PLAIN_CDR:
        assert_eq!(deserialize_v1_be::<AppendableType>(&[0, 7]), expected);
        assert_eq!(deserialize_v1_le::<AppendableType>(&[7, 0]), expected);
        // DELIMITED_CDR:
        assert_eq!(
            deserialize_v2_be::<AppendableType>(&[
                0, 0, 0, 2, // DHEADER
                0, 7 // value
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_le::<AppendableType>(&[
                2, 0, 0, 0, // DHEADER
                7, 0 // value
            ]),
            expected
        );
    }

    #[derive(Debug, PartialEq)]
    //@extensibility(MUTABLE)
    struct MutableType {
        // @id(0x005A) @key
        key: u8,
        // @id(0x0050)
        participant_key: u32,
    }

    impl<'de> XTypesDeserialize<'de> for MutableType {
        fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
            let mut des = deserializer.deserialize_mutable_struct()?;
            Ok(Self {
                key: des.deserialize_field(0x005A, "key")?,
                participant_key: des.deserialize_field(0x0050, "participant_key")?,
            })
        }
    }

    #[test]
    fn deserialize_mutable_struct() {
        let expected = Ok(MutableType {
            key: 7,
            participant_key: 8,
        });
        // PL_CDR:
        assert_eq!(
            deserialize_v1_be::<MutableType>(&[
                0x00, 0x05A, 0, 1, // PID | length
                7, 0, 0, 0, // key | padding
                0x00, 0x050, 0, 4, // PID | length
                0, 0, 0, 8, // participant_key
                0, 0, 0, 0, // Sentinel
            ]),
            expected
        );
        assert_eq!(
            deserialize_v1_le::<MutableType>(&[
                0x05A, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // key | padding
                0x050, 0x00, 4, 0, // PID | length
                8, 0, 0, 0, // participant_key
                0, 0, 0, 0, // Sentinel
            ]),
            expected
        );
        // PL_CDR2:
        assert_eq!(
            deserialize_v2_be::<MutableType>(&[
                0x00, 0x05A, 0, 1, // PID | length
                7, 0, 0, 0, // key | padding
                0x00, 0x050, 0, 4, // PID | length
                0, 0, 0, 8, // participant_key
                0, 0, 0, 0, // Sentinel
            ]),
            expected
        );
        assert_eq!(
            deserialize_v2_le::<MutableType>(&[
                0x05A, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // key | padding
                0x050, 0x00, 4, 0, // PID | length
                8, 0, 0, 0, // participant_key
                0, 0, 0, 0, // Sentinel
            ]),
            expected
        );
    }
}
