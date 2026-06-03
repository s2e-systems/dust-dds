use super::dynamic_type::ExtensibilityKind;
use crate::xtypes::{
    dynamic_type::{DynamicData, DynamicType, TypeKind},
    error::{XTypesError, XTypesResult},
    read_write::Write,
};
use alloc::{string::ToString, vec::Vec};

type RepresentationIdentifier = [u8; 2];
const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const CDR2_BE: RepresentationIdentifier = [0x00, 0x06];
const CDR2_LE: RepresentationIdentifier = [0x00, 0x07];
const D_CDR2_BE: RepresentationIdentifier = [0x00, 0x08];
const D_CDR2_LE: RepresentationIdentifier = [0x00, 0x09];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
const PL_CDR2_BE: RepresentationIdentifier = [0x00, 0x0a];
const PL_CDR2_LE: RepresentationIdentifier = [0x00, 0x0b];
const REPRESENTATION_OPTIONS: [u8; 2] = [0x00, 0x00];

pub fn serialize_cdr1_be(dynamic_data: &DynamicData) -> XTypesResult<Vec<u8>> {
    serialize_cdr1(dynamic_data, BigEndian)
}

pub fn serialize_cdr1_le(dynamic_data: &DynamicData) -> XTypesResult<Vec<u8>> {
    serialize_cdr1(dynamic_data, LittleEndian)
}

pub fn serialize_cdr2_be(dynamic_data: &DynamicData) -> XTypesResult<Vec<u8>> {
    serialize_cdr2(dynamic_data, BigEndian)
}

pub fn serialize_cdr2_le(dynamic_data: &DynamicData) -> XTypesResult<Vec<u8>> {
    serialize_cdr2(dynamic_data, LittleEndian)
}

pub fn serialize_rtps(dynamic_data: &DynamicData) -> XTypesResult<Vec<u8>> {
    let mut buffer = Vec::new();
    RtpsPlCdrSerializer::serialize(&mut buffer, dynamic_data)?;
    pad_entire_serialization(&mut buffer);
    Ok(buffer)
}

pub fn serialize_final_without_header<W: Write>(
    mut buffer: W,
    dynamic_data: &DynamicData,
) -> XTypesResult<W> {
    let mut s = Xcdr2Serializer {
        writer: CdrWriter::new(&mut buffer, BigEndian),
    };
    s.serialize_fstruct_type(dynamic_data)?;
    Ok(buffer)
}

fn serialize_cdr1(
    dynamic_data: &DynamicData,
    endianness: impl EndiannessWrite,
) -> XTypesResult<Vec<u8>> {
    let mut buffer = Vec::new();
    Xcdr1Serializer {
        writer: CdrWriter::new(&mut buffer, endianness),
    }
    .serialize_top_level_type(dynamic_data)?;
    pad_entire_serialization(&mut buffer);
    Ok(buffer)
}

fn serialize_cdr2(
    dynamic_data: &DynamicData,
    endianness: impl EndiannessWrite,
) -> XTypesResult<Vec<u8>> {
    let mut buffer = Vec::new();
    Xcdr2Serializer {
        writer: CdrWriter::new(&mut buffer, endianness),
    }
    .serialize_top_level_type(dynamic_data)?;
    pad_entire_serialization(&mut buffer);
    Ok(buffer)
}

fn pad_entire_serialization(buffer: &mut Vec<u8>) {
    let padding = match buffer.len() % 4 {
        1 => &[0, 0, 0][..],
        2 => &[0, 0][..],
        3 => &[0][..],
        _ => &[][..],
    };
    buffer.extend_from_slice(padding);
    buffer[3] = padding.len() as u8;
}

struct RtpsPlCdrSerializer<'a, W> {
    writer: CdrWriter<'a, W, LittleEndian>,
}

impl<'a, W: Write> RtpsPlCdrSerializer<'a, W> {
    fn serialize(buffer: &'a mut W, dynamic_data: &DynamicData) -> XTypesResult<()> {
        Self {
            writer: CdrWriter::new(buffer, LittleEndian),
        }
        .serialize_top_level_type(dynamic_data)
    }
}

struct Xcdr1Serializer<'a, W, E> {
    writer: CdrWriter<'a, W, E>,
}

struct Xcdr2Serializer<'a, W, E> {
    writer: CdrWriter<'a, W, E>,
}

