use crate::xtypes::{
    dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, ExtensibilityKind,
        TypeKind,
    },
    error::{
        XTypesError::{self, PidNotFound},
        XTypesResult,
    },
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
    fn read_i16(reader: &mut Reader) -> XTypesResult<i16>;
    fn read_u32(reader: &mut Reader) -> XTypesResult<u32>;
    fn read_u16(reader: &mut Reader) -> XTypesResult<u16>;
    fn read_i64(reader: &mut Reader) -> XTypesResult<i64>;
    fn read_i32(reader: &mut Reader) -> XTypesResult<i32>;
    fn read_u64(reader: &mut Reader) -> XTypesResult<u64>;
    fn read_i128(reader: &mut Reader) -> XTypesResult<i128>;
    fn read_f32(reader: &mut Reader) -> XTypesResult<f32>;
    fn read_f64(reader: &mut Reader) -> XTypesResult<f64>;
}

fn read_array<const N: usize>(reader: &mut Reader) -> XTypesResult<[u8; N]> {
    let mut array = [0u8; N];
    array.copy_from_slice(reader.read_bytes(N)?);
    Ok(array)
}

struct BigEndian;
impl EndiannessRead for BigEndian {
    fn read_i16(reader: &mut Reader) -> XTypesResult<i16> {
        Ok(i16::from_be_bytes(read_array(reader)?))
    }
    fn read_u16(reader: &mut Reader) -> XTypesResult<u16> {
        Ok(u16::from_be_bytes(read_array(reader)?))
    }
    fn read_i32(reader: &mut Reader) -> XTypesResult<i32> {
        Ok(i32::from_be_bytes(read_array(reader)?))
    }
    fn read_u32(reader: &mut Reader) -> XTypesResult<u32> {
        Ok(u32::from_be_bytes(read_array(reader)?))
    }
    fn read_i64(reader: &mut Reader) -> XTypesResult<i64> {
        Ok(i64::from_be_bytes(read_array(reader)?))
    }
    fn read_u64(reader: &mut Reader) -> XTypesResult<u64> {
        Ok(u64::from_be_bytes(read_array(reader)?))
    }
    fn read_i128(reader: &mut Reader) -> XTypesResult<i128> {
        Ok(i128::from_be_bytes(read_array(reader)?))
    }
    fn read_f32(reader: &mut Reader) -> XTypesResult<f32> {
        Ok(f32::from_be_bytes(read_array(reader)?))
    }
    fn read_f64(reader: &mut Reader) -> XTypesResult<f64> {
        Ok(f64::from_be_bytes(read_array(reader)?))
    }
}

struct LittleEndian;
impl EndiannessRead for LittleEndian {
    fn read_i16(reader: &mut Reader) -> XTypesResult<i16> {
        Ok(i16::from_le_bytes(read_array(reader)?))
    }
    fn read_u16(reader: &mut Reader) -> XTypesResult<u16> {
        Ok(u16::from_le_bytes(read_array(reader)?))
    }
    fn read_i32(reader: &mut Reader) -> XTypesResult<i32> {
        Ok(i32::from_le_bytes(read_array(reader)?))
    }
    fn read_u32(reader: &mut Reader) -> XTypesResult<u32> {
        Ok(u32::from_le_bytes(read_array(reader)?))
    }
    fn read_i64(reader: &mut Reader) -> XTypesResult<i64> {
        Ok(i64::from_le_bytes(read_array(reader)?))
    }
    fn read_u64(reader: &mut Reader) -> XTypesResult<u64> {
        Ok(u64::from_le_bytes(read_array(reader)?))
    }
    fn read_i128(reader: &mut Reader) -> XTypesResult<i128> {
        Ok(i128::from_le_bytes(read_array(reader)?))
    }
    fn read_f32(reader: &mut Reader) -> XTypesResult<f32> {
        Ok(f32::from_le_bytes(read_array(reader)?))
    }
    fn read_f64(reader: &mut Reader) -> XTypesResult<f64> {
        Ok(f64::from_le_bytes(read_array(reader)?))
    }
}

const PID_SENTINEL: u16 = 1;

