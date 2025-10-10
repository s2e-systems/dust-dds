use super::{
    error::XTypesError,
    serialize::Write,
    serializer::{
        SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct, XTypesSerializer,
    },
};
use crate::xtypes::{data_representation::DataKind, dynamic_type::DynamicData};

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

fn extend_with_padding_v2<const N: usize, C: Write>(
    writer: &mut CollectionWriter<'_, C>,
    data: &[u8; N],
) -> Result<(), XTypesError> {
    writer.pad(core::cmp::min(N, 4));
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

pub struct Xcdr1BeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C: Write> Xcdr1BeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
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
        value.serialize(&mut serializer)?;
        Ok(byte_counter.0)
    }
}

impl<C: Write> SerializeFinalStruct for &mut Xcdr1BeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        value.serialize(&mut **self)
    }

    fn serialize_optional_field(
        &mut self,
        value: &Option<DynamicData>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        self.writer.pad(4);
        if let Some(value) = value {
            let length = Xcdr1BeSerializer::bytes_len(value)? as u16;
            self.writer.write_slice(&0_u16.to_be_bytes());
            self.writer.write_slice(&length.to_be_bytes());
            value.serialize(&mut **self)
        } else {
            self.writer.write_slice(&0_u16.to_be_bytes());
            self.writer.write_slice(&0_u16.to_be_bytes());
            Ok(())
        }
    }
}
impl<C: Write> SerializeAppendableStruct for &mut Xcdr1BeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        value.serialize(&mut **self)
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
        self.writer.write_slice(&(pid as u16).to_be_bytes());
        self.writer.write_slice(&length.to_be_bytes());
        value.serialize(&mut **self)?;
        self.writer.pad(4);
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_be_bytes());
        self.writer.write_slice(&0u16.to_be_bytes());
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr1BeSerializer<'_, C> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_complex_value_list(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }
    fn serialize_complex_value_array(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &[v as u8])
    }

    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }

    fn serialize_char8(self, v: char) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &into_u8(v)?.to_be_bytes())
    }

    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes());
        self.writer.write_slice(&[0]);
        Ok(())
    }

    fn serialize_uint8_array(self, vs: &[u8]) -> Result<(), XTypesError> {
        Ok(self.writer.write_slice(vs))
    }

    fn serialize_string_list(self, vs: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_list(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_list(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_list(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_list(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_list(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint8_list(self, vs: &[u8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_list(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_list(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_list(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_list(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_list(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_list(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_array(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_array(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_array(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_array(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_array(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_array(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_array(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_array(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_array(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_array(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_array(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_string_array(self, vs: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_complex_value(self, vs: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

pub struct Xcdr1LeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C: Write> Xcdr1LeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
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
}

impl<C: Write> SerializeFinalStruct for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        value.serialize(&mut **self)
    }

    fn serialize_optional_field(
        &mut self,
        value: &Option<DynamicData>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        if let Some(value) = value {
            let length = Xcdr1LeSerializer::bytes_len(value)? as u16;
            self.writer.pad(4);
            self.writer.write_slice(&0_u16.to_le_bytes());
            self.writer.write_slice(&length.to_le_bytes());
            value.serialize(&mut **self)
        } else {
            self.writer.pad(4);
            self.writer.write_slice(&0_u16.to_le_bytes());
            self.writer.write_slice(&0_u16.to_le_bytes());
            Ok(())
        }
    }
}
impl<C: Write> SerializeAppendableStruct for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        todo!()
    }
}
impl<C: Write> SerializeMutableStruct for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        todo!()
        // let length = Xcdr1LeSerializer::bytes_len(value)? as u16;
        // self.writer.write_slice(&(pid as u16).to_le_bytes());
        // self.writer.write_slice(&length.to_le_bytes());
        // value.serialize(&mut **self)?;
        // self.writer.pad(4);
        // Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.write_slice(&0u16.to_le_bytes());
        Ok(())
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr1LeSerializer<'_, C> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_complex_value_list(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }
    fn serialize_complex_value_array(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
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

    fn serialize_uint8_array(self, v: &[u8]) -> Result<(), XTypesError> {
        self.writer.write_slice(v);
        Ok(())
    }

    fn serialize_string_list(self, v: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_list(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_list(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_list(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_list(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_list(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint8_list(self, vs: &[u8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_list(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_list(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_list(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_list(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_list(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_list(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_array(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_array(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_array(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_array(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_array(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_array(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_array(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_array(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_array(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_array(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_array(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_string_array(self, vs: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_complex_value(self, vs: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

pub struct Xcdr2BeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C: Write> Xcdr2BeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
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
}

pub struct Xcdr2LeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C: Write> Xcdr2LeSerializer<'a, C> {
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
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
}

impl<C: Write> SerializeMutableStruct for &mut Xcdr2BeSerializer<'_, C> {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        _name: &str,
    ) -> Result<(), XTypesError> {
        todo!()
        // let length = Xcdr2BeSerializer::bytes_len(value)? as u16;
        // self.writer.write_slice(&(pid as u16).to_be_bytes());
        // self.writer.write_slice(&length.to_be_bytes());
        // value.serialize(&mut **self)?;
        // self.writer.pad(4);
        // Ok(())
    }
    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_be_bytes());
        self.writer.write_slice(&0u16.to_be_bytes());
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
        todo!()
        // let length = Xcdr2LeSerializer::bytes_len(value)? as u16;
        // self.writer.write_slice(&(pid as u16).to_le_bytes());
        // self.writer.write_slice(&length.to_le_bytes());
        // value.serialize(&mut **self)?;
        // self.writer.pad(4);
        // Ok(())
    }
    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.write_slice(&0u16.to_le_bytes());
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
        value.serialize(&mut *self.serializer)
    }

    fn serialize_optional_field(
        &mut self,
        value: &Option<DynamicData>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        // if let Some(value) = value {
        //     true.serialize(&mut *self.serializer)?;
        //     value.serialize(&mut *self.serializer)
        // } else {
        //     false.serialize(&mut *self.serializer)
        // }
        todo!()
    }
}

impl<C: Write> SerializeAppendableStruct for &mut Xcdr2BeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        // let length = Xcdr2BeSerializer::bytes_len(value)? as u32;
        // // DHEADER
        // length.serialize(&mut **self)?;
        // value.serialize(&mut **self)?;
        // Ok(())
        todo!()
    }
}

impl<C: Write> SerializeAppendableStruct for &mut Xcdr2LeSerializer<'_, C> {
    fn serialize_field(&mut self, value: &DataKind, _name: &str) -> Result<(), XTypesError> {
        // let length = Xcdr2LeSerializer::bytes_len(value)? as u32;
        // // DHEADER
        // length.serialize(&mut **self)?;
        // value.serialize(&mut **self)?;
        // Ok(())
        todo!()
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr2BeSerializer<'_, C> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_complex_value_list(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }
    fn serialize_complex_value_array(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &(v as u8).to_be_bytes())
    }
    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &into_u8(v)?.to_be_bytes())
    }
    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes());
        self.writer.write_slice(&[0]);
        Ok(())
    }

    fn serialize_uint8_array(self, v: &[u8]) -> Result<(), XTypesError> {
        self.writer.write_slice(v);
        Ok(())
    }

    fn serialize_string_list(self, vs: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_list(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_list(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_list(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_list(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_list(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint8_list(self, vs: &[u8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_list(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_list(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_list(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_list(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_list(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_list(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_array(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_array(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_array(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_array(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_array(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_array(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_array(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_array(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_array(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_array(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_array(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_string_array(self, vs: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_complex_value(self, vs: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

impl<C: Write> XTypesSerializer for &mut Xcdr2LeSerializer<'_, C> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_complex_value_list(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }
    fn serialize_complex_value_array(self, vs: &[DynamicData]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        extend_with_padding_v2(&mut self.writer, &v.to_le_bytes())
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

    fn serialize_uint8_array(self, v: &[u8]) -> Result<(), XTypesError> {
        self.writer.write_slice(v);
        Ok(())
    }

    fn serialize_string_list(self, v: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_list(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_list(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_list(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_list(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_list(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint8_list(self, vs: &[u8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_list(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_list(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_list(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_list(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_list(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_list(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_boolean_array(self, vs: &[bool]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int8_array(self, vs: &[i8]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int16_array(self, vs: &[i16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int32_array(self, vs: &[i32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_int64_array(self, vs: &[i64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint16_array(self, vs: &[u16]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint32_array(self, vs: &[u32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_uint64_array(self, vs: &[u64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float32_array(self, vs: &[f32]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_float64_array(self, vs: &[f64]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_char8_array(self, vs: &[char]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_string_array(self, vs: &[String]) -> Result<(), XTypesError> {
        todo!()
    }

    fn serialize_complex_value(self, vs: &DynamicData) -> Result<(), XTypesError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::type_support::TypeSupport;

    use super::*;
    extern crate std;

    #[test]
    fn round_up_to_multiples_2() {
        assert_eq!(round_up_to_multiples(0, 2), 0);
        assert_eq!(round_up_to_multiples(1, 2), 2);
        assert_eq!(round_up_to_multiples(2, 2), 2);
        assert_eq!(round_up_to_multiples(3, 2), 4);
        assert_eq!(round_up_to_multiples(4, 2), 4);
    }

    #[test]
    fn round_up_to_multiples_4() {
        assert_eq!(round_up_to_multiples(0, 4), 0);
        assert_eq!(round_up_to_multiples(1, 4), 4);
        assert_eq!(round_up_to_multiples(2, 4), 4);
        assert_eq!(round_up_to_multiples(3, 4), 4);
        assert_eq!(round_up_to_multiples(4, 4), 4);
        assert_eq!(round_up_to_multiples(5, 4), 8);
    }
    #[test]
    fn round_up_to_multiples_8() {
        assert_eq!(round_up_to_multiples(0, 8), 0);
        assert_eq!(round_up_to_multiples(1, 8), 8);
        assert_eq!(round_up_to_multiples(2, 8), 8);
        assert_eq!(round_up_to_multiples(3, 8), 8);
        assert_eq!(round_up_to_multiples(4, 8), 8);
        assert_eq!(round_up_to_multiples(5, 8), 8);
        assert_eq!(round_up_to_multiples(6, 8), 8);
        assert_eq!(round_up_to_multiples(7, 8), 8);
        assert_eq!(round_up_to_multiples(8, 8), 8);
        assert_eq!(round_up_to_multiples(9, 8), 16);
    }

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

    // #[test]
    // fn serialize_octet() {
    //     let v = 0x20u8;
    //     assert_eq!(serialize_v1_be(&v), vec![0x20]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x20]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x20]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x20]);
    // }

    // #[test]
    // fn serialize_char() {
    //     let v = 'Z';
    //     assert_eq!(serialize_v1_be(&v), vec![0x5a]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x5a]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x5a]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x5a]);
    // }

    // #[test]
    // fn serialize_ushort() {
    //     let v = 65500u16;
    //     assert_eq!(serialize_v1_be(&v), vec![0xff, 0xdc]);
    //     assert_eq!(serialize_v1_le(&v), vec![0xdc, 0xff]);
    //     assert_eq!(serialize_v2_be(&v), vec![0xff, 0xdc]);
    //     assert_eq!(serialize_v2_le(&v), vec![0xdc, 0xff]);
    // }

    // #[test]
    // fn serialize_short() {
    //     let v = -32700i16;
    //     assert_eq!(serialize_v1_be(&v), vec![0x80, 0x44]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x44, 0x80]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x80, 0x44]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x44, 0x80]);
    // }

    // #[test]
    // fn serialize_ulong() {
    //     let v = 4294967200u32;
    //     assert_eq!(serialize_v1_be(&v), vec![0xff, 0xff, 0xff, 0xa0]);
    //     assert_eq!(serialize_v1_le(&v), vec![0xa0, 0xff, 0xff, 0xff]);
    //     assert_eq!(serialize_v2_be(&v), vec![0xff, 0xff, 0xff, 0xa0]);
    //     assert_eq!(serialize_v2_le(&v), vec![0xa0, 0xff, 0xff, 0xff]);
    // }

    // #[test]
    // fn serialize_long() {
    //     let v = -2147483600i32;
    //     assert_eq!(serialize_v1_be(&v), vec![0x80, 0x00, 0x00, 0x30]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x30, 0x00, 0x00, 0x80]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x80, 0x00, 0x00, 0x30]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x30, 0x00, 0x00, 0x80]);
    // }

    // #[test]
    // fn serialize_ulonglong() {
    //     let v = 18446744073709551600u64;
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    //     );
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    //     );
    // }

    // #[test]
    // fn serialize_longlong() {
    //     let v = -9223372036800i64;
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]
    //     );
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]
    //     );
    // }

    // #[test]
    // fn serialize_float() {
    //     let v = core::f32::MIN_POSITIVE;
    //     assert_eq!(serialize_v1_be(&v), vec![0x00, 0x80, 0x00, 0x00]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x00, 0x00, 0x80, 0x00]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x00, 0x80, 0x00, 0x00]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x00, 0x00, 0x80, 0x00]);
    // }

    // #[test]
    // fn serialize_double() {
    //     let v = core::f64::MIN_POSITIVE;
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
    //     );
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
    //     );
    // }

    // #[test]
    // fn serialize_bool() {
    //     let v = true;
    //     assert_eq!(serialize_v1_be(&v), vec![0x01]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x01]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x01]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x01]);
    // }

    // #[test]
    // fn serialize_string() {
    //     let v = "Hola";
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![
    //             0, 0, 0, 5, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![
    //             5, 0, 0, 0, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             0, 0, 0, 5, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
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
    //     assert_eq!(serialize_v1_be(&v), vec![0x00, 0x00, 0x00, 0x01, 0x00]);
    //     assert_eq!(serialize_v1_le(&v), vec![0x01, 0x00, 0x00, 0x00, 0x00]);
    //     assert_eq!(serialize_v2_be(&v), vec![0x00, 0x00, 0x00, 0x01, 0x00]);
    //     assert_eq!(serialize_v2_le(&v), vec![0x01, 0x00, 0x00, 0x00, 0x00]);
    // }

    // #[test]
    // fn serialize_byte_slice() {
    //     let v = &[1u8, 2, 3, 4, 5][..];
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![
    //             0, 0, 0, 5, // length
    //             1, 2, 3, 4, 5 // data
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![
    //             5, 0, 0, 0, // length
    //             1, 2, 3, 4, 5 // data
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             0, 0, 0, 5, // length
    //             1, 2, 3, 4, 5 // data
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![
    //             5, 0, 0, 0, // length
    //             1, 2, 3, 4, 5 // data
    //         ]
    //     );
    // }

    // #[test]
    // fn serialize_byte_array() {
    //     let v = [1u8, 2, 3, 4, 5];
    //     assert_eq!(serialize_v1_be(&v), vec![1, 2, 3, 4, 5]);
    //     assert_eq!(serialize_v1_le(&v), vec![1, 2, 3, 4, 5]);
    //     assert_eq!(serialize_v2_be(&v), vec![1, 2, 3, 4, 5]);
    //     assert_eq!(serialize_v2_le(&v), vec![1, 2, 3, 4, 5]);
    // }

    #[derive(TypeSupport, Clone)]
    // #[dust_dds(extensibility = "final")]
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

    // //@extensibility(FINAL)
    // struct FinalOptionalType {
    //     field: u8,
    //     optional_field: Option<u16>,
    // }

    // impl XTypesSerialize for FinalOptionalType {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut serializer = serializer.serialize_final_struct()?;
    //         serializer.serialize_field(&self.field, "field")?;
    //         serializer.serialize_optional_field(&self.optional_field, "optional_field")
    //     }
    // }
    // #[test]
    // fn serialize_final_optional_struct_some() {
    //     let some = FinalOptionalType {
    //         field: 6,
    //         optional_field: Some(7),
    //     };
    //     //PLAIN_CDR:
    //     assert_eq!(
    //         serialize_v1_be(&some),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 2, // HEADER (FLAGS+ID | length)
    //             0, 7, // optional_field value
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&some),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 2, 0, // HEADER (FLAGS+ID | length)
    //             7, 0 // optional_field value
    //         ]
    //     );
    //     //PLAIN_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(&some),
    //         vec![
    //             6, 1, // u8 | boolean for option
    //             0, 7 // optional_field value
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&some),
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
    //         serialize_v1_be(&none),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 0, // HEADER (FLAGS+ID | length)
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&none),
    //         vec![
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 0, // HEADER (FLAGS+ID | length)
    //         ]
    //     );
    //     // PLAIN_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(&none),
    //         vec![
    //         6, 0, // u8 | boolean for option
    //     ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&none),
    //         vec![
    //         6, 0, // u8 | boolean for option
    //     ]
    //     );
    // }

    // //@extensibility(FINAL)
    // struct NestedFinalType {
    //     field_nested: FinalType,
    //     field_u8: u8,
    // }
    // impl XTypesSerialize for NestedFinalType {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut serializer = serializer.serialize_final_struct()?;
    //         serializer.serialize_field(&self.field_nested, "field_nested")?;
    //         serializer.serialize_field(&self.field_u8, "field_u8")
    //     }
    // }
    // #[test]
    // fn serialize_nested_final_struct() {
    //     let v = NestedFinalType {
    //         field_nested: FinalType {
    //             field_u16: 7,
    //             field_u64: 9,
    //         },
    //         field_u8: 10,
    //     };
    //     // PLAIN_CDR:
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![
    //             0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
    //             0, 0, 0, 0, 0, 0, 0, 9,  // u64
    //             10, //u8
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![
    //             7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
    //             9, 0, 0, 0, 0, 0, 0, 0,  // u64
    //             10, //u8
    //         ]
    //     );
    //     // PLAIN_CDR2:
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![
    //             7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
    //             9, 0, 0, 0, 0, 0, 0, 0,  // u64
    //             10, //u8
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             0, 7, 0, 0, // nested FinalType (u16) | padding
    //             0, 0, 0, 0, 0, 0, 0, 9,  // u64
    //             10, //u8
    //         ]
    //     );
    // }

    // // @extensibility(APPENDABLE) @nested
    // struct AppendableType {
    //     value: u16,
    // }
    // impl XTypesSerialize for AppendableType {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut serializer = serializer.serialize_appendable_struct()?;
    //         serializer.serialize_field(&self.value, "value")
    //     }
    // }

    // #[test]
    // fn serialize_appendable_struct() {
    //     let v = AppendableType { value: 7 };
    //     // PLAIN_CDR:
    //     assert_eq!(serialize_v1_be(&v), vec![0, 7]);
    //     assert_eq!(serialize_v1_le(&v), vec![7, 0]);
    //     // DELIMITED_CDR:
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             0, 0, 0, 2, // DHEADER
    //             0, 7 // value
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![
    //             2, 0, 0, 0, // DHEADER
    //             7, 0 // value
    //         ]
    //     );
    // }

    // //@extensibility(MUTABLE)
    // struct MutableType {
    //     // @id(0x005A) @key
    //     key: u8,
    //     // @id(0x0050)
    //     participant_key: u16,
    // }
    // impl XTypesSerialize for MutableType {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut s = serializer.serialize_mutable_struct()?;
    //         s.serialize_field(&self.key, 0x005A, "key")?;
    //         s.serialize_field(&self.participant_key, 0x0050, "participant_key")?;
    //         s.end()
    //     }
    // }

    // #[test]
    // fn serialize_mutable_struct() {
    //     let v = MutableType {
    //         key: 7,
    //         participant_key: 8,
    //     };
    //     // PL_CDR:
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![
    //             0x00, 0x05A, 0, 1, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x00, 0x050, 0, 2, // PID | length
    //             0, 8, 0, 0, // participant_key | padding (2 bytes)
    //             0, 1, 0, 0, // Sentinel
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![
    //             0x05A, 0x00, 1, 0, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x050, 0x00, 2, 0, // PID | length
    //             8, 0, 0, 0, // participant_key | padding (2 bytes)
    //             1, 0, 0, 0, // Sentinel
    //         ]
    //     );
    //     // PL_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             0x00, 0x05A, 0, 1, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x00, 0x050, 0, 2, // PID | length
    //             0, 8, 0, 0, // participant_key | padding (2 bytes)
    //             0, 1, 0, 0, // Sentinel
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![
    //             0x05A, 0x00, 1, 0, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x050, 0x00, 2, 0, // PID | length
    //             8, 0, 0, 0, // participant_key | padding (2 bytes)
    //             1, 0, 0, 0, // Sentinel
    //         ]
    //     );
    // }

    // //@extensibility(FINAL)
    // struct TinyFinalType {
    //     primitive: u16,
    // }
    // impl XTypesSerialize for TinyFinalType {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut s = serializer.serialize_final_struct()?;
    //         s.serialize_field(&self.primitive, "primitive")
    //     }
    // }
    // //@extensibility(MUTABLE)
    // struct NestedMutableType {
    //     // @id(0x0060) @key
    //     field_primitive: u8,
    //     // @id(0x0061)
    //     field_mutable: MutableType,
    //     // @id(0x0062)
    //     field_final: TinyFinalType,
    // }
    // impl XTypesSerialize for NestedMutableType {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut s = serializer.serialize_mutable_struct()?;
    //         s.serialize_field(&self.field_primitive, 0x0060, "field_primitive")?;
    //         s.serialize_field(&self.field_mutable, 0x0061, "field_mutable")?;
    //         s.serialize_field(&self.field_final, 0x0062, "field_final")?;
    //         s.end()
    //     }
    // }

    // #[test]
    // fn serialize_nested_mutable_struct() {
    //     let v = NestedMutableType {
    //         field_primitive: 5,
    //         field_mutable: MutableType {
    //             key: 7,
    //             participant_key: 8,
    //         },
    //         field_final: TinyFinalType { primitive: 9 },
    //     };
    //     // PL_CDR:
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![
    //             0x00, 0x060, 0, 1, // PID | length
    //             5, 0, 0, 0, // field_primitive | padding (3 bytes)
    //             0x00, 0x061, 0, 20, // PID | length
    //             0x00, 0x05A, 0, 1, // field_mutable: PID | length
    //             7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
    //             0x00, 0x050, 0, 2, // field_mutable: PID | length
    //             0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
    //             0, 1, 0, 0, // field_mutable: Sentinel
    //             0x00, 0x062, 0, 2, // field_mutable: PID | length
    //             0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
    //             0, 1, 0, 0, // Sentinel
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![
    //             0x060, 0x00, 1, 0, // PID | length
    //             5, 0, 0, 0, // field_primitive | padding (3 bytes)
    //             0x061, 0x00, 20, 0, // PID | length
    //             0x05A, 0x00, 1, 0, // field_mutable: PID | length
    //             7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
    //             0x050, 0x00, 2, 0, // field_mutable: PID | length
    //             8, 0, 0, 0, // field_mutable: participant_key | padding (2 bytes)
    //             1, 0, 0, 0, // field_mutable: Sentinel
    //             0x062, 0x00, 2, 0, // field_mutable: PID | length
    //             9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
    //             1, 0, 0, 0, // Sentinel
    //         ]
    //     );
    //     // PL_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             0x00, 0x060, 0, 1, // PID | length
    //             5, 0, 0, 0, // field_primitive | padding (3 bytes)
    //             0x00, 0x061, 0, 20, // PID | length
    //             0x00, 0x05A, 0, 1, // field_mutable: PID | length
    //             7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
    //             0x00, 0x050, 0, 2, // field_mutable: PID | length
    //             0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
    //             0, 1, 0, 0, // field_mutable: Sentinel
    //             0x00, 0x062, 0, 2, // field_mutable: PID | length
    //             0, 9, 0, 0, // field_final: primitive | padding (2 bytes)
    //             0, 1, 0, 0, // Sentinel
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![
    //             0x060, 0x00, 1, 0, // PID | length
    //             5, 0, 0, 0, // field_primitive | padding (3 bytes)
    //             0x061, 0x00, 20, 0, // PID | length
    //             0x05A, 0x00, 1, 0, // field_mutable: PID | length
    //             7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
    //             0x050, 0x00, 2, 0, // field_mutable: PID | length
    //             8, 0, 0, 0, // field_mutable: participant_key | padding (2 bytes)
    //             1, 0, 0, 0, // field_mutable: Sentinel
    //             0x062, 0x00, 2, 0, // field_mutable: PID | length
    //             9, 0, 0, 0, // field_final: primitive | padding (2 bytes)
    //             1, 0, 0, 0, // Sentinel
    //         ]
    //     );
    // }

    // #[derive(Debug, PartialEq)]
    // struct BasicTypes {
    //     f1: bool,
    //     f2: i8,
    //     f3: i16,
    //     f4: i32,
    //     f5: i64,
    //     f6: u8,
    //     f7: u16,
    //     f8: u32,
    //     f9: u64,
    //     f10: f32,
    //     f11: f64,
    //     f12: char,
    // }

    // impl XTypesSerialize for BasicTypes {
    //     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
    //         let mut serializer = serializer.serialize_final_struct()?;
    //         serializer.serialize_field(&self.f1, "f1")?;
    //         serializer.serialize_field(&self.f2, "f2")?;
    //         serializer.serialize_field(&self.f3, "f3")?;
    //         serializer.serialize_field(&self.f4, "f4")?;
    //         serializer.serialize_field(&self.f5, "f5")?;
    //         serializer.serialize_field(&self.f6, "f6")?;
    //         serializer.serialize_field(&self.f7, "f7")?;
    //         serializer.serialize_field(&self.f8, "f8")?;
    //         serializer.serialize_field(&self.f9, "f9")?;
    //         serializer.serialize_field(&self.f10, "f10")?;
    //         serializer.serialize_field(&self.f11, "f11")?;
    //         serializer.serialize_field(&self.f12, "f12")
    //     }
    // }

    // #[test]
    // fn serialize_basic_types_struct() {
    //     let v = BasicTypes {
    //         f1: true,
    //         f2: 2,
    //         f3: 3,
    //         f4: 4,
    //         f5: 5,
    //         f6: 6,
    //         f7: 7,
    //         f8: 8,
    //         f9: 9,
    //         f10: 1.0,
    //         f11: 1.0,
    //         f12: 'a',
    //     };
    //     // PLAIN_CDR:
    //     assert_eq!(
    //         serialize_v1_be(&v),
    //         vec![
    //             1, 2, 0, 3, 0, 0, 0, 4, // f1: bool | f2: i8 | f3: i16 | f4: i32
    //             0, 0, 0, 0, 0, 0, 0, 5, // f5: i64
    //             6, 0, 0, 7, 0, 0, 0, 8, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
    //             0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
    //             0x3F, 0x80, 0x00, 0x00, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
    //             0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
    //             b'a', // f12: char
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v1_le(&v),
    //         vec![
    //             1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
    //             5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
    //             6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
    //             9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
    //             0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
    //             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
    //             b'a', // f12: char
    //         ]
    //     );
    //     //PLAIN_CDR2:
    //     assert_eq!(
    //         serialize_v2_be(&v),
    //         vec![
    //             1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
    //             0, 0, 0, 4, // f4: i32
    //             0, 0, 0, 0, // f5-1: i64
    //             0, 0, 0, 5, // f5-2: i64
    //             6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
    //             0, 0, 0, 8, // f8: u32
    //             0, 0, 0, 0, // f9-1: u64
    //             0, 0, 0, 9, // f9-2: u64
    //             0x3F, 0x80, 0x00, 0x00, // f10: f32
    //             0x3F, 0xF0, 0x00, 0x00, // f11-1: f64
    //             0x00, 0x00, 0x00, 0x00, // f11-2: f64
    //             b'a', // f12: char
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_v2_le(&v),
    //         vec![
    //             1, 2, 3, 0, // f1: bool | f2: i8 | f3: i16
    //             4, 0, 0, 0, // f4: i32
    //             5, 0, 0, 0, // f5-1: i64
    //             0, 0, 0, 0, // f5-2: i64
    //             6, 0, 7, 0, // f6: u8 | padding (1 byte) | f7: u16
    //             8, 0, 0, 0, // f8: u32
    //             9, 0, 0, 0, // f9-1: u64
    //             0, 0, 0, 0, // f9-2: u64
    //             0x00, 0x00, 0x80, 0x3F, // f10: f32
    //             0x00, 0x00, 0x00, 0x00, // f11-1: f64
    //             0x00, 0x00, 0xF0, 0x3F, // f11-2: f64
    //             b'a', // f12: char
    //         ]
    //     );
    // }
}
