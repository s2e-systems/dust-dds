use super::{
    error::XTypesError,
    serialize::Write,
    serializer::{
        SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct, XTypesSerializer,
    },
};
use crate::xtypes::{
    bytes::Bytes,
    data_representation::DataKind,
    dynamic_type::DynamicData,
    serializer::{LittleEndian, TryWriteAsBytes, WriteAsBytes, Writer, WriterV1},
};
use alloc::string::String;

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
    writer: WriterV1<'a, C>,
}

impl<'a, C: Write> PlCdrLeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: WriterV1 {
                writer: Writer::new(collection),
            },
        }
    }
}

impl<C: Write> SerializeFinalStruct for &mut PlCdrLeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        self.serialize_data_kind(value)
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
        self.serialize_data_kind(value)
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
            serializer.serialize_data_kind(value)?;
            Ok(byte_counter.0)
        }

        if let DataKind::ComplexValueList(items) = value {
            for item in items {
                let length = bytes_len_dynamic_data(item)?;
                let padded_length = (length + 3) & !3;
                self.writer.writer.write_slice(&(pid as u16).to_le_bytes());
                self.writer.writer.write_slice(&padded_length.to_le_bytes());
                item.serialize(&mut **self)?;
                self.writer.writer.pad(4);
            }
        } else {
            let length = bytes_len_data_kind(value)?;
            let padded_length = (length + 3) & !3;
            self.writer.writer.write_slice(&(pid as u16).to_le_bytes());
            self.writer.writer.write_slice(&padded_length.to_le_bytes());
            self.serialize_data_kind(value)?;
            self.writer.writer.pad(4);
        }
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.writer.write_slice(&0u16.to_le_bytes());
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut PlCdrLeSerializer<'_, C> {
    type Endianness = LittleEndian;

    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_data_kind(self, v: &DataKind) -> Result<(), XTypesError> {
        todo!()
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
            .serialize(&mut PlCdrLeSerializer::new(&mut buffer))
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
