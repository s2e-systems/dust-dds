use super::{error::XTypesError, serializer::XTypesSerializer};
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    serializer::{EncodingVersion, EncodingVersion1, EncodingVersion2, EndiannessWrite, LittleEndian, PlainCdrSerialize, Write, Writer},
};

const PID_SENTINEL: u16 = 1;

pub fn serialize_nested<W: Write, E: EndiannessWrite, V: EncodingVersion>(
    dynamic_data: &DynamicData,
    mut serializer: impl XTypesSerializer<W, E, V>,
) -> Result<impl XTypesSerializer<W, E, V>, super::error::XTypesError> {
    match dynamic_data.type_ref().get_kind() {
        TypeKind::ENUM => {
            serializer.serialize_enum(dynamic_data)?;
        }
        TypeKind::STRUCTURE => serializer.serialize_complex(dynamic_data)?,
        kind => unimplemented!("Should not reach for {kind:?}"),
    }
    Ok(serializer)
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

pub struct Xcdr1Serializer<W, E> {
    pub writer: Writer<W, E, EncodingVersion1>,
}

impl<W: Write, E> Xcdr1Serializer<W, E> {
    pub fn new(buffer: W, endianess: E) -> Self {
        Self {
            writer: Writer::new(buffer, endianess, EncodingVersion1),
        }
    }
}

fn count_bytes_xdr1(v: &DynamicData, member_id: u32) -> Result<usize, XTypesError> {
    let mut byte_conter_serializer = Xcdr1Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    Ok(byte_conter_serializer.into_inner().0)
}

fn count_bytes_xdr2(v: &DynamicData, member_id: u32) -> Result<usize, XTypesError> {
    let mut byte_conter_serializer = Xcdr2Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    Ok(byte_conter_serializer.into_inner().0)
}

impl<W: Write, E: EndiannessWrite> XTypesSerializer<W, E, EncodingVersion1> for Xcdr1Serializer<W, E> {
    fn into_inner(self) -> W {
        self.writer.buffer
    }
    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let length = count_bytes_xdr1(v, member_id)?;
            PlainCdrSerialize::serialize(member_id as u16, &mut self.writer);
            PlainCdrSerialize::serialize(length as u16, &mut self.writer);
            self.serialize_dynamic_data_member(v, member_id)?;
            self.writer.pad(4);
        }
        PlainCdrSerialize::serialize(PID_SENTINEL, &mut self.writer);
        PlainCdrSerialize::serialize(0u16, &mut self.writer);
        Ok(())
    }

    fn writer(&mut self) -> &mut Writer<W, E, EncodingVersion1> {
        &mut self.writer
    }
}

pub struct Xcdr2Serializer<C, E> {
    writer: Writer<C, E, EncodingVersion2>,
}

impl<C: Write, E> Xcdr2Serializer<C, E> {
    pub fn new(buffer: C, endianess: E) -> Self {
        Self {
            writer: Writer::new(buffer, endianess, EncodingVersion2),
        }
    }
}

