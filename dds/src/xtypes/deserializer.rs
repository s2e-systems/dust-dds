use crate::xtypes::{
    dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, ExtensibilityKind,
        TypeKind,
    },
    error::{XTypesError::{self, PidNotFound}, XTypesResult},
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

trait EndiannessRead {
    fn read_i16(reader: &mut NextCdrReader) -> XTypesResult<i16>;
    fn read_u32(reader: &mut NextCdrReader) -> XTypesResult<u32>;
    fn read_u16(reader: &mut NextCdrReader) -> XTypesResult<u16>;
    fn read_i64(reader: &mut NextCdrReader) -> XTypesResult<i64>;
    fn read_i32(reader: &mut NextCdrReader) -> XTypesResult<i32>;
    fn read_u64(reader: &mut NextCdrReader) -> XTypesResult<u64>;
    fn read_i128(reader: &mut NextCdrReader) -> XTypesResult<i128>;
    fn read_f32(reader: &mut NextCdrReader) -> XTypesResult<f32>;
    fn read_f64(reader: &mut NextCdrReader) -> XTypesResult<f64>;
}

fn read_array<const N: usize>(reader: &mut NextCdrReader) -> XTypesResult<[u8; N]> {
    Ok(*reader.read_bytes(N)?.as_array().expect("must have length"))
}

struct BigEndian;
impl EndiannessRead for BigEndian {
    fn read_i16(reader: &mut NextCdrReader) -> XTypesResult<i16> {
        Ok(i16::from_be_bytes(read_array(reader)?))
    }
    fn read_u16(reader: &mut NextCdrReader) -> XTypesResult<u16> {
        Ok(u16::from_be_bytes(read_array(reader)?))
    }
    fn read_i32(reader: &mut NextCdrReader) -> XTypesResult<i32> {
        Ok(i32::from_be_bytes(read_array(reader)?))
    }
    fn read_u32(reader: &mut NextCdrReader) -> XTypesResult<u32> {
        Ok(u32::from_be_bytes(read_array(reader)?))
    }
    fn read_i64(reader: &mut NextCdrReader) -> XTypesResult<i64> {
        Ok(i64::from_be_bytes(read_array(reader)?))
    }
    fn read_u64(reader: &mut NextCdrReader) -> XTypesResult<u64> {
        Ok(u64::from_be_bytes(read_array(reader)?))
    }
    fn read_i128(reader: &mut NextCdrReader) -> XTypesResult<i128> {
        Ok(i128::from_be_bytes(read_array(reader)?))
    }
    fn read_f32(reader: &mut NextCdrReader) -> XTypesResult<f32> {
        Ok(f32::from_be_bytes(read_array(reader)?))
    }
    fn read_f64(reader: &mut NextCdrReader) -> XTypesResult<f64> {
        Ok(f64::from_be_bytes(read_array(reader)?))
    }
}

struct LittleEndian;
impl EndiannessRead for LittleEndian {
    fn read_i16(reader: &mut NextCdrReader) -> XTypesResult<i16> {
        Ok(i16::from_le_bytes(read_array(reader)?))
    }
    fn read_u16(reader: &mut NextCdrReader) -> XTypesResult<u16> {
        Ok(u16::from_le_bytes(read_array(reader)?))
    }
    fn read_i32(reader: &mut NextCdrReader) -> XTypesResult<i32> {
        Ok(i32::from_le_bytes(read_array(reader)?))
    }
    fn read_u32(reader: &mut NextCdrReader) -> XTypesResult<u32> {
        Ok(u32::from_le_bytes(read_array(reader)?))
    }
    fn read_i64(reader: &mut NextCdrReader) -> XTypesResult<i64> {
        Ok(i64::from_le_bytes(read_array(reader)?))
    }
    fn read_u64(reader: &mut NextCdrReader) -> XTypesResult<u64> {
        Ok(u64::from_le_bytes(read_array(reader)?))
    }
    fn read_i128(reader: &mut NextCdrReader) -> XTypesResult<i128> {
        Ok(i128::from_le_bytes(read_array(reader)?))
    }
    fn read_f32(reader: &mut NextCdrReader) -> XTypesResult<f32> {
        Ok(f32::from_le_bytes(read_array(reader)?))
    }
    fn read_f64(reader: &mut NextCdrReader) -> XTypesResult<f64> {
        Ok(f64::from_le_bytes(read_array(reader)?))
    }
}