trait EncodingVersion: Sized {
    fn align<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        alignment: usize,
    ) -> XTypesResult<()>;

    fn seek_to_pid<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        pid: u16,
    ) -> XTypesResult<u16>;

    /// Serialization Rule (9) & (10)
    fn deserialize_array_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (12)
    fn deserialize_sequence_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (15) & (16)
    fn _deserialize_map_type<'a, E: EndiannessRead>(
        _deserializer: &mut XTypesDeserializer<'a, E, Self>,
    ) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (19) & (20)
    fn deserialize_opt_fmember<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (21) & (23)
    fn deserialize_mstruct_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (22) & (24) & (25)
    fn deserialize_mmember<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    /// Serialization Rule (27) & (28)
    fn _deserialize_munion_type(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Serialization Rule (29) & (30)
    fn deserialize_appendable_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;
}

struct EncodingVersion1;
impl EncodingVersion for EncodingVersion1 {
    fn align<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        alignment: usize,
    ) -> XTypesResult<()> {
        deserializer.reader.seek_padding(alignment)
    }

    fn seek_to_pid<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        pid: u16,
    ) -> XTypesResult<u16> {
        loop {
            let current_pid: u16 = deserializer.deserialize_primitive_type()?;
            let length: u16 = deserializer.deserialize_primitive_type()?;
            if current_pid == pid {
                return Ok(length);
            } else if current_pid == PID_SENTINEL {
                return Err(PidNotFound(pid));
            } else {
                deserializer.reader.seek(length as usize)?;
                Self::align(deserializer, 4)?;
            }
        }
    }

    /// Arrays (any extensibility) using version 1 encoding
    ///
    /// (10) XCDR[1] << {O : ARRAY_TYPE} =
    ///                  XCDR
    ///                    << { O[i] : O.element_type }*
    fn deserialize_array_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
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

    /// Sequences (any extensibility) using version 1 encoding
    ///
    /// (13) XCDR[1] << {O : SEQUENCE_TYPE} =
    ///                   XCDR
    ///                     << { O.length : UInt32 }
    ///                     << { O[i] : O.element_type }*
    fn deserialize_sequence_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let length = deserializer.deserialize_primitive_type::<u32>()?;
        deserializer.deserialize_sequence_elements(member, dynamic_data, length as usize)
    }

    /// Optional member of final Aggregated type (structure, union), version 1
    /// see (26) and (27) for MMEMBER serialization
    ///
    /// (19) XCDR[1] << {M : OPT_FMEMBER} =
    ///                    XCDR
    ///                    << { M : MMEMBER }
    fn deserialize_opt_fmember<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        Self::deserialize_mmember(deserializer, member, dynamic_data)
    }

    /// Structures with extensibility MUTABLE, version 1 encoding
    ///
    /// (23) XCDR[1] << {O : MSTRUCT_TYPE} =
    ///                  XCDR
    ///                    << { O.member[i] : MMEMBER }*
    ///                    << { PID_SENTINEL : UInt16 }
    ///                    << { length = 0 : UInt16 }
    fn deserialize_mstruct_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        deserializer.deserialize_members(dynamic_data)?;
        Self::seek_to_pid(deserializer, PID_SENTINEL)?;
        Ok(())
    }

    /// Member of mutable aggregated type (structure, union), version 1 encoding
    /// using short PL encoding when both M.id <= 2^14 and M.value.ssize <= 2^16
    ///
    /// (24) XCDR[1] << {M : MMEMBER} =
    ///                  XCDR
    ///                   << ALIGN(4)
    ///                   << { FLAG_I + FLAG_M + M.id : UInt16 }
    ///                   << { M.value.ssize : UInt16 }
    ///                   << PUSH( ORIGIN=0 )
    ///                   << { M.value : M.value.type }
    fn deserialize_mmember<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        Self::align(deserializer, 4)?;
        let pid: u16 = member.get_id() as u16;
        let orig_pos = deserializer.reader.pos;
        let result = if let Ok(length) = Self::seek_to_pid(deserializer, pid) {
            if length > 0 {
                deserializer.deserialize_nopt_fmember(member, dynamic_data)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        deserializer.reader.pos = orig_pos;
        result

        // TODO (25) using long PL encoding
    }

    /// Extensibility APPENDABLE (Collection or Aggregated types), version 1
    /// encoding
    /// (29) XCDR[1] << {O : APPENDABLE_TYPE} =
    ///                  XCDR
    ///                    << { O : AsFinal(O.type) }
    fn deserialize_appendable_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        deserializer.deserialize_fstruct_type(dynamic_data)
    }
}

