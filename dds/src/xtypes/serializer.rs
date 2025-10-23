use super::dynamic_type::ExtensibilityKind;
use crate::xtypes::{
    dynamic_type::{DynamicData, TypeKind},
    error::{XTypesError, XTypesResult},
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// A trait to Write bytes into a potentially growing buffer
pub trait Write {
    fn write(&mut self, buf: &[u8]);
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf)
    }
}

trait EncodingVersion {
    const MAX_ALIGN: usize;
}
struct EncodingVersion1;
impl EncodingVersion for EncodingVersion1 {
    const MAX_ALIGN: usize = 8;
}

struct EncodingVersion2;
impl EncodingVersion for EncodingVersion2 {
    const MAX_ALIGN: usize = 4;
}

struct Writer<W, E, V> {
    buffer: W,
    position: usize,
    _endianness: E,
    _encoding_version: V,
}

impl<W: Write, E, V: EncodingVersion> Writer<W, E, V> {
    fn new(buffer: W, endianness: E, encoding_version: V) -> Self {
        Self {
            buffer,
            position: 0,
            _endianness: endianness,
            _encoding_version: encoding_version,
        }
    }

    fn write_slice(&mut self, data: &[u8]) {
        self.buffer.write(data);
        self.position += data.len();
    }

    fn pad(&mut self, alignment: usize) {
        const ZEROS: [u8; 8] = [0; 8];
        let alignment = core::cmp::min(alignment, V::MAX_ALIGN);
        let alignment = self.position.div_ceil(alignment) * alignment - self.position;
        self.write_slice(&ZEROS[..alignment]);
    }
}

impl<W: Write, E, V: EncodingVersion> Write for Writer<W, E, V> {
    fn write(&mut self, buf: &[u8]) {
        self.write_slice(buf);
    }
}