trait EncodingVersion {
    const MAX_ALIGN: usize;

    fn seek_to_pid<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        pid: u16,
    ) -> XTypesResult<()>;

    /// Serialization Rule (9) & (10)
    fn deserialize_array_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (12)
    fn deserialize_sequence_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (15) & (16)
    fn _deserialize_map_type<'a, E: EndiannessRead, V: EncodingVersion>(
        _deserializer: &mut XTypesDeserializer<'a, E, V>,
    ) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (21) & (23)
    fn deserialize_mstruct_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (22) & (24) & (25)
    fn deserialize_mmember<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
        length: usize,
    ) -> XTypesResult<()>;

    /// Serialization Rule (27) & (28)
    fn _deserialize_munion_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (29) & (30)
    fn deserialize_appendable_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;
}

struct EncodingVersion1;
impl EncodingVersion for EncodingVersion1 {
    const MAX_ALIGN: usize = 8;

    fn seek_to_pid<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        pid: u16,
    ) -> XTypesResult<()> {
        const PID_SENTINEL: u16 = 1;
        loop {
            let current_pid: u16 = deserializer.deserialize_primitive_type()?;
            let length: u16 = deserializer.deserialize_primitive_type()?;
            if current_pid == pid {
                return Ok(());
            } else if current_pid == PID_SENTINEL {
                return Err(PidNotFound(pid));
            } else {
                deserializer.reader.seek(length as usize)?;
                deserializer.align(4)?;
            }
        }
    }

    fn deserialize_array_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let bound = member
            .descriptor
            .r#type
            .descriptor
            .bound
            .ok_or(XTypesError::InvalidType)?;
        deserializer.deserialize_sequence_elements(member, dynamic_data, bound as usize)
    }

    /// Serialization Rule (12)
    fn deserialize_sequence_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let length = deserializer.deserialize_primitive_type::<u32>()?;
        deserializer.deserialize_sequence_elements(member, dynamic_data, length as usize)
    }

    /// Serialization Rule (23)
    fn deserialize_mstruct_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            V::deserialize_mmember(deserializer, member, dynamic_data, 0)?;
        }
        const PID_SENTINEL: u16 = 1;
        V::seek_to_pid(deserializer, PID_SENTINEL)?;
        Ok(())
    }

    /// Serialization Rule (24) & (25)
    fn deserialize_mmember<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
        _length: usize,
    ) -> XTypesResult<()> {
        // (24) using short PL encoding when both M.id <= 2^14 and M.value.ssize <= 2^16
        deserializer.align(4);
        let pid: u16 = member.get_id() as u16;
        let orig_pos = deserializer.reader.pos;
        let result = if V::seek_to_pid(deserializer, pid).is_ok() {
            deserializer.deserialize_nopt_fmember(member, dynamic_data)
        } else {
            Ok(())
        };
        deserializer.reader.set_position(orig_pos);
        result

        // TODO (25) using long PL encoding
    }

    /// Serialization Rule (29)
    fn deserialize_appendable_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        deserializer.deserialize_fstruct_type(dynamic_type, dynamic_data)
    }
}

struct EncodingVersion2;
impl EncodingVersion for EncodingVersion2 {
    const MAX_ALIGN: usize = 4;

