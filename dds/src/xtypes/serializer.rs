use super::dynamic_type::ExtensibilityKind;
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    error::XTypesError,
};
use alloc::{string::ToString, vec::Vec};

/// A trait to Write bytes into a potentially growing buffer
pub trait Write {
    fn write(&mut self, buf: &[u8]);
    fn pos(&self) -> usize {
        0
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf)
    }
    fn pos(&self) -> usize {
        self.len()
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

pub trait Endianness {
    fn write_bool<C: Write>(v: bool, writer: &mut C) {
        writer.write(&[v as u8]);
    }
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
    fn write_f32<C: Write>(v: f32, writer: &mut C);
    fn write_f64<C: Write>(v: f64, writer: &mut C);
    fn write_char<C: Write>(v: char, writer: &mut C) {
        writer.write(v.to_string().as_bytes());
    }
    fn write_str<C: Write>(v: &str, writer: &mut C) {
        writer.write(v.as_bytes());
    }
    fn write_slice_u8<C: Write>(v: &[u8], writer: &mut C) {
        writer.write(v);
    }
}
pub struct BigEndian;
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
    fn write_f32<C: Write>(v: f32, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
    fn write_f64<C: Write>(v: f64, writer: &mut C) {
        writer.write(&v.to_be_bytes())
    }
}

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
    fn write_f32<C: Write>(v: f32, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
    fn write_f64<C: Write>(v: f64, writer: &mut C) {
        writer.write(&v.to_le_bytes())
    }
}

fn pad_writer(alignment: usize, writer: &mut impl Write) {
    const ZEROS: [u8; 8] = [0; 8];
    let padding = writer.pos().div_ceil(alignment) * alignment - writer.pos();
    writer.write(&ZEROS[..padding]);
}
pub trait Padding {
    fn pad(alignment: usize, writer: &mut impl Write);
}
pub struct PaddingV1;
impl Padding for PaddingV1 {
    fn pad(alignment: usize, writer: &mut impl Write) {
        pad_writer(alignment, writer)
    }
}
pub struct PaddingV2;
impl Padding for PaddingV2 {
    fn pad(alignment: usize, writer: &mut impl Write) {
        pad_writer(core::cmp::min(alignment, 4), writer)
    }
}

pub trait PadEndiannessWrite {
    type Endianness: Endianness;
    type Padding: Padding;
    fn writer(&mut self) -> &mut impl Write;
    fn pad(&mut self, alignment: usize) {
        Self::Padding::pad(alignment, self.writer());
    }
    fn write_bool(&mut self, v: bool) {
        Self::Endianness::write_bool(v, self.writer())
    }
    fn write_i8(&mut self, v: i8) {
        Self::Endianness::write_i8(v, self.writer())
    }
    fn write_u8(&mut self, v: u8) {
        Self::Endianness::write_u8(v, self.writer())
    }
    fn write_i16(&mut self, v: i16) {
        Self::Padding::pad(2, self.writer());
        Self::Endianness::write_i16(v, self.writer())
    }
    fn write_u16(&mut self, v: u16) {
        Self::Padding::pad(2, self.writer());
        Self::Endianness::write_u16(v, self.writer())
    }
    fn write_i32(&mut self, v: i32) {
        Self::Padding::pad(4, self.writer());
        Self::Endianness::write_i32(v, self.writer())
    }
    fn write_u32(&mut self, v: u32) {
        Self::Padding::pad(4, self.writer());
        Self::Endianness::write_u32(v, self.writer())
    }
    fn write_i64(&mut self, v: i64) {
        Self::Padding::pad(8, self.writer());
        Self::Endianness::write_i64(v, self.writer())
    }
    fn write_u64(&mut self, v: u64) {
        Self::Padding::pad(8, self.writer());
        Self::Endianness::write_u64(v, self.writer())
    }
    fn write_f32(&mut self, v: f32) {
        Self::Padding::pad(4, self.writer());
        Self::Endianness::write_f32(v, self.writer())
    }
    fn write_f64(&mut self, v: f64) {
        Self::Padding::pad(8, self.writer());
        Self::Endianness::write_f64(v, self.writer())
    }
    fn write_char(&mut self, v: char) {
        Self::Endianness::write_char(v, self.writer())
    }
    fn write_str(&mut self, v: &str) {
        Self::Endianness::write_str(v, self.writer())
    }
    fn write_slice_u8(&mut self, v: &[u8]) {
        Self::Endianness::write_slice_u8(v, self.writer())
    }
}

pub struct WriterBe1<W>(pub W);
impl<W: Write> PadEndiannessWrite for WriterBe1<W> {
    type Endianness = BigEndian;
    type Padding = PaddingV1;
    fn writer(&mut self) -> &mut impl Write {
        &mut self.0
    }
}
pub struct WriterLe1<W>(pub W);
impl<W: Write> PadEndiannessWrite for WriterLe1<W> {
    type Endianness = LittleEndian;
    type Padding = PaddingV1;
    fn writer(&mut self) -> &mut impl Write {
        &mut self.0
    }
}
pub struct WriterBe2<W>(pub W);
impl<W: Write> PadEndiannessWrite for WriterBe2<W> {
    type Endianness = BigEndian;
    type Padding = PaddingV2;
    fn writer(&mut self) -> &mut impl Write {
        &mut self.0
    }
}
pub struct WriterLe2<W>(pub W);
impl<W: Write> PadEndiannessWrite for WriterLe2<W> {
    type Endianness = LittleEndian;
    type Padding = PaddingV2;
    fn writer(&mut self) -> &mut impl Write {
        &mut self.0
    }
}

pub trait WriteBasicType: Copy {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite);
}
impl WriteBasicType for bool {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_bool(self);
    }
}
impl WriteBasicType for i8 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_i8(self);
    }
}
impl WriteBasicType for u8 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_u8(self);
    }
}
impl WriteBasicType for i16 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_i16(self);
    }
}
impl WriteBasicType for u16 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_u16(self);
    }
}
impl WriteBasicType for i32 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_i32(self);
    }
}
impl WriteBasicType for u32 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_u32(self);
    }
}
impl WriteBasicType for i64 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_i64(self);
    }
}
impl WriteBasicType for u64 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_u64(self);
    }
}
impl WriteBasicType for f32 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_f32(self);
    }
}
impl WriteBasicType for f64 {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_f64(self);
    }
}
impl WriteBasicType for char {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_char(self);
    }
}
impl WriteBasicType for &[u8] {
    fn write_basic_type(self, writer: &mut impl PadEndiannessWrite) {
        writer.write_slice_u8(self);
    }
}

