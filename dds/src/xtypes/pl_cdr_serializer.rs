use super::{
    error::XTypesError,
    serialize::Write,
    serializer::{
        SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct, XTypesSerializer,
    },
};
use crate::xtypes::{data_representation::DataKind, dynamic_type::DynamicData};

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

fn round_up_to_multiples(position: usize, alignment: usize) -> usize {
    position.div_ceil(alignment) * alignment
}

struct CollectionWriter<'a, C> {
    collection: &'a mut C,
    position: usize,
}

impl<'a, C: Write> CollectionWriter<'a, C> {
    fn new(collection: &'a mut C) -> Self {
        Self {
            collection,
            position: 0,
        }
    }

    fn write_slice(&mut self, data: &[u8]) {
        self.collection.write(data);
        self.position += data.len();
    }

    fn pad(&mut self, alignment: usize) {
        const ZEROS: [u8; 8] = [0; 8];
        let alignment = round_up_to_multiples(self.position, alignment) - self.position;
        self.write_slice(&ZEROS[..alignment]);
    }
}

fn extend_with_padding_v1<const N: usize, C: Write>(
    writer: &mut CollectionWriter<'_, C>,
    data: &[u8; N],
) -> Result<(), XTypesError> {
    writer.pad(N);
    writer.write_slice(data);
    Ok(())
}

fn into_u8(v: char) -> Result<u8, XTypesError> {
    if !v.is_ascii() {
        Err(XTypesError::InvalidData)
    } else {
        Ok(v as u8)
    }
}
fn into_u32(v: usize) -> Result<u32, XTypesError> {
    if v > u32::MAX as usize {
        Err(XTypesError::InvalidData)
    } else {
        Ok(v as u32)
    }
}
fn str_len(v: &str) -> Result<u32, XTypesError> {
    if !v.is_ascii() {
        Err(XTypesError::InvalidData)
    } else {
        into_u32(v.len() + 1)
    }
}
pub struct PlCdrLeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C: Write> PlCdrLeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
        }
    }
}