    fn seek_to_pid<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        pid: u16,
    ) -> XTypesResult<()> {
        loop {
            let emheader: u32 = deserializer.deserialize_primitive_type()?;
            let current_pid = (emheader & 0x0fffffff) as u16;
            let length = ((emheader & 0b01110000_00000000_00000000_00000000) >> 28) as usize;
            if current_pid == pid {
                return Ok(());
            } else {
                deserializer.reader.seek(length)?;
                deserializer.align(4)?;
            }
        }
    }

    fn deserialize_array_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = deserializer.deserialize_primitive_type::<u32>()?;
        let bound = member
            .descriptor
            .r#type
            .descriptor
            .bound
            .ok_or(XTypesError::InvalidType)?;
        deserializer.deserialize_sequence_elements(member, dynamic_data, bound as usize)
    }

    /// Serialization Rule (12)
    fn deserialize_sequence_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = deserializer.deserialize_primitive_type::<u32>()?;
        let length = deserializer.deserialize_primitive_type::<u32>()?;
        deserializer.deserialize_sequence_elements(member, dynamic_data, length as usize)
    }

    /// Serialization Rule (21)
    fn deserialize_mstruct_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let dheader = deserializer.deserialize_primitive_type::<u32>()?;
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            V::deserialize_mmember(deserializer, member, dynamic_data, dheader as usize)?;
        }
        Ok(())
    }

    /// Serialization Rule (22)
    fn deserialize_mmember<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
        length: usize,
    ) -> XTypesResult<()> {
        deserializer.align(4);
        // TODO: If LC(C)>=4
        //let _next_int = deserializer.deserialize_primitive_type::<u32>();
        deserializer.align(4);
        let pid: u16 = member.get_id() as u16;
        let orig_pos = deserializer.reader.pos;
        let result = if V::seek_to_pid(deserializer, pid).is_ok() {
            deserializer.deserialize_nopt_fmember(member, dynamic_data)
        } else if !member.descriptor.is_optional {
            Err(XTypesError::PidNotFound(pid))
        } else {
            Ok(())
        };
        deserializer.reader.set_position(orig_pos);
        result
    }

    /// Serialization Rule (30)
    fn deserialize_appendable_type<'a, E: EndiannessRead, V: EncodingVersion>(
        deserializer: &mut XTypesDeserializer<'a, E, V>,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = deserializer.deserialize_primitive_type::<u32>();
        deserializer.deserialize_fstruct_type(dynamic_type, dynamic_data)
    }
}

/// Serialization Rule (1)
pub fn deserialize_top_level_type<'a>(
    dynamic_type: DynamicType<'a>,
    buffer: &[u8],
) -> XTypesResult<DynamicData<'a>> {
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
    reader: NextCdrReader<'a>,
    _endianness: E,
    _encoding_version: V,
}

fn is_element_type_kind_primitive(member: &DynamicTypeMember) -> XTypesResult<bool> {
    Ok(matches!(
        member
            .descriptor
            .r#type
            .descriptor
            .element_type
            .ok_or(XTypesError::InvalidType)?
            .get_kind(),
        TypeKind::BOOLEAN
            | TypeKind::BYTE
            | TypeKind::INT16
            | TypeKind::INT32
            | TypeKind::INT64
            | TypeKind::UINT16
            | TypeKind::UINT32
            | TypeKind::UINT64
            | TypeKind::FLOAT32
            | TypeKind::FLOAT64
            | TypeKind::FLOAT128
            | TypeKind::INT8
            | TypeKind::UINT8
            | TypeKind::CHAR8
            | TypeKind::CHAR16
    ))
}

impl<'a, E: EndiannessRead, V: EncodingVersion> XTypesDeserializer<'a, E, V> {
    fn new(buffer: &'a [u8], encoding_version: V, endianness: E) -> Self {
        Self {
            reader: NextCdrReader::new(buffer),
            _endianness: endianness,
            _encoding_version: encoding_version,
        }
    }

    fn align(&mut self, alignment: usize) -> XTypesResult<()>{
        self.reader.seek_padding(alignment)
    }

    fn deserialize_primitive_sequence_elements<O: AsBytes + Align>(
        &mut self,
        length: usize,
    ) -> XTypesResult<Vec<O>> {
        let mut sequence = Vec::with_capacity(length);
        for _ in 0..length {
            sequence.push(self.deserialize_primitive_type()?);
        }
        Ok(sequence)
    }

