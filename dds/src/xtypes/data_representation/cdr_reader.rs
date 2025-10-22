use crate::xtypes::{
    data_representation::{
        deserialize::XTypesDeserialize, endianness::EndiannessRead, read_write::Read,
    },
    dynamic_type::{DynamicData, DynamicType, DynamicTypeMember, TypeKind},
    error::{XTypesError, XTypesResult},
};

pub trait CdrVersion {
    const MAX_ALIGN: usize;
}

pub struct CdrVersion1;
impl CdrVersion for CdrVersion1 {
    const MAX_ALIGN: usize = 8;
}

pub struct CdrVersion2;
impl CdrVersion for CdrVersion2 {
    const MAX_ALIGN: usize = 4;
}

trait CdrPrimitiveTypeDeserializer {
    fn deserialize_u8(&mut self) -> XTypesResult<u8>;
    fn deserialize_u16(&mut self) -> XTypesResult<u16>;
    fn deserialize_u32(&mut self) -> XTypesResult<u32>;
    fn deserialize_u64(&mut self) -> XTypesResult<u64>;
    fn deserialize_i8(&mut self) -> XTypesResult<i8>;
    fn deserialize_i16(&mut self) -> XTypesResult<i16>;
    fn deserialize_i32(&mut self) -> XTypesResult<i32>;
    fn deserialize_i64(&mut self) -> XTypesResult<i64>;
    fn deserialize_f32(&mut self) -> XTypesResult<f32>;
    fn deserialize_f64(&mut self) -> XTypesResult<f64>;
    fn deserialize_boolean(&mut self) -> XTypesResult<bool>;
    fn deserialize_char(&mut self) -> XTypesResult<char>;
}

// This trait is only meant to be implemented on Rust basic types
// to allow for generalizing the process of deserializing those types
pub trait CdrPrimitiveTypeDeserialize: Sized {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self>;
}

impl CdrPrimitiveTypeDeserialize for u8 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u8()
    }
}
impl CdrPrimitiveTypeDeserialize for u16 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u16()
    }
}
impl CdrPrimitiveTypeDeserialize for u32 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u32()
    }
}
impl CdrPrimitiveTypeDeserialize for u64 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u64()
    }
}
impl CdrPrimitiveTypeDeserialize for i8 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i8()
    }
}
impl CdrPrimitiveTypeDeserialize for i16 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i16()
    }
}
impl CdrPrimitiveTypeDeserialize for i32 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i32()
    }
}
impl CdrPrimitiveTypeDeserialize for i64 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i64()
    }
}
impl CdrPrimitiveTypeDeserialize for f32 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_f32()
    }
}
impl CdrPrimitiveTypeDeserialize for f64 {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_f64()
    }
}
impl CdrPrimitiveTypeDeserialize for bool {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_boolean()
    }
}
impl CdrPrimitiveTypeDeserialize for char {
    fn deserialize(d: &mut impl CdrPrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_char()
    }
}

struct CdrReader<'a, E, V> {
    buffer: &'a [u8],
    pos: usize,
    _endianness: E,
    _version: V,
}

impl<'a, E: EndiannessRead, V: CdrVersion> CdrReader<'a, E, V> {
    fn new(buffer: &'a [u8], endianness: E, version: V) -> Self {
        Self {
            buffer,
            pos: 0,
            _endianness: endianness,
            _version: version,
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

    pub fn seek_padding(&mut self, alignment: usize) {
        let alignment = core::cmp::min(alignment, V::MAX_ALIGN);
        let mask = alignment - 1;
        self.seek(((self.pos + mask) & !mask) - self.pos)
    }

    pub fn seek_to_pid_le(&mut self, pid: u16) -> XTypesResult<bool> {
        self.pos = 0;
        const PID_SENTINEL: u16 = 1;
        loop {
            let current_pid = E::read_u16(self)?;
            let length = E::read_u16(self)? as usize;
            if current_pid == pid {
                return Ok(true);
            } else if current_pid == PID_SENTINEL {
                return Ok(false);
            } else {
                self.seek(length);
                self.seek_padding(4);
            }
        }
    }
}

impl<'a, E: EndiannessRead, V: CdrVersion> Read for CdrReader<'a, E, V> {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]> {
        self.read_all(size)
    }
}

impl<'a, E: EndiannessRead, V: CdrVersion> CdrPrimitiveTypeDeserializer for CdrReader<'a, E, V> {
    fn deserialize_u8(&mut self) -> XTypesResult<u8> {
        self.seek_padding((u8::BITS / 8) as usize);
        E::read_u8(self)
    }