// Trait for the serialization of types defined in the XTypes standard.
// The definitions follow the order and nomenclature of Table 40 – Symbols and notation used in the serialization virtual machine
// defined in the DDS-XTypes, version 1.3 - 7.4.3.5.1 Notation used for the match criteria
trait XTypesSerializer<'a> {
    fn as_bytes(&mut self, v: &impl CdrPrimitiveTypeSerialize);

    fn align(&mut self, v: usize);
    fn push_origin_0(&mut self);

    fn serialize_enc_header(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()>;

    fn serialize_options(&mut self) -> XTypesResult<()> {
        self.serialize_p_array_type(&REPRESENTATION_OPTIONS);
        Ok(())
    }
    fn serialize_dheader(&mut self, ssize: usize) {
        self.serialize_primitive_type(&(ssize as u32))
    }

    fn serialize_emheader1(
        &mut self,
        dynamic_data: &DynamicData,
        member_id: u32,
    ) -> XTypesResult<()> {
        let mut byte_conter = ByteCounter::new();
        let mut byte_conter_serializer = Xcdr2Serializer {
            writer: CdrWriter::new(&mut byte_conter, LittleEndian),
        };
        byte_conter_serializer.serialize_m_value(dynamic_data, member_id)?;
        let length = byte_conter.0;
        if length == 1 || length == 2 || length == 4 || length == 8 {
            let m_flag = 0;
            let lc = match length {
                1 => 0,
                2 => 1,
                4 => 2,
                8 => 3,
                _ => unreachable!(),
            } as u32;
            let emheader = (m_flag << 31) + (lc << 28) + (member_id & 0x0fffffff);
            self.serialize_primitive_type(&emheader);
        } else {
            self.serialize_primitive_type(&member_id);
            //todo!("LC (length code) not yet supproted")
        };
        Ok(())
    }

    fn serialize_t_as_final(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()> {
        match dynamic_data.r#type().get_kind() {
            TypeKind::ENUM => {
                self.serialize_enum_type(dynamic_data)?;
            }
            TypeKind::STRUCTURE => {
                self.serialize_fstruct_type(dynamic_data)?;
            }
            kind => unimplemented!("Should not reach for {kind:?}"),
        }
        Ok(())
    }

    fn serialize_t_as_nested(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()> {
        match dynamic_data.r#type().get_kind() {
            TypeKind::ENUM => {
                self.serialize_enum_type(dynamic_data)?;
            }
            TypeKind::STRUCTURE => {
                match dynamic_data.r#type().get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => self.serialize_fstruct_type(dynamic_data),
                    ExtensibilityKind::Appendable => self.serialize_appendable_type(dynamic_data),
                    ExtensibilityKind::Mutable => self.serialize_mstruct_type(dynamic_data),
                }?
            }
            TypeKind::UNION => {
                match dynamic_data.r#type().get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => self.serialize_funion_type(dynamic_data),
                    ExtensibilityKind::Appendable => self.serialize_appendable_type(dynamic_data),
                    ExtensibilityKind::Mutable => self.serialize_munion_type(dynamic_data),
                }?
            }
            kind => unimplemented!("Should not reach for {kind:?}"),
        }
        Ok(())
    }

    fn serialize_m_value(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => self.serialize_primitive_type(v.get_boolean_value(member_id)?),
            TypeKind::BYTE => self.serialize_primitive_type(v.get_byte_value(member_id)?),
            TypeKind::INT16 => self.serialize_primitive_type(v.get_int16_value(member_id)?),
            TypeKind::INT32 => self.serialize_primitive_type(v.get_int32_value(member_id)?),
            TypeKind::INT64 => self.serialize_primitive_type(v.get_int64_value(member_id)?),
            TypeKind::UINT16 => self.serialize_primitive_type(v.get_uint16_value(member_id)?),
            TypeKind::UINT32 => self.serialize_primitive_type(v.get_uint32_value(member_id)?),
            TypeKind::UINT64 => self.serialize_primitive_type(v.get_uint64_value(member_id)?),
            TypeKind::FLOAT32 => self.serialize_primitive_type(v.get_float32_value(member_id)?),
            TypeKind::FLOAT64 => self.serialize_primitive_type(v.get_float64_value(member_id)?),
            TypeKind::FLOAT128 => self.serialize_primitive_type(v.get_float128_value(member_id)?),
            TypeKind::INT8 => self.serialize_primitive_type(v.get_int8_value(member_id)?),
            TypeKind::UINT8 => self.serialize_primitive_type(v.get_uint8_value(member_id)?),
            TypeKind::CHAR8 => self.serialize_primitive_type(v.get_char8_value(member_id)?),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => self.serialize_string_type(v.get_string_value(member_id)?),
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => self.serialize_enum_type(v.get_complex_value(member_id)?)?,
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => self.serialize_t_as_nested(v.get_complex_value(member_id)?)?,
            TypeKind::UNION => self.serialize_funion_type(v)?,
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => self.serialize_sequence_type(v, member_id)?,
            TypeKind::ARRAY => self.serialize_array_type(v, member_id)?,
            TypeKind::MAP => todo!(),
        }
        Ok(())
    }

    /// Serialization Rule (1)
    fn serialize_top_level_type(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()> {
        self.serialize_enc_header(dynamic_data)?;
        self.serialize_options()?;
        self.push_origin_0();
        self.serialize_t_as_nested(dynamic_data)?;
        Ok(())
    }

    /// Serialization Rule (2)
    fn serialize_primitive_type<C: CdrPrimitiveTypeSerialize>(&mut self, v: &C) {
        self.align(C::BYTES);
        self.as_bytes(v);
    }

    /// Serialization Rule (3)
    fn serialize_string_type(&mut self, v: &str) {
        self.serialize_primitive_type(&(v.len() as u32 + 1));
        self.serialize_primitive_type(&v.as_bytes());
        self.serialize_primitive_type(&0u8);
    }

    /// Serialization Rule (4)
    fn _serialize_wstring_type(&mut self, _v: &str) {
        todo!()
    }

    /// Serialization Rule (5)
    fn serialize_enum_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        let holder_type = v
            .r#type()
            .descriptor
            .discriminator_type
            .ok_or(XTypesError::InvalidType)?;
        match holder_type.descriptor.kind {
            TypeKind::INT8 => {
                self.serialize_primitive_type(v.get_int8_value(0)?);
                Ok(())
            }
            TypeKind::INT16 => {
                self.serialize_primitive_type(v.get_int16_value(0)?);
                Ok(())
            }
            TypeKind::INT32 => {
                self.serialize_primitive_type(v.get_int32_value(0)?);
                Ok(())
            }
            _ => Err(XTypesError::InvalidType),
        }
    }

    /// Serialization Rule (6)
    fn _serialize_bitmask_type(&mut self, _v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    /// Serialization Rule (7)
    fn _serialize_alias_type(&mut self, _v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    /// Serialization Rule (8)
    fn serialize_p_array_type(&mut self, v: &[impl CdrPrimitiveTypeSerialize]) {
        for vi in v {
            self.serialize_primitive_type(vi);
        }
    }

    /// Serialization Rule (9) & (10)
    fn serialize_array_type(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        let element_type = member_descriptor
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("array has element type");
        match element_type.get_descriptor().kind {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => self.serialize_p_array_type(v.get_boolean_values(member_id)?),
            TypeKind::BYTE => self.serialize_p_array_type(v.get_byte_values(member_id)?),
            TypeKind::INT16 => self.serialize_p_array_type(v.get_int16_values(member_id)?),
            TypeKind::INT32 => self.serialize_p_array_type(v.get_int32_values(member_id)?),
            TypeKind::INT64 => self.serialize_p_array_type(v.get_int64_values(member_id)?),
            TypeKind::UINT16 => self.serialize_p_array_type(v.get_uint16_values(member_id)?),
            TypeKind::UINT32 => self.serialize_p_array_type(v.get_uint32_values(member_id)?),
            TypeKind::UINT64 => self.serialize_p_array_type(v.get_uint64_values(member_id)?),
            TypeKind::FLOAT32 => self.serialize_p_array_type(v.get_float32_values(member_id)?),
            TypeKind::FLOAT64 => self.serialize_p_array_type(v.get_float64_values(member_id)?),
            TypeKind::FLOAT128 => self.serialize_p_array_type(v.get_float128_values(member_id)?),
            TypeKind::INT8 => self.serialize_p_array_type(v.get_int8_values(member_id)?),
            TypeKind::UINT8 => self.serialize_p_array_type(v.get_uint8_values(member_id)?),
            TypeKind::CHAR8 => self.serialize_p_array_type(v.get_char8_values(member_id)?),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                for v in v.get_string_values(member_id)? {
                    self.serialize_string_type(v);
                }
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                for v in v.get_complex_values(member_id)? {
                    self.serialize_t_as_nested(v)?;
                }
            }
            TypeKind::UNION => {
                for v in v.get_complex_values(member_id)? {
                    self.serialize_funion_type(v)?;
                }
            }
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }

        Ok(())
    }

    /// Serialization Rule (11)
    fn serialize_p_sequence_type(&mut self, v: &[impl CdrPrimitiveTypeSerialize]) {
        self.serialize_primitive_type(&(v.len() as u32));

        for vi in v {
            self.serialize_primitive_type(vi);
        }
    }

    /// Serialization Rule (12) & (13)
    fn serialize_sequence_type(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        let element_type = member_descriptor
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("sequence has element type");
        match element_type.get_descriptor().kind {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => self.serialize_p_sequence_type(v.get_boolean_values(member_id)?),
            TypeKind::BYTE => self.serialize_p_sequence_type(v.get_byte_values(member_id)?),
            TypeKind::INT16 => self.serialize_p_sequence_type(v.get_int16_values(member_id)?),
            TypeKind::INT32 => self.serialize_p_sequence_type(v.get_int32_values(member_id)?),
            TypeKind::INT64 => self.serialize_p_sequence_type(v.get_int64_values(member_id)?),
            TypeKind::UINT16 => self.serialize_p_sequence_type(v.get_uint16_values(member_id)?),
            TypeKind::UINT32 => self.serialize_p_sequence_type(v.get_uint32_values(member_id)?),
            TypeKind::UINT64 => self.serialize_p_sequence_type(v.get_uint64_values(member_id)?),
            TypeKind::FLOAT32 => self.serialize_p_sequence_type(v.get_float32_values(member_id)?),
            TypeKind::FLOAT64 => self.serialize_p_sequence_type(v.get_float64_values(member_id)?),
            TypeKind::FLOAT128 => self.serialize_p_sequence_type(v.get_float128_values(member_id)?),
            TypeKind::INT8 => self.serialize_p_sequence_type(v.get_int8_values(member_id)?),
            TypeKind::UINT8 => self.serialize_p_sequence_type(v.get_uint8_values(member_id)?),
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                let list = v.get_string_values(member_id)?;
                self.serialize_primitive_type(&(list.len() as u32));
                for v in list {
                    self.serialize_string_type(v);
                }
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                let list = v.get_complex_values(member_id)?;
                self.serialize_primitive_type(&(list.len() as u32));
                for v in list {
                    self.serialize_t_as_nested(v)?;
                }
            }
            TypeKind::UNION => {
                let list = v.get_complex_values(member_id)?;
                self.serialize_primitive_type(&(list.len() as u32));
                for v in list {
                    self.serialize_funion_type(v)?;
                }
            }
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }

        Ok(())
    }

    /// Serialization Rule (14)
    fn _serialize_pmap_type(&mut self, _v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    /// Serialization Rule (15) & (16)
    fn _serialize_map_type(&mut self, _v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    /// Serialization Rule (17)
    fn serialize_fstruct_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let dynamic_type = v.r#type();
            if let Ok(member_id) = v.get_member_id_at_index(field_index) {
                if let Some(base_type) = dynamic_type.descriptor.base_type {
                    let member_descriptor = &base_type.get_member(member_id)?.descriptor;

                    if member_descriptor.is_optional {
                        self.serialize_opt_fmember(v, member_id)?;
                    } else {
                        self.serialize_nopt_fmember(v, member_id)?;
                    }
                }
                if let Ok(member) = &dynamic_type.get_member(member_id) {
                    if member.descriptor.is_optional {
                        self.serialize_opt_fmember(v, member_id)?;
                    } else {
                        self.serialize_nopt_fmember(v, member_id)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Serialization Rule (18)
    fn serialize_nopt_fmember(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        self.serialize_m_value(v, member_id)
    }

    /// Serialization Rule (19) & (20)
    fn serialize_opt_fmember(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        self.serialize_mmember(v, member_id)
    }

    /// Serialization Rule (21) & (23)
    fn serialize_mstruct_type(&mut self, v: &DynamicData) -> Result<(), XTypesError>;

    /// Serialization Rule (22) & (24) & (25)
    fn serialize_mmember(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError>;

    /// Serialization Rule (26)
    fn serialize_funion_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.serialize_nopt_fmember(v, 0)?;
        if let Ok(member_id) = v.get_member_id_at_index(1) {
            self.serialize_nopt_fmember(v, member_id)?;
        }
        Ok(())
    }

    /// Serialization Rule (27) & (28)
    fn serialize_munion_type(&mut self, _v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    /// Serialization Rule (29) & (30)
    fn serialize_appendable_type(&mut self, v: &DynamicData) -> Result<(), XTypesError>;
}

const PID_SENTINEL: u16 = 1;

impl<'a, W: Write> XTypesSerializer<'a> for RtpsPlCdrSerializer<'a, W> {
    fn align(&mut self, v: usize) {
        self.writer.pad(v);
    }

    fn push_origin_0(&mut self) {
        self.writer.position = 0;
    }

    fn serialize_enc_header(&mut self, _dynamic_data: &DynamicData) -> XTypesResult<()> {
        self.serialize_p_array_type(&PL_CDR_LE);
        Ok(())
    }

    fn serialize_mstruct_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        // That differs in the if memeber_descriptor.kind == Sequence for Structure kinds

        fn serialize_all_members<'a, W: Write>(
            serializer: &mut RtpsPlCdrSerializer<'a, W>,
            dynamic_type: &DynamicType,
            v: &DynamicData,
        ) -> Result<(), XTypesError> {
            if let Some(b) = &dynamic_type.descriptor.base_type {
                serialize_all_members(serializer, b, v)?;
            }

            for field_index in 0..dynamic_type.get_member_count() {
                let member = dynamic_type.get_member_by_index(field_index)?;
                let member_id = member.get_id();
                let member_descriptor = &member.descriptor;

                if v.get_value(member_id).is_err() {
                    continue;
                }

                if member_descriptor.r#type.get_kind() == TypeKind::SEQUENCE {
                    let element_type = member_descriptor
                        .r#type
                        .get_descriptor()
                        .element_type
                        .as_ref()
                        .expect("sequence has element type");
                    if element_type.get_descriptor().kind == TypeKind::STRUCTURE {
                        for vi in v.get_complex_values(member_id)? {
                            let mut byte_conter = ByteCounter::new();
                            let mut byte_conter_serializer = RtpsPlCdrSerializer {
                                writer: CdrWriter::new(&mut byte_conter, LittleEndian),
                            };
                            byte_conter_serializer.serialize_fstruct_type(vi)?;
                            byte_conter_serializer.align(4);

                            serializer.serialize_primitive_type(&(member_id as u16));
                            serializer.serialize_primitive_type(&(byte_conter.0 as u16));
                            serializer.serialize_fstruct_type(vi)?;
                            serializer.align(4);
                        }
                    } else {
                        serializer.serialize_mmember(v, member_id)?;
                    }
                } else {
                    serializer.serialize_mmember(v, member_id)?;
                }
            }
            Ok(())
        }
        serialize_all_members(self, &v.r#type(), v)?;
        self.align(4);
        self.serialize_primitive_type(&PID_SENTINEL);
        self.serialize_primitive_type(&0u16);
        Ok(())
    }

    fn serialize_mmember(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        // This differs from the Xcdr1LeSerializer in the counter that also adds the padding to the size

        let mut byte_conter = ByteCounter::new();
        let mut byte_conter_serializer = Xcdr1Serializer {
            writer: CdrWriter::new(&mut byte_conter, LittleEndian),
        };
        byte_conter_serializer.serialize_m_value(v, member_id)?;
        byte_conter_serializer.align(4);

        self.align(4);
        self.serialize_primitive_type(&(member_id as u16));
        self.serialize_primitive_type(&(byte_conter.0 as u16));
        self.push_origin_0();
        self.serialize_m_value(v, member_id)?;
        self.align(4);
        Ok(())
    }
    fn serialize_appendable_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.serialize_t_as_final(v)
    }

    fn as_bytes(&mut self, v: &impl CdrPrimitiveTypeSerialize) {
        v.serialize(&mut self.writer)
    }
}

impl<'a, W: Write, E: EndiannessWrite> XTypesSerializer<'a> for Xcdr1Serializer<'a, W, E> {
    fn as_bytes(&mut self, v: &impl CdrPrimitiveTypeSerialize) {
        v.serialize(&mut self.writer)
    }
    fn align(&mut self, v: usize) {
        self.writer.pad(core::cmp::min(v, 8))
    }
    fn push_origin_0(&mut self) {
        self.writer.position = 0;
    }

    fn serialize_enc_header(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()> {
        self.serialize_p_array_type(
            &match dynamic_data.r#type().get_descriptor().extensibility_kind {
                ExtensibilityKind::Final | ExtensibilityKind::Appendable => match E::ENDIANNESS {
                    Endianness::Big => CDR_BE,
                    Endianness::Little => CDR_LE,
                },
                ExtensibilityKind::Mutable => match E::ENDIANNESS {
                    Endianness::Big => PL_CDR_BE,
                    Endianness::Little => PL_CDR_LE,
                },
            },
        );
        Ok(())
    }

    fn serialize_mstruct_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.r#type().get_member_count() {
            if let Ok(member_id) = v.get_member_id_at_index(field_index) {
                self.serialize_mmember(v, member_id)?;
            }
        }
        // TODO: The alignment is not done in the Xtypes specification (possibly this needs to be deleted)
        self.align(4);
        self.serialize_primitive_type(&PID_SENTINEL);
        self.serialize_primitive_type(&0u16);
        Ok(())
    }

    fn serialize_mmember(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        // using short PL encoding when both M.id <= 2^14 and M.value.ssize <= 2^16

        let mut byte_conter = ByteCounter::new();
        let mut byte_conter_serializer = Xcdr1Serializer {
            writer: CdrWriter::new(&mut byte_conter, LittleEndian),
        };
        byte_conter_serializer.serialize_m_value(v, member_id)?;

        self.align(4);
        self.serialize_primitive_type(&(member_id as u16));
        self.serialize_primitive_type(&(byte_conter.0 as u16));
        self.push_origin_0();
        self.serialize_m_value(v, member_id)
    }

    fn serialize_appendable_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.serialize_t_as_final(v)
    }
}

