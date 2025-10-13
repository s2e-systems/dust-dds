use crate::xtypes::{
    bytes::Bytes, data_representation::DataKind, dynamic_type::DynamicData, error::XTypesError, serialize::Write
};
use alloc::string::String;

pub struct Writer<'a, W> {
    buffer: &'a mut W,
    position: usize,
}

impl<'a, W: Write> Writer<'a, W> {
    pub fn new(buffer: &'a mut W) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn write_slice(&mut self, data: &[u8]) {
        self.buffer.write(data);
        self.position += data.len();
    }

    pub fn pad(&mut self, alignment: usize) {
        const ZEROS: [u8; 8] = [0; 8];
        let alignment = self.position.div_ceil(alignment) * alignment - self.position;
        self.write_slice(&ZEROS[..alignment]);
    }
}

pub struct WriterV1<'a, W> {
    pub writer: Writer<'a, W>,
}
impl<'a, W: Write> Write for WriterV1<'a, W> {
    fn write(&mut self, buf: &[u8]) {
        self.writer.pad(buf.len());
        self.writer.write_slice(buf);
    }
}
pub struct WriterV2<'a, W> {
    pub writer: Writer<'a, W>,
}

impl<'a, W: Write> Write for WriterV2<'a, W> {
    fn write(&mut self, buf: &[u8]) {
        self.writer.pad(core::cmp::min(buf.len(), 4));
        self.writer.write_slice(buf);
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

pub trait Endianness {}
pub struct BigEndian;
pub struct LittleEndian;

impl Endianness for LittleEndian {}
impl Endianness for BigEndian {}

pub trait TryWriteAsBytes<E> {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError>;
}
pub trait WriteAsBytes<E> {
    fn write_as_bytes<C: Write>(self, writer: &mut C);
}
impl<E: Endianness> TryWriteAsBytes<E> for char {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError> {
        writer.write(self.to_string().as_bytes());
        Ok(())
    }
}
impl<E: Endianness> WriteAsBytes<E> for &str {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(self.as_bytes());
        writer.write(&[0]);
    }
}

trait Length {
    fn length(&self) -> usize;
}
impl Length for &str {
    fn length(&self) -> usize {
        self.len()
    }
}
pub struct WithLength<T>(T);

impl<T: Length + WriteAsBytes<BigEndian>> TryWriteAsBytes<BigEndian> for WithLength<T> {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError> {
        WriteAsBytes::<BigEndian>::write_as_bytes(into_u32(self.0.length())?, writer);
        WriteAsBytes::<BigEndian>::write_as_bytes(self.0, writer);
        Ok(())
    }
}
impl<T: Length + WriteAsBytes<LittleEndian>> TryWriteAsBytes<LittleEndian> for WithLength<T> {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError> {
        WriteAsBytes::<LittleEndian>::write_as_bytes(into_u32(self.0.length())?, writer);
        WriteAsBytes::<LittleEndian>::write_as_bytes(self.0, writer);
        Ok(())
    }
}
impl TryWriteAsBytes<LittleEndian> for &str {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError> {
        TryWriteAsBytes::<LittleEndian>::try_write_as_bytes(WithLength(self), writer)
    }
}
impl TryWriteAsBytes<BigEndian> for &str {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError> {
        TryWriteAsBytes::<BigEndian>::try_write_as_bytes(WithLength(self), writer)
    }
}

impl<E: Endianness> WriteAsBytes<E> for bool {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&[self as u8])
    }
}

impl WriteAsBytes<LittleEndian> for i64 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u64 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u32 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for i32 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for i16 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u16 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for i8 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u8 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for f32 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for f64 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}


impl WriteAsBytes<BigEndian> for i64 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u64 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for i32 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u32 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for i16 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u16 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for i8 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u8 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for f32 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for f64 {
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}

impl<E, T> WriteAsBytes<E> for &[T]
where
    E: Endianness,
    T: WriteAsBytes<E> + Copy,
{
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        for v in self.iter() {
            v.write_as_bytes(writer);
        }
    }
}
impl<E:Endianness> TryWriteAsBytes<E> for &[char] {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError>{
        for &v in self.iter() {
            TryWriteAsBytes::<E>::try_write_as_bytes(v, writer)?;
        }
        Ok(())
    }
}

