use crate::xtypes::{
    dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, ExtensibilityKind,
        TypeKind,
    },
    error::{XTypesError, XTypesResult},
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

pub trait Read {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]>;

    fn read_array<const N: usize>(&mut self) -> XTypesResult<&[u8; N]> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_e| XTypesError::InvalidData)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeserializeKind {
    Full,
    KeyOnly,
}

pub fn deserialize_full(dynamic_type: DynamicType, buffer: &[u8]) -> XTypesResult<DynamicData> {
    if buffer.len() < 4 {
        return Err(XTypesError::NotEnoughData);
    }
    let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
    let representation_identifier = [buffer[0], buffer[1]];
    match representation_identifier {
        CDR_BE | PL_CDR_BE => {
            let mut deserializer = Cdr1Deserializer::new(&buffer[4..], BigEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        CDR_LE | PL_CDR_LE => {
            let mut deserializer = Cdr1Deserializer::new(&buffer[4..], LittleEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        CDR2_BE | D_CDR2_BE | PL_CDR2_BE => {
            let mut deserializer = Cdr2Deserializer::new(&buffer[4..], BigEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        CDR2_LE | D_CDR2_LE | PL_CDR2_LE => {
            let mut deserializer = Cdr2Deserializer::new(&buffer[4..], LittleEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        _ => return Err(XTypesError::InvalidData),
    }
    Ok(dynamic_data)
}

pub fn deserialize_builtin(dynamic_type: DynamicType, buffer: &[u8]) -> XTypesResult<DynamicData> {
    if buffer.len() < 4 {
        return Err(XTypesError::NotEnoughData);
    }
    let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
    let representation_identifier = [buffer[0], buffer[1]];
    match representation_identifier {
        PL_CDR_LE => {
            let mut deserializer = RtpsPlCdrDeserializer::new(&buffer[4..]);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        _ => return Err(XTypesError::NotSupported(representation_identifier)),
    }
    Ok(dynamic_data)
}

pub fn deserialize_key_only(dynamic_type: DynamicType, buffer: &[u8]) -> XTypesResult<DynamicData> {
    if buffer.len() < 4 {
        return Err(XTypesError::NotEnoughData);
    }
    let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
    let representation_identifier = [buffer[0], buffer[1]];
    match representation_identifier {
        CDR_BE => {
            let mut deserializer = Cdr1KeyDeserializer::new(&buffer[4..], BigEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        CDR_LE => {
            let mut deserializer = Cdr1KeyDeserializer::new(&buffer[4..], LittleEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        CDR2_BE => {
            let mut deserializer = Cdr2KeyDeserializer::new(&buffer[4..], BigEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        CDR2_LE => {
            let mut deserializer = Cdr2KeyDeserializer::new(&buffer[4..], LittleEndian);
            deserializer.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        }
        _ => return Err(XTypesError::InvalidData),
    }
    Ok(dynamic_data)
}

struct Cdr1KeyDeserializer<'a, E: EndiannessRead> {
    reader: CdrReader<'a, E, CdrVersion1>,
}

impl<'a, E: EndiannessRead> Cdr1KeyDeserializer<'a, E> {
    fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: CdrReader::new(buffer, endianness, CdrVersion1),
        }
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for Cdr1KeyDeserializer<'a, E> {
    fn deserialize_final_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            if member.get_descriptor()?.is_key {
                self.deserialize_final_member(member, dynamic_data)?;
            }
        }
        Ok(())
    }

    fn deserialize_mutable_struct(
        &mut self,
        _dynamic_type: &DynamicType,
        _dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        unimplemented!("Not available for key")
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.reader)
    }
}

struct Cdr2KeyDeserializer<'a, E: EndiannessRead> {
    reader: CdrReader<'a, E, CdrVersion2>,
}

impl<'a, E: EndiannessRead> Cdr2KeyDeserializer<'a, E> {
    fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: CdrReader::new(buffer, endianness, CdrVersion2),
        }
    }
}

impl<'a, E: EndiannessRead> XTypesDeserialize for Cdr2KeyDeserializer<'a, E> {
    fn deserialize_mutable_struct(
        &mut self,
        _dynamic_type: &DynamicType,
        _dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        unimplemented!("Not available for key")
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T> {
        T::deserialize(&mut self.reader)
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
    fn read_i128<R: Read>(reader: &mut R) -> XTypesResult<i128>;
    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32>;
    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64>;
}


struct Buffer<'a>(&'a mut [u8]);

impl<'a> Buffer<'a> {
    fn read_array<const N: usize>(&mut self) -> [u8; N] {
        self.0[..N].try_into().unwrap()
    }
}

trait ReadArray {
    fn read_array<const N: usize>(&mut self) -> [u8; N] ;
}


trait ReadFromReader<E: EndiannessRead> {
    fn read(reader: impl ReadArray) -> Self;

}
impl ReadFromReader<BigEndian> for u16 {
    fn read(mut reader: impl ReadArray) -> Self {
        u16::from_be_bytes(reader.read_array())
    }
}
// impl ReadFromReader<LittleEndian> for u16 {
//     fn read(reader: impl ReadArray -> Self {
//         u16::from_le_bytes(reader.read_array())
//     }
// }


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

    fn read_i128<R: Read>(reader: &mut R) -> XTypesResult<i128> {
        Ok(i128::from_be_bytes(*reader.read_array::<16>()?))
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

    fn read_i128<R: Read>(reader: &mut R) -> XTypesResult<i128> {
        Ok(i128::from_le_bytes(*reader.read_array::<16>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_le_bytes(*reader.read_array::<8>()?))
    }
}

trait EncodingVersion {
    const MAX_ALIGN: usize;

    /// Serialization Rule (9) & (10)
    fn serialize_array_type(&mut self);

    /// Serialization Rule (12)
    fn deserialize_sequence_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (15) & (16)
    fn deserialize_map_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (21) & (23)
    fn deserialize_mstruct_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
    ) -> XTypesResult<DynamicData> {
        todo!()
    }

    /// Serialization Rule (27) & (28)
    fn deserialize_munion_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (29) & (30)
    fn deserialize_appendable_type(&mut self) -> XTypesResult<DynamicData> {
        todo!()
    }
}

struct EncodingVersion1;
impl EncodingVersion for EncodingVersion1 {
    const MAX_ALIGN: usize = 8;

    fn serialize_array_type(&mut self) {
        todo!()
    }
}

struct EncodingVersion2;
impl EncodingVersion for EncodingVersion2 {
    const MAX_ALIGN: usize = 4;

    fn serialize_array_type(&mut self) {
        todo!()
    }
}

/// Serialization Rule (1)
fn deserialize_top_level_type(
    dynamic_type: DynamicType,
    buffer: &[u8],
) -> XTypesResult<DynamicData> {
    if buffer.len() < 4 {
        return Err(XTypesError::NotEnoughData);
    }
    let representation_identifier = [buffer[0], buffer[1]];
    let data = &buffer[4..];
    match representation_identifier {
        CDR_BE | PL_CDR_BE => XTypesDeserializer::new(data, EncodingVersion1, BigEndian)
            .deserialize_as_nested(dynamic_type),
        CDR_LE | PL_CDR_LE => XTypesDeserializer::new(data, EncodingVersion1, LittleEndian)
            .deserialize_as_nested(dynamic_type),
        CDR2_BE | D_CDR2_BE | PL_CDR2_BE => {
            XTypesDeserializer::new(data, EncodingVersion2, BigEndian)
                .deserialize_as_nested(dynamic_type)
        }
        CDR2_LE | D_CDR2_LE | PL_CDR2_LE => {
            XTypesDeserializer::new(data, EncodingVersion2, LittleEndian)
                .deserialize_as_nested(dynamic_type)
        }
        _ => Err(XTypesError::InvalidData),
    }
}

struct XTypesDeserializer<'a, E, V> {
    reader: NextCdrReader<'a, E>,
    _encoding_version: V,
}

impl<'a, E: EndiannessRead, V: EncodingVersion> XTypesDeserializer<'a, E, V> {
    fn new(buffer: &'a [u8], encoding_version: V, endianness: E) -> Self {
        Self {
            reader: NextCdrReader::new(buffer, endianness),
            _encoding_version: encoding_version,
        }
    }

    fn deserialize_as_nested(&mut self, dynamic_type: DynamicType) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);

        // We start by deserializing the base type if it exists before proceeding with the rest of the type
        if let Some(base_dynamic_type) = dynamic_type.descriptor.base_type {
            self.deserialize_as_nested(base_dynamic_type)?;
        }

        // Deserialize the top-level which must be either a struct, an enum or a union.
        match dynamic_type.get_descriptor().kind {
            TypeKind::STRUCTURE => match dynamic_type.get_descriptor().extensibility_kind {
                ExtensibilityKind::Final => {
                    self.deserialize_fstruct_type(dynamic_type, &mut dynamic_data)?;
                }
                ExtensibilityKind::Appendable => todo!(),
                ExtensibilityKind::Mutable => todo!(), //V::deserialize_mstruct_type(self),
            },
            TypeKind::ENUM => todo!(),
            TypeKind::UNION => todo!(),
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                return Err(XTypesError::InvalidType);
            }
        }
        Ok(dynamic_data)
    }

    /// Serialization Rule (2)
    fn deserialize_primitive_type<O: AsBytes + Align>(&mut self) -> XTypesResult<O> {
        O::align::<_, V>(&mut self.reader);
        O::as_bytes(&mut self.reader)
    }

    /// Serialization Rule (3) & (4)
    fn deserialize_string_type(&mut self) -> XTypesResult<String> {
        todo!()
        // let length = self.deserialize_primitive_type::<u32>()?;
        // let values = self.reader.read_all(length as usize)?.to_vec();
        // String::from_utf8(values).map_err(|_| XTypesError::InvalidData)
    }

    /// Serialization Rule (5)
    fn _deserialize_enum_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (6)
    fn _deserialize_bitmask_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (7)
    fn _deserialize_alias_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (8)
    fn deserialize_parray_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (11)
    fn deserialize_psequence_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (14)
    fn deserialize_pmap_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (17)
    fn deserialize_fstruct_type(
        &mut self,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            self.deserialize_nopt_member(member, dynamic_data)?;
        }
        Ok(())
    }

    /// Serialization Rule (18)
    fn deserialize_nopt_member(
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
            TypeKind::BYTE => {
                dynamic_data.set_byte_value(member.get_id(), self.deserialize_primitive_type()?)
            }
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
            TypeKind::FLOAT128 => {
                dynamic_data.set_float128_value(member.get_id(), self.deserialize_primitive_type()?)
            }
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
                dynamic_data.set_string_value(member.get_id(), self.deserialize_string_type()?)
            }
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
            _ => todo!()
        }
    }

    /// Serialization Rule (26)
    fn deserialize_funion_type(&mut self) -> XTypesResult<DynamicData> {
        todo!()
    }
}

trait XTypesDeserialize {
    fn deserialize_complex_value(
        &mut self,
        dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
        let dynamic_type = dynamic_data.r#type();
        self.deserialize_dynamic_data(&dynamic_type, &mut dynamic_data)?;
        Ok(dynamic_data)
    }

    fn deserialize_dynamic_data(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        // We start by deserializing the base type if it exists before proceeding with the rest of the type
        if let Some(b) = &dynamic_type.descriptor.base_type {
            self.deserialize_dynamic_data(b, dynamic_data)?;
        }

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
            TypeKind::UNION => todo!(),
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
            TypeKind::BYTE => {
                dynamic_data.set_byte_value(member.get_id(), self.deserialize_primitive_type()?)
            }
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
            TypeKind::FLOAT128 => {
                dynamic_data.set_float128_value(member.get_id(), self.deserialize_primitive_type()?)
            }
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
                self.deserialize_complex_value(member_descriptor.r#type)?,
            ),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => dynamic_data.set_complex_value(
                member.get_id(),
                self.deserialize_complex_value(member_descriptor.r#type)?,
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
        let bound = member
            .get_descriptor()?
            .r#type
            .get_descriptor()
            .bound
            .ok_or(XTypesError::InvalidType)?;
        match sequence_type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => dynamic_data.set_boolean_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
            TypeKind::BYTE => dynamic_data.set_byte_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
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
            TypeKind::FLOAT128 => dynamic_data.set_float128_values(
                member.get_id(),
                self.deserialize_primitive_type_array(bound)?,
            ),
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
            TypeKind::BYTE => dynamic_data
                .set_byte_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
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
            TypeKind::FLOAT128 => dynamic_data
                .set_float128_values(member.get_id(), self.deserialize_primitive_type_sequence()?),
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

trait Align {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>);
}
impl Align for bool {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(_reader: &mut NextCdrReader<'a, E>) {}
}
impl Align for u8 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(_reader: &mut NextCdrReader<'a, E>) {}
}
impl Align for u16 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for u32 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for u64 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(core::cmp::min(Self::BITS as usize / 8, V::MAX_ALIGN));
    }
}
impl Align for i8 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(_reader: &mut NextCdrReader<'a, E>) {}
}
impl Align for i16 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for i32 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for i64 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(core::cmp::min(Self::BITS as usize / 8, V::MAX_ALIGN));
    }
}
impl Align for i128 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(core::cmp::min(Self::BITS as usize / 8, V::MAX_ALIGN));
    }
}
impl Align for f32 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(32 / 8);
    }
}
impl Align for f64 {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {
        reader.seek_padding(core::cmp::min(64 / 8, V::MAX_ALIGN));
    }
}
impl Align for char {
    fn align<'a, E: EndiannessRead, V: EncodingVersion>(reader: &mut NextCdrReader<'a, E>) {}
}

