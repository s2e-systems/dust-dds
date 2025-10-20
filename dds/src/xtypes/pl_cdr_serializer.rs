use super::{error::XTypesError, serializer::XTypesSerializer};
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    serializer::{EndiannessWriter, Write},
    xcdr_serializer::Xcdr1LeSerializer,
};

const PID_SENTINEL: u16 = 1;

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
    fn pos(&self) -> usize {
        self.0
    }
}
fn count_bytes_xdr1_le(v: &DynamicData, member_id: u32) -> Result<u16, XTypesError> {
    let mut byte_conter_serializer = Xcdr1LeSerializer::new(ByteCounter::new());
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    let length = byte_conter_serializer.into_inner().0 as u16;
    Ok((length + 3) & !3)
}
fn count_bytes_xdr1_le_complex(v: &DynamicData) -> Result<u16, XTypesError> {
    let mut byte_conter_serializer = Xcdr1LeSerializer::new(ByteCounter::new());
    byte_conter_serializer.serialize_complex(v)?;
    let length = byte_conter_serializer.into_inner().0 as u16;
    Ok((length + 3) & !3)
}

pub struct PlCdrLeSerializer<C> {
    cdr1_le_serializer: Xcdr1LeSerializer<C>,
}

impl<C: Write> PlCdrLeSerializer<C> {
    pub fn new(collection: C) -> Self {
        Self {
            cdr1_le_serializer: Xcdr1LeSerializer::new(collection),
        }
    }
}

impl<C: Write> XTypesSerializer<C> for PlCdrLeSerializer<C> {
    fn into_inner(self) -> C {
        self.cdr1_le_serializer.into_inner()
    }
    fn writer(&mut self) -> &mut impl EndiannessWriter {
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
            let member_descriptor = v.get_descriptor(member_id)?;
            if member_descriptor.is_optional {
                if let Some(default_value) = &member_descriptor.default_value {
                    if v.get_value(member_id)? == default_value {
                        continue;
                    }
                }
            }

            let element_type = member_descriptor
                .r#type
                .get_descriptor()
                .element_type
                .as_ref();
            if let Some(element_type) = element_type {
                // Sequence
                if element_type.get_kind() == TypeKind::STRUCTURE {
                    for vi in &v.get_complex_values(member_id)? {
                        let padded_length = count_bytes_xdr1_le_complex(vi)?;
                        self.writer().write_u16(member_id as u16);
                        self.writer().write_u16(padded_length as u16);
                        self.serialize_complex(vi)?
                    }

                    self.writer().pad(4);
                } else {
                    let padded_length = count_bytes_xdr1_le(v, member_id)?;
                    self.writer().write_u16(member_id as u16);
                    self.writer().write_u16(padded_length as u16);
                    self.serialize_dynamic_data_member(v, member_id)?;
                    self.writer().pad(4);
                }
            } else {
                // Structure
                let padded_length = count_bytes_xdr1_le(v, member_id)?;
                self.writer().write_u16(member_id as u16);
                self.writer().write_u16(padded_length as u16);
                self.serialize_dynamic_data_member(v, member_id)?;
                self.writer().pad(4);
            }
        }
        self.writer().write_u16(PID_SENTINEL);
        self.writer().write_u16(0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::type_support::TypeSupport;
    extern crate std;

    fn test_serialize_type_support<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        v.create_dynamic_sample()
            .serialize(PlCdrLeSerializer::new(Vec::new()))
            .unwrap()
            .into_inner()
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
                41, 0x00, 4, 0, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_locator() {
        #[derive(TypeSupport)]
        pub struct Locator {
            kind: i32,
            port: u32,
            address: [u8; 4],
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        pub struct LocatorContainer {
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
                41, 0x00, 12, 0, // PID, length
                1, 0, 0, 0, // kind
                2, 0, 0, 0, // port
                7, 7, 7, 7, // address
                1, 0, 0, 0, // Sentinel
            ]
        );
    }
}