trait PlainCdrSerialize {
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
        writer.pad(i16::BITS as usize / 8);
        E::write_i16(self, writer);
    }
}
impl PlainCdrSerialize for u16 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(u16::BITS as usize / 8);
        E::write_u16(self, writer);
    }
}
impl PlainCdrSerialize for i32 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(i32::BITS as usize / 8);
        E::write_i32(self, writer);
    }
}
impl PlainCdrSerialize for u32 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(u32::BITS as usize / 8);
        E::write_u32(self, writer);
    }
}
impl PlainCdrSerialize for i64 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(i64::BITS as usize / 8);
        E::write_i64(self, writer);
    }
}
impl PlainCdrSerialize for u64 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(u64::BITS as usize / 8);
        E::write_u64(self, writer);
    }
}
impl PlainCdrSerialize for f32 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(32 / 8);
        E::write_f32(self, writer);
    }
}
impl PlainCdrSerialize for f64 {
    fn serialize<W: Write, E: EndiannessWrite, V: EncodingVersion>(
        self,
        writer: &mut Writer<W, E, V>,
    ) {
        writer.pad(64 / 8);
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
trait EndiannessWrite {
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

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer<W> {
    fn into_inner(self) -> W;

    fn serialize(&mut self, dynamic_data: &DynamicData) -> XTypesResult<()> {
        // todo header CDR type
        match dynamic_data.type_ref().get_kind() {
            TypeKind::ENUM => {
                self.serialize_enum(dynamic_data)?;
            }
            TypeKind::STRUCTURE => self.serialize_complex(dynamic_data)?,
            kind => unimplemented!("Should not reach for {kind:?}"),
        }
        // sentinel ?
        Ok(())
    }

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.serialize_all_dynamic_data_members(v)
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.serialize_all_dynamic_data_members(v)
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError>;

    fn serialize_all_dynamic_data_members(&mut self, v: &DynamicData) -> XTypesResult<()> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    fn serialize_array_basic(&mut self, v: Vec<impl PlainCdrSerialize>) {
        for vi in v {
            self.serialize_writable(vi);
        }
    }
    fn serialize_sequence_basic(&mut self, v: Vec<impl PlainCdrSerialize>) {
        self.serialize_writable(v.len() as u32);

        for vi in v {
            self.serialize_writable(vi);
        }
    }

    fn serialize_string(&mut self, v: &String) {
        self.serialize_writable(v.len() as u32 + 1);
        self.serialize_writable(v.as_bytes());
        self.serialize_writable(0u8);
    }

    fn serialize_dynamic_data_member(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        let member_descriptor = v.get_descriptor(member_id)?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => self.serialize_writable(*v.get_boolean_value(member_id)?),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => self.serialize_writable(*v.get_int16_value(member_id)?),
            TypeKind::INT32 => self.serialize_writable(*v.get_int32_value(member_id)?),
            TypeKind::INT64 => self.serialize_writable(*v.get_int64_value(member_id)?),
            TypeKind::UINT16 => self.serialize_writable(*v.get_uint16_value(member_id)?),
            TypeKind::UINT32 => self.serialize_writable(*v.get_uint32_value(member_id)?),
            TypeKind::UINT64 => self.serialize_writable(*v.get_uint64_value(member_id)?),
            TypeKind::FLOAT32 => self.serialize_writable(*v.get_float32_value(member_id)?),
            TypeKind::FLOAT64 => self.serialize_writable(*v.get_float64_value(member_id)?),
            TypeKind::FLOAT128 => unimplemented!("not supported by Rust"),
            TypeKind::INT8 => self.serialize_writable(*v.get_int8_value(member_id)?),
            TypeKind::UINT8 => self.serialize_writable(*v.get_uint8_value(member_id)?),
            TypeKind::CHAR8 => self.serialize_writable(*v.get_char8_value(member_id)?),
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
            TypeKind::BOOLEAN => self.serialize_sequence_basic(v.get_boolean_values(member_id)?),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => self.serialize_sequence_basic(v.get_int16_values(member_id)?),
            TypeKind::INT32 => self.serialize_sequence_basic(v.get_int32_values(member_id)?),
            TypeKind::INT64 => self.serialize_sequence_basic(v.get_int64_values(member_id)?),
            TypeKind::UINT16 => self.serialize_sequence_basic(v.get_uint16_values(member_id)?),
            TypeKind::UINT32 => self.serialize_sequence_basic(v.get_uint32_values(member_id)?),
            TypeKind::UINT64 => self.serialize_sequence_basic(v.get_uint64_values(member_id)?),
            TypeKind::FLOAT32 => self.serialize_sequence_basic(v.get_float32_values(member_id)?),

            TypeKind::FLOAT64 => self.serialize_sequence_basic(v.get_float64_values(member_id)?),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => self.serialize_sequence_basic(v.get_int8_values(member_id)?),
            TypeKind::UINT8 => self.serialize_sequence_basic(v.get_uint8_values(member_id)?),
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                let list = v.get_string_values(member_id)?;
                self.serialize_writable(list.len() as u32);
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
                self.serialize_writable(list.len() as u32);
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
            TypeKind::UINT8 => self.serialize_array_basic(v.get_uint8_values(member_id)?),
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
        self.serialize_writable(*v.get_int32_value(0)?);
        Ok(())
    }

    fn serialize_complex(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        match v.type_ref().get_descriptor().extensibility_kind {
            ExtensibilityKind::Final => self.serialize_final_struct(v),
            ExtensibilityKind::Appendable => self.serialize_appendable_struct(v),
            ExtensibilityKind::Mutable => self.serialize_mutable_struct(v),
        }
    }

    fn serialize_writable(&mut self, v: impl PlainCdrSerialize);
}

const PID_SENTINEL: u16 = 1;


fn count_bytes_cdr1(v: &DynamicData, member_id: u32) -> Result<usize, XTypesError> {
    let mut byte_conter_serializer = Xcdr1Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    Ok(byte_conter_serializer.into_inner().0)
}

fn count_bytes_cdr2(v: &DynamicData, member_id: u32) -> Result<usize, XTypesError> {
    let mut byte_conter_serializer = Xcdr2Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    Ok(byte_conter_serializer.into_inner().0)
}

fn count_bytes_pl_cdr(v: &DynamicData, member_id: u32) -> Result<u16, XTypesError> {
    let mut byte_conter_serializer = Xcdr1Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    let length = byte_conter_serializer.into_inner().0 as u16;
    Ok((length + 3) & !3)
}
fn count_bytes_pl_cdr_complex(v: &DynamicData) -> Result<u16, XTypesError> {
    let mut byte_conter_serializer = Xcdr1Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_complex(v)?;
    let length = byte_conter_serializer.into_inner().0 as u16;
    Ok((length + 3) & !3)
}

fn count_bytes_xdr2(v: &DynamicData, member_id: u32) -> Result<usize, XTypesError> {
    let mut byte_conter_serializer = Xcdr2Serializer::new(ByteCounter::new(), LittleEndian);
    byte_conter_serializer.serialize_dynamic_data_member(v, member_id)?;
    Ok(byte_conter_serializer.into_inner().0)
}


pub struct PlCdrSerializer<C> {
    cdr1_le_serializer: Xcdr1Serializer<C, LittleEndian>,
}

impl<C: Write> PlCdrSerializer<C> {
    pub fn new(collection: C) -> Self {
        Self {
            cdr1_le_serializer: Xcdr1Serializer::new(collection, LittleEndian),
        }
    }
}

impl<W: Write> XTypesSerializer<W> for PlCdrSerializer<W> {
    fn into_inner(self) -> W {
        self.cdr1_le_serializer.into_inner()
    }

    fn serialize_final_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.cdr1_le_serializer.serialize_final_struct(v)
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        self.cdr1_le_serializer.serialize_appendable_struct(v)
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let member_descriptor = v.get_descriptor(member_id)?;
            if member_descriptor.is_optional {
                if let Some(default_value) = &member_descriptor.default_value {
                    if &v.get_value(member_id)? == default_value {
                        continue;
                    }
                }
            }

            let element_type = member_descriptor
                .r#type
                .get_descriptor()
                .element_type
                .as_ref();
            if let Some(element_type) = element_type {
                // Sequence
                if element_type.get_kind() == TypeKind::STRUCTURE {
                    for vi in &v.get_complex_values(member_id)? {
                        let padded_length = count_bytes_pl_cdr_complex(vi)?;
                        LittleEndian::write_u16(
                            member_id as u16,
                            &mut self.cdr1_le_serializer.writer,
                        );
                        LittleEndian::write_u16(padded_length, &mut self.cdr1_le_serializer.writer);
                        self.serialize_complex(vi)?
                    }
                    self.cdr1_le_serializer.writer.pad(4);
                } else {
                    let padded_length = count_bytes_pl_cdr(v, member_id)?;
                    LittleEndian::write_u16(member_id as u16, &mut self.cdr1_le_serializer.writer);
                    LittleEndian::write_u16(padded_length, &mut self.cdr1_le_serializer.writer);
                    self.serialize_dynamic_data_member(v, member_id)?;
                    self.cdr1_le_serializer.writer.pad(4);
                }
            } else {
                // Structure
                let padded_length = count_bytes_pl_cdr(v, member_id)?;
                LittleEndian::write_u16(member_id as u16, &mut self.cdr1_le_serializer.writer);
                LittleEndian::write_u16(padded_length, &mut self.cdr1_le_serializer.writer);
                self.serialize_dynamic_data_member(v, member_id)?;
                self.cdr1_le_serializer.writer.pad(4);
            }
        }
        LittleEndian::write_u16(PID_SENTINEL, &mut self.cdr1_le_serializer.writer);
        LittleEndian::write_u16(0, &mut self.cdr1_le_serializer.writer);
        Ok(())
    }

    fn serialize_dynamic_data_member(
        &mut self,
        v: &DynamicData,
        member_id: u32,
    ) -> Result<(), XTypesError> {
        self.cdr1_le_serializer
            .serialize_dynamic_data_member(v, member_id)
    }

    fn serialize_writable(&mut self, v: impl super::serializer::PlainCdrSerialize) {
        v.serialize(&mut self.cdr1_le_serializer.writer);
    }
}

struct ByteCounter(usize);

impl ByteCounter {
    fn new() -> Self {
        Self(0)
    }
}

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) {
        self.0 += buf.len();
    }
}