    fn deserialize_sequence_elements(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
        length: usize,
    ) -> XTypesResult<()> {
        let element_type = member
            .descriptor
            .r#type
            .descriptor
            .element_type
            .ok_or(XTypesError::InvalidType)?;
        match element_type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => dynamic_data.set_boolean_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::BYTE => dynamic_data.set_byte_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::INT16 => dynamic_data.set_int16_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::INT32 => dynamic_data.set_int32_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::INT64 => dynamic_data.set_int64_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::UINT16 => dynamic_data.set_uint16_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::UINT32 => dynamic_data.set_uint32_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::UINT64 => dynamic_data.set_uint64_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::FLOAT32 => dynamic_data.set_float32_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::FLOAT64 => dynamic_data.set_float64_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::FLOAT128 => dynamic_data.set_float128_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::INT8 => dynamic_data.set_int8_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::UINT8 => dynamic_data.set_uint8_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::CHAR8 => dynamic_data.set_char8_values(
                member.get_id(),
                self.deserialize_primitive_sequence_elements(length)?,
            ),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.deserialize_string_type()?);
                }
                dynamic_data.set_string_values(member.get_id(), values)
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.deserialize_as_nested(element_type)?);
                }
                dynamic_data.set_complex_values(member.get_id(), values)
            }
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }

    fn deserialize_as_nested<'b>(
        &mut self,
        dynamic_type: DynamicType<'b>,
    ) -> XTypesResult<DynamicData<'b>> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);

        fn deserialize_as_nested_inner<'a, E: EndiannessRead, V: EncodingVersion>(
            deserializer: &mut XTypesDeserializer<'a, E, V>,
            dynamic_type: DynamicType,
            dynamic_data: &mut DynamicData,
        ) -> XTypesResult<()> {
            // Deserialize the top-level which must be either a struct, an enum or a union.
            match dynamic_type.get_descriptor().kind {
                TypeKind::STRUCTURE => match dynamic_type.get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => {
                        deserializer.deserialize_fstruct_type(dynamic_type, dynamic_data)
                    }
                    ExtensibilityKind::Appendable => {
                        V::deserialize_appendable_type(deserializer, dynamic_type, dynamic_data)
                    }
                    ExtensibilityKind::Mutable => {
                        V::deserialize_mstruct_type(deserializer, dynamic_type, dynamic_data)
                    }
                },
                TypeKind::ENUM => deserializer.deserialize_enum_type(dynamic_type, dynamic_data),

                TypeKind::UNION => todo!(),
                kind => {
                    debug!("Expected structure, enum or union. Got kind {kind:?} ");
                    Err(XTypesError::InvalidType)
                }
            }
        }

        // We start by deserializing the base type if it exists before proceeding with the rest of the type
        if let Some(base_dynamic_type) = dynamic_type.descriptor.base_type {
            deserialize_as_nested_inner(self, base_dynamic_type, &mut dynamic_data)?;
        }
        deserialize_as_nested_inner(self, dynamic_type, &mut dynamic_data)?;

        Ok(dynamic_data)
    }

    fn deserialize_as_value(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        match member.descriptor.r#type.get_kind() {
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
            TypeKind::ENUM => dynamic_data.set_complex_value(
                member.get_id(),
                self.deserialize_as_nested(member.descriptor.r#type)?,
            ),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => dynamic_data.set_complex_value(
                member.get_id(),
                self.deserialize_as_nested(member.descriptor.r#type)?,
            ),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => {
                if is_element_type_kind_primitive(member)? {
                    self.deserialize_psequence_type(member, dynamic_data)
                } else {
                    V::deserialize_sequence_type(self, member, dynamic_data)
                }
            }
            TypeKind::ARRAY => {
                if is_element_type_kind_primitive(member)? {
                    self.deserialize_parray_type(member, dynamic_data)
                } else {
                    V::deserialize_array_type(self, member, dynamic_data)
                }
            }
            TypeKind::MAP => todo!(),
        }
    }

    /// Serialization Rule (2)
    fn deserialize_primitive_type<O: AsBytes + Align>(&mut self) -> XTypesResult<O> {
        O::align::<V>(&mut self.reader);
        O::as_bytes::<E>(&mut self.reader)
    }

    /// Serialization Rule (3) & (4)
    fn deserialize_string_type(&mut self) -> XTypesResult<String> {
        let length = self.deserialize_primitive_type::<u32>()?;
        let values = self.reader.read_bytes(length as usize - 1)?.to_vec();
        self.reader.read_byte()?; // 0-termination
        String::from_utf8(values).map_err(|_| XTypesError::InvalidData)
    }

    /// Serialization Rule (5)
    fn deserialize_enum_type(
        &mut self,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let discriminator_type = dynamic_type
            .descriptor
            .discriminator_type
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

    /// Serialization Rule (6)
    fn _deserialize_bitmask_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (7)
    fn _deserialize_alias_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (8)
    fn deserialize_parray_type(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let bound = member
            .descriptor
            .r#type
            .descriptor
            .bound
            .ok_or(XTypesError::InvalidType)?;
        self.deserialize_sequence_elements(member, dynamic_data, bound as usize)
    }

    /// Serialization Rule (11)
    fn deserialize_psequence_type(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let length = self.deserialize_primitive_type::<u32>()?;
        self.deserialize_sequence_elements(member, dynamic_data, length as usize)
    }

    /// Serialization Rule (14)
    fn _deserialize_pmap_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (17)
    fn deserialize_fstruct_type(
        &mut self,
        dynamic_type: DynamicType,
        dynamic_data: &mut DynamicData<'_>,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            self.deserialize_nopt_fmember(member, dynamic_data)?;
        }
        Ok(())
    }

    /// Serialization Rule (18)
    fn deserialize_nopt_fmember(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData<'_>,
    ) -> XTypesResult<()> {
        self.deserialize_as_value(member, dynamic_data)
    }

    /// Serialization Rule (26)
    fn _deserialize_funion_type(&mut self) -> XTypesResult<DynamicData<'_>> {
        todo!()
    }
}

