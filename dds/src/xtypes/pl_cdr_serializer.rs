use super::{error::XTypesError, serializer::XTypesSerializer};
use crate::xtypes::{
    dynamic_type::DynamicData, serializer::{LittleEndian, PaddedWrite, Write}, xcdr_serializer::Xcdr1LeSerializer,
};

const PID_SENTINEL: u16 = 1;

struct ByteCounter(u16);
impl ByteCounter {
    pub fn new() -> Self {
        Self(0)
    }
}
impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) {
        self.0 += buf.len() as u16;
    }
}
fn count_bytes_xdr1_le(v: &DynamicData, member_id: u32) -> Result<u16, XTypesError> {
    let mut byte_counter = ByteCounter::new();
    let mut byte_conter_serializer = Xcdr1LeSerializer::new(&mut byte_counter);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    Ok(byte_counter.0)
}


pub struct PlCdrLeSerializer<'a, C> {
    cdr1_le_serializer: Xcdr1LeSerializer<'a, C>,
}

impl<'a, C: Write> PlCdrLeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            cdr1_le_serializer: Xcdr1LeSerializer::new(collection),
        }
    }
}

impl<C: Write> XTypesSerializer for PlCdrLeSerializer<'_, C> {
    type Endianness = LittleEndian;

    fn writer(&mut self) -> &mut impl PaddedWrite {
        &mut self.cdr1_le_serializer.writer
    }

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.cdr1_le_serializer.serialize_final_struct(v)
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.cdr1_le_serializer.serialize_appendable_struct(v)
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let length = count_bytes_xdr1_le(v, member_id)?;
            let padded_length = (length + 3) & !3;
            self.serialize_u16(member_id as u16);
            self.serialize_u16(padded_length as u16);
            self.serialize_dynamic_data_member(v, member_id)?;
            self.cdr1_le_serializer.writer.writer.pad(4);
        }
        self.serialize_u16(PID_SENTINEL);
        self.serialize_u16(0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::type_support::TypeSupport;
    extern crate std;

    fn test_serialize_type_support<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(PlCdrLeSerializer::new(&mut buffer))
            .unwrap();
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
                10, 0, 4, 0, // PID | length
                7, 0, 0, 0, // field_u16 | padding (2 bytes)
                20, 0, 4, 0, // PID | length
                8, 0, 0, 0, // field_u8 | padding (3 bytes)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[derive(TypeSupport)]
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
                0x29, 0x00, 20, 0, // PID, length
                2, 0, 0, 0, // vec length
                4, 0, 0, 0, // String length
                b'o', b'n', b'e', 0, // String
                4, 0, 0, 0, // String length
                b't', b'w', b'o', 0, // String
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[derive(TypeSupport)]
    struct NestedFinal {
        basic: MutableBasicType,
        time: MutableTimeType,
    }
}
