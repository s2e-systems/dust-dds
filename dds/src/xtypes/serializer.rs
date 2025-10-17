use super::dynamic_type::ExtensibilityKind;
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    error::XTypesError,
};
use alloc::{string::ToString, vec::Vec};

/// A trait to Write bytes into a potentially growing buffer
pub trait Write {
    fn write(&mut self, buf: &[u8]);
    // fn pos(&self) -> usize;
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf)
    }
}

pub struct Writer<'a, W> {
    buffer: &'a mut W,
    pub position: usize,
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
        self.writer.write_slice(buf);
    }
}
pub struct WriterV2<'a, W> {
    pub writer: Writer<'a, W>,
}

impl<'a, W: Write> Write for WriterV2<'a, W> {
    fn write(&mut self, buf: &[u8]) {
        self.writer.write_slice(buf);
    }
}

pub trait PaddedWrite: Write {
    fn padded_write(&mut self, buf: &[u8]);
}

impl<'a, W: Write> PaddedWrite for WriterV2<'a, W> {
    fn padded_write(&mut self, buf: &[u8]) {
        self.writer.write_slice(buf);
        self.writer.pad(core::cmp::min(buf.len(), 4));
    }
}

impl<'a, W: Write> PaddedWrite for WriterV1<'a, W> {
    fn padded_write(&mut self, buf: &[u8]) {
        self.writer.write_slice(buf);
        self.writer.pad(buf.len());
    }
}

pub trait Endianness {
    fn write_bool<C: PaddedWrite>(v: bool, writer: &mut C) {
        writer.write(&[v as u8]);
    }
    fn write_i8<C: PaddedWrite>(v: i8, writer: &mut C) {
        writer.write(&[v as u8]);
    }
    fn write_u8<C: PaddedWrite>(v: u8, writer: &mut C) {
        writer.write(&[v]);
    }
    fn write_i16<C: PaddedWrite>(v: i16, writer: &mut C);
    fn write_u16<C: PaddedWrite>(v: u16, writer: &mut C);
    fn write_i32<C: PaddedWrite>(v: i32, writer: &mut C);
    fn write_u32<C: PaddedWrite>(v: u32, writer: &mut C);
    fn write_i64<C: PaddedWrite>(v: i64, writer: &mut C);
    fn write_u64<C: PaddedWrite>(v: u64, writer: &mut C);
    fn write_f32<C: PaddedWrite>(v: f32, writer: &mut C);
    fn write_f64<C: PaddedWrite>(v: f64, writer: &mut C);
    fn write_char<C: PaddedWrite>(v: char, writer: &mut C) {
        writer.write(v.to_string().as_bytes());
    }
    fn write_str<C: PaddedWrite>(v: &str, writer: &mut C) {
        writer.write(&v.as_bytes());
        writer.write(&[0])
    }
    fn write_slice_u8<C: PaddedWrite>(v: &[u8], writer: &mut C) {
        writer.write(v);
    }
}
pub struct BigEndian;
pub struct LittleEndian;