trait AsBytes {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self>
    where
        Self: Sized;
}
impl AsBytes for bool {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        match reader.read_exact(1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(XTypesError::InvalidData),
        }
    }
}
impl AsBytes for u8 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        reader.read_byte()
    }
}
impl AsBytes for u16 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_u16(reader)
    }
}
impl AsBytes for u32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_u32(reader)
    }
}
impl AsBytes for u64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_u64(reader)
    }
}
impl AsBytes for i8 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        Ok(reader.read_byte()? as Self)
    }
}
impl AsBytes for i16 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_i16(reader)
    }
}
impl AsBytes for i32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_i32(reader)
    }
}
impl AsBytes for i64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_i64(reader)
    }
}
impl AsBytes for i128 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_i128(reader)
    }
}
impl AsBytes for f32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_f32(reader)
    }
}
impl AsBytes for f64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        E::read_f64(reader)
    }
}
impl AsBytes for char {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a, E>) -> XTypesResult<Self> {
        Ok(char::from(reader.read_byte()?))
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
impl CdrPrimitiveTypeDeserialize for i128 {
    fn deserialize<'a, E: EndiannessRead, V: CdrVersion>(
        reader: &mut CdrReader<'a, E, V>,
    ) -> XTypesResult<Self> {
        reader.seek_padding((i64::BITS / 8) as usize);
        E::read_i128(reader)
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

struct NextCdrReader<'a, E> {
    buffer: &'a [u8],
    pos: usize,
    _endianness: E,
}

// trait ReadArray<const N: usize> {
//     fn read_array_n(&mut self) -> XTypesResult<[u8; N]>;
// }
// impl<'a, E> ReadArray<2> for NextCdrReader<'a, E> {
//     fn read_array_n(&mut self) -> XTypesResult<[u8; 2]> {
//         todo!()
//     }
// }
// impl<'a, E> ReadArray<4> for NextCdrReader<'a, E> {
//     fn read_array_n(&mut self) -> XTypesResult<[u8; 4]> {
//         todo!()
//     }
// }


impl<'a, E: EndiannessRead> NextCdrReader<'a, E> {
    fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            buffer,
            pos: 0,
            _endianness: endianness,
        }
    }
    fn read_byte(&mut self) -> XTypesResult<u8> {
        if self.pos + 1 > self.buffer.len() {
            return Err(XTypesError::NotEnoughData);
        }
        let ret = self.buffer[self.pos];
        self.pos += 1;
        Ok(ret)
    }

    fn read_array<const N: usize>(&mut self) -> XTypesResult<[u8; N]> {
        self.read_all(N)?
            .try_into()
            .map_err(|_e| XTypesError::InvalidData)
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
impl<'a, E: EndiannessRead> Read for NextCdrReader<'a, E> {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]> {
        self.read_all(size)
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
            } else if !member_descriptor.is_optional {
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
            } else if !member_descriptor.is_optional {
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
                let mut sequence = Vec::new();
                while self.cdr1_deserializer.reader.seek_to_pid_le(pid)? {
                    sequence.push(
                        self.deserialize_complex_value(
                            member_descriptor
                                .r#type
                                .get_descriptor()
                                .element_type
                                .expect("must have element_type"),
                        )?,
                    );
                }
                // Only add empty sequences to non-optional members
                if !(sequence.is_empty() && member_descriptor.is_optional) {
                    dynamic_data.set_complex_values(pid as u32, sequence)?;
                }
            } else if self.cdr1_deserializer.reader.seek_to_pid_le(pid)? {
                self.deserialize_final_member(member, dynamic_data)?;
            } else if member_descriptor.is_key {
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
    use crate::{
        dcps::xtypes_glue::key_and_instance_handle::get_instance_handle_from_dynamic_data,
        infrastructure::type_support::TypeSupport,
    };

    #[test]
    fn cyclone_dispose_message() {
        #[derive(Debug, PartialEq, TypeSupport)]
        pub struct DisposeDataType {
            #[dust_dds(key)]
            pub name: String,
            pub value: u8,
        }
        let dispose_serialized_key_from_data_message = deserialize_key_only(
            DisposeDataType::TYPE,
            &[
                0x0, 0x1, 0x0, 0x1, 0xf, 0x0, 0x0, 0x0, 0x56, 0x65, 0x72, 0x79, 0x20, 0x4c, 0x6f,
                0x6e, 0x67, 0x20, 0x4e, 0x61, 0x6d, 0x65, 0x0, 0x0,
            ],
        )
        .unwrap();
        let instance_handle =
            get_instance_handle_from_dynamic_data(dispose_serialized_key_from_data_message.clone())
                .unwrap();
        assert_eq!(
            instance_handle,
            [
                170, 156, 253, 79, 26, 61, 29, 0, 160, 59, 3, 163, 8, 9, 203, 167,
            ]
        )
    }

    #[test]
    fn deserialize_final_struct() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalType {
            field_u16: u16,
            field_u64: u64,
        }
        let mut expected = DynamicDataFactory::create_data(FinalType::TYPE);
        FinalType {
            field_u16: 7,
            field_u64: 9,
        }
        .create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_top_level_type(
                FinalType::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalType::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalType::TYPE,
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
            deserialize_full(
                FinalType::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // field_u16 | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                ],
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

        let mut expected = DynamicDataFactory::create_data(NestedFinalType::TYPE);
        NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        }
        .create_dynamic_sample(&mut expected);

        assert_eq!(
            deserialize_full(
                NestedFinalType::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                NestedFinalType::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                    10, //u8
                ],
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            deserialize_full(
                NestedFinalType::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 7, 0, 0, // nested FinalType (u16) | padding
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ],
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            deserialize_full(
                NestedFinalType::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                    10, //u8
                ],
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

        let mut expected = DynamicDataFactory::create_data(FinalTypeWithSequence::TYPE);
        FinalTypeWithSequence {
            field_u16: 7,
            field_u64: 9,
            field_seq_u32: vec![1, 4],
        }
        .create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                FinalTypeWithSequence::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    0, 0, 0, 2, // field_seq_u32 Length (u32)
                    0, 0, 0, 1, // field_seq_u32[0]
                    0, 0, 0, 4, // field_seq_u32[0]
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalTypeWithSequence::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    2, 0, 0, 0, // field_seq_u32 Length (u32)
                    1, 0, 0, 0, // field_seq_u32[0]
                    4, 0, 0, 0, // field_seq_u32[0]
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalTypeWithSequence::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 7, 0, 0, // field_u16 | padding (2 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    0, 0, 0, 2, // field_seq_u32 Length (u32)
                    0, 0, 0, 1, // field_seq_u32[0]
                    0, 0, 0, 4, // field_seq_u32[0]
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalTypeWithSequence::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // field_u16 | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    2, 0, 0, 0, // field_seq_u32 Length (u32)
                    1, 0, 0, 0, // field_seq_u32[0]
                    4, 0, 0, 0, // field_seq_u32[0]
                ],
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_string() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalString(String);
        let mut expected = DynamicDataFactory::create_data(FinalString::TYPE);
        FinalString(String::from("Hola")).create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                FinalString::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 0, 0, 5, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalString::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    5, 0, 0, 0, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalString::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 0, 0, 5, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                FinalString::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    5, 0, 0, 0, //length
                    b'H', b'o', b'l', b'a', // str
                    0x00, // terminating 0
                ],
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_bytes() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct ByteArray([u8; 2]);
        let mut expected = DynamicDataFactory::create_data(ByteArray::TYPE);
        ByteArray([1u8, 2]).create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                ByteArray::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    1, 2
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                ByteArray::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    1, 2
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                ByteArray::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    1, 2
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                ByteArray::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // CD2R_LE
                    1, 2
                ],
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

        let mut expected = DynamicDataFactory::create_data(Sequence::TYPE);
        Sequence(vec![Atype(1), Atype(2)]).create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                Sequence::TYPE,
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
        let mut expected = DynamicDataFactory::create_data(MutableType::TYPE);
        MutableType {
            key: 7,
            participant_key: 8,
        }
        .create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                MutableType::TYPE,
                &[
                    0x00, 0x02, 0x00, 0x00, // PL_CDR_BE
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
            deserialize_full(
                MutableType::TYPE,
                &[
                    0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
                    0x05A, 0x00, 1, 0, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x050, 0x00, 4, 0, // PID | length
                    8, 0, 0, 0, // participant_key
                    0, 0, 0, 0, // Sentinel
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                MutableType::TYPE,
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
            deserialize_full(
                MutableType::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // PL_CDR2_LE
                    0x05A, 0x00, 1, 0, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x050, 0x00, 4, 0, // PID | length
                    8, 0, 0, 0, // participant_key
                    0, 0, 0, 0, // Sentinel
                ],
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
        let mut expected = DynamicDataFactory::create_data(AppendableType::TYPE);
        AppendableType {
            key: 7,
            participant_key: 8,
        }
        .create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                AppendableType::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    7, 0, 0, 0, // key | padding
                    0, 0, 0, 8, // participant_key
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                AppendableType::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, // key | padding
                    8, 0, 0, 0, // participant_key
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                AppendableType::TYPE,
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
            deserialize_full(
                AppendableType::TYPE,
                &[
                    0x00, 0x09, 0x00, 0x00, // D_CDR2_LE
                    0, 0, 0, 8, // DHEADER
                    7, 0, 0, 0, // key | padding
                    8, 0, 0, 0, // participant_key
                ],
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
        let mut expected = DynamicDataFactory::create_data(AppendableShapesType::TYPE);
        AppendableShapesType {
            color: String::from("BLUE"),
            x: 10,
            y: 20,
            shapesize: 30,
            additional_payload_size: vec![],
        }
        .create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_full(
                AppendableShapesType::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
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
            deserialize_full(
                AppendableShapesType::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    5, 0, 0, 0, // color: length
                    b'B', b'L', b'U', b'E', // color
                    0, 0, 0, 0, // color: terminating 0 | padding
                    10, 0, 0, 0, // x
                    20, 0, 0, 0, // y
                    30, 0, 0, 0, // shapesize
                    0, 0, 0, 0, // additional_payload_size: length
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_full(
                AppendableShapesType::TYPE,
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
            deserialize_full(
                AppendableShapesType::TYPE,
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
                ],
            )
            .unwrap(),
            expected
        );
    }
}
