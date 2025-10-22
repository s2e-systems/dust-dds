use crate::xtypes::{
    data_representation::{
        deserialize::XTypesDeserialize, endianness::EndiannessRead, read_write::Read,
    },
    dynamic_type::{DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, TypeKind},
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
    const MAX_ALIGN: usize = 8;
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

    fn deserialize_primitive_type_sequence<T: CdrPrimitiveTypeDeserialize>(
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

    fn deserialize_type_kind(
        &mut self,
        id: u32,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        match member.get_descriptor()?.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => {
                dynamic_data.set_boolean_value(id, self.reader.deserialize_boolean()?)?
            }
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => dynamic_data.set_int16_value(id, self.reader.deserialize_i16()?)?,
            TypeKind::INT32 => dynamic_data.set_int32_value(id, self.reader.deserialize_i32()?)?,
            TypeKind::INT64 => dynamic_data.set_int64_value(id, self.reader.deserialize_i64()?)?,
            TypeKind::UINT16 => {
                dynamic_data.set_uint16_value(id, self.reader.deserialize_u16()?)?
            }
            TypeKind::UINT32 => {
                dynamic_data.set_uint32_value(id, self.reader.deserialize_u32()?)?
            }
            TypeKind::UINT64 => {
                dynamic_data.set_uint64_value(id, self.reader.deserialize_u64()?)?
            }
            TypeKind::FLOAT32 => {
                dynamic_data.set_float32_value(id, self.reader.deserialize_f32()?)?
            }
            TypeKind::FLOAT64 => {
                dynamic_data.set_float64_value(id, self.reader.deserialize_f64()?)?
            }
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => dynamic_data.set_int8_value(id, self.reader.deserialize_i8()?)?,
            TypeKind::UINT8 => dynamic_data.set_uint8_value(id, self.reader.deserialize_u8()?)?,
            TypeKind::CHAR8 => dynamic_data.set_char8_value(id, self.reader.deserialize_char()?)?,
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => todo!(),
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                let value =
                    self.deserialize_final_struct(member.get_descriptor()?.r#type.clone())?;
                dynamic_data.set_complex_value(id, value)?;
            }
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => {
                let sequence_type = member
                    .get_descriptor()?
                    .r#type
                    .get_descriptor()
                    .element_type
                    .as_ref()
                    .expect("Sequence must have element type");
                match sequence_type.get_kind() {
                    TypeKind::NONE => todo!(),
                    TypeKind::BOOLEAN => todo!(),
                    TypeKind::BYTE => todo!(),
                    TypeKind::INT16 => todo!(),
                    TypeKind::INT32 => todo!(),
                    TypeKind::INT64 => todo!(),
                    TypeKind::UINT16 => todo!(),
                    TypeKind::UINT32 => {
                        dynamic_data.set_uint32_values(
                            id,
                            self.reader.deserialize_primitive_type_sequence::<u32>()?,
                        )?;
                    }
                    TypeKind::UINT64 => todo!(),
                    TypeKind::FLOAT32 => todo!(),
                    TypeKind::FLOAT64 => todo!(),
                    TypeKind::FLOAT128 => todo!(),
                    TypeKind::INT8 => todo!(),
                    TypeKind::UINT8 => todo!(),
                    TypeKind::CHAR8 => todo!(),
                    TypeKind::CHAR16 => todo!(),
                    TypeKind::STRING8 => todo!(),
                    TypeKind::STRING16 => todo!(),
                    TypeKind::ALIAS => todo!(),
                    TypeKind::ENUM => todo!(),
                    TypeKind::BITMASK => todo!(),
                    TypeKind::ANNOTATION => todo!(),
                    TypeKind::STRUCTURE => {
                        dynamic_data.set_complex_values(
                            id,
                            self.deserialize_type_sequence(
                                member.get_descriptor()?.r#type.clone(),
                            )?,
                        )?;
                    }
                    TypeKind::UNION => todo!(),
                    TypeKind::BITSET => todo!(),
                    TypeKind::SEQUENCE => todo!(),
                    TypeKind::ARRAY => todo!(),
                    TypeKind::MAP => todo!(),
                }
            }
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
        Ok(())
    }

    fn deserialize_final_member(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        todo!()
    }

    fn deserialize_type_sequence(&mut self, r#type: DynamicType) -> XTypesResult<Vec<DynamicData>> {
        let length = self.reader.deserialize_u32()?;
        let values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            todo!("Deserialize each element into a vec")
        }
        Ok(values)
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for Cdr1Deserializer<'a, E> {
    fn deserialize_final_struct(&mut self, dynamic_type: DynamicType) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type.clone());
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            self.deserialize_final_member(member, &mut dynamic_data)?;
        }
        Ok(dynamic_data)
    }

    fn deserialize_appendable_struct(
        &mut self,
        dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData> {
        self.deserialize_final_struct(dynamic_type)
    }

    fn deserialize_mutable_struct(
        &mut self,
        _dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData> {
        Err(XTypesError::IllegalOperation)
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
    fn deserialize_final_struct(&mut self, dynamic_type: DynamicType) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
        for member_index in 0..dynamic_data.type_ref().get_member_count() {
            let member = dynamic_data.type_ref().get_member_by_index(member_index)?;
            match member.get_descriptor()?.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => dynamic_data
                    .set_boolean_value(member.get_id(), self.reader.deserialize_boolean()?)?,
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => {
                    dynamic_data.set_int16_value(member.get_id(), self.reader.deserialize_i16()?)?
                }
                TypeKind::INT32 => {
                    dynamic_data.set_int32_value(member.get_id(), self.reader.deserialize_i32()?)?
                }
                TypeKind::INT64 => {
                    dynamic_data.set_int64_value(member.get_id(), self.reader.deserialize_i64()?)?
                }
                TypeKind::UINT16 => dynamic_data
                    .set_uint16_value(member.get_id(), self.reader.deserialize_u16()?)?,
                TypeKind::UINT32 => dynamic_data
                    .set_uint32_value(member.get_id(), self.reader.deserialize_u32()?)?,
                TypeKind::UINT64 => dynamic_data
                    .set_uint64_value(member.get_id(), self.reader.deserialize_u64()?)?,
                TypeKind::FLOAT32 => dynamic_data
                    .set_float32_value(member.get_id(), self.reader.deserialize_f32()?)?,
                TypeKind::FLOAT64 => dynamic_data
                    .set_float64_value(member.get_id(), self.reader.deserialize_f64()?)?,
                TypeKind::FLOAT128 => todo!(),
                TypeKind::INT8 => {
                    dynamic_data.set_int8_value(member.get_id(), self.reader.deserialize_i8()?)?
                }
                TypeKind::UINT8 => {
                    dynamic_data.set_uint8_value(member.get_id(), self.reader.deserialize_u8()?)?
                }
                TypeKind::CHAR8 => dynamic_data
                    .set_char8_value(member.get_id(), self.reader.deserialize_char()?)?,
                TypeKind::CHAR16 => todo!(),
                TypeKind::STRING8 => todo!(),
                TypeKind::STRING16 => todo!(),
                TypeKind::ALIAS => todo!(),
                TypeKind::ENUM => todo!(),
                TypeKind::BITMASK => todo!(),
                TypeKind::ANNOTATION => todo!(),
                TypeKind::STRUCTURE => {
                    let value =
                        self.deserialize_final_struct(member.get_descriptor()?.r#type.clone())?;
                    dynamic_data.set_complex_value(member.get_id(), value)?;
                }
                TypeKind::UNION => todo!(),
                TypeKind::BITSET => todo!(),
                TypeKind::SEQUENCE => {
                    let sequence_type = member
                        .get_descriptor()?
                        .r#type
                        .get_descriptor()
                        .element_type
                        .as_ref()
                        .expect("Sequence must have element type");
                    match sequence_type.get_kind() {
                        TypeKind::NONE => todo!(),
                        TypeKind::BOOLEAN => todo!(),
                        TypeKind::BYTE => todo!(),
                        TypeKind::INT16 => todo!(),
                        TypeKind::INT32 => todo!(),
                        TypeKind::INT64 => todo!(),
                        TypeKind::UINT16 => todo!(),
                        TypeKind::UINT32 => {
                            dynamic_data.set_uint32_values(
                                member.get_id(),
                                todo!(), // self.deserialize_primitive_type_sequence::<u32>()?,
                            )?;
                        }
                        TypeKind::UINT64 => todo!(),
                        TypeKind::FLOAT32 => todo!(),
                        TypeKind::FLOAT64 => todo!(),
                        TypeKind::FLOAT128 => todo!(),
                        TypeKind::INT8 => todo!(),
                        TypeKind::UINT8 => todo!(),
                        TypeKind::CHAR8 => todo!(),
                        TypeKind::CHAR16 => todo!(),
                        TypeKind::STRING8 => todo!(),
                        TypeKind::STRING16 => todo!(),
                        TypeKind::ALIAS => todo!(),
                        TypeKind::ENUM => todo!(),
                        TypeKind::BITMASK => todo!(),
                        TypeKind::ANNOTATION => todo!(),
                        TypeKind::STRUCTURE => {
                            dynamic_data.set_complex_values(
                                member.get_id(),
                                todo!(), // self.deserialize_type_sequence(
                                         //     member.get_descriptor()?.r#type.clone(),
                                         // )?,
                            )?;
                        }
                        TypeKind::UNION => todo!(),
                        TypeKind::BITSET => todo!(),
                        TypeKind::SEQUENCE => todo!(),
                        TypeKind::ARRAY => todo!(),
                        TypeKind::MAP => todo!(),
                    }
                }
                TypeKind::ARRAY => todo!(),
                TypeKind::MAP => todo!(),
            }
        }
        Ok(dynamic_data)
    }

    fn deserialize_appendable_struct(
        &mut self,
        dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData> {
        todo!()
    }

    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData> {
        todo!()
    }
}
