use crate::xtypes::{
    bytes::Bytes,
    data_representation::DataKind,
    dynamic_type::{DynamicData, MemberDescriptor, TypeKind},
    error::XTypesError,
    serialize::Write,
};
use alloc::{string::ToString, vec::Vec};
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

pub trait Endianness {
    fn write_i8<C: Write>(v: i8, writer: &mut C) {
        writer.write(&[v as u8]);
    }
    fn write_u8<C: Write>(v: u8, writer: &mut C) {
        writer.write(&[v]);
    }
    fn write_i16<C: Write>(v: i16, writer: &mut C);
    fn write_u16<C: Write>(v: u16, writer: &mut C);
    fn write_i32<C: Write>(v: i32, writer: &mut C);
    fn write_u32<C: Write>(v: u32, writer: &mut C);
    fn write_i64<C: Write>(v: i64, writer: &mut C);
    fn write_u64<C: Write>(v: u64, writer: &mut C);
    fn write_str<C: Write>(v: &str, writer: &mut C) {
        writer.write(&v.as_bytes());
        writer.write(&[0])
    }
}
pub struct BigEndian;
pub struct LittleEndian;

impl Endianness for LittleEndian {
    fn write_i16<C: Write>(v: i16, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_u16<C: Write>(v: u16, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_i32<C: Write>(v: i32, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_u32<C: Write>(v: u32, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_i64<C: Write>(v: i64, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_u64<C: Write>(v: u64, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
}
impl Endianness for BigEndian {
    fn write_i16<C: Write>(v: i16, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_u16<C: Write>(v: u16, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_i32<C: Write>(v: i32, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_u32<C: Write>(v: u32, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_i64<C: Write>(v: i64, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_u64<C: Write>(v: u64, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
}

pub trait WriteAsBytes<E> {
    fn write_as_bytes<C: Write>(&self, writer: &mut C);
}
impl<E: Endianness> WriteAsBytes<E> for char {
    fn write_as_bytes<C: Write>(&self, writer: &mut C) {
        writer.write(self.to_string().as_bytes());
    }
}
impl<E: Endianness> WriteAsBytes<E> for String {
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
    fn serialize_members(&mut self, v: &DynamicData) -> Result<(), XTypesError>;
}
pub trait SerializeAppendableStruct {
    fn serialize_members(&mut self, v: &DynamicData) -> Result<(), XTypesError>;
}
pub trait SerializeMutableStruct {
    fn serialize_members(&mut self, value: &DynamicData) -> Result<(), XTypesError>;
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

    fn serialize_sequence(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        // self.serialize_u32(v.len() as u32);
        // for vi in v {
        //     self.serialize_complex(vi)?;
        // }
        todo!();
        Ok(())
    }

    fn serialize_complex(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let member_descriptor = v.get_descriptor(member_id)?;

            match member_descriptor.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => todo!(),
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => self.serialize_i16(*v.get_int16_value(member_id)?),
                TypeKind::INT32 => self.serialize_i32(*v.get_int32_value(member_id)?),
                TypeKind::INT64 => todo!(),
                TypeKind::UINT16 => self.serialize_u16(*v.get_uint16_value(member_id)?),
                TypeKind::UINT32 => todo!(),
                TypeKind::UINT64 => self.serialize_u64(*v.get_uint64_value(member_id)?),
                TypeKind::FLOAT32 => todo!(),
                TypeKind::FLOAT64 => todo!(),
                TypeKind::FLOAT128 => todo!(),
                TypeKind::INT8 => self.serialize_i8(*v.get_int8_value(member_id)?),
                TypeKind::UINT8 => self.serialize_u8(*v.get_uint8_value(member_id)?),
                TypeKind::CHAR8 => todo!(),
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
        }
        Ok(())
    }
    fn serialize_string(&mut self, v: &str) {
        self.serialize_u32(v.len() as u32 + 1);
        Self::Endianness::write_str(v, self.writer());
    }
    fn serialize_u8(&mut self, v: u8) {
        Self::Endianness::write_u8(v, self.writer());
    }
    fn serialize_i8(&mut self, v: i8) {
        Self::Endianness::write_i8(v, self.writer());
    }
    fn serialize_u16(&mut self, v: u16) {
        Self::Endianness::write_u16(v, self.writer());
    }
    fn serialize_i16(&mut self, v: i16) {
        Self::Endianness::write_i16(v, self.writer());
    }
    fn serialize_u32(&mut self, v: u32) {
        Self::Endianness::write_u32(v, self.writer());
    }
    fn serialize_i32(&mut self, v: i32) {
        Self::Endianness::write_i32(v, self.writer());
    }
    fn serialize_u64(&mut self, v: u64) {
        Self::Endianness::write_u64(v, self.writer());
    }

    fn writer(&mut self) -> &mut impl Write;
}