struct EncodingVersion2;
impl EncodingVersion for EncodingVersion2 {
    fn seek_to_pid<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        pid: u16,
    ) -> XTypesResult<u16> {
        loop {
            let emheader: u32 = deserializer.deserialize_primitive_type()?;
            let current_pid = (emheader & 0x0fffffff) as u16;
            let lc = (emheader & 0b01110000_00000000_00000000_00000000) >> 28;
            let length = match lc {
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 8,
                4 => deserializer.deserialize_primitive_type::<u32>()?,
                5 => deserializer.deserialize_primitive_type::<u32>()?,
                6 => 4 * deserializer.deserialize_primitive_type::<u32>()?,
                7 => 8 * deserializer.deserialize_primitive_type::<u32>()?,
                _ => unimplemented!("LC not possible"),
            };

            if current_pid == pid {
                if lc == 5 {
                    deserializer.reader.pos -= 4;
                }
                return Ok(length as u16);
            } else {
                deserializer.reader.seek(length as usize)?;
                Self::align(deserializer, 4)?;
            }
        }
    }

    /// Arrays (any extensibility) using version 2 encoding
    ///
    /// (9) XCDR[2] << {O : ARRAY_TYPE} =
    ///                 XCDR
    ///                   << { DHEADER(O) : UINT32 }
    ///                   << { O[i] : O.element_type }*
    fn deserialize_array_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
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

    /// Sequences (any extensibility) using version 2 encoding
    ///
    /// (12) XCDR[2] << {O : SEQUENCE_TYPE} =
    ///                  XCDR
    ///                    << { DHEADER(O) : UINT32 }
    ///                    << { O.length : UINT32 }
    ///                    << { O[i] : O.element_type }*
    fn deserialize_sequence_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = deserializer.deserialize_primitive_type::<u32>()?;
        let length = deserializer.deserialize_primitive_type::<u32>()?;
        deserializer.deserialize_sequence_elements(member, dynamic_data, length as usize)
    }

    /// (21) XCDR[2] << {O : MSTRUCT_TYPE} =
    ///                  XCDR
    ///                    << { DHEADER(O) : UInt32 }
    ///                    << { O.member[i] : MMEMBER }*
    fn deserialize_mstruct_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = deserializer.deserialize_primitive_type::<u32>()?;
        deserializer.deserialize_members(dynamic_data)?;
        Ok(())
    }

    /// Member of mutable aggregated type (structure, union), version 2 encoding
    ///
    /// (22) XCDR[2] << {M : MMEMBER} =
    ///                  XCDR
    ///                    << { EMHEADER1(M) : UInt32 }
    ///                    << IF (LC(M)>=4) { NEXTINT(M) : UInt32 }
    ///                    << IF (LC(M)>=5) XCDR.offset = XCDR.offset-4
    ///                    << { M.value : M.value.type }
    fn deserialize_mmember<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        Self::align(deserializer, 4)?;
        // TODO: If LC(C)>=4
        //let _next_int = deserializer.deserialize_primitive_type::<u32>();
        let pid: u16 = member.get_id() as u16;
        let orig_pos = deserializer.reader.pos;
        let result = if Self::seek_to_pid(deserializer, pid).is_ok() {
            deserializer.deserialize_value(member, dynamic_data)
        } else if !member.descriptor.is_optional {
            Err(XTypesError::PidNotFound(pid))
        } else {
            Ok(())
        };
        deserializer.reader.pos = orig_pos;
        result
    }

    /// Extensibility APPENDABLE (Collection or Aggregated types), version 2
    /// encoding
    /// (30) XCDR[2] << {O : APPENDABLE_TYPE} =
    ///                   XCDR
    ///                     << { DHEADER(O) : UInt32 }
    ///                     << { O : AsFinal(O.type) }
    fn deserialize_appendable_type<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let _dheader = deserializer.deserialize_primitive_type::<u32>();
        deserializer.deserialize_fstruct_type(dynamic_data)
    }

    fn align<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        alignment: usize,
    ) -> XTypesResult<()> {
        deserializer
            .reader
            .seek_padding(core::cmp::min(alignment, 4))
    }

    /// Optional member of final aggregated type (structure, union), version 2
    ///
    /// (20) XCDR[2] << {M : OPT_FMEMBER} =
    ///                   XCDR
    ///                   << { <is_present> : BOOLEAN }
    ///                   << IF (<is_present>) { M.value : M.value.type }
    fn deserialize_opt_fmember<'a, E: EndiannessRead>(
        deserializer: &mut XTypesDeserializer<'a, E, Self>,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let is_present = deserializer.deserialize_primitive_type::<bool>()?;
        if is_present {
            deserializer.deserialize_value(member, dynamic_data)
        } else {
            Ok(())
        }
    }
}

