use super::{
    error::XTypesError,
    serialize::Write,
    serializer::{
        SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct, XTypesSerializer,
    },
};
use crate::xtypes::{
    data_representation::DataKind,
    dynamic_type::{DynamicData, ExtensibilityKind, TypeKind},
    serializer::{BigEndian, LittleEndian, WriteAsBytes, Writer, WriterV1, WriterV2},
};
use alloc::vec::Vec;

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

impl Xcdr1BeSerializer<'_, ()> {
    pub fn bytes_len(value: &DynamicData) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr1BeSerializer::new(&mut byte_counter);
        value.serialize(&mut serializer)?;
        Ok(byte_counter.0)
    }
    pub fn bytes_len_data_kind(value: &DataKind) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr1BeSerializer::new(&mut byte_counter);
        serializer.serialize_data_kind(value)?;
        Ok(byte_counter.0)
    }
}

impl<C: Write> SerializeFinalStruct for &mut Xcdr1BeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        self.serialize_data_kind(value)
    }
}
impl<C: Write> SerializeAppendableStruct for &mut Xcdr1BeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        self.serialize_data_kind(value)
    }
}
impl<C: Write> SerializeMutableStruct for &mut Xcdr1BeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        let length = Xcdr1BeSerializer::bytes_len_data_kind(value)? as u16;
        self.writer.writer.write_slice(&(pid as u16).to_be_bytes());
        self.writer.writer.write_slice(&length.to_be_bytes());
        self.serialize_data_kind(value)?;
        self.writer.writer.pad(4);
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.writer.write_slice(&PID_SENTINEL.to_be_bytes());
        self.writer.writer.write_slice(&0u16.to_be_bytes());
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr1BeSerializer<'_, C> {
    type Endianness = BigEndian;

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
        match v {
            DataKind::UInt8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Boolean(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::String(v) => {
                self.serialize_data_kind(&DataKind::UInt32(v.len() as u32 + 1))?;
                WriteAsBytes::<Self::Endianness>::write_as_bytes(
                    v.as_str(),
                    &mut self.writer.writer,
                );
            }
            DataKind::ComplexValue(dynamic_data) => serialize_nested(dynamic_data, &mut *self)?,
            DataKind::Sequence(v) => seralize_sequence(self, v)?,
            DataKind::Array(v) => seralize_array(self, v)?,
            DataKind::Char8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
        };
        Ok(())
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

impl Xcdr1LeSerializer<'_, ()> {
    pub fn bytes_len(value: &DynamicData) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr1LeSerializer::new(&mut byte_counter);
        value.serialize(&mut serializer)?;
        Ok(byte_counter.0)
    }
    pub fn bytes_len_data_kind(value: &DataKind) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr1LeSerializer::new(&mut byte_counter);
        serializer.serialize_data_kind(value)?;
        Ok(byte_counter.0)
    }
}

impl<C: Write> SerializeFinalStruct for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        self.serialize_data_kind(value)
    }
}
impl<C: Write> SerializeAppendableStruct for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        self.serialize_data_kind(value)
    }
}
impl<C: Write> SerializeMutableStruct for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        let length = Xcdr1LeSerializer::bytes_len_data_kind(value)? as u16;
        self.writer.writer.write_slice(&(pid as u16).to_le_bytes());
        self.writer.writer.write_slice(&length.to_le_bytes());
        self.serialize_data_kind(value)?;
        self.writer.writer.pad(4);
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.writer.write_slice(&0u16.to_le_bytes());
        Ok(())
    }
}