fn serialize_sequence(v: &[impl WriteBasicType], writer: &mut impl PadEndiannessWrite) {
    writer.write_u32(v.len() as u32);
    for vi in v.iter() {
        vi.write_basic_type(writer);
    }
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer<C> {
    fn into_inner(self) -> C;

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

    fn serialize_sequence(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        let element_type = member_descriptor
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("sequence has element type");
        match element_type.get_descriptor().kind {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => todo!(),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => todo!(),
            TypeKind::INT32 => todo!(),
            TypeKind::INT64 => todo!(),
            TypeKind::UINT16 => serialize_sequence(&v.get_uint16_values(member_id)?, self.writer()),
            TypeKind::UINT32 => todo!(),
            TypeKind::UINT64 => todo!(),
            TypeKind::FLOAT32 => todo!(),
            TypeKind::FLOAT64 => todo!(),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => todo!(),
            TypeKind::UINT8 => serialize_sequence(&v.get_uint8_values(member_id)?, self.writer()),
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                let list = v.get_string_values(member_id)?;
                self.writer().write_u32(list.len() as u32);
                for v in list {
                    self.serialize_string(v.as_str());
                }
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                let list = v.get_complex_values(member_id)?;
                self.writer().write_u32(list.len() as u32);
                for v in &list {
                    self.serialize_complex(v)?;
                }
            }
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }

        Ok(())
    }

    fn serialize_array(&mut self, v: &DynamicData, member_id: u32) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        let element_type = member_descriptor
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("array has element type");
        match element_type.get_descriptor().kind {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => todo!(),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => todo!(),
            TypeKind::INT32 => todo!(),
            TypeKind::INT64 => todo!(),
            TypeKind::UINT16 => todo!(),
            TypeKind::UINT32 => todo!(),
            TypeKind::UINT64 => todo!(),
            TypeKind::FLOAT32 => todo!(),
            TypeKind::FLOAT64 => todo!(),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => todo!(),
            TypeKind::UINT8 => {
                for v in v.get_uint8_values(member_id)? {
                    self.writer().write_u8(v);
                }
            }
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                for v in v.get_string_values(member_id)? {
                    self.serialize_string(v.as_str());
                }
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                for v in &v.get_complex_values(member_id)? {
                    self.serialize_complex(v)?;
                }
            }
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }

        Ok(())
    }

    fn serialize_enum(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        let _discriminator_type = v.type_ref().get_descriptor().discriminator_type.as_ref();
        self.writer().write_i32(*v.get_int32_value(0)?);
        Ok(())
    }

    fn serialize_dynamic_data_member(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => v
                .get_boolean_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => v
                .get_int16_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::INT32 => v
                .get_int32_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::INT64 => v
                .get_int64_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::UINT16 => v
                .get_uint16_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::UINT32 => v
                .get_uint32_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::UINT64 => v
                .get_uint64_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::FLOAT32 => v
                .get_float32_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::FLOAT64 => v
                .get_float64_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::FLOAT128 => unimplemented!("not supported by Rust"),
            TypeKind::INT8 => v.get_int8_value(member_id)?.write_basic_type(self.writer()),
            TypeKind::UINT8 => v
                .get_uint8_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::CHAR8 => v
                .get_char8_value(member_id)?
                .write_basic_type(self.writer()),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => self.serialize_string(v.get_string_value(member_id)?),
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => self.serialize_enum(v.get_complex_value(member_id)?)?,
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => self.serialize_complex(v.get_complex_value(member_id)?)?,
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => self.serialize_sequence(v, member_id)?,
            TypeKind::ARRAY => self.serialize_array(v, member_id)?,
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
        self.writer().write_u32(v.len() as u32 + 1);
        self.writer().write_str(v);
        self.writer().write_u8(0);
    }

    fn serialize_writable(&mut self, v: impl WriteBasicType) {
        v.write_basic_type(self.writer());
    }

    fn writer(&mut self) -> &mut impl PadEndiannessWrite;
}