    fn deserialize_u16(&mut self) -> XTypesResult<u16> {
        self.seek_padding((u16::BITS / 8) as usize);
        E::read_u16(self)
    }

    fn deserialize_u32(&mut self) -> XTypesResult<u32> {
        self.seek_padding((u32::BITS / 8) as usize);
        E::read_u32(self)
    }

    fn deserialize_u64(&mut self) -> XTypesResult<u64> {
        self.seek_padding((u64::BITS / 8) as usize);
        E::read_u64(self)
    }

    fn deserialize_i8(&mut self) -> XTypesResult<i8> {
        self.seek_padding((i8::BITS / 8) as usize);
        E::read_i8(self)
    }

    fn deserialize_i16(&mut self) -> XTypesResult<i16> {
        self.seek_padding((i16::BITS / 8) as usize);
        E::read_i16(self)
    }

    fn deserialize_i32(&mut self) -> XTypesResult<i32> {
        self.seek_padding((i32::BITS / 8) as usize);
        E::read_i32(self)
    }

    fn deserialize_i64(&mut self) -> XTypesResult<i64> {
        self.seek_padding((i64::BITS / 8) as usize);
        E::read_i64(self)
    }

    fn deserialize_f32(&mut self) -> XTypesResult<f32> {
        self.seek_padding(4);
        E::read_f32(self)
    }

    fn deserialize_f64(&mut self) -> XTypesResult<f64> {
        self.seek_padding(8);
        E::read_f64(self)
    }

    fn deserialize_boolean(&mut self) -> XTypesResult<bool> {
        self.seek_padding(1);
        E::read_bool(self)
    }

    fn deserialize_char(&mut self) -> XTypesResult<char> {
        self.seek_padding(1);
        E::read_char(self)
    }
}

pub struct Cdr1Deserializer<'a, E: EndiannessRead> {
    reader: CdrReader<'a, E, CdrVersion1>,
}

impl<'a, E: EndiannessRead> Cdr1Deserializer<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: CdrReader::new(buffer, endianness, CdrVersion1),
        }
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for Cdr1Deserializer<'a, E> {
    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            let member_descriptor = member.get_descriptor()?;
            let pid = member.get_id() as u16;

            if self.reader.seek_to_pid_le(pid)? {
                self.deserialize_final_member(member, dynamic_data)?;
            } else {
                if member_descriptor.is_optional {
                    let default_value = member_descriptor
                        .default_value
                        .as_ref()
                        .cloned()
                        .expect("default value must exist for optional type");
                    dynamic_data.set_value(pid as u32, default_value);
                } else {
                    return Err(XTypesError::PidNotFound(pid));
                }
            }
        }
        Ok(())
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.reader)
    }
}

pub struct Cdr2Deserializer<'a, E: EndiannessRead> {
    reader: CdrReader<'a, E, CdrVersion2>,
}

impl<'a, E: EndiannessRead> Cdr2Deserializer<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: CdrReader::new(buffer, endianness, CdrVersion2),
        }
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for Cdr2Deserializer<'a, E> {
    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.reader)
    }
}

pub struct PlCdr1Deserializer<'a, E: EndiannessRead> {
    cdr1_deserializer: Cdr1Deserializer<'a, E>,
}

impl<'a, E: EndiannessRead> PlCdr1Deserializer<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            cdr1_deserializer: Cdr1Deserializer::new(buffer, endianness),
        }
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for PlCdr1Deserializer<'a, E> {
    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            let member_descriptor = member.get_descriptor()?;
            let pid = member.get_id() as u16;

            if member_descriptor.r#type.get_kind() == TypeKind::SEQUENCE {
                todo!()
            } else {
                if self.cdr1_deserializer.reader.seek_to_pid_le(pid)? {
                    self.deserialize_final_member(member, dynamic_data)?;
                } else {
                    if member_descriptor.is_optional {
                        let default_value = member_descriptor
                            .default_value
                            .as_ref()
                            .cloned()
                            .expect("default value must exist for optional type");
                        dynamic_data.set_value(pid as u32, default_value);
                    } else {
                        return Err(XTypesError::PidNotFound(pid));
                    }
                }
            }
        }
        Ok(())
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.cdr1_deserializer.reader)
    }
}