impl TryWriteAsBytes<LittleEndian> for &[String] {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError>{
        for v in self.iter().cloned() {
            TryWriteAsBytes::<LittleEndian>::try_write_as_bytes(v.as_str(), writer)?;
        }
        Ok(())
    }
}
impl TryWriteAsBytes<BigEndian> for &[String] {
    fn try_write_as_bytes<C: Write>(self, writer: &mut C) -> Result<(), XTypesError>{
        for v in self.iter().cloned() {
            TryWriteAsBytes::<BigEndian>::try_write_as_bytes(v.as_str(), writer)?;
        }
        Ok(())
    }
}

impl<'a, E> WriteAsBytes<E> for Bytes<'a>
where
    E: Endianness,
{
    fn write_as_bytes<C: Write>(self, writer: &mut C) {
        writer.write(self.0);
    }
}

#[test]
fn test_to_bytes_write() {
    let mut buffer = Vec::new();
    WriteAsBytes::<BigEndian>::write_as_bytes(true, &mut buffer);
    assert_eq!(buffer, [1]);
    let mut buffer = Vec::new();
    WriteAsBytes::<BigEndian>::write_as_bytes(7i32, &mut buffer);
    assert_eq!(buffer, [0, 0, 0, 7]);
    let mut buffer = Vec::new();
    WriteAsBytes::<LittleEndian>::write_as_bytes(7i32, &mut buffer);
    assert_eq!(buffer, [7, 0, 0, 0]);
    let mut buffer = Vec::new();
    WriteAsBytes::<LittleEndian>::write_as_bytes(7i16, &mut buffer);
    assert_eq!(buffer, [7, 0]);
}

pub trait ToBytes<const N: usize, E: Endianness> {
    fn to_bytes(self) -> [u8; N];
}
impl<E: Endianness> ToBytes<1, E> for bool {
    fn to_bytes(self) -> [u8; 1] {
        [self as u8]
    }
}
impl ToBytes<4, LittleEndian> for i32 {
    fn to_bytes(self) -> [u8; 4] {
        self.to_le_bytes()
    }
}
impl ToBytes<2, LittleEndian> for i16 {
    fn to_bytes(self) -> [u8; 2] {
        self.to_le_bytes()
    }
}
impl ToBytes<4, BigEndian> for i32 {
    fn to_bytes(self) -> [u8; 4] {
        self.to_be_bytes()
    }
}
impl ToBytes<2, BigEndian> for i16 {
    fn to_bytes(self) -> [u8; 2] {
        self.to_be_bytes()
    }
}