impl<W: Write, E: EndiannessWrite> XTypesSerializer<W, E, EncodingVersion2> for Xcdr2Serializer<W, E> {
    fn into_inner(self) -> W {
        self.writer.buffer
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            // DHEADER
            let length = count_bytes_xdr1(v, member_id)? as u32;
            length.serialize(&mut self.writer);
            // Value
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let length = count_bytes_xdr2(v, member_id)?;
            PlainCdrSerialize::serialize(member_id as u16, &mut self.writer);
            PlainCdrSerialize::serialize(length as u16, &mut self.writer);
            self.serialize_dynamic_data_member(v, member_id)?;
            self.writer.pad(4);
        }
        PlainCdrSerialize::serialize(PID_SENTINEL, &mut self.writer);
        PlainCdrSerialize::serialize(0u16, &mut self.writer);
        Ok(())
    }

    fn writer(&mut self) -> &mut Writer<W, E, EncodingVersion2> {
        &mut self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{infrastructure::type_support::TypeSupport, xtypes::serializer::BigEndian};
    extern crate std;

    fn serialize_v1_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        v.create_dynamic_sample()
            .serialize(Xcdr1Serializer::new(Vec::new(), BigEndian))
            .unwrap()
            .into_inner()
    }

    fn serialize_v1_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        v.create_dynamic_sample()
            .serialize(Xcdr1Serializer::new(Vec::new(), LittleEndian))
            .unwrap()
            .into_inner()
    }

    fn serialize_v2_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        v.create_dynamic_sample()
            .serialize(Xcdr2Serializer::new(Vec::new(), BigEndian))
            .unwrap()
            .into_inner()
    }

    fn serialize_v2_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        v.create_dynamic_sample()
            .serialize(Xcdr2Serializer::new(Vec::new(), LittleEndian))
            .unwrap()
            .into_inner()
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
        assert_eq!(
            serialize_v1_le(v.clone()),
            vec![
                1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
                5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
                6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
                9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
                0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
                b'a', // f12: char
            ]
        );
        //PLAIN_CDR2:
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
                0, 0, 0, 4, // f4: i32
                0, 0, 0, 0, // f5-1: i64
                0, 0, 0, 5, // f5-2: i64
                6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
                0, 0, 0, 8, // f8: u32
                0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
                0x3F, 0x80, 0x00, 0x00, // f10: f32
                0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
                b'a', // f12: char
            ]
        );
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
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
                b'a', // f12: char
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
            serialize_v1_be(v),
            vec![
                0x00, 41, 0, 2, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                0, 1, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_locator() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        pub struct LocatorContainer {
            #[dust_dds(id = 73)]
            locator: Locator,
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "final", nested)]
        pub struct Locator {
            kind: i32,
            address1: [u8; 2],
            address2: [u8; 3],
        }

        let v = LocatorContainer {
            locator: Locator {
                kind: 1,
                address1: [3, 4],
                address2: [5, 6, 7],
            },
        };
        assert_eq!(
            serialize_v1_be(v),
            vec![
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

        let v = StringData(String::from("Hola"));
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        assert_eq!(
            serialize_v1_le(v.clone()),
            vec![
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
    }

    #[test]
    fn serialize_string_list() {
        #[derive(TypeSupport)]
        struct StringList {
            name: Vec<String>,
        }

        let v = StringList {
            name: vec!["one".to_string(), "two".to_string()],
        };
        assert_eq!(
            serialize_v1_be(v),
            vec![
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
        assert_eq!(
            serialize_v1_le(v.clone()),
            vec![
                7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                0, 7, 0, 0, // field_u16 | padding (2 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]
        );
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
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
        assert_eq!(
            serialize_v1_le(v.clone()),
            vec![
                7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0,  // u64
                10, //u8
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
                7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0,  // u64
                10, //u8
            ]
        );
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                0, 7, 0, 0, // nested FinalType (u16) | padding
                0, 0, 0, 0, 0, 0, 0, 9,  // u64
                10, //u8
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
        let v = AppendableType { value: 7 };
        // PLAIN_CDR:
        assert_eq!(serialize_v1_be(v.clone()), vec![0, 7]);
        assert_eq!(serialize_v1_le(v.clone()), vec![7, 0]);
        // DELIMITED_CDR:
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                0, 0, 0, 2, // DHEADER
                0, 7 // value
            ]
        );
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
                2, 0, 0, 0, // DHEADER
                7, 0 // value
            ]
        );
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
        assert_eq!(
            serialize_v1_le(v.clone()),
            vec![
                0x050, 0x00, 2, 0, // PID | length
                8, 0, 0, 0, // participant_key | padding (2 bytes)
                0x05A, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // key | padding
                1, 0, 0, 0, // Sentinel
            ]
        );
        // PL_CDR2:
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                0x00, 0x050, 0, 2, // PID | length
                0, 8, 0, 0, // participant_key | padding (2 bytes)
                0x00, 0x05A, 0, 1, // PID | length
                7, 0, 0, 0, // key | padding
                0, 1, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
                0x050, 0x00, 2, 0, // PID | length
                8, 0, 0, 0, // participant_key | padding (2 bytes)
                0x05A, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // key | padding
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[derive(TypeSupport, Clone)]
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
        let v = NestedMutableType {
            field_primitive: 5,
            field_mutable: MutableType {
                key: 7,
                participant_key: 8,
            },
            field_final: TinyFinalType { primitive: 9 },
        };
        // PL_CDR:
        assert_eq!(
            serialize_v1_be(v.clone()),
            vec![
                0x00, 96, 0, 1, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x00, 97, 0, 20, // PID | length
                0x00, 80, 0, 2, // field_mutable: PID | length
                0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
                0x00, 90, 0, 1, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
                0, 1, 0, 0, // field_mutable: Sentinel
                0x00, 98, 0, 2, // field_mutable: PID | length
                0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
                0, 1, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_v1_le(v.clone()),
            vec![
                0x060, 0x00, 1, 0, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x061, 0x00, 20, 0, // PID | length
                0x050, 0x00, 2, 0, // field_mutable: PID | length
                8, 0, 0, 0, // field_mutable: participant_key | padding (2 bytes)
                0x05A, 0x00, 1, 0, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
                1, 0, 0, 0, // field_mutable: Sentinel
                0x062, 0x00, 2, 0, // field_mutable: PID | length
                9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
                1, 0, 0, 0, // Sentinel
            ]
        );
        // PL_CDR2:
        assert_eq!(
            serialize_v2_be(v.clone()),
            vec![
                0x00, 0x060, 0, 1, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x00, 0x061, 0, 20, // PID | length
                0x00, 0x050, 0, 2, // field_mutable: PID | length
                0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
                0x00, 0x05A, 0, 1, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
                0, 1, 0, 0, // field_mutable: Sentinel
                0x00, 0x062, 0, 2, // field_mutable: PID | length
                0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
                0, 1, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_v2_le(v.clone()),
            vec![
                0x060, 0x00, 1, 0, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x061, 0x00, 20, 0, // PID | length
                0x050, 0x00, 2, 0, // field_mutable: PID | length
                8, 0, 0, 0, // field_mutable: participant_key | padding (2 bytes)
                0x05A, 0x00, 1, 0, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
                1, 0, 0, 0, // field_mutable: Sentinel
                0x062, 0x00, 2, 0, // field_mutable: PID | length
                9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
                1, 0, 0, 0, // Sentinel
            ]
        );
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
