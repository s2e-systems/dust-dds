use tracing::debug;

use crate::xtypes::{
    data_representation::{
        endianness::{BigEndian, EndiannessRead, LittleEndian},
        plain_cdr::{CdrVersion, PlainCdrDecoder},
    },
    dynamic_type::{DynamicData, DynamicDataFactory, DynamicType, ExtensibilityKind, TypeKind},
    error::{XTypesError, XTypesResult},
};

const PLAIN_CDR_BE: &[u8] = &[0x00, 0x00];
const PLAIN_CDR_LE: &[u8] = &[0x00, 0x01];
const PLAIN_CDR2_BE: &[u8] = &[0x00, 0x10];
const PLAIN_CDR2_LE: &[u8] = &[0x00, 0x11];

impl DynamicData {
    pub fn xcdr_deserialize(dynamic_type: DynamicType, buffer: &[u8]) -> XTypesResult<Self> {
        // Buffer must have at least a header
        if buffer.len() < 4 {
            debug!("Buffer too small. Expected at least header");
            return Err(XTypesError::InvalidData);
        }

        // Deserialize the top-level which must be either a struct, an enum or a union.
        // No other types are supported at this stage because this is what we can get from DDS
        match dynamic_type.get_descriptor().kind {
            TypeKind::STRUCTURE => {
                // Extensibility kind must match the representation
                match dynamic_type.get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => match &buffer[0..2] {
                        PLAIN_CDR_BE => {
                            let mut decoder =
                                PlainCdrDecoder::new(&buffer[4..], BigEndian, CdrVersion::Version1);
                            Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                        }
                        PLAIN_CDR_LE => {
                            let mut decoder = PlainCdrDecoder::new(
                                &buffer[4..],
                                LittleEndian,
                                CdrVersion::Version1,
                            );
                            Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                        }
                        PLAIN_CDR2_BE => {
                            let mut decoder =
                                PlainCdrDecoder::new(&buffer[4..], BigEndian, CdrVersion::Version2);
                            Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                        }
                        PLAIN_CDR2_LE => {
                            let mut decoder = PlainCdrDecoder::new(
                                &buffer[4..],
                                LittleEndian,
                                CdrVersion::Version2,
                            );
                            Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                        }
                        enc_header => {
                            debug!("Invalid encoding header for final struct {enc_header:?}");
                            return Err(XTypesError::InvalidData);
                        }
                    },
                    ExtensibilityKind::Appendable => todo!(),
                    ExtensibilityKind::Mutable => todo!(),
                }
            }
            TypeKind::ENUM => todo!(),
            TypeKind::UNION => todo!(),
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                return Err(XTypesError::InvalidType);
            }
        }
    }

    fn xcdr_deserialize_final_struct<'a, E: EndiannessRead>(
        dynamic_type: DynamicType,
        decoder: &mut PlainCdrDecoder<'a, E>,
    ) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
        for member_index in 0..dynamic_data.type_ref().get_member_count() {
            let member = dynamic_data.type_ref().get_member_by_index(member_index)?;
            match member.get_descriptor()?.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => {
                    dynamic_data.set_boolean_value(member.get_id(), decoder.read_boolean()?)?
                }
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => {
                    dynamic_data.set_int16_value(member.get_id(), decoder.read_i16()?)?
                }
                TypeKind::INT32 => {
                    dynamic_data.set_int32_value(member.get_id(), decoder.read_i32()?)?
                }
                TypeKind::INT64 => {
                    dynamic_data.set_int64_value(member.get_id(), decoder.read_i64()?)?
                }
                TypeKind::UINT16 => {
                    dynamic_data.set_uint16_value(member.get_id(), decoder.read_u16()?)?
                }
                TypeKind::UINT32 => {
                    dynamic_data.set_uint32_value(member.get_id(), decoder.read_u32()?)?
                }
                TypeKind::UINT64 => {
                    dynamic_data.set_uint64_value(member.get_id(), decoder.read_u64()?)?
                }
                TypeKind::FLOAT32 => {
                    dynamic_data.set_float32_value(member.get_id(), decoder.read_f32()?)?
                }
                TypeKind::FLOAT64 => {
                    dynamic_data.set_float64_value(member.get_id(), decoder.read_f64()?)?
                }
                TypeKind::FLOAT128 => todo!(),
                TypeKind::INT8 => {
                    dynamic_data.set_int8_value(member.get_id(), decoder.read_i8()?)?
                }
                TypeKind::UINT8 => {
                    dynamic_data.set_uint8_value(member.get_id(), decoder.read_u8()?)?
                }
                TypeKind::CHAR8 => {
                    dynamic_data.set_char8_value(member.get_id(), decoder.read_char()?)?
                }
                TypeKind::CHAR16 => todo!(),
                TypeKind::STRING8 => todo!(),
                TypeKind::STRING16 => todo!(),
                TypeKind::ALIAS => todo!(),
                TypeKind::ENUM => todo!(),
                TypeKind::BITMASK => todo!(),
                TypeKind::ANNOTATION => todo!(),
                TypeKind::STRUCTURE => {
                    let value = DynamicData::xcdr_deserialize_final_struct(
                        member.get_descriptor()?.r#type.clone(),
                        decoder,
                    )?;
                    dynamic_data.set_complex_value(member.get_id(), value)?;
                }
                TypeKind::UNION => todo!(),
                TypeKind::BITSET => todo!(),
                TypeKind::SEQUENCE => todo!(),
                TypeKind::ARRAY => todo!(),
                TypeKind::MAP => todo!(),
            }
        }
        Ok(dynamic_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::{infrastructure::type_support::TypeSupport, xtypes::dynamic_type::DynamicData};

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

    #[test]
    fn deserialize_final_struct() {
        let expected = FinalType {
            field_u16: 7,
            field_u64: 9,
        }
        .create_dynamic_sample();
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalType::get_type(),
                &[
                    0, 0, // PLAIN_CDR_BIG_ENDIAN
                    0, 0, // OPTIONS
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalType::get_type(),
                &[
                    0, 1, // PLAIN_CDR_LITTLE_ENDIAN
                    0, 0, // OPTIONS
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalType::get_type(),
                &[
                    0, 0x10, // PLAIN_CDR2_BIG_ENDIAN
                    0, 0, // OPTIONS
                    0, 7, 0, 0, // field_u16 | padding (2 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalType::get_type(),
                &[
                    0, 0x11, // PLAIN_CDR2_LITTLE_ENDIAN
                    0, 0, // OPTIONS
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
        let expected = NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        }
        .create_dynamic_sample();

        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &[
                    0, 0, // PLAIN_CDR_BIG_ENDIAN
                    0, 0, // OPTIONS
                    0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &[
                    0, 1, // PLAIN_CDR_LITTLE_ENDIAN
                    0, 0, // OPTIONS
                    7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &[
                    0, 0x10, // PLAIN_CDR_BIG_ENDIAN
                    0, 0, // OPTIONS
                    7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &[
                    0, 0x11, // PLAIN_CDR_LITTLE_ENDIAN
                    0, 0, // OPTIONS
                    0, 7, 0, 0, // nested FinalType (u16) | padding
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );
    }
}
