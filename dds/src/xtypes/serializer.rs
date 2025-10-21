use super::dynamic_type::ExtensibilityKind;
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    error::XTypesError,
};
use alloc::{string::{String, ToString}, vec::Vec};

/// A trait to Write bytes into a potentially growing buffer
pub trait Write {
    fn write(&mut self, buf: &[u8]);
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf)
    }
}

pub trait EncodingVersion {
    const MAX_ALIGN: usize;
}
pub struct EncodingVersion1;
impl EncodingVersion for EncodingVersion1 {
    const MAX_ALIGN: usize = 8;
}

pub struct EncodingVersion2;
impl EncodingVersion for EncodingVersion2 {
    const MAX_ALIGN: usize = 4;
}

pub struct Writer<W, E, V> {
    pub buffer: W,
    position: usize,
    _endianness: E,
    _encoding_version: V,
}

impl<W: Write, E, V> Writer<W, E, V> {
    pub fn new(buffer: W, endianness: E, encoding_version: V) -> Self {
        Self {
            buffer,
            position: 0,
            _endianness: endianness,
            _encoding_version: encoding_version,
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

impl<W: Write, E, V> Write for Writer<W, E, V> {
    fn write(&mut self, buf: &[u8]) {
        self.write_slice(buf);
    }
}

pub trait PlainCdrSerialize {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    );
}
impl PlainCdrSerialize for bool {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        E::write_bool(self, writer);
    }
}
impl PlainCdrSerialize for i8 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        E::write_i8(self, writer);
    }
}
impl PlainCdrSerialize for u8 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        E::write_u8(self, writer);
    }
}
impl PlainCdrSerialize for i16 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(i16::BITS as usize / 8, V::MAX_ALIGN));
        E::write_i16(self, writer);
    }
}
impl PlainCdrSerialize for u16 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(u16::BITS as usize / 8, V::MAX_ALIGN));
        E::write_u16(self, writer);
    }
}
impl PlainCdrSerialize for i32 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(i32::BITS as usize / 8, V::MAX_ALIGN));
        E::write_i32(self, writer);
    }
}
impl PlainCdrSerialize for u32 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(u32::BITS as usize / 8, V::MAX_ALIGN));
        E::write_u32(self, writer);
    }
}
impl PlainCdrSerialize for i64 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(i64::BITS as usize / 8, V::MAX_ALIGN));
        E::write_i64(self, writer);
    }
}
impl PlainCdrSerialize for u64 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(u64::BITS as usize / 8, V::MAX_ALIGN));
        E::write_u64(self, writer);
    }
}
impl PlainCdrSerialize for f32 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(32 / 8, V::MAX_ALIGN));
        E::write_f32(self, writer);
    }
}
impl PlainCdrSerialize for f64 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(core::cmp::min(64 / 8, V::MAX_ALIGN));
        E::write_f64(self, writer);
    }
}
impl PlainCdrSerialize for char {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        E::write_char(self, writer);
    }
}
impl PlainCdrSerialize for &[u8] {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        E::write_slice_u8(self, writer);
    }
}
pub trait EndiannessWrite {
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
impl EndiannessWrite for BigEndian {
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
impl EndiannessWrite for LittleEndian {
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

fn serialize_sequence_basic<W: Write, E: EndiannessWrite, V: EncodingVersion>(
    v: Vec<impl PlainCdrSerialize>,
    writer: &mut Writer<W, E, V>,
) {
    PlainCdrSerialize::serialize(v.len() as u32, writer);
    for vi in v {
        PlainCdrSerialize::serialize(vi, writer);
    }
}

fn serialize_array_basic<W: Write, E: EndiannessWrite, V: EncodingVersion>(
    v: Vec<impl PlainCdrSerialize>,
    writer: &mut Writer<W, E, V>,
) {
    for vi in v {
        PlainCdrSerialize::serialize(vi, writer);
    }
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer<W: Write, E: EndiannessWrite, V: EncodingVersion> {
    fn into_inner(self) -> W;

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError>;

    fn serialize_dynamic_data_member(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => v.get_boolean_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => {
                v.get_int16_value(member_id)?.serialize(&mut self.writer());
            }
            TypeKind::INT32 => {
                v.get_int32_value(member_id)?.serialize(&mut self.writer());
            }
            TypeKind::INT64 => v.get_int64_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::UINT16 => v.get_uint16_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::UINT32 => v.get_uint32_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::UINT64 => v.get_uint64_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::FLOAT32 => v.get_float32_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::FLOAT64 => v.get_float64_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::FLOAT128 => unimplemented!("not supported by Rust"),
            TypeKind::INT8 => v.get_int8_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::UINT8 => v.get_uint8_value(member_id)?.serialize(&mut self.writer()),
            TypeKind::CHAR8 => v.get_char8_value(member_id)?.serialize(&mut self.writer()),
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
            TypeKind::BOOLEAN => {
                serialize_sequence_basic(v.get_boolean_values(member_id)?, self.writer())
            }
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => {
                serialize_sequence_basic(v.get_int16_values(member_id)?, self.writer())
            }
            TypeKind::INT32 => {
                serialize_sequence_basic(v.get_int32_values(member_id)?, self.writer())
            }
            TypeKind::INT64 => {
                serialize_sequence_basic(v.get_int64_values(member_id)?, self.writer())
            }
            TypeKind::UINT16 => {
                serialize_sequence_basic(v.get_uint16_values(member_id)?, self.writer())
            }
            TypeKind::UINT32 => {
                serialize_sequence_basic(v.get_uint32_values(member_id)?, self.writer())
            }
            TypeKind::UINT64 => {
                serialize_sequence_basic(v.get_uint64_values(member_id)?, self.writer())
            }
            TypeKind::FLOAT32 => {
                serialize_sequence_basic(v.get_float32_values(member_id)?, self.writer())
            }
            TypeKind::FLOAT64 => {
                serialize_sequence_basic(v.get_float64_values(member_id)?, self.writer())
            }
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => {
                serialize_sequence_basic(v.get_int8_values(member_id)?, self.writer())
            }
            TypeKind::UINT8 => {
                serialize_sequence_basic(v.get_uint8_values(member_id)?, self.writer())
            }
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                let list = v.get_string_values(member_id)?;
                PlainCdrSerialize::serialize(list.len() as u32, self.writer());
                for v in &list {
                    self.serialize_string(v);
                }
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => {
                let list = v.get_complex_values(member_id)?;
                PlainCdrSerialize::serialize(list.len() as u32, self.writer());
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
            TypeKind::UINT8 => serialize_array_basic(v.get_uint8_values(member_id)?, self.writer()),
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                for v in &v.get_string_values(member_id)? {
                    self.serialize_string(v);
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
        PlainCdrSerialize::serialize(*v.get_int32_value(0)?, self.writer());
        Ok(())
    }

    fn serialize_complex(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        match v.type_ref().get_descriptor().extensibility_kind {
            ExtensibilityKind::Final => self.serialize_final_struct(v),
            ExtensibilityKind::Appendable => self.serialize_appendable_struct(v),
            ExtensibilityKind::Mutable => self.serialize_mutable_struct(v),
        }
    }
    fn serialize_string(&mut self, v: &String) {
        PlainCdrSerialize::serialize(v.len() as u32 + 1, self.writer());
        PlainCdrSerialize::serialize(v.as_bytes(), self.writer());
        PlainCdrSerialize::serialize(0u8, self.writer());
    }

    fn serialize_writable(&mut self, v: impl PlainCdrSerialize) {
        v.serialize(self.writer());
    }

    fn writer(&mut self) -> &mut Writer<W, E, V>;
}
