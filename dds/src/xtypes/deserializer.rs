use crate::xtypes::{
    dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, ExtensibilityKind,
        TypeKind,
    },
    error::{XTypesError, XTypesResult},
    read_write::Read,
};
use alloc::{string::String, vec::Vec};
use tracing::debug;

type RepresentationIdentifier = [u8; 2];
const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
const CDR2_BE: RepresentationIdentifier = [0x00, 0x06];
const CDR2_LE: RepresentationIdentifier = [0x00, 0x07];
const D_CDR2_BE: RepresentationIdentifier = [0x00, 0x08];
const D_CDR2_LE: RepresentationIdentifier = [0x00, 0x09];
const PL_CDR2_BE: RepresentationIdentifier = [0x00, 0x0a];
const PL_CDR2_LE: RepresentationIdentifier = [0x00, 0x0b];

pub struct CdrDeserializer;
impl CdrDeserializer {
    pub fn deserialize(dynamic_type: DynamicType, buffer: &[u8]) -> XTypesResult<DynamicData> {
        if buffer.len() < 4 {
            return Err(XTypesError::NotEnoughData);
        }
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type.clone());
        let representation_identifier = [buffer[0], buffer[1]];
        match representation_identifier {
            CDR_BE | PL_CDR_BE => {
                let mut deserializer = Cdr1Deserializer::new(&buffer[4..], BigEndian);
                deserializer.deserialize_structure(&dynamic_type, &mut dynamic_data)?;
            }
            CDR_LE | PL_CDR_LE => {
                let mut deserializer = Cdr1Deserializer::new(&buffer[4..], LittleEndian);
                deserializer.deserialize_structure(&dynamic_type, &mut dynamic_data)?;
            }
            CDR2_BE | D_CDR2_BE | PL_CDR2_BE => {
                let mut deserializer = Cdr2Deserializer::new(&buffer[4..], BigEndian);
                deserializer.deserialize_structure(&dynamic_type, &mut dynamic_data)?;
            }
            CDR2_LE | D_CDR2_LE | PL_CDR2_LE => {
                let mut deserializer = Cdr2Deserializer::new(&buffer[4..], LittleEndian);
                deserializer.deserialize_structure(&dynamic_type, &mut dynamic_data)?;
            }
            _ => return Err(XTypesError::InvalidData),
        }
        Ok(dynamic_data)
    }

    pub fn deserialize_builtin(
        dynamic_type: DynamicType,
        buffer: &[u8],
    ) -> XTypesResult<DynamicData> {
        if buffer.len() < 4 {
            return Err(XTypesError::NotEnoughData);
        }
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type.clone());
        let representation_identifier = [buffer[0], buffer[1]];
        match representation_identifier {
            PL_CDR_LE => {
                let mut deserializer = RtpsPlCdrDeserializer::new(&buffer[4..]);
                deserializer.deserialize_structure(&dynamic_type, &mut dynamic_data)?;
            }
            _ => return Err(XTypesError::NotSupported(representation_identifier)),
        }
        Ok(dynamic_data)
    }
}

struct Cdr1Deserializer<'a, E: EndiannessRead> {
    reader: CdrReader<'a, E, CdrVersion1>,
}

impl<'a, E: EndiannessRead> Cdr1Deserializer<'a, E> {
    fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: CdrReader::new(buffer, endianness, CdrVersion1),
        }
    }
}

struct Cdr2Deserializer<'a, E: EndiannessRead> {
    reader: CdrReader<'a, E, CdrVersion2>,
}

impl<'a, E: EndiannessRead> Cdr2Deserializer<'a, E> {
    fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: CdrReader::new(buffer, endianness, CdrVersion2),
        }
    }
}

struct RtpsPlCdrDeserializer<'a> {
    cdr1_deserializer: Cdr1Deserializer<'a, LittleEndian>,
}

impl<'a> RtpsPlCdrDeserializer<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            cdr1_deserializer: Cdr1Deserializer::new(buffer, LittleEndian),
        }
    }
}

trait CdrVersion {
    const MAX_ALIGN: usize;
}

struct CdrVersion1;
impl CdrVersion for CdrVersion1 {
    const MAX_ALIGN: usize = 8;
}

struct CdrVersion2;
impl CdrVersion for CdrVersion2 {
    const MAX_ALIGN: usize = 4;
}