/// Serialization Rule
/// (1) XCDR << {O : TOP_LEVEL_TYPE} =
///             XCDR
///               << INIT( OFFSET=0, ORIGIN=0,
///                        CENDIAN=<E>, EVERSION=<eversion> )
///               << { ENC_HEADER(<E>, <eversion>, O.type) : Byte[2] }
///               << PUSH( EVERSION = <eversion> )
///               << PUSH( MAXALIGN = MAXALIGN(<eversion>) )
///               << PUSH( ORIGIN = 0 )
///               << { <OPTIONS> : Byte[2] }
///               << { O : AsNested(O.type) }
pub fn deserialize_top_level_type<'a>(
    dynamic_type: DynamicType<'a>,
    buffer: &[u8],
) -> XTypesResult<DynamicData<'a>> {
    if buffer.len() < 4 {
        return Err(XTypesError::NotEnoughData);
    }
    deserialize_top_level_type_from_representation_identifier(
        dynamic_type,
        [buffer[0], buffer[1]],
        &buffer[4..],
    )
}

/// This is an addidtional function to the "Serialization Rule (1)" deserialization function.
/// It can be used if the representation identifier is already known
pub fn deserialize_top_level_type_from_representation_identifier<'a>(
    dynamic_type: DynamicType<'a>,
    representation_identifier: RepresentationIdentifier,
    data: &[u8],
) -> XTypesResult<DynamicData<'a>> {
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
    reader: Reader<'a>,
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
            reader: Reader { buffer, pos: 0 },
            _endianness: endianness,
            _encoding_version: encoding_version,
        }
    }

    /// Serialization rule: { O.member[i] : MMEMBER }*
    fn deserialize_members(&mut self, dynamic_data: &mut DynamicData) -> XTypesResult<()> {
        let dynamic_type = dynamic_data.r#type();
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            V::deserialize_mmember(self, member, dynamic_data)?;
        }
        Ok(())
    }

    /// Serialization rule: { O[i] : O.element_type }*
    fn deserialize_sequence_elements(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
        length: usize,
    ) -> XTypesResult<()> {
        fn deserialize_primitive_sequence_elements<
            'a,
            O: AsBytes + Align,
            E: EndiannessRead,
            V: EncodingVersion,
        >(
            deserializer: &mut XTypesDeserializer<'a, E, V>,
            length: usize,
        ) -> XTypesResult<Vec<O>> {
            let mut sequence = Vec::with_capacity(length);
            for _ in 0..length {
                sequence.push(deserializer.deserialize_primitive_type()?);
            }
            Ok(sequence)
        }

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
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::BYTE => dynamic_data
                .set_byte_values(member.get_id(), self.reader.read_bytes(length)?.to_vec()),
            TypeKind::INT16 => dynamic_data.set_int16_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::INT32 => dynamic_data.set_int32_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::INT64 => dynamic_data.set_int64_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::UINT16 => dynamic_data.set_uint16_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::UINT32 => dynamic_data.set_uint32_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::UINT64 => dynamic_data.set_uint64_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::FLOAT32 => dynamic_data.set_float32_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::FLOAT64 => dynamic_data.set_float64_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::FLOAT128 => dynamic_data.set_float128_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::INT8 => dynamic_data.set_int8_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
            ),
            TypeKind::UINT8 => dynamic_data
                .set_uint8_values(member.get_id(), self.reader.read_bytes(length)?.to_vec()),
            TypeKind::CHAR8 => dynamic_data.set_char8_values(
                member.get_id(),
                deserialize_primitive_sequence_elements(self, length)?,
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
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::ENUM | TypeKind::STRUCTURE | TypeKind::UNION => {
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.deserialize_as_nested(element_type)?);
                }
                dynamic_data.set_complex_values(member.get_id(), values)
            }
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }

    /// Serialization rule: { O : AsNested(O.type) }
    fn deserialize_as_nested<'b>(
        &mut self,
        dynamic_type: DynamicType<'b>,
    ) -> XTypesResult<DynamicData<'b>> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);

        let descriptor = dynamic_data.r#type().descriptor;
        match descriptor.kind {
            TypeKind::STRUCTURE => match descriptor.extensibility_kind {
                ExtensibilityKind::Final => self.deserialize_fstruct_type(&mut dynamic_data)?,
                ExtensibilityKind::Appendable => {
                    V::deserialize_appendable_type(self, &mut dynamic_data)?
                }
                ExtensibilityKind::Mutable => V::deserialize_mstruct_type(self, &mut dynamic_data)?,
            },
            TypeKind::ENUM => self.deserialize_enum_type(&mut dynamic_data)?,
            TypeKind::UNION => match descriptor.extensibility_kind {
                ExtensibilityKind::Final => self.deserialize_funion_type(&mut dynamic_data)?,
                ExtensibilityKind::Appendable => {
                    let _dheader = self.deserialize_primitive_type::<u32>()?;
                    self.deserialize_funion_type(&mut dynamic_data)?
                }
                ExtensibilityKind::Mutable => todo!(),
            },
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                Err(XTypesError::InvalidType)?
            }
        }
        Ok(dynamic_data)
    }

    /// Serialization rule: { M.value : M.value.type }
    fn deserialize_value(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        match member.descriptor.r#type.get_kind() {
            TypeKind::NONE => Ok(()),
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
            TypeKind::ENUM | TypeKind::STRUCTURE | TypeKind::UNION => dynamic_data
                .set_complex_value(
                    member.get_id(),
                    self.deserialize_as_nested(member.descriptor.r#type)?,
                ),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
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

    /// Serialization rule: { O.member : FMEMBER }
    fn deserialize_fmember(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        if member.descriptor.is_optional {
            V::deserialize_opt_fmember(self, member, dynamic_data)
        } else {
            self.deserialize_nopt_fmember(member, dynamic_data)
        }
    }
    /// (2) XCDR << {O : PRIMITIVE_TYPE} =
    ///              XCDR
    ///                << ALIGN( O.ssize )
    ///                << ESWAP( AsBytes(O) )
    fn deserialize_primitive_type<O: AsBytes + Align>(&mut self) -> XTypesResult<O> {
        V::align(self, O::SSIZE)?;
        O::as_bytes::<E>(&mut self.reader)
    }

    /// (3) XCDR << {O : STRING_TYPE} =
    ///               XCDR
    ///                << { O.ssize : UInt32 } // includes NUL
    ///                << { O[i] : Byte }* // includes NUL
    fn deserialize_string_type(&mut self) -> XTypesResult<String> {
        let length = self.deserialize_primitive_type::<u32>()?;
        let values = self.reader.read_bytes(length as usize - 1)?.to_vec();
        self.reader.read_byte()?; // 0-termination
        String::from_utf8(values).map_err(|_| XTypesError::InvalidData)
    }

    /// (5) XCDR << {O : ENUM_TYPE} =
    ///              XCDR
    ///                << { O.value : O.holder_type }
    fn deserialize_enum_type(&mut self, dynamic_data: &mut DynamicData) -> XTypesResult<()> {
        let discriminator_type = dynamic_data
            .r#type()
            .descriptor
            .discriminator_type
            .ok_or(XTypesError::InvalidType)?;
        match discriminator_type.get_kind() {
            TypeKind::INT8 => {
                let value = self.deserialize_primitive_type::<i8>()?;
                dynamic_data.set_int8_value(0, value)
            }
            TypeKind::INT16 => {
                let value = self.deserialize_primitive_type::<i16>()?;
                dynamic_data.set_int16_value(0, value)
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

    /// Arrays of primitive element type (version 1 and 2 encoding)
    ///
    /// (8) XCDR << {O : PARRAY_TYPE} =
    ///              XCDR
    ///                << { O[i] : O.element_type }*
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

    /// Sequences of primitive element type (version 1 and 2 encoding)
    ///
    /// (11) XCDR << { O : PSEQUENCE_TYPE } =
    ///               XCDR
    ///                 << { O.length : UInt32 }
    ///                 << { O[i] : O.element_type }*
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

    /// Structures with extensibility FINAL (version 1 and 2 encoding)
    /// FMMEBER can be NOPT_FMEMBER (18) or OPT_FMEMBER (19)
    ///
    /// (17) XCDR << {O : FSTRUCT_TYPE} =
    ///               XCDR
    ///                << { O.member[i] : FMEMBER }*
    fn deserialize_fstruct_type(&mut self, dynamic_data: &mut DynamicData) -> XTypesResult<()> {
        let dynamic_type = dynamic_data.r#type();
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            self.deserialize_fmember(member, dynamic_data)?;
        }
        Ok(())
    }

    /// Non-optional member of final Aggregated type (structure, union)
    ///
    /// (18) XCDR << {M : NOPT_FMEMBER} =
    ///                XCDR
    ///                  << { M.value : M.value.type }
    fn deserialize_nopt_fmember(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData<'_>,
    ) -> XTypesResult<()> {
        self.deserialize_value(member, dynamic_data)
    }

    /// Unions with extensibility FINAL (version 1 and 2 encoding)
    /// see (18) to (20) for NOPT_FMEMBER and FMEMBER serialization
    ///
    /// (26) XCDR << {O : FUNION_TYPE} =
    ///               XCDR
    ///                 << { O.disc : NOPT_FMEMBER }
    ///                 << { O.selected_member : FMEMBER }?
    fn deserialize_funion_type(&mut self, dynamic_data: &mut DynamicData) -> XTypesResult<()> {
        let dynamic_type = dynamic_data.r#type();
        // Deserialize the discriminator
        let disc_member = dynamic_type.get_member_by_index(0)?;
        self.deserialize_nopt_fmember(disc_member, dynamic_data)?;

        // The discriminator value represents the id of a member
        let disc_id = match dynamic_data.get_value(disc_member.get_id())? {
            crate::xtypes::data_storage::DataStorage::UInt8(x) => *x as u32,
            crate::xtypes::data_storage::DataStorage::Int8(x) => *x as u32,
            crate::xtypes::data_storage::DataStorage::UInt16(x) => *x as u32,
            crate::xtypes::data_storage::DataStorage::Int16(x) => *x as u32,
            crate::xtypes::data_storage::DataStorage::Int32(x) => *x as u32,
            crate::xtypes::data_storage::DataStorage::UInt32(x) => *x,
            _ => return Err(XTypesError::InvalidType),
        };

        // Deserialize the member based on its discriminator
        let member = dynamic_type.get_member(disc_id)?;
        self.deserialize_fmember(member, dynamic_data)
    }
}

trait Align {
    const SSIZE: usize;
}
impl Align for u8 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for u16 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for u32 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for u64 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for i8 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for i16 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for i32 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for i64 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for i128 {
    const SSIZE: usize = Self::BITS as usize / 8;
}
impl Align for f32 {
    const SSIZE: usize = 4;
}
impl Align for f64 {
    const SSIZE: usize = 8;
}
impl Align for bool {
    const SSIZE: usize = 1;
}
impl Align for char {
    const SSIZE: usize = 1;
}

trait AsBytes {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self>
    where
        Self: Sized;
}
impl AsBytes for bool {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        match reader.read_byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(XTypesError::InvalidData),
        }
    }
}
impl AsBytes for u8 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        reader.read_byte()
    }
}
impl AsBytes for u16 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_u16(reader)
    }
}
impl AsBytes for u32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_u32(reader)
    }
}
impl AsBytes for u64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_u64(reader)
    }
}
impl AsBytes for i8 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        Ok(reader.read_byte()? as Self)
    }
}
impl AsBytes for i16 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_i16(reader)
    }
}
impl AsBytes for i32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_i32(reader)
    }
}
impl AsBytes for i64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_i64(reader)
    }
}
impl AsBytes for i128 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_i128(reader)
    }
}
impl AsBytes for f32 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_f32(reader)
    }
}
impl AsBytes for f64 {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        E::read_f64(reader)
    }
}
impl AsBytes for char {
    fn as_bytes<'a, E: EndiannessRead>(reader: &mut Reader<'a>) -> XTypesResult<Self> {
        Ok(char::from(reader.read_byte()?))
    }
}