trait Align {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>);
}
impl Align for bool {
    fn align<'a, V: EncodingVersion>(_reader: &mut NextCdrReader<'a>) {}
}
impl Align for u8 {
    fn align<'a, V: EncodingVersion>(_reader: &mut NextCdrReader<'a>) {}
}
impl Align for u16 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for u32 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for u64 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(core::cmp::min(Self::BITS as usize / 8, V::MAX_ALIGN));
    }
}
impl Align for i8 {
    fn align<'a, V: EncodingVersion>(_reader: &mut NextCdrReader<'a>) {}
}
impl Align for i16 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for i32 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(Self::BITS as usize / 8);
    }
}
impl Align for i64 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(core::cmp::min(Self::BITS as usize / 8, V::MAX_ALIGN));
    }
}
impl Align for i128 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(core::cmp::min(Self::BITS as usize / 8, V::MAX_ALIGN));
    }
}
impl Align for f32 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(32 / 8);
    }
}
impl Align for f64 {
    fn align<'a, V: EncodingVersion>(reader: &mut NextCdrReader<'a>) {
        reader.seek_padding(core::cmp::min(64 / 8, V::MAX_ALIGN));
    }
}
impl Align for char {
    fn align<'a, V: EncodingVersion>(_reader: &mut NextCdrReader<'a>) {}
}

trait AsBytes {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self>
    where
        Self: Sized;
}
impl AsBytes for bool {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        match reader.read_byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(XTypesError::InvalidData),
        }
    }
}
impl AsBytes for u8 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        reader.read_byte()
    }
}
impl AsBytes for u16 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_u16(reader)
    }
}
impl AsBytes for u32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_u32(reader)
    }
}
impl AsBytes for u64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_u64(reader)
    }
}
impl AsBytes for i8 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        Ok(reader.read_byte()? as Self)
    }
}
impl AsBytes for i16 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_i16(reader)
    }
}
impl AsBytes for i32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_i32(reader)
    }
}
impl AsBytes for i64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_i64(reader)
    }
}
impl AsBytes for i128 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_i128(reader)
    }
}
impl AsBytes for f32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_f32(reader)
    }
}
impl AsBytes for f64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        E::read_f64(reader)
    }
}
impl AsBytes for char {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut NextCdrReader<'a>) -> XTypesResult<Self> {
        Ok(char::from(reader.read_byte()?))
    }
}

