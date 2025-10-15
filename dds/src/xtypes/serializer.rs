use crate::xtypes::{
    bytes::Bytes, data_representation::DataKind, error::XTypesError, serialize::Write,
};
use alloc::{vec::Vec, string::ToString};
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
impl<'a, W: Write> Write for Writer<'a, W> {
    fn write(&mut self, buf: &[u8]) {
        self.write_slice(buf);
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

pub trait Endianness {}
pub struct BigEndian;
pub struct LittleEndian;

impl Endianness for LittleEndian {}
impl Endianness for BigEndian {}

pub trait WriteAsBytes<E> {
    fn write_as_bytes<C: Write>(&self, writer: &mut C);
}
impl<E: Endianness> WriteAsBytes<E> for char {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(self.to_string().as_bytes());
    }
}
impl<E: Endianness> WriteAsBytes<E> for str {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(self.as_bytes());
        writer.write(&[0]);
    }
}

impl<E: Endianness> WriteAsBytes<E> for bool {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&[*self as u8])
    }
}

impl WriteAsBytes<LittleEndian> for i64 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u64 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u32 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for i32 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for i16 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u16 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for i8 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for u8 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for f32 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}
impl WriteAsBytes<LittleEndian> for f64 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_le_bytes())
    }
}

impl WriteAsBytes<BigEndian> for i64 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u64 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for i32 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u32 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for i16 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u16 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for i8 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for u8 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for f32 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}
impl WriteAsBytes<BigEndian> for f64 {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(&self.to_be_bytes())
    }
}

impl<E, T> WriteAsBytes<E> for &[T]
where
    E: Endianness,
    T: WriteAsBytes<E> + Copy,
{
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        for v in self.iter() {
            v.write_as_bytes(writer);
        }
    }
}
impl<E, T> WriteAsBytes<E> for Vec<T>
where
    E: Endianness,
    T: WriteAsBytes<E> + Copy,
{
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        for v in self.iter() {
            v.write_as_bytes(writer);
        }
    }
}

impl<'a, E> WriteAsBytes<E> for Bytes<'a>
where
    E: Endianness,
{
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(self.0);
    }
}

pub trait SerializeFinalStruct {
    fn serialize_field(&mut self, value: &DataKind, name: &str) -> Result<(), XTypesError>;
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
pub trait XTypesSerializer {
    type Endianness: Endianness;

    /// Start serializing a type with final extensibility.
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError>;

    /// Start serializing a type with appendable extensibility.
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError>;

    /// Start serializing a type with mutable extensibility.
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError>;

    /// Serializing data
    fn serialize_data_kind(self, v: &DataKind) -> Result<(), XTypesError>;
}