impl Endianness for LittleEndian {
    fn write_i16<C: PaddedWrite>(v: i16, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_u16<C: PaddedWrite>(v: u16, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_i32<C: PaddedWrite>(v: i32, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_u32<C: PaddedWrite>(v: u32, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_i64<C: PaddedWrite>(v: i64, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_u64<C: PaddedWrite>(v: u64, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_f32<C: PaddedWrite>(v: f32, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
    fn write_f64<C: PaddedWrite>(v: f64, writer: &mut C) {
        writer.padded_write(&v.to_le_bytes())
    }
}
impl Endianness for BigEndian {
    fn write_i16<C: PaddedWrite>(v: i16, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_u16<C: PaddedWrite>(v: u16, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_i32<C: PaddedWrite>(v: i32, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_u32<C: PaddedWrite>(v: u32, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_i64<C: PaddedWrite>(v: i64, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_u64<C: PaddedWrite>(v: u64, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_f32<C: PaddedWrite>(v: f32, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
    fn write_f64<C: PaddedWrite>(v: f64, writer: &mut C) {
        writer.padded_write(&v.to_be_bytes())
    }
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer {
    type Endianness: Endianness;

    /// Start serializing a type with final extensibility.
    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    /// Start serializing a type with appendable extensibility.
    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    /// Start serializing a type with mutable extensibility.
    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError>;

    fn serialize_sequence(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        // self.serialize_u32(v.len() as u32);
        // for vi in v {
        //     self.serialize_complex(vi)?;
        // }
        // Ok(())
        todo!();
    }
    fn serialize_dynamic_data_member(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => self.serialize_bool(*v.get_boolean_value(member_id)?),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => self.serialize_i16(*v.get_int16_value(member_id)?),
            TypeKind::INT32 => self.serialize_i32(*v.get_int32_value(member_id)?),
            TypeKind::INT64 => self.serialize_i64(*v.get_int64_value(member_id)?),
            TypeKind::UINT16 => self.serialize_u16(*v.get_uint16_value(member_id)?),
            TypeKind::UINT32 => self.serialize_u32(*v.get_uint32_value(member_id)?),
            TypeKind::UINT64 => self.serialize_u64(*v.get_uint64_value(member_id)?),
            TypeKind::FLOAT32 => self.serialize_f32(*v.get_float32_value(member_id)?),
            TypeKind::FLOAT64 => self.serialize_f64(*v.get_float64_value(member_id)?),
            TypeKind::FLOAT128 => unimplemented!("not supported by Rust"),
            TypeKind::INT8 => self.serialize_i8(*v.get_int8_value(member_id)?),
            TypeKind::UINT8 => self.serialize_u8(*v.get_uint8_value(member_id)?),
            TypeKind::CHAR8 => self.serialize_char(*v.get_char8_value(member_id)?),
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
        Ok(())
    }

    fn serialize_complex(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        match v.type_ref().get_descriptor().extensibility_kind {
            ExtensibilityKind::Final => self.serialize_final_struct(v),
            ExtensibilityKind::Appendable => self.serialize_appendable_struct(v),
            ExtensibilityKind::Mutable => self.serialize_mutable_struct(v),
        }
    }
    fn serialize_string(&mut self, v: &str) {
        self.serialize_u32(v.len() as u32 + 1);
        Self::Endianness::write_str(v, self.writer());
    }
    fn serialize_char(&mut self, v: char) {
        Self::Endianness::write_char(v, self.writer());
    }
    fn serialize_bool(&mut self, v: bool) {
        Self::Endianness::write_bool(v, self.writer());
    }
    fn serialize_i8(&mut self, v: i8) {
        Self::Endianness::write_i8(v, self.writer());
    }
    fn serialize_u8(&mut self, v: u8) {
        Self::Endianness::write_u8(v, self.writer());
    }
    fn serialize_i16(&mut self, v: i16) {
        Self::Endianness::write_i16(v, self.writer());
    }
    fn serialize_u16(&mut self, v: u16) {
        Self::Endianness::write_u16(v, self.writer());
    }
    fn serialize_i32(&mut self, v: i32) {
        Self::Endianness::write_i32(v, self.writer());
    }
    fn serialize_u32(&mut self, v: u32) {
        Self::Endianness::write_u32(v, self.writer());
    }
    fn serialize_i64(&mut self, v: i64) {
        Self::Endianness::write_i64(v, self.writer());
    }
    fn serialize_u64(&mut self, v: u64) {
        Self::Endianness::write_u64(v, self.writer());
    }
    fn serialize_f32(&mut self, v: f32) {
        Self::Endianness::write_f32(v, self.writer());
    }
    fn serialize_f64(&mut self, v: f64) {
        Self::Endianness::write_f64(v, self.writer());
    }
    fn writer(&mut self) -> &mut impl PaddedWrite;
}