impl<'a, W: Write, E: EndiannessWrite> XTypesSerializer<'a> for Xcdr2Serializer<'a, W, E> {
    fn as_bytes(&mut self, v: &impl CdrPrimitiveTypeSerialize) {
        v.serialize(&mut self.writer)
    }
    fn align(&mut self, v: usize) {
        self.writer.pad(core::cmp::min(v, 4))
    }
    fn push_origin_0(&mut self) {
        self.writer.position = 0;
    }

    fn serialize_enc_header(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()> {
        self.serialize_p_array_type(
            &match dynamic_data.r#type().get_descriptor().extensibility_kind {
                ExtensibilityKind::Final => match E::ENDIANNESS {
                    Endianness::Big => CDR2_BE,
                    Endianness::Little => CDR2_LE,
                },
                ExtensibilityKind::Appendable => match E::ENDIANNESS {
                    Endianness::Big => D_CDR2_BE,
                    Endianness::Little => D_CDR2_LE,
                },
                ExtensibilityKind::Mutable => match E::ENDIANNESS {
                    Endianness::Big => PL_CDR2_BE,
                    Endianness::Little => PL_CDR2_LE,
                },
            },
        );
        Ok(())
    }

    fn serialize_mstruct_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        fn serialize_this<'b, W: Write, E: EndiannessWrite>(
            serializer: &mut Xcdr2Serializer<'b, W, E>,
            v: &DynamicData,
        ) -> Result<(), XTypesError> {
            for field_index in 0..v.get_item_count() {
                if let Ok(member_id) = v.get_member_id_at_index(field_index) {
                    serializer.serialize_mmember(v, member_id)?;
                }
            }
            Ok(())
        }
        let mut byte_counter = ByteCounter::new();
        let mut byte_counter_serializer = Xcdr2Serializer {
            writer: CdrWriter::new(&mut byte_counter, LittleEndian),
        };
        serialize_this(&mut byte_counter_serializer, v)?;
        self.serialize_dheader(byte_counter.0);
        serialize_this(self, v)
    }

    fn serialize_mmember(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        self.serialize_emheader1(v, member_id)?;
        self.serialize_m_value(v, member_id)
    }

    fn serialize_appendable_type(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut byte_counter_serializer = Xcdr2Serializer {
            writer: CdrWriter::new(&mut byte_counter, LittleEndian),
        };
        byte_counter_serializer.serialize_t_as_final(v)?;
        self.serialize_dheader(byte_counter.0);
        self.serialize_t_as_final(v)
    }
}

