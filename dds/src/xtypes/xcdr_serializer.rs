use super::{error::XTypesError, serialize::Write, serializer::XTypesSerializer};
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    serializer::{BigEndian, LittleEndian, Writer, WriterV1, WriterV2},
};
use alloc::vec::Vec;

const PID_SENTINEL: u16 = 1;

pub fn serialize_nested(
    dynamic_data: &DynamicData,
    mut serializer: impl XTypesSerializer,
) -> Result<(), super::error::XTypesError> {
    match dynamic_data.type_ref().get_kind() {
        TypeKind::ENUM => {
            // serializer.serialize_data_kind(dynamic_data.get_value(0)?)?;
            todo!()
        }
        TypeKind::STRUCTURE => serializer.serialize_complex(dynamic_data)?,
        kind => unimplemented!("Should not reach for {kind:?}"),
    }
    Ok(())
}

struct ByteCounter(usize);

impl ByteCounter {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) {
        self.0 += buf.len();
    }
}

pub struct Xcdr1BeSerializer<'a, C> {
    writer: WriterV1<'a, C>,
}

impl<'a, C: Write> Xcdr1BeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: WriterV1 {
                writer: Writer::new(collection),
            },
        }
    }
}

impl<C: Write> XTypesSerializer for Xcdr1BeSerializer<'_, C> {
    type Endianness = BigEndian;

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }
    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }
    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let member_descriptor = v.get_descriptor(member_id)?;

            self.serialize_u16(member_id as u16);

            let mut byte_counter = ByteCounter::new();
            let mut byte_conter_serializer = Xcdr1BeSerializer::new(&mut byte_counter);

            match member_descriptor.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => {
                    byte_conter_serializer.serialize_bool(*v.get_boolean_value(member_id)?)
                }
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => {
                    byte_conter_serializer.serialize_i16(*v.get_int16_value(member_id)?)
                }
                TypeKind::INT32 => {
                    byte_conter_serializer.serialize_i32(*v.get_int32_value(member_id)?)
                }
                TypeKind::INT64 => {
                    byte_conter_serializer.serialize_i64(*v.get_int64_value(member_id)?)
                }
                TypeKind::UINT16 => {
                    byte_conter_serializer.serialize_u16(*v.get_uint16_value(member_id)?)
                }
                TypeKind::UINT32 => {
                    byte_conter_serializer.serialize_u32(*v.get_uint32_value(member_id)?)
                }
                TypeKind::UINT64 => {
                    byte_conter_serializer.serialize_u64(*v.get_uint64_value(member_id)?)
                }
                TypeKind::FLOAT32 => {
                    byte_conter_serializer.serialize_f32(*v.get_float32_value(member_id)?)
                }
                TypeKind::FLOAT64 => {
                    byte_conter_serializer.serialize_f64(*v.get_float64_value(member_id)?)
                }
                TypeKind::FLOAT128 => unimplemented!("not supported by Rust"),
                TypeKind::INT8 => {
                    byte_conter_serializer.serialize_i8(*v.get_int8_value(member_id)?)
                }
                TypeKind::UINT8 => {
                    byte_conter_serializer.serialize_u8(*v.get_uint8_value(member_id)?)
                }
                TypeKind::CHAR8 => {
                    byte_conter_serializer.serialize_char(*v.get_char8_value(member_id)?)
                }
                TypeKind::CHAR16 => todo!(),
                TypeKind::STRING8 => {
                    byte_conter_serializer.serialize_string(v.get_string_value(member_id)?)
                }
                TypeKind::STRING16 => todo!(),
                TypeKind::ALIAS => todo!(),
                TypeKind::ENUM => todo!(),
                TypeKind::BITMASK => todo!(),
                TypeKind::ANNOTATION => todo!(),
                TypeKind::STRUCTURE => {
                    byte_conter_serializer.serialize_complex(v.get_complex_value(member_id)?)?
                }
                TypeKind::UNION => todo!(),
                TypeKind::BITSET => todo!(),
                TypeKind::SEQUENCE => byte_conter_serializer.serialize_sequence(v)?,
                TypeKind::ARRAY => todo!(),
                TypeKind::MAP => todo!(),
            }

            let length = byte_counter.0;
            self.serialize_u16(length as u16);

            match member_descriptor.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => self.serialize_bool(*v.get_boolean_value(member_id)?),
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => self.serialize_i16(*v.get_int16_value(member_id)?),
                TypeKind::INT32 => self.serialize_i32(*v.get_int32_value(member_id)?),
                TypeKind::INT64 => self.serialize_i64(*v.get_int64_value(member_id)?),
                TypeKind::UINT16 => self.serialize_u16(*v.get_uint16_value(member_id)?),
                TypeKind::UINT32 => self.serialize_u32(*v.get_uint32_value(member_id)?),
                TypeKind::UINT64 => self.serialize_u64(*v.get_uint64_value(member_id)?),
                TypeKind::FLOAT32 => self.serialize_f32(*v.get_float32_value(member_id)?),
                TypeKind::FLOAT64 => self.serialize_f64(*v.get_float64_value(member_id)?),
                TypeKind::FLOAT128 => unimplemented!("not supported by Rust"),
                TypeKind::INT8 => self.serialize_i8(*v.get_int8_value(member_id)?),
                TypeKind::UINT8 => self.serialize_u8(*v.get_uint8_value(member_id)?),
                TypeKind::CHAR8 => self.serialize_char(*v.get_char8_value(member_id)?),
                TypeKind::CHAR16 => todo!(),
                TypeKind::STRING8 => self.serialize_string(v.get_string_value(member_id)?),
                TypeKind::STRING16 => todo!(),
                TypeKind::ALIAS => todo!(),
                TypeKind::ENUM => todo!(),
                TypeKind::BITMASK => todo!(),
                TypeKind::ANNOTATION => todo!(),
                TypeKind::STRUCTURE => self.serialize_complex(v.get_complex_value(member_id)?)?,
                TypeKind::UNION => todo!(),
                TypeKind::BITSET => todo!(),
                TypeKind::SEQUENCE => self.serialize_sequence(v)?,
                TypeKind::ARRAY => todo!(),
                TypeKind::MAP => todo!(),
            }

            self.writer.writer.pad(4);
        }

        self.writer.writer.write_slice(&PID_SENTINEL.to_be_bytes());
        self.writer.writer.write_slice(&0u16.to_be_bytes());

        Ok(())
    }
    fn writer(&mut self) -> &mut impl Write {
        &mut self.writer
    }
}

