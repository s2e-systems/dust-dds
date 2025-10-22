use tracing::debug;

use crate::xtypes::{
    data_representation::deserialize::XTypesDeserialize,
    dynamic_type::{DynamicData, DynamicType, ExtensibilityKind, TypeKind},
    error::{XTypesError, XTypesResult},
};

impl DynamicData {
    pub fn xcdr_deserialize(
        dynamic_type: DynamicType,
        deserializer: &mut impl XTypesDeserialize,
    ) -> XTypesResult<Self> {
        // Deserialize the top-level which must be either a struct, an enum or a union.
        // No other types are supported at this stage because this is what we can get from DDS
        match dynamic_type.get_descriptor().kind {
            TypeKind::STRUCTURE => {
                // Extensibility kind must match the representation
                match dynamic_type.get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => deserializer.deserialize_final_struct(dynamic_type),
                    ExtensibilityKind::Appendable => {
                        deserializer.deserialize_appendable_struct(dynamic_type)
                    }
                    ExtensibilityKind::Mutable => {
                        deserializer.deserialize_mutable_struct(dynamic_type)
                    }
                }
            }
            //     TypeKind::ENUM => todo!(),
            //     TypeKind::UNION => todo!(),
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                return Err(XTypesError::InvalidType);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::type_support::TypeSupport,
        xtypes::{
            data_representation::{
                cdr_reader::{Cdr1Deserializer, PlainCdr2Deserializer},
                endianness::{BigEndian, LittleEndian},
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
            DynamicData::xcdr_deserialize(
                FinalType::get_type(),
                &mut Cdr1Deserializer::new(
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
            DynamicData::xcdr_deserialize(
                FinalType::get_type(),
                &mut Cdr1Deserializer::new(
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
            DynamicData::xcdr_deserialize(
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
            DynamicData::xcdr_deserialize(
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
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &mut Cdr1Deserializer::new(
                    &[
                        0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                        0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                        10, //u8
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &mut Cdr1Deserializer::new(
                    &[
                        7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                        9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                        10, //u8
                    ],
                    LittleEndian
                )
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &mut PlainCdr2Deserializer::new(
                    &[
                        0, 7, 0, 0, // nested FinalType (u16) | padding
                        0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                        10, //u8
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );

        assert_eq!(
            DynamicData::xcdr_deserialize(
                NestedFinalType::get_type(),
                &mut PlainCdr2Deserializer::new(
                    &[
                        7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                        9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
                        10, //u8
                    ],
                    LittleEndian
                )
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
            DynamicData::xcdr_deserialize(
                FinalTypeWithSequence::get_type(),
                &mut Cdr1Deserializer::new(
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
            DynamicData::xcdr_deserialize(
                FinalTypeWithSequence::get_type(),
                &mut Cdr1Deserializer::new(
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
            DynamicData::xcdr_deserialize(
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
            DynamicData::xcdr_deserialize(
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