trait CdrPrimitiveTypeSerialize {
    const BYTES: usize;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>);
}
impl CdrPrimitiveTypeSerialize for bool {
    const BYTES: usize = u8::BITS as usize / 8;

    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        writer.write(&[*self as u8]);
    }
}
impl CdrPrimitiveTypeSerialize for i8 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        writer.write(&[*self as u8]);
    }
}
impl CdrPrimitiveTypeSerialize for u8 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        writer.write(&[*self]);
    }
}
impl CdrPrimitiveTypeSerialize for i16 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_i16(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for u16 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_u16(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for i32 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_i32(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for u32 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_u32(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for i64 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_i64(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for u64 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_u64(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for i128 {
    const BYTES: usize = Self::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_i128(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for f32 {
    const BYTES: usize = 32 / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_f32(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for f64 {
    const BYTES: usize = 64 / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        E::write_f64(self, writer);
    }
}
impl CdrPrimitiveTypeSerialize for char {
    const BYTES: usize = u8::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        writer.write_slice(self.to_string().as_bytes());
    }
}
impl CdrPrimitiveTypeSerialize for &[u8] {
    const BYTES: usize = u8::BITS as usize / 8;
    fn serialize<W: Write, E: EndiannessWrite>(&self, writer: &mut CdrWriter<W, E>) {
        writer.write_slice(self);
    }
}
enum Endianness {
    Big,
    Little,
}
trait EndiannessWrite {
    const ENDIANNESS: Endianness;
    fn write_i16<C: Write>(v: &i16, writer: &mut C);
    fn write_u16<C: Write>(v: &u16, writer: &mut C);
    fn write_i32<C: Write>(v: &i32, writer: &mut C);
    fn write_u32<C: Write>(v: &u32, writer: &mut C);
    fn write_i64<C: Write>(v: &i64, writer: &mut C);
    fn write_u64<C: Write>(v: &u64, writer: &mut C);
    fn write_i128<C: Write>(v: &i128, writer: &mut C);
    fn write_f32<C: Write>(v: &f32, writer: &mut C);
    fn write_f64<C: Write>(v: &f64, writer: &mut C);
}

struct BigEndian;
impl EndiannessWrite for BigEndian {
    const ENDIANNESS: Endianness = Endianness::Big;

    fn write_i16<C: Write>(v: &i16, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_u16<C: Write>(v: &u16, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_i32<C: Write>(v: &i32, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_u32<C: Write>(v: &u32, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_i64<C: Write>(v: &i64, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_u64<C: Write>(v: &u64, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_i128<C: Write>(v: &i128, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_f32<C: Write>(v: &f32, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }

    fn write_f64<C: Write>(v: &f64, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
}

struct LittleEndian;
impl EndiannessWrite for LittleEndian {
    const ENDIANNESS: Endianness = Endianness::Little;

    fn write_i16<C: Write>(v: &i16, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_u16<C: Write>(v: &u16, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_i32<C: Write>(v: &i32, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_u32<C: Write>(v: &u32, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_i64<C: Write>(v: &i64, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_u64<C: Write>(v: &u64, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_i128<C: Write>(v: &i128, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_f32<C: Write>(v: &f32, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_f64<C: Write>(v: &f64, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
}

struct CdrWriter<'a, W, E> {
    buffer: &'a mut W,
    position: usize,
    _endianness: E,
}
impl<'a, W: Write, E> CdrWriter<'a, W, E> {
    fn new(buffer: &'a mut W, endianness: E) -> Self {
        Self {
            buffer,
            position: 0,
            _endianness: endianness,
        }
    }

    fn write_slice(&mut self, data: &[u8]) {
        self.buffer.write(data);
        self.position += data.len();
    }

    fn pad(&mut self, alignment: usize) {
        const ZEROS: [u8; 8] = [0; 8];
        let alignment = self.position.div_ceil(alignment) * alignment - self.position;
        self.write_slice(&ZEROS[..alignment]);
    }
}
impl<'a, W: Write, E> Write for CdrWriter<'a, W, E> {
    fn write(&mut self, buf: &[u8]) {
        self.write_slice(buf);
    }
}

struct ByteCounter(usize);
impl ByteCounter {
    fn new() -> Self {
        Self(0)
    }
}
impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) {
        self.0 += buf.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        infrastructure::type_support::TypeSupport, xtypes::dynamic_type::DynamicDataFactory,
    };
    extern crate std;

    #[test]
    fn serialize_basic_types_struct() {
        #[derive(TypeSupport, Clone)]
        struct BasicTypes {
            f1: bool,
            f2: i8,
            f3: i16,
            f4: i32,
            f5: i64,
            f6: u8,
            f7: u16,
            f8: u32,
            f9: u64,
            f10: f32,
            f11: f64,
            f12: char,
        }

        let mut v = DynamicDataFactory::create_data(BasicTypes::TYPE);
        BasicTypes {
            f1: true,
            f2: 2,
            f3: 3,
            f4: 4,
            f5: 5,
            f6: 6,
            f7: 7,
            f8: 8,
            f9: 9,
            f10: 1.0,
            f11: 1.0,
            f12: 'a',
        }
        .create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x03, // CDR Header (incl. padding length)
                1, 2, 0, 3, 0, 0, 0, 4, // f1: bool | f2: i8 | f3: i16 | f4: i32
                0, 0, 0, 0, 0, 0, 0, 5, // f5: i64
                6, 0, 0, 7, 0, 0, 0, 8, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
                0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
                0x3F, 0x80, 0x00, 0x00, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
                0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
                b'a', 0, 0, 0 // f12: char | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x01, 0x00, 0x03, // CDR Header (incl. padding length)
                1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
                5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
                6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
                9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
                0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
                b'a', 0, 0, 0 // f12: char | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x06, 0x00, 0x03, // CDR Header (incl. padding length)
                1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
                0, 0, 0, 4, // f4: i32
                0, 0, 0, 0, // f5-1: i64
                0, 0, 0, 5, // f5-2: i64
                6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
                0, 0, 0, 8, // f8: u32
                0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
                0x3F, 0x80, 0x00, 0x00, // f10: f32
                0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
                b'a', 0, 0, 0 // f12: char | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
                0x00, 0x07, 0x00, 0x03, // CDR Header (incl. padding length)
                1, 2, 3, 0, // f1: bool | f2: i8 | f3: i16
                4, 0, 0, 0, // f4: i32
                5, 0, 0, 0, // f5-1: i64
                0, 0, 0, 0, // f5-2: i64
                6, 0, 7, 0, // f6: u8 | padding (1 byte) | f7: u16
                8, 0, 0, 0, // f8: u32
                9, 0, 0, 0, // f9-1: u64
                0, 0, 0, 0, // f9-2: u64
                0x00, 0x00, 0x80, 0x3F, // f10: f32
                0x00, 0x00, 0x00, 0x00, // f11-1: f64
                0x00, 0x00, 0xF0, 0x3F, // f11-2: f64
                b'a', 0, 0, 0 // f12: char | padding (3 bytes)
            ]
        );
    }

    #[test]
    fn serialize_u8_array() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct U8Array {
            #[dust_dds(id = 41)]
            version: [u8; 2],
        }

        let mut v = DynamicDataFactory::create_data(U8Array::TYPE);
        U8Array { version: [1, 2] }.create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x02, 0x00, 0x00, // CDR Header
                0x00, 41, 0, 2, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                0, 1, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                41, 0x00, 2, 0, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_locator() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct LocatorContainer {
            #[dust_dds(id = 73)]
            locator: Locator,
        }
        #[derive(TypeSupport, Default, PartialEq, Eq)]
        #[dust_dds(extensibility = "final", nested)]
        struct Locator {
            kind: i32,
            address1: [u8; 2],
            address2: [u8; 3],
        }

        let mut v = DynamicDataFactory::create_data(LocatorContainer::TYPE);
        LocatorContainer {
            locator: Locator {
                kind: 1,
                address1: [3, 4],
                address2: [5, 6, 7],
            },
        }
        .create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x02, 0x00, 0x00, // CDR Header
                0, 73, 0, 9, // PID | length
                0, 0, 0, 1, // kind
                3, 4, 5, 6, // address1 and 2
                7, 0, 0, 0, // address2 | pading (3 bytes)
                0, 1, 0, 0
            ]
        );
    }

    #[test]
    fn serialize_string() {
        #[derive(TypeSupport, Clone)]
        struct StringData(String);

        let mut v = DynamicDataFactory::create_data(StringData::TYPE);
        StringData(String::from("Hola")).create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x03, // CDR Header (incl padding length)
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, 0, 0, 0 // terminating 0 | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x01, 0x00, 0x03, // CDR Header (incl padding length)
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, 0, 0, 0 // terminating 0 | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x06, 0x00, 0x03, // CDR Header (incl padding length)
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, 0, 0, 0 // terminating 0 | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
                0x00, 0x07, 0x00, 0x03, // CDR Header (incl padding length)
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, 0, 0, 0 // terminating 0 | padding (3 bytes)
            ]
        );
    }

    #[test]
    fn serialize_string_list() {
        #[derive(TypeSupport)]
        struct StringList {
            name: Vec<String>,
        }

        let mut v = DynamicDataFactory::create_data(StringList::TYPE);
        StringList {
            name: vec!["one".to_string(), "two".to_string()],
        }
        .create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x00, // CDR Header
                0, 0, 0, 2, // vec length
                0, 0, 0, 4, // String length
                b'o', b'n', b'e', 0, // String
                0, 0, 0, 4, // String length
                b't', b'w', b'o', 0, // String
            ]
        );
    }

    #[derive(TypeSupport, Clone)]
    struct FinalType {
        field_u16: u16,
        field_u64: u64,
    }

    #[test]
    fn serialize_final_struct() {
        let mut v = DynamicDataFactory::create_data(FinalType::TYPE);
        FinalType {
            field_u16: 7,
            field_u64: 9,
        }
        .create_dynamic_sample(&mut v);
        // PLAIN_CDR:
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x00, // CDR Header
                0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x01, 0x00, 0x00, // CDR Header
                7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x06, 0x00, 0x00, // CDR Header
                0, 7, 0, 0, // field_u16 | padding (2 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
                0x00, 0x07, 0x00, 0x00, // CDR Header
                7, 0, 0, 0, // field_u16 | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]
        );
    }

    #[derive(TypeSupport, Clone)]
    struct NestedFinalType {
        field_nested: FinalType,
        field_u8: u8,
    }

    #[test]
    fn serialize_nested_final_struct() {
        let mut v = DynamicDataFactory::create_data(NestedFinalType::TYPE);
        NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        }
        .create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x03, // CDR Header (incl padding length)
                0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // u64
                10, 0, 0, 0 // u8 | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x01, 0x00, 0x03, // CDR Header (incl padding length)
                7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // u64
                10, 0, 0, 0 // u8 | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x06, 0x00, 0x03, // CDR Header (incl padding length)
                0, 7, 0, 0, // nested FinalType (u16) | padding
                0, 0, 0, 0, 0, 0, 0, 9, // u64
                10, 0, 0, 0 // u8 | padding (3 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
                0x00, 0x07, 0x00, 0x03, // CDR Header (incl padding length)
                7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // u64
                10, 0, 0, 0 // u8 | padding (3 bytes)
            ]
        );
    }

    #[derive(TypeSupport, Clone)]
    #[dust_dds(extensibility = "appendable", nested)]
    struct AppendableType {
        value: u16,
    }

    #[test]
    fn serialize_appendable_struct() {
        let mut v = DynamicDataFactory::create_data(AppendableType::TYPE);
        AppendableType { value: 7 }.create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x02, // CDR Header (incl padding length)
                0, 7, 0, 0 // value | padding (2 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x01, 0x00, 0x02, // CDR Header (incl padding length)
                7, 0, 0, 0 // value | padding (2 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x08, 0x00, 0x02, // CDR Header (incl padding length)
                0, 0, 0, 2, // DHEADER
                0, 7, 0, 0 // value | padding (2 bytes)
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
                0x00, 0x09, 0x00, 0x02, // CDR Header (incl padding length)
                2, 0, 0, 0, // DHEADER
                7, 0, 0, 0 // value | padding (2 bytes)
            ]
        );
    }

    #[derive(TypeSupport, Clone, Default, PartialEq)]
    #[dust_dds(extensibility = "mutable")]
    struct MutableType {
        #[dust_dds(id = 90, key)]
        one_byte: u8,
        #[dust_dds(id = 80)]
        two_bytes: u16,
    }

    #[test]
    fn serialize_mutable_struct() {
        let mut v = DynamicDataFactory::create_data(MutableType::TYPE);
        MutableType {
            one_byte: 7,
            two_bytes: 8,
        }
        .create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x02, 0x00, 0x00, // CDR Header
                0x00, 80, 0, 2, // PID | length
                0, 8, 0, 0, // two_bytes | padding (2 bytes)
                0x00, 90, 0, 1, // PID | length
                7, 0, 0, 0, // one_byte | padding
                0, 1, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                80, 0x00, 2, 0, // PID | length
                8, 0, 0, 0, // two_bytes | padding (2 bytes)
                90, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // one_byte | padding
                1, 0, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x0a, 0x00, 0x03, // CDR Header
                0, 0, 0, 13, // DHEADER (length)
                0x10, 0, 0, 80, // EMHEADER1 incl. LC 0b001 (2 bytes)
                0, 8, 0, 0, // two_bytes | padding (2 bytes)
                0, 0, 0, 90, // EMHEADER1 incl. LC 0b000 (1 bytes)
                7, 0, 0, 0 // one_byte | padding 3 bytes
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
                0x00, 0x0b, 0x00, 0x03, // CDR Header
                13, 0, 0, 0, // DHEADER (length)
                80, 0, 0, 0x10, // EMHEADER1 incl. LC 0b001 (2 bytes)
                8, 0, 0, 0, // two_bytes | padding (2 bytes)
                90, 0, 0, 0, // EMHEADER1 incl. LC 0b000 (1 bytes)
                7, 0, 0, 0 // one_byte | padding 3 bytes
            ]
        );
    }

    #[derive(TypeSupport, Clone, Default, PartialEq)]
    struct TinyFinalType {
        primitive: u16,
    }

    #[derive(TypeSupport, Clone)]
    #[dust_dds(extensibility = "mutable")]
    struct NestedMutableType {
        #[dust_dds(id = 96, key)]
        field_primitive: u8,
        #[dust_dds(id = 97)]
        field_mutable: MutableType,
        #[dust_dds(id = 98)]
        field_final: TinyFinalType,
    }

    #[test]
    fn serialize_nested_mutable_struct() {
        let mut v = DynamicDataFactory::create_data(NestedMutableType::TYPE);
        NestedMutableType {
            field_primitive: 5,
            field_mutable: MutableType {
                one_byte: 7,
                two_bytes: 8,
            },
            field_final: TinyFinalType { primitive: 9 },
        }
        .create_dynamic_sample(&mut v);
        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x02, 0x00, 0x00, // CDR Header
                0x00, 96, 0, 1, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x00, 97, 0, 20, // PID | length
                0x00, 80, 0, 2, // field_mutable: PID | length
                0, 8, 0, 0, // field_mutable: two_bytes | padding (2 bytes)
                0x00, 90, 0, 1, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: one_byte | padding (3 bytes)
                0, 1, 0, 0, // field_mutable: Sentinel
                0x00, 98, 0, 2, // field_mutable: PID | length
                0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
                0, 1, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                96, 0x00, 1, 0, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                97, 0x00, 20, 0, // PID | length
                0x050, 0x00, 2, 0, // field_mutable: PID | length
                8, 0, 0, 0, // field_mutable: two_bytes | padding (2 bytes)
                0x05A, 0x00, 1, 0, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: one_byte | padding (3 bytes)
                1, 0, 0, 0, // field_mutable: Sentinel
                0x062, 0x00, 2, 0, // field_mutable: PID | length
                9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_appendable_shapes() {
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
        let mut v = DynamicDataFactory::create_data(AppendableShapesType::TYPE);
        AppendableShapesType {
            color: String::from("BLUE"),
            x: 10,
            y: 20,
            shapesize: 30,
            additional_payload_size: vec![],
        }
        .create_dynamic_sample(&mut v);

        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x00, // CDR_BE
                0, 0, 0, 5, // color: length
                b'B', b'L', b'U', b'E', // color
                0, 0, 0, 0, // color: terminating 0 | padding
                0, 0, 0, 10, // x
                0, 0, 0, 20, // y
                0, 0, 0, 30, // shapesize
                0, 0, 0, 0, // additional_payload_size: length
            ]
        );
        assert_eq!(
            serialize_cdr1_le(&v).unwrap(),
            vec![
                0x00, 0x01, 0x00, 0x00, // CDR_LE
                5, 0, 0, 0, // color: length
                b'B', b'L', b'U', b'E', // color
                0, 0, 0, 0, // color: terminating 0 | padding
                10, 0, 0, 0, // x
                20, 0, 0, 0, // y
                30, 0, 0, 0, // shapesize
                0, 0, 0, 0, // additional_payload_size: length
            ]
        );
        assert_eq!(
            serialize_cdr2_be(&v).unwrap(),
            vec![
                0x00, 0x08, 0x00, 0x00, // D_CDR2_BE
                0, 0, 0, 28, // Dheader
                0, 0, 0, 5, // color: length
                b'B', b'L', b'U', b'E', // color
                0, 0, 0, 0, // color: terminating 0 | padding
                0, 0, 0, 10, // x
                0, 0, 0, 20, // y
                0, 0, 0, 30, // shapesize
                0, 0, 0, 0, // additional_payload_size: length
            ]
        );
        assert_eq!(
            serialize_cdr2_le(&v).unwrap(),
            vec![
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
        );
    }

    #[test]
    fn serialize_union_type() {
        #[derive(Clone, Debug, PartialEq, TypeSupport)]
        struct MyInnerType(u32);

        #[derive(Debug, PartialEq, TypeSupport)]
        #[dust_dds(extensibility = "final", switch(u16))]
        enum MyDynamicType {
            #[dust_dds(case = 5)]
            _VariantA(MyInnerType),
            #[dust_dds(case = 6)]
            VariantB { a: u32 },
            #[dust_dds(case = 7)]
            _VariantC,
        }
        let mut v = DynamicDataFactory::create_data(MyDynamicType::TYPE);
        MyDynamicType::VariantB { a: 10 }.create_dynamic_sample(&mut v);

        assert_eq!(
            serialize_cdr1_be(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x00, // CDR_BE
                0, 6, 0, 0, // discriminant (u16) | padding (3 bytes)
                0, 0, 0, 10, // u32 (VariantB)
            ]
        );
    }

    #[test]
    fn mutable_struct_with_optional_id() {
        #[derive(TypeSupport, Clone)]
        #[dust_dds(extensibility = "mutable")]
        struct MutableTypeAutoId {
            #[dust_dds(key)]
            key: u8,
            participant_key: u16,
        }

        let type_info = MutableTypeAutoId::TYPE;
        assert_eq!(
            type_info
                .get_member_by_index(0)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            0
        );
        assert_eq!(
            type_info
                .get_member_by_index(1)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            1
        );
    }

    #[test]
    fn mutable_struct_with_mixed_ids() {
        #[derive(TypeSupport, Clone)]
        #[dust_dds(extensibility = "mutable")]
        struct MutableTypeMixedId {
            #[dust_dds(id = 10)]
            key: u8,
            participant_key: u16,
            #[dust_dds(id = 20)]
            extra_field_1: u32,
            extra_field_2: u32,
            extra_field_3: u32,
        }

        let type_info = MutableTypeMixedId::TYPE;
        assert_eq!(
            type_info
                .get_member_by_index(0)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            10
        );
        assert_eq!(
            type_info
                .get_member_by_index(1)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            11
        );
        assert_eq!(
            type_info
                .get_member_by_index(2)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            20
        );
        assert_eq!(
            type_info
                .get_member_by_index(3)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            21
        );
        assert_eq!(
            type_info
                .get_member_by_index(4)
                .unwrap()
                .get_descriptor()
                .unwrap()
                .id,
            22
        );
    }
}

#[cfg(test)]
mod rtps_pl_tests {
    use super::*;
    use crate::{
        infrastructure::type_support::TypeSupport, xtypes::dynamic_type::DynamicDataFactory,
    };
    extern crate std;

    fn test_serialize_type_support<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut data = DynamicDataFactory::create_data(T::TYPE);
        v.create_dynamic_sample(&mut data);
        let mut buffer = Vec::new();
        RtpsPlCdrSerializer::serialize(&mut buffer, &data).unwrap();
        buffer
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "mutable")]
    struct MutableBasicType {
        #[dust_dds(id = 10)]
        field_u16: u16,
        #[dust_dds(id = 20)]
        field_u8: u8,
    }

    #[test]
    fn serialize_mutable_struct() {
        let v = MutableBasicType {
            field_u16: 7,
            field_u8: 8,
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                10, 0, 4, 0, // PID | length
                7, 0, 0, 0, // field_u16 | padding (2 bytes)
                20, 0, 4, 0, // PID | length
                8, 0, 0, 0, // field_u8 | padding (3 bytes)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[derive(TypeSupport, Default, PartialEq)]
    struct Time {
        sec: u32,
        nanosec: i32,
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "mutable")]
    struct MutableTimeType {
        #[dust_dds(id = 30)]
        field_time: Time,
    }

    #[test]
    fn serialize_mutable_time_struct() {
        let v = MutableTimeType {
            field_time: Time { sec: 5, nanosec: 6 },
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                30, 0, 8, 0, // PID | length
                5, 0, 0, 0, // Time: sec
                6, 0, 0, 0, // Time: nanosec
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "mutable")]
    struct MutableCollectionType {
        #[dust_dds(id = 30)]
        field_times: Vec<Time>,
    }

    #[test]
    fn serialize_mutable_collection_struct() {
        let v = MutableCollectionType {
            field_times: vec![Time { sec: 5, nanosec: 6 }, Time { sec: 7, nanosec: 8 }],
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                30, 0, 8, 0, // PID | length
                5, 0, 0, 0, // Time: sec
                6, 0, 0, 0, // Time: nanosec
                30, 0, 8, 0, // PID | length
                7, 0, 0, 0, // Time: sec
                8, 0, 0, 0, // Time: nanosec
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_string() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct StringData {
            #[dust_dds(id = 41)]
            name: String,
        }

        let v = StringData {
            name: "one".to_string(),
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                41, 0x00, 8, 0, // PID, length
                4, 0, 0, 0, // String length
                b'o', b'n', b'e', 0, // String
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_string_list() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct StringList {
            #[dust_dds(id = 41)]
            name: Vec<String>,
        }

        let v = StringList {
            name: vec!["one".to_string(), "two".to_string()],
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                41, 0x00, 20, 0, // PID, length
                2, 0, 0, 0, // vec length
                4, 0, 0, 0, // String length
                b'o', b'n', b'e', 0, // String
                4, 0, 0, 0, // String length
                b't', b'w', b'o', 0, // String
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_u8_array() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct U8Array {
            #[dust_dds(id = 41)]
            version: [u8; 2],
        }

        let v = U8Array { version: [1, 2] };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                41, 0x00, 4, 0, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_locator() {
        #[derive(TypeSupport, Default, PartialEq)]
        struct Locator {
            kind: i32,
            port: u32,
            address: [u8; 4],
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct LocatorContainer {
            #[dust_dds(id = 41)]
            locator: Locator,
        }

        let v = LocatorContainer {
            locator: Locator {
                kind: 1,
                port: 2,
                address: [7; 4],
            },
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                0x00, 0x03, 0x00, 0x00, // CDR Header
                41, 0x00, 12, 0, // PID, length
                1, 0, 0, 0, // kind
                2, 0, 0, 0, // port
                7, 7, 7, 7, // address
                1, 0, 0, 0, // Sentinel
            ]
        );
    }
}
