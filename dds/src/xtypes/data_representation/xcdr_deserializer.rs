use crate::xtypes::{
    data_representation::deserialize::XTypesDeserialize,
    dynamic_type::{DynamicData, DynamicDataFactory, DynamicType},
    error::XTypesResult,
};

impl DynamicData {
    pub fn xcdr_deserialize(
        dynamic_type: DynamicType,
        deserializer: &mut impl XTypesDeserialize,
    ) -> XTypesResult<Self> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type.clone());
        deserializer.deserialize_structure(&dynamic_type, &mut dynamic_data)?;
        Ok(dynamic_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::type_support::TypeSupport,
        xtypes::{
            data_representation::{
                cdr_reader::{Cdr1Deserializer, Cdr2Deserializer},
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
                &mut Cdr2Deserializer::new(
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
                &mut Cdr2Deserializer::new(
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
                &mut Cdr2Deserializer::new(
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
                &mut Cdr2Deserializer::new(
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
                &mut Cdr2Deserializer::new(
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
                &mut Cdr2Deserializer::new(
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

    #[test]
    fn deserialize_string() {
        #[derive(Debug, PartialEq, TypeSupport)]
        struct FinalString(String);
        let expected = FinalString(String::from("Hola")).create_dynamic_sample();
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalString::get_type(),
                &mut Cdr1Deserializer::new(
                    &[
                        0, 0, 0, 5, //length
                        b'H', b'o', b'l', b'a', // str
                        0x00, // terminating 0
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalString::get_type(),
                &mut Cdr1Deserializer::new(
                    &[
                        5, 0, 0, 0, //length
                        b'H', b'o', b'l', b'a', // str
                        0x00, // terminating 0
                    ],
                    LittleEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalString::get_type(),
                &mut Cdr2Deserializer::new(
                    &[
                        0, 0, 0, 5, //length
                        b'H', b'o', b'l', b'a', // str
                        0x00, // terminating 0
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                FinalString::get_type(),
                &mut Cdr2Deserializer::new(
                    &[
                        5, 0, 0, 0, //length
                        b'H', b'o', b'l', b'a', // str
                        0x00, // terminating 0
                    ],
                    LittleEndian
                )
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
            DynamicData::xcdr_deserialize(
                ByteArray::get_type(),
                &mut Cdr1Deserializer::new(&[1, 2], BigEndian)
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                ByteArray::get_type(),
                &mut Cdr1Deserializer::new(&[1, 2], LittleEndian)
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                ByteArray::get_type(),
                &mut Cdr2Deserializer::new(&[1, 2], BigEndian)
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            DynamicData::xcdr_deserialize(
                ByteArray::get_type(),
                &mut Cdr2Deserializer::new(&[1, 2], LittleEndian)
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
            DynamicData::xcdr_deserialize(
                Sequence::get_type(),
                &mut Cdr1Deserializer::new(
                    &[
                        0, 0, 0, 2, // length
                        1, 2, 77
                    ],
                    BigEndian
                )
            )
            .unwrap(),
            expected
        );
    }
}
