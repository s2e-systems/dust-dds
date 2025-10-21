use tracing::debug;

use crate::xtypes::{
    data_representation::{
        deserialize::PrimitiveTypeDeserializer,
        endianness::{BigEndian, EndiannessRead, LittleEndian},
    },
    dynamic_type::{DynamicData, DynamicDataFactory, DynamicType, ExtensibilityKind, TypeKind},
    error::{XTypesError, XTypesResult},
};

impl DynamicData {
    pub fn xcdr_deserialize(
        dynamic_type: DynamicType,
        deserializer: impl PrimitiveTypeDeserializer,
    ) -> XTypesResult<Self> {
        // Deserialize the top-level which must be either a struct, an enum or a union.
        // No other types are supported at this stage because this is what we can get from DDS
        match dynamic_type.get_descriptor().kind {
            TypeKind::STRUCTURE => {
                todo!()
                //         // Extensibility kind must match the representation
                //         match dynamic_type.get_descriptor().extensibility_kind {
                //             ExtensibilityKind::Final => match &buffer[0..2] {
                //                 PLAIN_CDR_BE => {
                //                     let mut decoder =
                //                         PlainCdrDecoder::new(&buffer[4..], BigEndian, CdrVersion::Version1);
                //                     Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                //                 }
                //                 PLAIN_CDR_LE => {
                //                     let mut decoder = PlainCdrDecoder::new(
                //                         &buffer[4..],
                //                         LittleEndian,
                //                         CdrVersion::Version1,
                //                     );
                //                     Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                //                 }
                //                 PLAIN_CDR2_BE => {
                //                     let mut decoder =
                //                         PlainCdrDecoder::new(&buffer[4..], BigEndian, CdrVersion::Version2);
                //                     Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                //                 }
                //                 PLAIN_CDR2_LE => {
                //                     let mut decoder = PlainCdrDecoder::new(
                //                         &buffer[4..],
                //                         LittleEndian,
                //                         CdrVersion::Version2,
                //                     );
                //                     Self::xcdr_deserialize_final_struct(dynamic_type, &mut decoder)
                //                 }
                //                 enc_header => {
                //                     debug!("Invalid encoding header for final struct {enc_header:?}");
                //                     return Err(XTypesError::InvalidData);
                //                 }
                //             },
                //             ExtensibilityKind::Appendable => todo!(),
                //             ExtensibilityKind::Mutable => todo!(),
                // }
            }
            //     TypeKind::ENUM => todo!(),
            //     TypeKind::UNION => todo!(),
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                return Err(XTypesError::InvalidType);
            }
        }
    }

    fn xcdr_deserialize_final_struct(
        dynamic_type: DynamicType,
        deserializer: &mut impl PrimitiveTypeDeserializer,
    ) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
        for member_index in 0..dynamic_data.type_ref().get_member_count() {
            let member = dynamic_data.type_ref().get_member_by_index(member_index)?;
            match member.get_descriptor()?.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => dynamic_data
                    .set_boolean_value(member.get_id(), deserializer.deserialize_boolean()?)?,
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => dynamic_data
                    .set_int16_value(member.get_id(), deserializer.deserialize_i16()?)?,
                TypeKind::INT32 => dynamic_data
                    .set_int32_value(member.get_id(), deserializer.deserialize_i32()?)?,
                TypeKind::INT64 => dynamic_data
                    .set_int64_value(member.get_id(), deserializer.deserialize_i64()?)?,
                TypeKind::UINT16 => dynamic_data
                    .set_uint16_value(member.get_id(), deserializer.deserialize_u16()?)?,
                TypeKind::UINT32 => dynamic_data
                    .set_uint32_value(member.get_id(), deserializer.deserialize_u32()?)?,
                TypeKind::UINT64 => dynamic_data
                    .set_uint64_value(member.get_id(), deserializer.deserialize_u64()?)?,
                TypeKind::FLOAT32 => dynamic_data
                    .set_float32_value(member.get_id(), deserializer.deserialize_f32()?)?,
                TypeKind::FLOAT64 => dynamic_data
                    .set_float64_value(member.get_id(), deserializer.deserialize_f64()?)?,
                TypeKind::FLOAT128 => todo!(),
                TypeKind::INT8 => {
                    dynamic_data.set_int8_value(member.get_id(), deserializer.deserialize_i8()?)?
                }
                TypeKind::UINT8 => {
                    dynamic_data.set_uint8_value(member.get_id(), deserializer.deserialize_u8()?)?
                }
                TypeKind::CHAR8 => dynamic_data
                    .set_char8_value(member.get_id(), deserializer.deserialize_char()?)?,
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
                        deserializer,
                    )?;
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
                                deserializer.deserialize_primitive_type_sequence::<u32>()?,
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
                        TypeKind::STRUCTURE => todo!(),
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
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::type_support::TypeSupport,
        xtypes::{
            data_representation::{
                endianness::{BigEndian, LittleEndian},
                plain_cdr::{PlainCdr1Decoder, PlainCdr2Deserializer},
            },
            dynamic_type::DynamicData,
        },
    };

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
            DynamicData::xcdr_deserialize_final_struct(
                FinalType::get_type(),
                &mut PlainCdr1Decoder::new(
                    &[
                        0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                        0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize_final_struct(
                FinalType::get_type(),
                &mut PlainCdr1Decoder::new(
                    &[
                        7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                        9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    ],
                    LittleEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize_final_struct(
                FinalType::get_type(),
                &mut PlainCdr2Deserializer::new(
                    &[
                        0, 7, 0, 0, // field_u16 | padding (2 bytes)
                        0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize_final_struct(
                FinalType::get_type(),
                &mut PlainCdr2Deserializer::new(
                    &[
                        7, 0, 0, 0, // field_u16 | padding (2 bytes)
                        9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                    ],
                    LittleEndian
                )
            )
            .unwrap(),
            expected
        );
    }

    // #[test]
    // fn deserialize_nested_final_struct() {
    //     #[derive(Debug, PartialEq, TypeSupport)]
    //     struct FinalType {
    //         field_u16: u16,
    //         field_u64: u64,
    //     }

    //     #[derive(Debug, PartialEq, TypeSupport)]
    //     //@extensibility(FINAL)
    //     struct NestedFinalType {
    //         field_nested: FinalType,
    //         field_u8: u8,
    //     }

    //     let expected = NestedFinalType {
    //         field_nested: FinalType {
    //             field_u16: 7,
    //             field_u64: 9,
    //         },
    //         field_u8: 10,
    //     }
    //     .create_dynamic_sample();

    //     assert_eq!(
    //         DynamicData::xcdr_deserialize(
    //             NestedFinalType::get_type(),
    //             &[
    //                 0, 0, // PLAIN_CDR_BIG_ENDIAN
    //                 0, 0, // OPTIONS
    //                 0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
    //                 0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
    //                 10, //u8
    //             ]
    //         )
    //         .unwrap(),
    //         expected
    //     );
    //     assert_eq!(
    //         DynamicData::xcdr_deserialize(
    //             NestedFinalType::get_type(),
    //             &[
    //                 0, 1, // PLAIN_CDR_LITTLE_ENDIAN
    //                 0, 0, // OPTIONS
    //                 7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
    //                 9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
    //                 10, //u8
    //             ]
    //         )
    //         .unwrap(),
    //         expected
    //     );

    //     assert_eq!(
    //         DynamicData::xcdr_deserialize(
    //             NestedFinalType::get_type(),
    //             &[
    //                 0, 0x10, // PLAIN_CDR_BIG_ENDIAN
    //                 0, 0, // OPTIONS
    //                 7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
    //                 9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
    //                 10, //u8
    //             ]
    //         )
    //         .unwrap(),
    //         expected
    //     );
    //     assert_eq!(
    //         DynamicData::xcdr_deserialize(
    //             NestedFinalType::get_type(),
    //             &[
    //                 0, 0x11, // PLAIN_CDR_LITTLE_ENDIAN
    //                 0, 0, // OPTIONS
    //                 0, 7, 0, 0, // nested FinalType (u16) | padding
    //                 0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
    //                 10, //u8
    //             ]
    //         )
    //         .unwrap(),
    //         expected
    //     );
    // }

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
            DynamicData::xcdr_deserialize_final_struct(
                FinalTypeWithSequence::get_type(),
                &mut PlainCdr1Decoder::new(
                    &[
                        0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                        0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                        0, 0, 0, 2, // field_seq_u32 Length (u32)
                        0, 0, 0, 1, // field_seq_u32[0]
                        0, 0, 0, 4, // field_seq_u32[0]
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize_final_struct(
                FinalTypeWithSequence::get_type(),
                &mut PlainCdr1Decoder::new(
                    &[
                        7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                        9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                        2, 0, 0, 0, // field_seq_u32 Length (u32)
                        1, 0, 0, 0, // field_seq_u32[0]
                        4, 0, 0, 0, // field_seq_u32[0]
                    ],
                    LittleEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize_final_struct(
                FinalTypeWithSequence::get_type(),
                &mut PlainCdr2Deserializer::new(
                    &[
                        0, 7, 0, 0, // field_u16 | padding (2 bytes)
                        0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                        0, 0, 0, 2, // field_seq_u32 Length (u32)
                        0, 0, 0, 1, // field_seq_u32[0]
                        0, 0, 0, 4, // field_seq_u32[0]
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize_final_struct(
                FinalTypeWithSequence::get_type(),
                &mut PlainCdr2Deserializer::new(
                    &[
                        7, 0, 0, 0, // field_u16 | padding (2 bytes)
                        9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                        2, 0, 0, 0, // field_seq_u32 Length (u32)
                        1, 0, 0, 0, // field_seq_u32[0]
                        4, 0, 0, 0, // field_seq_u32[0]
                    ],
                    LittleEndian
                )
            )
            .unwrap(),
            expected
        );
    }
}