impl<C: Write> SerializeFinalStruct for &mut PlCdrLeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        value.serialize(&mut **self)
    }

    fn serialize_optional_field(
        &mut self,
        _value: &Option<DynamicData>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        unimplemented!()
    }
}
impl<C: Write> SerializeAppendableStruct for &mut PlCdrLeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        value.serialize(&mut **self)
    }
}
impl<C: Write> SerializeMutableStruct for &mut PlCdrLeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        fn bytes_len_dynamic_data(value: &DynamicData) -> Result<u16, XTypesError> {
            let mut byte_counter = ByteCounter::new();
            let mut serializer = PlCdrLeSerializer::new(&mut byte_counter);
            value.serialize(&mut serializer)?;
            Ok(byte_counter.0)
        }
        fn bytes_len_data_kind(value: &DataKind) -> Result<u16, XTypesError> {
            let mut byte_counter = ByteCounter::new();
            let mut serializer = PlCdrLeSerializer::new(&mut byte_counter);
            value.serialize(&mut serializer)?;
            Ok(byte_counter.0)
        }

        if let DataKind::ComplexValueList(items) = value {
            for item in items {
                let length = bytes_len_dynamic_data(item)?;
                let padded_length = (length + 3) & !3;
                self.writer.write_slice(&(pid as u16).to_le_bytes());
                self.writer.write_slice(&padded_length.to_le_bytes());
                item.serialize(&mut **self)?;
                self.writer.pad(4);
            }
        } else {
            let length = bytes_len_data_kind(value)?;
            let padded_length = (length + 3) & !3;
            self.writer.write_slice(&(pid as u16).to_le_bytes());
            self.writer.write_slice(&padded_length.to_le_bytes());
            value.serialize(&mut **self)?;
            self.writer.pad(4);
        }
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.write_slice(&0u16.to_le_bytes());
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut PlCdrLeSerializer<'_, C> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_complex_value(self, v: &DynamicData) -> Result<(), XTypesError> {
        v.serialize_nested(self)
    }

    fn serialize_complex_value_list(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        self.serialize_uint32(into_u32(vs.len())?)?;
        for v in vs {
            v.serialize_nested(&mut *self)?;
        }
        Ok(())
    }
    fn serialize_complex_value_array(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        for v in vs {
            v.serialize_nested(&mut *self)?;
        }
        Ok(())
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &[v as u8])
    }

    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }

    fn serialize_char8(self, v: char) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &into_u8(v)?.to_le_bytes())
    }

    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes());
        self.writer.write_slice(&[0]);
        Ok(())
    }

    fn serialize_boolean_list(self, _vs: &[bool]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint8_list(self, _vs: &[u8]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int8_list(self, _vs: &[i8]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int16_list(self, _vs: &[i16]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int32_list(self, _vs: &[i32]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int64_list(self, _vs: &[i64]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint16_list(self, _vs: &[u16]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint32_list(self, _vs: &[u32]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint64_list(self, _vs: &[u64]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_float32_list(self, _vs: &[f32]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_float64_list(self, _vs: &[f64]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_char8_list(self, _vs: &[char]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_string_list(self, v: &[String]) -> Result<(), XTypesError> {
        self.serialize_uint32(into_u32(v.len())?)?;
        for value in v {
            self.serialize_string(value)?;
        }
        Ok(())
    }

    fn serialize_boolean_array(self, _vs: &[bool]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint8_array(self, v: &[u8]) -> Result<(), XTypesError> {
        self.writer.write_slice(v);
        Ok(())
    }

    fn serialize_int8_array(self, _vs: &[i8]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int16_array(self, _vs: &[i16]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int32_array(self, _vs: &[i32]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_int64_array(self, _vs: &[i64]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint16_array(self, _vs: &[u16]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint32_array(self, _vs: &[u32]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_uint64_array(self, _vs: &[u64]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_float32_array(self, _vs: &[f32]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_float64_array(self, _vs: &[f64]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_char8_array(self, _vs: &[char]) -> Result<(), XTypesError> {
        unimplemented!()
    }

    fn serialize_string_array(self, _vs: &[String]) -> Result<(), XTypesError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{infrastructure::type_support::TypeSupport, xtypes::binding::XTypesBinding};
    extern crate std;

    fn test_serialize_type_support<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(&mut PlCdrLeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    // #[test]
    // fn serialize_octet() {
    //     let v = 0x20u8;
    //     assert_eq!(test_serialize_type_support(&v), vec![0x20]);
    // }

    // #[test]
    // fn serialize_char() {
    //     let v = 'Z';
    //     assert_eq!(test_serialize_type_support(&v), vec![0x5a]);
    // }

    // #[test]
    // fn serialize_ushort() {
    //     let v = 65500u16;
    //     assert_eq!(test_serialize_type_support(&v), vec![0xdc, 0xff,]);
    // }

    // #[test]
    // fn serialize_short() {
    //     let v = -32700i16;
    //     assert_eq!(test_serialize_type_support(&v), vec![0x44, 0x80,]);
    // }

    // #[test]
    // fn serialize_ulong() {
    //     let v = 4294967200u32;
    //     assert_eq!(
    //         test_serialize_type_support(&v),
    //         vec![0xa0, 0xff, 0xff, 0xff]
    //     );
    // }

    // #[test]
    // fn serialize_long() {
    //     let v = -2147483600i32;
    //     assert_eq!(
    //         test_serialize_type_support(&v),
    //         vec![0x30, 0x00, 0x00, 0x80,]
    //     );
    // }

    // #[test]
    // fn serialize_ulonglong() {
    //     let v = 18446744073709551600u64;
    //     assert_eq!(
    //         test_serialize_type_support(&v),
    //         vec![0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,]
    //     );
    // }

    // #[test]
    // fn serialize_longlong() {
    //     let v = -9223372036800i64;
    //     assert_eq!(
    //         test_serialize_type_support(&v),
    //         vec![0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff,]
    //     );
    // }

    // #[test]
    // fn serialize_float() {
    //     let v = core::f32::MIN_POSITIVE;
    //     assert_eq!(
    //         test_serialize_type_support(&v),
    //         vec![0x00, 0x00, 0x80, 0x00]
    //     );
    // }

    // #[test]
    // fn serialize_double() {
    //     let v = core::f64::MIN_POSITIVE;
    //     assert_eq!(
    //         test_serialize_type_support(&v),
    //         vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
    //     );
    // }

    // #[test]
    // fn serialize_bool() {
    //     let v = true;
    //     assert_eq!(test_serialize_type_support(&v), vec![0x01]);
    // }

    // #[test]
    // fn serialize_string() {
    //     let v = "Hola";
    //     assert_eq!(
    //         test_serialize_type_support(v),
    //         vec![
    //             5, 0, 0, 0, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]
    //     );
    // }

    // #[test]
    // fn serialize_empty_string() {
    //     let v = "";
    //     assert_eq!(
    //         test_serialize_type_support(v),
    //         vec![0x01, 0x00, 0x00, 0x00, 0x00]
    //     );
    // }

    // #[test]
    // fn serialize_byte_slice() {
    //     let v = &[1u8, 2, 3, 4, 5][..];
    //     assert_eq!(
    //         test_serialize(&v),
    //         vec![
    //             5, 0, 0, 0, // length
    //             1, 2, 3, 4, 5 // data
    //         ]
    //     );
    // }

    // #[test]
    // fn serialize_byte_array() {
    //     let v = [1u8, 2, 3, 4, 5];
    //     assert_eq!(test_serialize_type_support(v), vec![1, 2, 3, 4, 5]);
    // }

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

    #[derive(TypeSupport)]
    struct NestedFinal {
        basic: MutableBasicType,
        time: MutableTimeType,
    }

    #[test]
    fn serialize_mutable_nested_struct() {
        let v = NestedFinal {
            basic: MutableBasicType {
                field_u16: 7,
                field_u8: 8,
            },
            time: MutableTimeType {
                field_time: Time { sec: 5, nanosec: 6 },
            },
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                10, 0, 4, 0, // PID | length
                7, 0, 0, 0, // field_u16 | padding (2 bytes)
                20, 0, 4, 0, // PID | length
                8, 0, 0, 0, // field_u8 | padding (3 bytes)
                1, 0, 0, 0, // Sentinel
                30, 0, 8, 0, // PID | length
                5, 0, 0, 0, // Time: sec
                6, 0, 0, 0, // Time: nanosec
                1, 0, 0, 0, // Sentinel
            ]
        );
    }
}