trait EndiannessRead {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16>;
    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16>;
    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32>;
    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32>;
    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64>;
    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64>;
    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32>;
    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64>;
}

struct BigEndian;

impl EndiannessRead for BigEndian {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Ok(i16::from_be_bytes(*reader.read_array::<2>()?))
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Ok(u16::from_be_bytes(*reader.read_array::<2>()?))
    }

    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32> {
        Ok(i32::from_be_bytes(*reader.read_array::<4>()?))
    }

    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32> {
        Ok(u32::from_be_bytes(*reader.read_array::<4>()?))
    }

    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64> {
        Ok(i64::from_be_bytes(*reader.read_array::<8>()?))
    }

    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64> {
        Ok(u64::from_be_bytes(*reader.read_array::<8>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_be_bytes(*reader.read_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_be_bytes(*reader.read_array::<8>()?))
    }
}

struct LittleEndian;

impl EndiannessRead for LittleEndian {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Ok(i16::from_le_bytes(*reader.read_array::<2>()?))
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Ok(u16::from_le_bytes(*reader.read_array::<2>()?))
    }

    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32> {
        Ok(i32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32> {
        Ok(u32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64> {
        Ok(i64::from_le_bytes(*reader.read_array::<8>()?))
    }

    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64> {
        Ok(u64::from_le_bytes(*reader.read_array::<8>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_le_bytes(*reader.read_array::<8>()?))
    }
}

trait XTypesDeserialize {
    fn deserialize_complex_value(
        &mut self,
        dynamic_type: &DynamicType,
    ) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type.clone());
        self.deserialize_structure(dynamic_type, &mut dynamic_data)?;
        Ok(dynamic_data)
    }

    fn deserialize_structure(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        // Deserialize the top-level which must be either a struct, an enum or a union.
        // No other types are supported at this stage because this is what we can get from DDS
        match dynamic_type.get_descriptor().kind {
            TypeKind::STRUCTURE => {
                // Extensibility kind must match the representation
                match dynamic_type.get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => {
                        self.deserialize_final_struct(dynamic_type, dynamic_data)
                    }
                    ExtensibilityKind::Appendable => {
                        self.deserialize_appendable_struct(dynamic_type, dynamic_data)
                    }
                    ExtensibilityKind::Mutable => {
                        self.deserialize_mutable_struct(dynamic_type, dynamic_data)
                    }
                }
            }
            TypeKind::ENUM => {
                let discriminator_type = dynamic_type
                    .get_descriptor()
                    .discriminator_type
                    .as_ref()
                    .ok_or(XTypesError::InvalidType)?;
                match discriminator_type.get_kind() {
                    TypeKind::INT8 => {
                        let value = self.deserialize_primitive_type::<i8>()?;
                        dynamic_data.set_int8_value(0, value)
                    }
                    TypeKind::INT32 => {
                        let value = self.deserialize_primitive_type::<i32>()?;
                        dynamic_data.set_int32_value(0, value)
                    }
                    d => panic!("Invalid discriminator {d:?}"),
                }
            }
            //     TypeKind::UNION => todo!(),
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                Err(XTypesError::InvalidType)
            }
        }
    }

    fn deserialize_final_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            self.deserialize_final_member(member, dynamic_data)?;
        }
        Ok(())
    }

    fn deserialize_appendable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        self.deserialize_final_struct(dynamic_type, dynamic_data)
    }

    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    fn deserialize_string(&mut self) -> Result<String, XTypesError> {
        let length = self.deserialize_primitive_type::<u32>()?;
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length - 1 {
            values.push(self.deserialize_primitive_type::<u8>()?);
        }
        self.deserialize_primitive_type::<u8>()?;
        String::from_utf8(values).map_err(|_| XTypesError::InvalidData)
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T>;

    fn deserialize_final_member(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let member_descriptor = member.get_descriptor()?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => {
                dynamic_data.set_boolean_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => {
                dynamic_data.set_int16_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::INT32 => {
                dynamic_data.set_int32_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::INT64 => {
                dynamic_data.set_int64_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT16 => {
                dynamic_data.set_uint16_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT32 => {
                dynamic_data.set_uint32_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT64 => {
                dynamic_data.set_uint64_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::FLOAT32 => {
                dynamic_data.set_float32_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::FLOAT64 => {
                dynamic_data.set_float64_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => {
                dynamic_data.set_int8_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT8 => {
                dynamic_data.set_uint8_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::CHAR8 => {
                dynamic_data.set_char8_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                dynamic_data.set_string_value(member.get_id(), self.deserialize_string()?)
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => dynamic_data.set_complex_value(
                member.get_id(),
                self.deserialize_complex_value(&member_descriptor.r#type)?,
            ),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => dynamic_data.set_complex_value(
                member.get_id(),
                self.deserialize_complex_value(&member_descriptor.r#type)?,
            ),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => self.deserialize_sequence(member, dynamic_data),
            TypeKind::ARRAY => self.deserialize_array(member, dynamic_data),
            TypeKind::MAP => todo!(),
        }
    }

    fn deserialize_array(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let sequence_type = member
            .get_descriptor()?
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("Sequence must have element type");
        let bound = member.get_descriptor()?.r#type.get_descriptor().bound[0];
        match sequence_type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => dynamic_data.set_boolean_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => dynamic_data.set_int16_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::INT32 => dynamic_data.set_int32_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::INT64 => dynamic_data.set_int64_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::UINT16 => dynamic_data.set_uint16_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::UINT32 => dynamic_data.set_uint32_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::UINT64 => dynamic_data.set_uint64_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::FLOAT32 => dynamic_data.set_float32_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::FLOAT64 => dynamic_data.set_float64_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => dynamic_data.set_int8_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::UINT8 => dynamic_data.set_uint8_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::CHAR8 => dynamic_data.set_char8_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => todo!(),
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => todo!(),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }

    fn deserialize_primitive_type_sequence<T: CdrPrimitiveTypeDeserialize>(
        &mut self,
    ) -> XTypesResult<Vec<T>> {
        let length = self.deserialize_primitive_type::<u32>()?;
        self.deserialize_primitive_type_array(length)
    }

    fn deserialize_primitive_type_array<T: CdrPrimitiveTypeDeserialize>(
        &mut self,
        length: u32,
    ) -> XTypesResult<Vec<T>> {
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(self.deserialize_primitive_type::<T>()?);
        }
        Ok(values)
    }

    fn deserialize_string_sequence(&mut self) -> XTypesResult<Vec<String>> {
        let mut sequence_of_string = Vec::new();
        let length = self.deserialize_primitive_type::<u32>()?;
        for _ in 0..length {
            sequence_of_string.push(self.deserialize_string()?);
        }
        Ok(sequence_of_string)
    }

    fn deserialize_complex_sequence(
        &mut self,
        member: &DynamicTypeMember,
    ) -> XTypesResult<Vec<DynamicData>> {
        let mut sequence_of_dynamic_data = Vec::new();
        let element_type = member
            .get_descriptor()?
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("must have element type");
        let length = self.deserialize_primitive_type::<u32>()?;
        for _ in 0..length {
            sequence_of_dynamic_data.push(self.deserialize_complex_value(element_type)?);
        }
        Ok(sequence_of_dynamic_data)
    }

    fn deserialize_sequence(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let sequence_type = member
            .get_descriptor()?
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("Sequence must have element type");
        match sequence_type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => dynamic_data
                .set_boolean_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => dynamic_data
                .set_int16_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::INT32 => dynamic_data
                .set_int32_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::INT64 => dynamic_data
                .set_int64_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::UINT16 => dynamic_data
                .set_uint16_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::UINT32 => dynamic_data
                .set_uint32_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::UINT64 => dynamic_data
                .set_uint64_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::FLOAT32 => dynamic_data
                .set_float32_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::FLOAT64 => dynamic_data
                .set_float64_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => dynamic_data
                .set_int8_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::UINT8 => dynamic_data
                .set_uint8_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::CHAR8 => dynamic_data
                .set_char8_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                dynamic_data.set_string_values(member.get_id(), self.deserialize_string_sequence()?)
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => dynamic_data
                .set_complex_values(member.get_id(), self.deserialize_complex_sequence(member)?),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }
}

// This trait is only meant to be implemented on Rust basic types
// to allow for generalizing the process of deserializing those types
trait CdrPrimitiveTypeDeserialize: Sized {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self>;
}

impl CdrPrimitiveTypeDeserialize for u8 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        Ok(reader.read_exact(1)?[0])
    }
}
impl CdrPrimitiveTypeDeserialize for u16 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((u16::BITS / 8) as usize);
        E::read_u16(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for u32 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((u32::BITS / 8) as usize);
        E::read_u32(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for u64 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((u64::BITS / 8) as usize);
        E::read_u64(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for i8 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        Ok(reader.read_exact(1)?[0] as i8)
    }
}
impl CdrPrimitiveTypeDeserialize for i16 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((i16::BITS / 8) as usize);
        E::read_i16(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for i32 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((i32::BITS / 8) as usize);
        E::read_i32(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for i64 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((i64::BITS / 8) as usize);
        E::read_i64(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for f32 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((32 / 8) as usize);
        E::read_f32(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for f64 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((64 / 8) as usize);
        E::read_f64(reader)
    }
}
impl CdrPrimitiveTypeDeserialize for bool {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        let buf = reader.read_exact(1)?;
        match buf[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(XTypesError::InvalidData),
        }
    }
}
impl CdrPrimitiveTypeDeserialize for char {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        Ok(char::from(reader.read_exact(1)?[0]))
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
            return Err(XTypesError::NotEnoughData);
        }
        let ret = &self.buffer[self.pos..self.pos + length];
        self.pos += length;
        Ok(ret)
    }

    fn seek(&mut self, v: usize) {
        self.pos += v
    }

    fn seek_padding(&mut self, alignment: usize) {
        let alignment = core::cmp::min(alignment, V::MAX_ALIGN);
        let mask = alignment - 1;
        self.seek(((self.pos + mask) & !mask) - self.pos)
    }

    fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn seek_to_pid_le(&mut self, pid: u16) -> XTypesResult<bool> {
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

            self.reader.set_position(0);
            if self.reader.seek_to_pid_le(pid)? {
                self.deserialize_final_member(member, dynamic_data)?;
            } else if member_descriptor.is_optional {
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
        Ok(())
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.reader)
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for Cdr2Deserializer<'a, E> {
    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.reader)
    }

    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            let member_descriptor = member.get_descriptor()?;
            let pid = member.get_id() as u16;

            self.reader.set_position(0);
            if self.reader.seek_to_pid_le(pid)? {
                self.deserialize_final_member(member, dynamic_data)?;
            } else if member_descriptor.is_optional {
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
        Ok(())
    }

    fn deserialize_appendable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = self.deserialize_primitive_type::<u32>()?;
        self.deserialize_final_struct(dynamic_type, dynamic_data)
    }
}

impl<'a> XTypesDeserialize for RtpsPlCdrDeserializer<'a> {
    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            let member_descriptor = member.get_descriptor()?;
            let pid = member.get_id() as u16;

            self.cdr1_deserializer.reader.set_position(0);
            if member_descriptor.r#type.get_kind() == TypeKind::SEQUENCE {
                // Check if this is a sequence of complex types (structures) or primitives
                let element_type = member_descriptor
                    .r#type
                    .get_descriptor()
                    .element_type
                    .as_ref()
                    .expect("must have element_type");

                if element_type.get_kind() == TypeKind::STRUCTURE {
                    // Sequence of structures: each element is serialized as separate PID entry
                    let mut sequence = Vec::new();
                    while self.cdr1_deserializer.reader.seek_to_pid_le(pid)? {
                        sequence.push(self.deserialize_complex_value(element_type)?);
                    }
                    dynamic_data.set_complex_values(pid as u32, sequence)?;
                } else if self.cdr1_deserializer.reader.seek_to_pid_le(pid)? {
                    // Sequence of primitives: single PID entry with length + values
                    self.deserialize_final_member(member, dynamic_data)?;
                } else if member_descriptor.is_optional {
                    let default_value = member_descriptor
                        .default_value
                        .as_ref()
                        .cloned()
                        .expect("default value must exist for optional type");
                    dynamic_data.set_value(pid as u32, default_value);
                } else {
                    return Err(XTypesError::PidNotFound(pid));
                }
            } else if self.cdr1_deserializer.reader.seek_to_pid_le(pid)? {
                self.deserialize_final_member(member, dynamic_data)?;
            } else if member_descriptor.is_optional {
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
        Ok(())
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.cdr1_deserializer.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::type_support::TypeSupport;

    #[test]
    fn deserialize_final_struct() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalType {
            field_u16: u16,
            field_u64: u64,
        }
        let expected = FinalType {
            field_u16: 7,
            field_u64: 9,
        }
        .create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalType::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalType::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalType::get_type(),
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 7, 0, 0, // field_u16 | padding (2 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalType::get_type(),
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // field_u16 | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_nested_final_struct() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalType {
            field_u16: u16,
            field_u64: u64,
        }

        #[derive(Debug, PartialEq, TypeSupport)]
        //@extensibility(FINAL)
        struct NestedFinalType {
            field_nested: FinalType,
            field_u8: u8,
        }

        let expected = NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        }
        .create_dynamic_sample();

        assert_eq!(
            CdrDeserializer::deserialize(
                NestedFinalType::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                NestedFinalType::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            CdrDeserializer::deserialize(
                NestedFinalType::get_type(),
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 7, 0, 0, // nested FinalType (u16) | padding
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            CdrDeserializer::deserialize(
                NestedFinalType::get_type(),
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_final_struct_with_sequence() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalTypeWithSequence {
            field_u16: u16,
            field_u64: u64,
            field_seq_u32: Vec<u32>,
        }

        let expected = FinalTypeWithSequence {
            field_u16: 7,
            field_u64: 9,
            field_seq_u32: vec![1, 4],
        }
        .create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalTypeWithSequence::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    0, 0, 0, 2, // field_seq_u32 Length (u32)
                    0, 0, 0, 1, // field_seq_u32[0]
                    0, 0, 0, 4, // field_seq_u32[0]
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalTypeWithSequence::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    2, 0, 0, 0, // field_seq_u32 Length (u32)
                    1, 0, 0, 0, // field_seq_u32[0]
                    4, 0, 0, 0, // field_seq_u32[0]
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalTypeWithSequence::get_type(),
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 7, 0, 0, // field_u16 | padding (2 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    0, 0, 0, 2, // field_seq_u32 Length (u32)
                    0, 0, 0, 1, // field_seq_u32[0]
                    0, 0, 0, 4, // field_seq_u32[0]
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalTypeWithSequence::get_type(),
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // field_u16 | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    2, 0, 0, 0, // field_seq_u32 Length (u32)
                    1, 0, 0, 0, // field_seq_u32[0]
                    4, 0, 0, 0, // field_seq_u32[0]
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_string() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalString(String);
        let expected = FinalString(String::from("Hola")).create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalString::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 0, 0, 5, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalString::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    5, 0, 0, 0, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalString::get_type(),
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 0, 0, 5, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                FinalString::get_type(),
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    5, 0, 0, 0, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_bytes() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct ByteArray([u8; 2]);
        let expected = ByteArray([1u8, 2]).create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                ByteArray::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    1, 2
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                ByteArray::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    1, 2
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                ByteArray::get_type(),
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    1, 2
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                ByteArray::get_type(),
                &[
                    0x00, 0x07, 0x00, 0x00, // CD2R_LE
                    1, 2
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_complex_sequence() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct Atype(u8);
        #[derive(Debug, PartialEq, TypeSupport)]
        struct Sequence(Vec<Atype>);

        let expected = Sequence(vec![Atype(1), Atype(2)]).create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                Sequence::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 0, 0, 2, // length
                    1, 2, 77
                ],
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_mutable_struct() {
        #[derive(Debug, PartialEq, TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct MutableType {
            #[dust_dds(id = 0x5A, key)]
            key: u8,
            #[dust_dds(id = 0x50)]
            participant_key: u32,
        }
        let expected = MutableType {
            key: 7,
            participant_key: 8,
        }
        .create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                MutableType::get_type(),
                &[
                    0x00, 0x02, 0x00, 0x00, // PL_CDR_BE
                    0x00, 0x05A, 0, 1, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x00, 0x050, 0, 4, // PID | length
                    0, 0, 0, 8, // participant_key
                    0, 0, 0, 0, // Sentinel
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                MutableType::get_type(),
                &[
                    0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
                    0x05A, 0x00, 1, 0, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x050, 0x00, 4, 0, // PID | length
                    8, 0, 0, 0, // participant_key
                    0, 0, 0, 0, // Sentinel
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                MutableType::get_type(),
                &[
                    0x00, 0x06, 0x00, 0x00, // PL_CDR2_BE
                    0x00, 0x05A, 0, 1, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x00, 0x050, 0, 4, // PID | length
                    0, 0, 0, 8, // participant_key
                    0, 0, 0, 0, // Sentinel
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                MutableType::get_type(),
                &[
                    0x00, 0x07, 0x00, 0x00, // PL_CDR2_LE
                    0x05A, 0x00, 1, 0, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x050, 0x00, 4, 0, // PID | length
                    8, 0, 0, 0, // participant_key
                    0, 0, 0, 0, // Sentinel
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_appendable_struct() {
        #[derive(Debug, PartialEq, TypeSupport)]
        #[dust_dds(extensibility = "appendable")]
        struct AppendableType {
            #[dust_dds(key)]
            key: u8,
            participant_key: u32,
        }
        let expected = AppendableType {
            key: 7,
            participant_key: 8,
        }
        .create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableType::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    7, 0, 0, 0, // key | padding
                    0, 0, 0, 8, // participant_key
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableType::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, // key | padding
                    8, 0, 0, 0, // participant_key
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableType::get_type(),
                &[
                    0x00, 0x08, 0x00, 0x00, // D_CDR2_BE
                    0, 0, 0, 8, // DHEADER
                    7, 0, 0, 0, // key | padding
                    0, 0, 0, 8, // participant_key
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableType::get_type(),
                &[
                    0x00, 0x09, 0x00, 0x00, // D_CDR2_LE
                    0, 0, 0, 8, // DHEADER
                    7, 0, 0, 0, // key | padding
                    8, 0, 0, 0, // participant_key
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_appendable_shapes() {
        #[derive(Debug, PartialEq, TypeSupport)]
        #[dust_dds(extensibility = "appendable")]
        struct AppendableShapesType {
            #[dust_dds(key)]
            color: String,
            x: i32,
            y: i32,
            shapesize: i32,
            additional_payload_size: Vec<u8>,
        }
        let expected = AppendableShapesType {
            color: String::from("BLUE"),
            x: 10,
            y: 20,
            shapesize: 30,
            additional_payload_size: vec![],
        }
        .create_dynamic_sample();
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableShapesType::get_type(),
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 0, 0, 5, // color: length
                    b'B', b'L', b'U', b'E', // color
                    0, 0, 0, 0, // color: terminating 0 | padding
                    0, 0, 0, 10, // x
                    0, 0, 0, 20, // y
                    0, 0, 0, 30, // shapesize
                    0, 0, 0, 0, // additional_payload_size: length
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableShapesType::get_type(),
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    5, 0, 0, 0, // color: length
                    b'B', b'L', b'U', b'E', // color
                    0, 0, 0, 0, // color: terminating 0 | padding
                    10, 0, 0, 0, // x
                    20, 0, 0, 0, // y
                    30, 0, 0, 0, // shapesize
                    0, 0, 0, 0, // additional_payload_size: length
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableShapesType::get_type(),
                &[
                    0x00, 0x08, 0x00, 0x00, // D_CDR2_BE
                    0, 0, 0, 28, // Dheader
                    0, 0, 0, 5, // color: length
                    b'B', b'L', b'U', b'E', // color
                    0, 0, 0, 0, // color: terminating 0 | padding
                    0, 0, 0, 10, // x
                    0, 0, 0, 20, // y
                    0, 0, 0, 30, // shapesize
                    0, 0, 0, 0, // additional_payload_size: length
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            CdrDeserializer::deserialize(
                AppendableShapesType::get_type(),
                &[
                    0x00, 0x09, 0x00, 0x00, // D_CDR2_LE
                    28, 0, 0, 0, // Dheader
                    5, 0, 0, 0, // color: length
                    b'B', b'L', b'U', b'E', // color
                    0, 0, 0, 0, // color: terminating 0 | padding
                    10, 0, 0, 0, // x
                    20, 0, 0, 0, // y
                    30, 0, 0, 0, // shapesize
                    0, 0, 0, 0, // additional_payload_size: length
                ]
            )
            .unwrap(),
            expected
        );
    }
}