struct Reader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
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

    fn seek(&mut self, v: usize) -> XTypesResult<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dcps::{
            data_representation_builtin_endpoints::type_lookup::{
                RequestHeader, SampleIdentity, TypeLookupCall, TypeLookupGetTypesIn,
                TypeLookupRequest,
            },
            xtypes_glue::key_and_instance_handle::get_instance_handle_from_dynamic_data,
        },
        transport::types::{EntityId, Guid},
        xtypes::{
            type_object::{
                TypeIdentifier, TypeIdentifierWithDependencies, TypeIdentifierWithSize,
                TypeInformation,
            },
            type_support::{Type, TypeSupport},
        },
    };

    #[test]
    fn cyclone_dispose_message() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct DisposeDataType {
            #[dust_dds(key)]
            name: String,
            value: u8,
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
            get_instance_handle_from_dynamic_data(&dispose_serialized_key_from_data_message)
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
        let expected = FinalType {
            field_u16: 7,
            field_u64: 9,
            field_u32: 10,
        }
        .create_dynamic_sample();
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

        let expected = NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        }
        .create_dynamic_sample();

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

        let expected = FinalTypeWithSequence {
            field_u16: 7,
            field_u64: 9,
            field_seq_u32: vec![1, 4],
        }
        .create_dynamic_sample();
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
        let expected = FinalString(String::from("Hola")).create_dynamic_sample();
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
        let expected = ByteArray([1u8, 2]).create_dynamic_sample();
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
    fn deserialize_array_with_lc4() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct TestType {
            #[dust_dds(id = 41)]
            member: [u8; 3],
        }

        let expected = TestType { member: [1, 2, 3] }.create_dynamic_sample();
        assert_eq!(
            deserialize_top_level_type(
                TestType::TYPE,
                &[
                    0x00, 0x02, 0x00, 0x00, // CDR Header
                    0x00, 41, 0, 3, // PID, length
                    1, 2, 3, 0, // member | padding (1 bytres)
                    0, 1, 0, 0, // Sentinel
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_top_level_type(
                TestType::TYPE,
                &[
                    0x00, 0x03, 0x00, 0x00, // CDR Header
                    41, 0x00, 3, 0, // PID, length
                    1, 2, 3, 0, // member | padding (2 bytres)
                    1, 0, 0, 0, // Sentinel
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_top_level_type(
                TestType::TYPE,
                &[
                    0x00, 0x0a, 0x00, 0x01, // CDR Header
                    0, 0, 0, 11, // DHEADER
                    0b100_0000, 0, 0, 41, // EMHEADER1 incl. LC 4
                    0, 0, 0, 3, // NEXTINT
                    1, 2, 3, 0, // member | padding (1 bytres)
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_top_level_type(
                TestType::TYPE,
                &[
                    0x00, 0x0b, 0x00, 0x01, // CDR Header
                    11, 0, 0, 0, // DHEADER
                    41, 0, 0, 0b100_0000, // EMHEADER1 incl. LC 4
                    3, 0, 0, 0, // NEXTINT
                    1, 2, 3, 0, // member | padding (1 bytres)
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_array_with_lc_bigger_5() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct TestType16 {
            #[dust_dds(id = 41)]
            m1: [u8; 16],
            #[dust_dds(id = 42)]
            m2: u32,
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct TestType24 {
            #[dust_dds(id = 41)]
            m1: [u8; 24],
            #[dust_dds(id = 42)]
            m2: u32,
        }
        assert_eq!(
            deserialize_top_level_type(
                TestType16::TYPE,
                &[
                    0x00, 0x0a, 0x00, 0x00, // CDR Header
                    0, 0, 0, 32, // DHEADER
                    0b110_0000, 0, 0, 41, // EMHEADER1 incl. LC 6
                    0, 0, 0, 4, // NEXTINT: length == 4 * 4
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    0b010_0000, 0, 0, 42, // EMHEADER1 incl. LC 2
                    0, 0, 0, 6 // m2
                ]
            )
            .unwrap(),
            TestType16 { m1: [1; 16], m2: 6 }.create_dynamic_sample()
        );
        assert_eq!(
            deserialize_top_level_type(
                TestType24::TYPE,
                &[
                    0x00, 0x0a, 0x00, 0x00, // CDR Header
                    0, 0, 0, 32, // DHEADER
                    0b111_0000, 0, 0, 41, // EMHEADER1 incl. LC 7
                    0, 0, 0, 3, // NEXTINT: length == 8 * 3
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    1, 1, 1, 1, // m1
                    0b010_0000, 0, 0, 42, // EMHEADER1 incl. LC 2
                    0, 0, 0, 6 // m2
                ]
            )
            .unwrap(),
            TestType24 { m1: [1; 24], m2: 6 }.create_dynamic_sample()
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
        let expected = MutableType {
            key: 7,
            participant_key: 8,
        }
        .create_dynamic_sample();
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

        let expected = UserType { kind: Kind::One }.create_dynamic_sample();

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
        let expected = AppendableType {
            key: 7,
            participant_key: 8,
        }
        .create_dynamic_sample();
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
        let expected = AppendableShapesType {
            color: String::from("BLUE"),
            x: 10,
            y: 20,
            shapesize: 30,
            additional_payload_size: vec![],
        }
        .create_dynamic_sample();
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

    #[test]
    fn deserialize_type_lookup_get_types_in() {
        let expected = TypeLookupCall::TypeLookupGetTypesHashId {
            get_types: TypeLookupGetTypesIn {
                type_ids: vec![TypeIdentifier::EkComplete {
                    equivalence_hash: [5; 14],
                }],
            },
        }
        .create_dynamic_sample();

        assert_eq!(
            deserialize_top_level_type(
                TypeLookupCall::TYPE,
                &[
                    0x00u8, 0x09, 0x00, 0x01, // D_CDR2_LE + padding
                    35, 0, 0, 0, // TypeLookupCall DHEADER
                    0xd3, 0x52, 0x82, 0x01, // TypeLookupGetTypesHashId DISCRIMINATOR
                    27, 0, 0, 0, // type_ids: DHEADER
                    101, 96, 83, 92, // type_ids EMHEADER
                    // 23, 0, 0, 0, // type_ids NEXTINT
                    19, 0, 0, 0, // type_ids: DHEADER
                    1, 0, 0, 0,   // type_ids: Length
                    242, //EK_COMPLETE
                    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // equivalence_hash
                    0, // padding
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_type_lookup_request() {
        let expected = TypeLookupRequest {
            header: RequestHeader {
                request_id: SampleIdentity {
                    writer_guid: Guid::new([1; 12], EntityId::new([1; 3], 1)),
                    sequence_number: 5.into(),
                },
                instance_name: String::from(""),
            },
            call: TypeLookupCall::TypeLookupGetTypesHashId {
                get_types: TypeLookupGetTypesIn { type_ids: vec![] },
            },
        }
        .create_dynamic_sample();

        assert_eq!(
            deserialize_top_level_type(
                TypeLookupRequest::TYPE,
                &[
                    0x00u8, 0x07, 0x00, 0x00, // CDR2_LE + padding
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // writer_guid
                    0, 0, 0, 0, // sequence_number: high
                    5, 0, 0, 0, // sequence_number: low
                    1, 0, 0, 0, // instance_name: length
                    0, // instance_name: length
                    0, 0, 0, // Padding
                    20, 0, 0, 0, //TypeLookupCall DHEADER
                    211, 82, 130, 1, // TypeLookupGetTypesHashId Discriminator
                    12, 0, 0, 0, // type_ids: DHEADER
                    101, 96, 83, 92, // type_ids EMHEADER
                    // 19, 0, 0, 0, // type_ids NEXTINT (skipped, replaced by DHEADER)
                    4, 0, 0, 0, // type_ids: DHEADER
                    0, 0, 0, 0, // type_ids: Length
                ]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_type_identifier() {
        let expected = TypeInformation {
            minimal: TypeIdentifierWithDependencies {
                typeid_with_size: TypeIdentifierWithSize {
                    type_id: TypeIdentifier::EkMinimal {
                        equivalence_hash: [5; 14],
                    },
                    typeobject_serialized_size: 10,
                },
                dependent_typeid_count: 0,
                dependent_typeids: Vec::new(),
            },
            complete: TypeIdentifierWithDependencies {
                typeid_with_size: TypeIdentifierWithSize {
                    type_id: TypeIdentifier::EkComplete {
                        equivalence_hash: [5; 14],
                    },
                    typeobject_serialized_size: 10,
                },
                dependent_typeid_count: 0,
                dependent_typeids: Vec::new(),
            },
        }
        .create_dynamic_sample();

        assert_eq!(
            deserialize_top_level_type(
                TypeInformation::TYPE,
                &[
                    0x00u8, 0x0B, 0x00, 0x00, // PL_CDR2_LE + 0 padding
                    88, 0, 0, 0, // DHEADER
                    0x01, 0x10, 0, 80, // EMHEADER
                    // 40, 0, 0, 0, // NEXTINT (Reused in the EMHEADER)
                    36, 0, 0, 0, // DHEADER
                    20, 0, 0, 0,   // DHEADER
                    241, //EK_MINIMAL
                    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // equivalence_hash
                    0, // padding
                    10, 0, 0, 0, // typeobject_serialized_size
                    0, 0, 0, 0, // dependent_typeid_count
                    4, 0, 0, 0, // Sequence DHEADER
                    0, 0, 0, 0, // Sequence length
                    0x02, 0x10, 0, 80, // EMHEADER
                    // 40, 0, 0, 0, // NEXTINT (Reused in the EMHEADER)
                    36, 0, 0, 0, // DHEADER
                    20, 0, 0, 0,   // DHEADER
                    242, //EK_COMPLETE
                    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // equivalence_hash
                    0, // padding
                    10, 0, 0, 0, // typeobject_serialized_size
                    0, 0, 0, 0, // dependent_typeid_count
                    4, 0, 0, 0, // Sequence DHEADER
                    0, 0, 0, 0, // Sequence length
                ]
            )
            .unwrap(),
            expected
        );
    }
}