pub fn serialize_nested(
    dynamic_data: &DynamicData,
    serializer: impl XTypesSerializer,
) -> Result<(), super::error::XTypesError> {
    match dynamic_data.type_ref().get_kind() {
        TypeKind::ENUM => {
            serializer.serialize_data_kind(dynamic_data.get_value(0)?)?;
        }
        TypeKind::STRUCTURE => match dynamic_data.type_ref().get_descriptor().extensibility_kind {
            ExtensibilityKind::Final => {
                let mut final_serializer = serializer.serialize_final_struct()?;
                for field_index in 0..dynamic_data.get_item_count() {
                    let member_id = dynamic_data.get_member_id_at_index(field_index)?;
                    let member_descriptor = dynamic_data.get_descriptor(member_id)?;
                    final_serializer.serialize_field(
                        dynamic_data.get_value(member_id)?,
                        &member_descriptor.name,
                    )?;
                }
            }
            ExtensibilityKind::Appendable => {
                let mut appendable_serializer = serializer.serialize_appendable_struct()?;
                for field_index in 0..dynamic_data.get_item_count() {
                    let member_id = dynamic_data.get_member_id_at_index(field_index)?;
                    let member_descriptor = dynamic_data.get_descriptor(member_id)?;
                    appendable_serializer.serialize_field(
                        dynamic_data.get_value(member_id)?,
                        &member_descriptor.name,
                    )?;
                }
            }
            ExtensibilityKind::Mutable => {
                let mut mutable_serializer = serializer.serialize_mutable_struct()?;
                for field_index in 0..dynamic_data.get_item_count() {
                    let member_id = dynamic_data.get_member_id_at_index(field_index)?;
                    let member_descriptor = dynamic_data.get_descriptor(member_id)?;
                    let value = dynamic_data.get_value(member_id)?;
                    if member_descriptor.is_optional {
                        if let Some(default_value) = &member_descriptor.default_value {
                            if value == default_value {
                                continue;
                            }
                        }
                    }
                    mutable_serializer.serialize_field(
                        value,
                        member_id,
                        &member_descriptor.name,
                    )?;
                }
                mutable_serializer.end()?;
            }
        },

        kind => unimplemented!("Should not reach for {kind:?}"),
    }
    Ok(())
}

fn seralize_sequence<T>(serializer: &mut T, values: &Vec<DataKind>) -> Result<(), XTypesError>
where
    for<'a> &'a mut T: XTypesSerializer,
{
    serializer.serialize_data_kind(&DataKind::UInt32(values.len() as u32))?;
    seralize_array(serializer, values)
}
fn seralize_array<T>(serializer: &mut T, values: &Vec<DataKind>) -> Result<(), XTypesError>
where
    for<'a> &'a mut T: XTypesSerializer,
{
    for value in values {
        serializer.serialize_data_kind(value)?;
    }
    Ok(())
}

impl<C: Write> XTypesSerializer for &mut Xcdr1LeSerializer<'_, C> {
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
        match v {
            DataKind::UInt8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Boolean(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::String(v) => {
                self.serialize_data_kind(&DataKind::UInt32(v.len() as u32 + 1))?;
                WriteAsBytes::<Self::Endianness>::write_as_bytes(
                    v.as_str(),
                    &mut self.writer.writer,
                );
            }
            DataKind::ComplexValue(dynamic_data) => serialize_nested(dynamic_data, &mut *self)?,
            DataKind::Sequence(v) => seralize_sequence(self, v)?,
            DataKind::Array(v) => seralize_array(self, v)?,
            DataKind::Char8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
        };
        Ok(())
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

impl Xcdr2BeSerializer<'_, ()> {
    pub fn bytes_len(value: &DynamicData) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr2BeSerializer::new(&mut byte_counter);
        value.serialize(&mut serializer)?;
        Ok(byte_counter.0)
    }
    pub fn bytes_len_data_kind(value: &DataKind) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr2BeSerializer::new(&mut byte_counter);
        serializer.serialize_data_kind(value)?;
        Ok(byte_counter.0)
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

impl Xcdr2LeSerializer<'_, ()> {
    pub fn bytes_len(value: &DynamicData) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr2LeSerializer::new(&mut byte_counter);
        value.serialize(&mut serializer)?;
        Ok(byte_counter.0)
    }
    pub fn bytes_len_data_kind(value: &DataKind) -> Result<usize, XTypesError> {
        let mut byte_counter = ByteCounter::new();
        let mut serializer = Xcdr2LeSerializer::new(&mut byte_counter);
        serializer.serialize_data_kind(value)?;
        Ok(byte_counter.0)
    }
}

impl<C: Write> SerializeMutableStruct for &mut Xcdr2BeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        let length = Xcdr2BeSerializer::bytes_len_data_kind(value)? as u16;
        self.writer.writer.write_slice(&(pid as u16).to_be_bytes());
        self.writer.writer.write_slice(&length.to_be_bytes());
        self.serialize_data_kind(value)?;
        self.writer.writer.pad(4);
        Ok(())
    }
    fn end(self) -> Result<(), XTypesError> {
        self.writer.writer.write_slice(&PID_SENTINEL.to_be_bytes());
        self.writer.writer.write_slice(&0u16.to_be_bytes());
        Ok(())
    }
}