pub struct Xcdr1Serializer<W, E> {
    writer: Writer<W, E, EncodingVersion1>,
}

impl<W: Write, E> Xcdr1Serializer<W, E> {
    pub fn new(buffer: W, endianess: E) -> Self {
        Self {
            writer: Writer::new(buffer, endianess, EncodingVersion1),
        }
    }
}

impl<W: Write, E: EndiannessWrite> XTypesSerializer<W> for Xcdr1Serializer<W, E> {
    fn into_inner(self) -> W {
        self.writer.buffer
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let length = count_bytes_cdr1(v, member_id)?;
            self.serialize_writable(member_id as u16);
            self.serialize_writable(length as u16);
            self.serialize_dynamic_data_member(v, member_id)?;
            self.writer.pad(4);
        }
        self.serialize_writable(PID_SENTINEL);
        self.serialize_writable(0u16);
        Ok(())
    }
    fn serialize_writable(&mut self, v: impl PlainCdrSerialize) {
        v.serialize(&mut self.writer);
    }
}

pub struct Xcdr2Serializer<C, E> {
    writer: Writer<C, E, EncodingVersion2>,
}

impl<C: Write, E> Xcdr2Serializer<C, E> {
    pub fn new(buffer: C, endianess: E) -> Self {
        Self {
            writer: Writer::new(buffer, endianess, EncodingVersion2),
        }
    }
}