pub struct Xcdr1LeSerializer<'a, C> {
    pub writer: WriterV1<'a, C>,
}

impl<'a, C: Write> Xcdr1LeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: WriterV1 {
                writer: Writer::new(collection),
            },
        }
    }
}

impl<C: Write> XTypesSerializer for Xcdr1LeSerializer<'_, C> {
    type Endianness = LittleEndian;

    fn writer(&mut self) -> &mut impl Write {
        &mut self.writer
    }

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

pub struct Xcdr2BeSerializer<'a, C> {
    writer: WriterV2<'a, C>,
}

impl<'a, C: Write> Xcdr2BeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: WriterV2 {
                writer: Writer::new(collection),
            },
        }
    }
}

pub struct Xcdr2LeSerializer<'a, C> {
    writer: WriterV2<'a, C>,
}

impl<'a, C: Write> Xcdr2LeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: WriterV2 {
                writer: Writer::new(collection),
            },
        }
    }
}

struct PlainCdr2Encoder<'a, S> {
    serializer: &'a mut S,
}

impl<C: Write> XTypesSerializer for Xcdr2BeSerializer<'_, C> {
    type Endianness = BigEndian;

    fn writer(&mut self) -> &mut impl Write {
        &mut self.writer
    }

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

impl<C: Write> XTypesSerializer for Xcdr2LeSerializer<'_, C> {
    type Endianness = LittleEndian;

    fn writer(&mut self) -> &mut impl Write {
        &mut self.writer
    }

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::type_support::TypeSupport;

    use super::*;
    extern crate std;

    fn serialize_v1_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(Xcdr1BeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v1_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v2_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(Xcdr2BeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v2_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(Xcdr2LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

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

        let v = BasicTypes {
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
        };
        // PLAIN_CDR:
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                1, 2, 0, 3, 0, 0, 0, 4, // f1: bool | f2: i8 | f3: i16 | f4: i32
                0, 0, 0, 0, 0, 0, 0, 5, // f5: i64
                6, 0, 0, 7, 0, 0, 0, 8, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
                0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
                0x3F, 0x80, 0x00, 0x00, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
                0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
                b'a', // f12: char
            ]
        );
        // assert_eq!(
        //     serialize_v1_le(v.clone()),
        //     vec![
        //         1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
        //         5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
        //         6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
        //         9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
        //         0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
        //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
        //         b'a', // f12: char
        //     ]
        // );
        // //PLAIN_CDR2:
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
        //         0, 0, 0, 4, // f4: i32
        //         0, 0, 0, 0, // f5-1: i64
        //         0, 0, 0, 5, // f5-2: i64
        //         6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
        //         0, 0, 0, 8, // f8: u32
        //         0, 0, 0, 0, // f9-1: u64
        //         0, 0, 0, 9, // f9-2: u64
        //         0x3F, 0x80, 0x00, 0x00, // f10: f32
        //         0x3F, 0xF0, 0x00, 0x00, // f11-1: f64
        //         0x00, 0x00, 0x00, 0x00, // f11-2: f64
        //         b'a', // f12: char
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         1, 2, 3, 0, // f1: bool | f2: i8 | f3: i16
        //         4, 0, 0, 0, // f4: i32
        //         5, 0, 0, 0, // f5-1: i64
        //         0, 0, 0, 0, // f5-2: i64
        //         6, 0, 7, 0, // f6: u8 | padding (1 byte) | f7: u16
        //         8, 0, 0, 0, // f8: u32
        //         9, 0, 0, 0, // f9-1: u64
        //         0, 0, 0, 0, // f9-2: u64
        //         0x00, 0x00, 0x80, 0x3F, // f10: f32
        //         0x00, 0x00, 0x00, 0x00, // f11-1: f64
        //         0x00, 0x00, 0xF0, 0x3F, // f11-2: f64
        //         b'a', // f12: char
        //     ]
        // );
    }

    #[test]
    fn serialize_string() {
        #[derive(TypeSupport, Clone)]
        struct StringData(String);

        let v = StringData(String::from("Hola"));
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        // assert_eq!(
        //     serialize_v1_le(v.clone()),
        //     vec![
        //         5, 0, 0, 0, //length
        //         b'H', b'o', b'l', b'a', // str
        //         0x00, // terminating 0
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         0, 0, 0, 5, //length
        //         b'H', b'o', b'l', b'a', // str
        //         0x00, // terminating 0
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         5, 0, 0, 0, //length
        //         b'H', b'o', b'l', b'a', // str
        //         0x00, // terminating 0
        //     ]
        // );
    }

    #[derive(TypeSupport, Clone)]
    struct FinalType {
        field_u16: u16,
        field_u64: u64,
    }

    #[test]
    fn serialize_final_struct() {
        let v = FinalType {
            field_u16: 7,
            field_u64: 9,
        };
        // PLAIN_CDR:
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]
        );
        // assert_eq!(
        //     serialize_v1_le(v.clone()),
        //     vec![
        //         7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
        //         9, 0, 0, 0, 0, 0, 0, 0, // field_u64
        //     ]
        // );
        // // PLAIN_CDR2:
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         0, 7, 0, 0, // field_u16 | padding (2 bytes)
        //         0, 0, 0, 0, 0, 0, 0, 9, // field_u64
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         7, 0, 0, 0, // field_u16 | padding (2 bytes)
        //         9, 0, 0, 0, 0, 0, 0, 0, // field_u64
        //     ]
        // );
    }

    #[derive(TypeSupport, Clone)]
    struct NestedFinalType {
        field_nested: FinalType,
        field_u8: u8,
    }

    #[test]
    fn serialize_nested_final_struct() {
        let v = NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        };
        // PLAIN_CDR:
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9,  // u64
                10, //u8
            ]
        );
        // assert_eq!(
        //     serialize_v1_le(v.clone()),
        //     vec![
        //         7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
        //         9, 0, 0, 0, 0, 0, 0, 0,  // u64
        //         10, //u8
        //     ]
        // );
        // // PLAIN_CDR2:
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
        //         9, 0, 0, 0, 0, 0, 0, 0,  // u64
        //         10, //u8
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         0, 7, 0, 0, // nested FinalType (u16) | padding
        //         0, 0, 0, 0, 0, 0, 0, 9,  // u64
        //         10, //u8
        //     ]
        // );
    }

    #[derive(TypeSupport, Clone)]
    #[dust_dds(extensibility = "appendable", nested)]
    struct AppendableType {
        value: u16,
    }

    #[test]
    fn serialize_appendable_struct() {
        let v = AppendableType { value: 7 };
        // PLAIN_CDR:
        assert_eq!(serialize_v1_be(v.clone()), vec![0, 7]);
        // assert_eq!(serialize_v1_le(v.clone()), vec![7, 0]);
        // // DELIMITED_CDR:
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         0, 0, 0, 2, // DHEADER
        //         0, 7 // value
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         2, 0, 0, 0, // DHEADER
        //         7, 0 // value
        //     ]
        // );
    }

    #[derive(TypeSupport, Clone)]
    #[dust_dds(extensibility = "mutable")]
    struct MutableType {
        #[dust_dds(id = 90, key)]
        key: u8,
        #[dust_dds(id = 80)]
        participant_key: u16,
    }

    #[test]
    fn serialize_mutable_struct() {
        let v = MutableType {
            key: 7,
            participant_key: 8,
        };
        // PL_CDR:
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                0x00, 80, 0, 2, // PID | length
                0, 8, 0, 0, // participant_key | padding (2 bytes)
                0x00, 90, 0, 1, // PID | length
                7, 0, 0, 0, // key | padding
                0, 1, 0, 0, // Sentinel
            ]
        );
        // assert_eq!(
        //     serialize_v1_le(v.clone()),
        //     vec![
        //         0x050, 0x00, 2, 0, // PID | length
        //         8, 0, 0, 0, // participant_key | padding (2 bytes)
        //         0x05A, 0x00, 1, 0, // PID | length
        //         7, 0, 0, 0, // key | padding
        //         1, 0, 0, 0, // Sentinel
        //     ]
        // );
        // // PL_CDR2:
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         0x00, 0x050, 0, 2, // PID | length
        //         0, 8, 0, 0, // participant_key | padding (2 bytes)
        //         0x00, 0x05A, 0, 1, // PID | length
        //         7, 0, 0, 0, // key | padding
        //         0, 1, 0, 0, // Sentinel
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         0x050, 0x00, 2, 0, // PID | length
        //         8, 0, 0, 0, // participant_key | padding (2 bytes)
        //         0x05A, 0x00, 1, 0, // PID | length
        //         7, 0, 0, 0, // key | padding
        //         1, 0, 0, 0, // Sentinel
        //     ]
        // );
    }

    #[derive(TypeSupport, Clone)]
    struct TinyFinalType {
        primitive: u16,
    }

    #[derive(TypeSupport, Clone)]
    #[dust_dds(extensibility = "mutable")]
    struct NestedMutableType {
        // #[dust_dds(id = 96, key)]
        // field_primitive: u8,
        #[dust_dds(id = 97)]
        field_mutable: MutableType,
        // #[dust_dds(id = 98)]
        // field_final: TinyFinalType,
    }

    #[test]
    fn serialize_nested_mutable_struct() {
        let v = NestedMutableType {
            // field_primitive: 5,
            field_mutable: MutableType {
                key: 7,
                participant_key: 8,
            },
            // field_final: TinyFinalType { primitive: 9 },
        };
        // PL_CDR:
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                // 0x00, 96, 0, 1, // PID | length
                // 5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x00, 97, 0, 20, // PID | length
                0x00, 80, 0, 2, // field_mutable: PID | length
                0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
                0x00, 90, 0, 1, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
                0, 1, 0, 0, // field_mutable: Sentinel
                // 0x00, 98, 0, 2, // field_mutable: PID | length
                // 0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
                0, 1, 0, 0, // Sentinel
            ]
        );
        // assert_eq!(
        //     serialize_v1_le(v.clone()),
        //     vec![
        //         0x060, 0x00, 1, 0, // PID | length
        //         5, 0, 0, 0, // field_primitive | padding (3 bytes)
        //         0x061, 0x00, 20, 0, // PID | length
        //         0x050, 0x00, 2, 0, // field_mutable: PID | length
        //         8, 0, 0, 0, // field_mutable: participant_key | padding (2 bytes)
        //         0x05A, 0x00, 1, 0, // field_mutable: PID | length
        //         7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
        //         1, 0, 0, 0, // field_mutable: Sentinel
        //         0x062, 0x00, 2, 0, // field_mutable: PID | length
        //         9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
        //         1, 0, 0, 0, // Sentinel
        //     ]
        // );
        // // PL_CDR2:
        // assert_eq!(
        //     serialize_v2_be(v.clone()),
        //     vec![
        //         0x00, 0x060, 0, 1, // PID | length
        //         5, 0, 0, 0, // field_primitive | padding (3 bytes)
        //         0x00, 0x061, 0, 20, // PID | length
        //         0x00, 0x050, 0, 2, // field_mutable: PID | length
        //         0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
        //         0x00, 0x05A, 0, 1, // field_mutable: PID | length
        //         7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
        //         0, 1, 0, 0, // field_mutable: Sentinel
        //         0x00, 0x062, 0, 2, // field_mutable: PID | length
        //         0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
        //         0, 1, 0, 0, // Sentinel
        //     ]
        // );
        // assert_eq!(
        //     serialize_v2_le(v.clone()),
        //     vec![
        //         0x060, 0x00, 1, 0, // PID | length
        //         5, 0, 0, 0, // field_primitive | padding (3 bytes)
        //         0x061, 0x00, 20, 0, // PID | length
        //         0x050, 0x00, 2, 0, // field_mutable: PID | length
        //         8, 0, 0, 0, // field_mutable: participant_key | padding (2 bytes)
        //         0x05A, 0x00, 1, 0, // field_mutable: PID | length
        //         7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
        //         1, 0, 0, 0, // field_mutable: Sentinel
        //         0x062, 0x00, 2, 0, // field_mutable: PID | length
        //         9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
        //         1, 0, 0, 0, // Sentinel
        //     ]
        // );
    }

    // #[derive(TypeSupport, Clone)]
    // struct FinalOptionalType {
    //     field: u8,
    //     #[dust_dds(optional)]
    //     optional_field: i32,
    // }

    // #[test]
    // fn serialize_final_optional_struct_some() {
    //     let some = FinalOptionalType {
    //         field: 6,
    //         optional_field: Some(7),
    //     };
    //     //PLAIN_CDR:
    //     assert_eq!(
    //         serialize_v1_be(some.clone()),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 2, // HEADER (FLAGS+ID | length)
    //             0, 7, // optional_field value
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(some.clone()),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 2, 0, // HEADER (FLAGS+ID | length)
    //             7, 0 // optional_field value
    //         ]
    //     );
    //     //PLAIN_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(some.clone()),
    //         vec![
    //             6, 1, // u8 | boolean for option
    //             0, 7 // optional_field value
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(some.clone()),
    //         vec![
    //             6, 1, // u8 | boolean for option
    //             7, 0 // optional_field value
    //         ]
    //     );
    // }

    // #[test]
    // fn serialize_final_optional_struct_none() {
    //     let none = FinalOptionalType {
    //         field: 6,
    //         optional_field: None,
    //     };
    //     // PLAIN_CDR:
    //     assert_eq!(
    //         serialize_v1_be(none.clone()),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 0, // HEADER (FLAGS+ID | length)
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(none.clone()),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 0, // HEADER (FLAGS+ID | length)
    //         ]
    //     );
    //     // PLAIN_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(none.clone()),
    //         vec![
    //         6, 0, // u8 | boolean for option
    //     ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(none.clone()),
    //         vec![
    //         6, 0, // u8 | boolean for option
    //     ]
    //     );
    // }
}