impl<C: Write> SerializeMutableStruct for &mut Xcdr2LeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        let length = Xcdr2LeSerializer::bytes_len_data_kind(value)? as u16;
        self.writer.writer.write_slice(&(pid as u16).to_le_bytes());
        self.writer.writer.write_slice(&length.to_le_bytes());
        self.serialize_data_kind(value)?;
        self.writer.writer.pad(4);
        Ok(())
    }
    fn end(self) -> Result<(), XTypesError> {
        self.writer.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.writer.write_slice(&0u16.to_le_bytes());
        Ok(())
    }
}

struct PlainCdr2Encoder<'a, S> {
    serializer: &'a mut S,
}

impl<S> SerializeFinalStruct for PlainCdr2Encoder<'_, S>
where
    for<'a> &'a mut S: XTypesSerializer,
{
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        self.serializer.serialize_data_kind(value)
    }
}

impl<C: Write> SerializeAppendableStruct for &mut Xcdr2BeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        let length = Xcdr2BeSerializer::bytes_len_data_kind(value)? as u32;
        // DHEADER
        self.serialize_data_kind(&DataKind::UInt32(length))?;
        self.serialize_data_kind(value)
    }
}

impl<C: Write> SerializeAppendableStruct for &mut Xcdr2LeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        let length = Xcdr2LeSerializer::bytes_len_data_kind(value)? as u32;
        // DHEADER
        self.serialize_data_kind(&DataKind::UInt32(length))?;
        self.serialize_data_kind(value)?;
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr2BeSerializer<'_, C> {
    type Endianness = BigEndian;

    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_data_kind(self, v: &DataKind) -> Result<(), XTypesError> {
        match v {
            DataKind::UInt8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Boolean(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::String(v) => {
                self.serialize_data_kind(&DataKind::UInt32(v.len() as u32 + 1))?;
                WriteAsBytes::<Self::Endianness>::write_as_bytes(
                    v.as_str(),
                    &mut self.writer.writer,
                );
            }
            DataKind::ComplexValue(dynamic_data) => serialize_nested(dynamic_data, &mut *self)?,
            DataKind::Sequence(v) => seralize_sequence(self, v)?,
            DataKind::Array(v) => seralize_array(self, v)?,
            DataKind::Char8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
        };
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr2LeSerializer<'_, C> {
    type Endianness = LittleEndian;

    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_data_kind(self, v: &DataKind) -> Result<(), XTypesError> {
        match v {
            DataKind::UInt8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int16(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Int64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::UInt64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float32(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Float64(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::Boolean(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
            DataKind::String(v) => {
                self.serialize_data_kind(&DataKind::UInt32(v.len() as u32 + 1))?;
                WriteAsBytes::<Self::Endianness>::write_as_bytes(
                    v.as_str(),
                    &mut self.writer.writer,
                );
            }
            DataKind::ComplexValue(dynamic_data) => serialize_nested(dynamic_data, &mut *self)?,
            DataKind::Sequence(v) => seralize_sequence(self, v)?,
            DataKind::Array(v) => seralize_array(self, v)?,
            DataKind::Char8(v) => {
                WriteAsBytes::<Self::Endianness>::write_as_bytes(v, &mut self.writer)
            }
        };
        Ok(())
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
            .serialize(&mut Xcdr1BeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v1_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v2_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(&mut Xcdr2BeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v2_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut buffer = std::vec::Vec::new();
        v.create_dynamic_sample()
            .serialize(&mut Xcdr2LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
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
        // @id(0x005A) @key
        #[dust_dds(id = 0x5A, key)]
        key: u8,
        // @id(0x0050)
        #[dust_dds(id = 0x50)]
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
                0x00, 0x050, 0, 2, // PID | length
                0, 8, 0, 0, // participant_key | padding (2 bytes)
                0x00, 0x05A, 0, 1, // PID | length
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
        #[dust_dds(id = 0x60, key)]
        field_primitive: u8,
        #[dust_dds(id = 0x61)]
        field_mutable: MutableType,
        #[dust_dds(id = 0x62)]
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

    #[test]
    fn serialize_basic_types_struct() {
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
                0, 0, 0, 0, // f9-1: u64
                0, 0, 0, 9, // f9-2: u64
                0x3F, 0x80, 0x00, 0x00, // f10: f32
                0x3F, 0xF0, 0x00, 0x00, // f11-1: f64
                0x00, 0x00, 0x00, 0x00, // f11-2: f64
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
}