impl<W: Write, E: EndiannessWrite> XTypesSerializer<W> for Xcdr2Serializer<W, E> {
    fn into_inner(self) -> W {
        self.writer.buffer
    }

    fn serialize_writable(&mut self, v: impl PlainCdrSerialize) {
        v.serialize(&mut self.writer);
    }

    fn serialize_appendable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            // DHEADER
            let length = count_bytes_cdr1(v, member_id)? as u32;
            length.serialize(&mut self.writer);
            // Value
            self.serialize_dynamic_data_member(v, member_id)?;
        }
        Ok(())
    }

    fn serialize_mutable_struct(&mut self, v: &DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.get_item_count() {
            let member_id = v.get_member_id_at_index(field_index)?;
            let length = count_bytes_xdr2(v, member_id)?;
            self.serialize_writable(member_id as u16);
            self.serialize_writable(length as u16);
            self.serialize_dynamic_data_member(v, member_id)?;
            self.writer.pad(4);
        }
        self.serialize_writable(PID_SENTINEL);
        self.serialize_writable(0u16);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{infrastructure::type_support::TypeSupport, xtypes::serializer::BigEndian};
    extern crate std;

    fn serialize_v1_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut serializer = Xcdr1Serializer::new(Vec::new(), BigEndian);
        serializer.serialize(&v.create_dynamic_sample()).unwrap();
        serializer.into_inner()
    }

    fn serialize_v1_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut serializer = Xcdr1Serializer::new(Vec::new(), LittleEndian);
        serializer.serialize(&v.create_dynamic_sample()).unwrap();
        serializer.into_inner()
    }

    fn serialize_v2_be<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut serializer = Xcdr2Serializer::new(Vec::new(), BigEndian);
        serializer.serialize(&v.create_dynamic_sample()).unwrap();
        serializer.into_inner()
    }

    fn serialize_v2_le<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut serializer = Xcdr2Serializer::new(Vec::new(), LittleEndian);
        serializer.serialize(&v.create_dynamic_sample()).unwrap();
        serializer.into_inner()
    }

    #[test]
    fn serialize_basic_types_struct() {
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
                0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
                0x3F, 0x80, 0x00, 0x00, // f10: f32
                0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
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

    #[test]
    fn serialize_u8_array() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct U8Array {
            #[dust_dds(id = 41)]
            version: [u8; 2],
        }

        let v = U8Array { version: [1, 2] };
        assert_eq!(
            serialize_v1_be(v),
            vec![
                0x00, 41, 0, 2, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                0, 1, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_locator() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct LocatorContainer {
            #[dust_dds(id = 73)]
            locator: Locator,
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "final", nested)]
        struct Locator {
            kind: i32,
            address1: [u8; 2],
            address2: [u8; 3],
        }

        let v = LocatorContainer {
            locator: Locator {
                kind: 1,
                address1: [3, 4],
                address2: [5, 6, 7],
            },
        };
        assert_eq!(
            serialize_v1_be(v),
            vec![
                0, 73, 0, 9, // PID | length
                0, 0, 0, 1, // kind
                3, 4, 5, 6, // address1 and 2
                7, 0, 0, 0, // address2 | pading (3 bytes)
                0, 1, 0, 0
            ]
        );
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

    #[test]
    fn serialize_string_list() {
        #[derive(TypeSupport)]
        struct StringList {
            name: Vec<String>,
        }

        let v = StringList {
            name: vec!["one".to_string(), "two".to_string()],
        };
        assert_eq!(
            serialize_v1_be(v),
            vec![
                0, 0, 0, 2, // vec length
                0, 0, 0, 4, // String length
                b'o', b'n', b'e', 0, // String
                0, 0, 0, 4, // String length
                b't', b'w', b'o', 0, // String
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
        #[dust_dds(id = 90, key)]
        key: u8,
        #[dust_dds(id = 80)]
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
                0x00, 80, 0, 2, // PID | length
                0, 8, 0, 0, // participant_key | padding (2 bytes)
                0x00, 90, 0, 1, // PID | length
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
        #[dust_dds(id = 96, key)]
        field_primitive: u8,
        #[dust_dds(id = 97)]
        field_mutable: MutableType,
        #[dust_dds(id = 98)]
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
                0x00, 96, 0, 1, // PID | length
                5, 0, 0, 0, // field_primitive | padding (3 bytes)
                0x00, 97, 0, 20, // PID | length
                0x00, 80, 0, 2, // field_mutable: PID | length
                0, 8, 0, 0, // field_mutable: participant_key | padding (2 bytes)
                0x00, 90, 0, 1, // field_mutable: PID | length
                7, 0, 0, 0, // field_mutable: key | padding (3 bytes)
                0, 1, 0, 0, // field_mutable: Sentinel
                0x00, 98, 0, 2, // field_mutable: PID | length
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
}

#[cfg(test)]
mod rtps_pl_tests {
    use super::*;
    use crate::infrastructure::type_support::TypeSupport;
    extern crate std;

    fn test_serialize_type_support<T: TypeSupport>(v: T) -> std::vec::Vec<u8> {
        let mut serializer = PlCdrSerializer::new(Vec::new());
        serializer.serialize(&v.create_dynamic_sample()).unwrap();
        serializer.into_inner()
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

    #[test]
    fn serialize_string() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct StringData {
            #[dust_dds(id = 41)]
            name: String,
        }

        let v = StringData {
            name: "one".to_string(),
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                41, 0x00, 8, 0, // PID, length
                4, 0, 0, 0, // String length
                b'o', b'n', b'e', 0, // String
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_string_list() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct StringList {
            #[dust_dds(id = 41)]
            name: Vec<String>,
        }

        let v = StringList {
            name: vec!["one".to_string(), "two".to_string()],
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                41, 0x00, 20, 0, // PID, length
                2, 0, 0, 0, // vec length
                4, 0, 0, 0, // String length
                b'o', b'n', b'e', 0, // String
                4, 0, 0, 0, // String length
                b't', b'w', b'o', 0, // String
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_u8_array() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct U8Array {
            #[dust_dds(id = 41)]
            version: [u8; 2],
        }

        let v = U8Array { version: [1, 2] };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                41, 0x00, 4, 0, // PID, length
                1, 2, 0, 0, // version | padding (2 bytres)
                1, 0, 0, 0, // Sentinel
            ]
        );
    }

    #[test]
    fn serialize_locator() {
        #[derive(TypeSupport)]
        struct Locator {
            kind: i32,
            port: u32,
            address: [u8; 4],
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct LocatorContainer {
            #[dust_dds(id = 41)]
            locator: Locator,
        }

        let v = LocatorContainer {
            locator: Locator {
                kind: 1,
                port: 2,
                address: [7; 4],
            },
        };
        assert_eq!(
            test_serialize_type_support(v),
            vec![
                41, 0x00, 12, 0, // PID, length
                1, 0, 0, 0, // kind
                2, 0, 0, 0, // port
                7, 7, 7, 7, // address
                1, 0, 0, 0, // Sentinel
            ]
        );
    }
}