pub trait SerializeFinalStruct {
    fn serialize_field(&mut self, value: &DataKind, name: &str) -> Result<(), XTypesError>;
    fn serialize_optional_field(
        &mut self,
        value: &Option<DynamicData>,
        name: &str,
    ) -> Result<(), XTypesError>;
}
pub trait SerializeAppendableStruct {
    fn serialize_field(&mut self, value: &DataKind, name: &str) -> Result<(), XTypesError>;
}
pub trait SerializeMutableStruct {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        name: &str,
    ) -> Result<(), XTypesError>;

    fn end(self) -> Result<(), XTypesError>;
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer: Sized {
    type Endianness: Endianness;

    /// Start serializing a type with final extensibility.
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError>;

    /// Start serializing a type with appendable extensibility.
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError>;

    /// Start serializing a type with mutable extensibility.
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError>;

    /// Serializing a sequence with a dynamic length
    fn serialize_complex_value(self, v: &DynamicData) -> Result<(), XTypesError>;

    /// Serializing a sequence with a dynamic length
    fn serialize_complex_value_list(self, v: &[DynamicData]) -> Result<(), XTypesError>;

    /// Serializing a sequence with a static length
    fn serialize_complex_value_array(self, v: &[DynamicData]) -> Result<(), XTypesError>;

    /// Serialize a [`bool`] value.
    fn serialize_boolean(self, v: bool);

    /// Serialize an [`i8`] value.
    fn serialize_int8(self, v: i8);

    /// Serialize an [`i16`] value.
    fn serialize_int16(self, v: i16);

    /// Serialize an [`i32`] value.
    fn serialize_int32(self, v: i32);

    /// Serialize an [`i64`] value.
    fn serialize_int64(self, v: i64);

    /// Serialize a [`u8`] value.
    fn serialize_uint8(self, v: u8);

    /// Serialize a [`u16`] value.
    fn serialize_uint16(self, v: u16);

    /// Serialize a [`u32`] value.
    fn serialize_uint32(self, v: u32);

    /// Serialize a [`u64`] value.
    fn serialize_uint64(self, v: u64);

    /// Serialize an [`f32`] value.
    fn serialize_float32(self, v: f32);

    /// Serialize an [`f64`] value.
    fn serialize_float64(self, v: f64);

    /// Serialize a [`char`] value.
    fn serialize_char8(self, v: char) -> Result<(), XTypesError>;

    /// Serialize a [`str`] value.
    fn serialize_string(self, v: &str) -> Result<(), XTypesError>;

    /// Serialize a list of [`bool`] values.
    fn serialize_boolean_list(self, v: &[bool]);

    /// Serialize a list of [`u8`] values.
    fn serialize_uint8_list(self, v: Bytes);

    /// Serialize a list of [`i8`] values.
    fn serialize_int8_list(self, v: &[i8]);

    /// Serialize a list of [`i16`] values.
    fn serialize_int16_list(self, v: &[i16]);

    /// Serialize a list of [`i32`] values.
    fn serialize_int32_list(self, v: &[i32]);

    /// Serialize a list of [`i64`] values.
    fn serialize_int64_list(self, v: &[i64]);

    /// Serialize a list of [`u16`] values.
    fn serialize_uint16_list(self, v: &[u16]);

    /// Serialize a list of [`u32`] values.
    fn serialize_uint32_list(self, v: &[u32]);

    /// Serialize a list of [`u64`] values.
    fn serialize_uint64_list(self, v: &[u64]);

    /// Serialize a list of [`f32`] values.
    fn serialize_float32_list(self, v: &[f32]);

    /// Serialize a list of [`f64`] values.
    fn serialize_float64_list(self, v: &[f64]);

    /// Serialize a list of [`char`] values.
    fn serialize_char8_list(self, v: &[char]) -> Result<(), XTypesError>;

    /// Serialize a list of [`String`] values.
    fn serialize_string_list(self, v: &[String]) -> Result<(), XTypesError>;

    /// Serialize a list of [`bool`] values.
    fn serialize_boolean_array(self, v: &[bool]);

    /// Serialize an array of [`u8`] values.
    fn serialize_uint8_array(self, v: Bytes);

    /// Serialize an array of [`i8`] values.
    fn serialize_int8_array(self, v: &[i8]);

    /// Serialize an array of [`i16`] values.
    fn serialize_int16_array(self, v: &[i16]);

    /// Serialize an array of [`i32`] values.
    fn serialize_int32_array(self, v: &[i32]);

    /// Serialize an array of [`i64`] values.
    fn serialize_int64_array(self, v: &[i64]);

    /// Serialize an array of [`u16`] values.
    fn serialize_uint16_array(self, v: &[u16]);

    /// Serialize an array of [`u32`] values.
    fn serialize_uint32_array(self, v: &[u32]);

    /// Serialize an array of [`u64`] values.
    fn serialize_uint64_array(self, v: &[u64]);

    /// Serialize an array of [`f32`] values.
    fn serialize_float32_array(self, v: &[f32]);

    /// Serialize an array of [`f64`] values.
    fn serialize_float64_array(self, v: &[f64]);

    /// Serialize an array of [`char`] values.
    fn serialize_char8_array(self, v: &[char]) -> Result<(), XTypesError>;

    /// Serialize an array of [`String`] values.
    fn serialize_string_array(self, v: &[String]) -> Result<(), XTypesError>;
}