struct NextCdrReader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> NextCdrReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, pos: 0 }
    }
    fn read_byte(&mut self) -> XTypesResult<u8> {
        if self.pos + 1 > self.buffer.len() {
            return Err(XTypesError::NotEnoughData);
        }
        let ret = self.buffer[self.pos];
        self.pos += 1;
        Ok(ret)
    }

    fn read_bytes(&mut self, length: usize) -> XTypesResult<&'a [u8]> {
        if self.pos + length > self.buffer.len() {
            return Err(XTypesError::NotEnoughData);
        }
        let ret = &self.buffer[self.pos..self.pos + length];
        self.pos += length;
        Ok(ret)
    }

    fn seek(&mut self, v: usize) -> XTypesResult<()>{
        if self.pos + v > self.buffer.len() {
            Err(XTypesError::NotEnoughData)
        } else {
            self.pos += v;
            Ok(())
        }
    }

    fn seek_padding(&mut self, alignment: usize) -> XTypesResult<()> {
        let mask = alignment - 1;
        self.seek(((self.pos + mask) & !mask) - self.pos)
    }

    fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dcps::xtypes_glue::key_and_instance_handle::get_instance_handle_from_dynamic_data,
        xtypes::type_support::{Type, TypeSupport},
    };

    #[test]
    fn cyclone_dispose_message() {
        #[derive(Debug, PartialEq, TypeSupport)]
        pub struct DisposeDataType {
            #[dust_dds(key)]
            pub name: String,
            pub value: u8,
        }

        let mut dispose_data_type_key_holder = DisposeDataType::TYPE;
        let mut member_list = dispose_data_type_key_holder.member_list.to_vec();
        member_list.retain(|f| f.descriptor.is_key);
        dispose_data_type_key_holder.member_list = &member_list;

        let dispose_serialized_key_from_data_message = deserialize_top_level_type(
            dispose_data_type_key_holder,
            &[
                0x0, 0x1, 0x0, 0x1, // CDR Header
                0xf, 0x0, 0x0, 0x0, // String length
                0x56, 0x65, 0x72, 0x79, // String
                0x20, 0x4c, 0x6f, 0x6e, // String
                0x67, 0x20, 0x4e, 0x61, // String
                0x6d, 0x65, 0x0, 0x0, // String | terminating 0 | 1 byte padding
            ],
        )
        .unwrap();
        let instance_handle =
            get_instance_handle_from_dynamic_data(dispose_serialized_key_from_data_message)
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
            field_u32: u32,
        }
        let mut expected = DynamicDataFactory::create_data(FinalType::TYPE);
        FinalType {
            field_u16: 7,
            field_u64: 9,
            field_u32: 10,
        }
        .create_dynamic_sample(&mut expected);
        assert_eq!(
            deserialize_top_level_type(
                FinalType::TYPE,
                &[
                    0x00, 0x00, 0x00, 0x00, // CDR_BE
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    0, 0, 0, 10, // field_u32
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_top_level_type(
                FinalType::TYPE,
                &[
                    0x00, 0x01, 0x00, 0x00, // CDR_LE
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    10, 0, 0, 0, // field_u32
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_top_level_type(
                FinalType::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 7, 0, 0, // field_u16 | padding (2 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    0, 0, 0, 10, // field_u32
                ],
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_top_level_type(
                FinalType::TYPE,
                &[
                    0x00, 0x07, 0x00, 0x00, // CDR2_LE
                    7, 0, 0, 0, // field_u16 | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    10, 0, 0, 0, // field_u32
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
        assert_eq!(
            deserialize_top_level_type(
                Sequence::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // CDR2_BE
                    0, 0, 0, 6, // DHEADER
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
            deserialize_top_level_type(
                MutableType::TYPE,
                &[
                    0x00, 0x02, 0x00, 0x00, // PL_CDR_BE
                    0x00, 0x05A, 0, 1, // PID | length
                    7, 0, 0, 0, // key | padding
                    0x00, 0x050, 0, 4, // PID | length
                    0, 0, 0, 8, // participant_key
                    0, 1, 0, 0, // Sentinel
                ],
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            deserialize_top_level_type(
                MutableType::TYPE,
                &[
                    0x00, 0x06, 0x00, 0x00, // PL_CDR2_BE
                    0, 0, 0, 16, // DHEADER
                    0, 0, 0x00, 0x05A, // LC=0 (=> length = 1 byte) + PID  (EMHEADER)
                    7, 0, 0, 0, // key | padding
                    0x20, 0x00, 0x00, 0x050, // LC=2 (=> length = 4 bytes) + PID  (EMHEADER)
                    0, 0, 0, 8, // participant_key
                ],
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_mutable_struct_with_enum() {
        #[derive(Debug, PartialEq, TypeSupport)]
        enum Kind {
            Zero,
            One,
        }

        #[derive(Debug, PartialEq, TypeSupport)]
        struct UserType {
            kind: Kind,
        }

        let mut expected = DynamicDataFactory::create_data(UserType::TYPE);
        UserType { kind: Kind::One }.create_dynamic_sample(&mut expected);

        assert_eq!(
            deserialize_top_level_type(
                UserType::TYPE,
                &[
                    0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
                    1, 0, 0, 0, // Kind one
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
            deserialize_top_level_type(
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
    }
}
