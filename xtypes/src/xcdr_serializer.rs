use crate::{
    error::XcdrError,
    serialize::XTypesSerialize,
    serializer::{
        SerializeAppendableStruct, SerializeCollection, SerializeFinalStruct,
        SerializeMutableStruct, XTypesSerializer,
    },
};

pub struct Xcdr1BeSerializer<'a> {
    writer: Writer<'a>,
}

impl<'a> Xcdr1BeSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            writer: Writer::new(buffer),
        }
    }
}

pub struct Xcdr1LeSerializer<'a> {
    writer: Writer<'a>,
}

impl<'a> Xcdr1LeSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            writer: Writer::new(buffer),
        }
    }
}

pub struct Xcdr2BeSerializer<'a> {
    writer: Writer<'a>,
}

impl<'a> Xcdr2BeSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            writer: Writer::new(buffer),
        }
    }
}

pub struct Xcdr2LeSerializer<'a> {
    writer: Writer<'a>,
}

impl<'a> Xcdr2LeSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            writer: Writer::new(buffer),
        }
    }
}

struct Writer<'a> {
    data: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn write_slice(&mut self, data: &[u8]) -> Result<(), XcdrError> {
        let end = self.pos + data.len();
        if end > self.data.len() {
            Err(XcdrError::OutOfMemory)
        } else {
            self.data[self.pos..self.pos + data.len()].copy_from_slice(data);
            self.pos += data.len();
            Ok(())
        }
    }

    fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn pad(&mut self, alignment: usize) -> Result<(), XcdrError> {
        const ZEROS: [u8; 8] = [0; 8];
        let over_alignment = self.pos & (alignment - 1);
        if over_alignment > 0 {
            self.write_slice(&ZEROS[..alignment - over_alignment])?;
        };
        Ok(())
    }
}

fn write_with_padding_v1<const N: usize>(
    writer: &mut Writer,
    data: &[u8; N],
) -> Result<(), XcdrError> {
    writer.pad(N)?;
    writer.write_slice(data)
}

fn write_with_padding_v2<const N: usize>(
    writer: &mut Writer,
    data: &[u8; N],
) -> Result<(), XcdrError> {
    writer.pad(core::cmp::min(N, 4))?;
    writer.write_slice(data)
}

fn into_u8(v: char) -> Result<u8, XcdrError> {
    if !v.is_ascii() {
        Err(XcdrError::InvalidData)
    } else {
        Ok(v as u8)
    }
}
fn into_u32(v: usize) -> Result<u32, XcdrError> {
    if v > u32::MAX as usize {
        Err(XcdrError::InvalidData)
    } else {
        Ok(v as u32)
    }
}
fn str_len(v: &str) -> Result<u32, XcdrError> {
    if !v.is_ascii() {
        Err(XcdrError::InvalidData)
    } else {
        into_u32(v.len() + 1)
    }
}
const SENTINEL: [u8; 4] = [1u8, 0, 0, 0];

fn write_pl_header_be(writer: &mut Writer, header_pos: usize, pid: u16) -> Result<(), XcdrError> {
    let pos = writer.position();
    let length = (pos - header_pos - 4) as u16;
    writer.set_position(header_pos);
    writer.write_slice(&pid.to_be_bytes())?;
    writer.write_slice(&length.to_be_bytes())?;
    writer.set_position(pos);
    Ok(())
}
fn write_pl_header_le(writer: &mut Writer, header_pos: usize, pid: u16) -> Result<(), XcdrError> {
    let pos = writer.position();
    let length = (pos - header_pos - 4) as u16;
    writer.set_position(header_pos);
    writer.write_slice(&pid.to_le_bytes())?;
    writer.write_slice(&length.to_le_bytes())?;
    writer.set_position(pos);
    Ok(())
}

struct PlCdrBeEncoder<'a, 'b> {
    serializer: &'a mut Xcdr1BeSerializer<'b>,
}
impl<'a> SerializeMutableStruct for PlCdrBeEncoder<'a, '_> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        _name: &str,
    ) -> Result<(), XcdrError> {
        let header_pos = self.serializer.writer.position();
        self.serializer.writer.set_position(header_pos + 4);
        value.serialize(&mut *self.serializer)?;
        write_pl_header_be(&mut self.serializer.writer, header_pos, pid)?;
        self.serializer.writer.pad(4)
    }
    fn end(&mut self) -> Result<(), XcdrError> {
        SENTINEL.serialize(&mut *self.serializer)
    }
}

struct PlCdrLeEncoder<'a, 'b> {
    serializer: &'a mut Xcdr1LeSerializer<'b>,
}
impl<'a> SerializeMutableStruct for PlCdrLeEncoder<'a, '_> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        _name: &str,
    ) -> Result<(), XcdrError> {
        let header_pos = self.serializer.writer.position();
        self.serializer.writer.set_position(header_pos + 4);
        value.serialize(&mut *self.serializer)?;
        write_pl_header_le(&mut self.serializer.writer, header_pos, pid)?;
        self.serializer.writer.pad(4)
    }
    fn end(&mut self) -> Result<(), XcdrError> {
        SENTINEL.serialize(&mut *self.serializer)
    }
}

struct PlCdr2BeEncoder<'a, 'b> {
    serializer: &'a mut Xcdr2BeSerializer<'b>,
}
impl<'a> SerializeMutableStruct for PlCdr2BeEncoder<'a, '_> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        _name: &str,
    ) -> Result<(), XcdrError> {
        let header_pos = self.serializer.writer.position();
        self.serializer.writer.set_position(header_pos + 4);
        value.serialize(&mut *self.serializer)?;
        write_pl_header_be(&mut self.serializer.writer, header_pos, pid)?;
        self.serializer.writer.pad(4)
    }
    fn end(&mut self) -> Result<(), XcdrError> {
        SENTINEL.serialize(&mut *self.serializer)
    }
}

struct PlCdr2LeEncoder<'a, 'b> {
    serializer: &'a mut Xcdr2LeSerializer<'b>,
}
impl<'a> SerializeMutableStruct for PlCdr2LeEncoder<'a, '_> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        _name: &str,
    ) -> Result<(), XcdrError> {
        let header_pos = self.serializer.writer.position();
        self.serializer.writer.set_position(header_pos + 4);
        value.serialize(&mut *self.serializer)?;
        write_pl_header_le(&mut self.serializer.writer, header_pos, pid)?;
        self.serializer.writer.pad(4)
    }
    fn end(&mut self) -> Result<(), XcdrError> {
        SENTINEL.serialize(&mut *self.serializer)
    }
}

struct PlainCdrBeEncoder<'a, 'b> {
    serializer: &'a mut Xcdr1BeSerializer<'b>,
}

impl<'a> SerializeFinalStruct for PlainCdrBeEncoder<'a, '_> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XcdrError> {
        XTypesSerialize::serialize(value, &mut *self.serializer)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XcdrError> {
        if let Some(value) = value {
            self.serializer.writer.pad(4)?;
            let header_pos = self.serializer.writer.position();
            self.serializer.writer.set_position(header_pos + 4);
            value.serialize(&mut *self.serializer)?;
            write_pl_header_be(&mut self.serializer.writer, header_pos, 0)
        } else {
            Ok(())
        }
    }
}

struct PlainCdrLeEncoder<'a, 'b> {
    serializer: &'a mut Xcdr1LeSerializer<'b>,
}

impl<'a> SerializeFinalStruct for PlainCdrLeEncoder<'a, '_> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XcdrError> {
        XTypesSerialize::serialize(value, &mut *self.serializer)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XcdrError> {
        if let Some(value) = value {
            self.serializer.writer.pad(4)?;
            let header_pos = self.serializer.writer.position();
            self.serializer.writer.set_position(header_pos + 4);
            value.serialize(&mut *self.serializer)?;
            write_pl_header_le(&mut self.serializer.writer, header_pos, 0)
        } else {
            Ok(())
        }
    }
}

struct PlainCdrEncoder<'a, S> {
    serializer: &'a mut S,
}
impl<S> SerializeAppendableStruct for PlainCdrEncoder<'_, S>
where
    for<'a> &'a mut S: XTypesSerializer,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XcdrError> {
        XTypesSerialize::serialize(value, &mut *self.serializer)
    }
}

struct PlainCdr2Encoder<'a, S> {
    serializer: &'a mut S,
}

impl<S> SerializeFinalStruct for PlainCdr2Encoder<'_, S>
where
    for<'a> &'a mut S: XTypesSerializer,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XcdrError> {
        XTypesSerialize::serialize(value, &mut *self.serializer)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XcdrError> {
        if let Some(value) = value {
            true.serialize(&mut *self.serializer)?;
            value.serialize(&mut *self.serializer)
        } else {
            false.serialize(&mut *self.serializer)
        }
    }
}

struct DelimitedCdrBeEncoder<'a> {
    serializer: Xcdr2BeSerializer<'a>,
}

impl<'a> SerializeAppendableStruct for DelimitedCdrBeEncoder<'a> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XcdrError> {
        let d_header_position = self.serializer.writer.position();
        self.serializer.writer.set_position(d_header_position + 4);
        let data_position = self.serializer.writer.position();
        value.serialize(&mut self.serializer)?;
        let data_end_pos = self.serializer.writer.position();
        let data_length = (data_end_pos - data_position) as u32;
        self.serializer.writer.set_position(d_header_position);
        // DHEADER
        self.serializer
            .writer
            .write_slice(&data_length.to_be_bytes())?;
        self.serializer.writer.set_position(data_end_pos);
        Ok(())
    }
}

struct DelimitedCdrLeEncoder<'a> {
    serializer: Xcdr2LeSerializer<'a>,
}

impl<'a> SerializeAppendableStruct for DelimitedCdrLeEncoder<'a> {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XcdrError> {
        let d_header_position = self.serializer.writer.position();
        self.serializer.writer.set_position(d_header_position + 4);
        let data_position = self.serializer.writer.position();
        value.serialize(&mut self.serializer)?;
        let data_end_pos = self.serializer.writer.position();
        let data_length = (data_end_pos - data_position) as u32;
        self.serializer.writer.set_position(d_header_position);
        // DHEADER
        self.serializer
            .writer
            .write_slice(&data_length.to_le_bytes())?;
        self.serializer.writer.set_position(data_end_pos);
        Ok(())
    }
}

struct CollectionSerializer<'a, S> {
    serializer: &'a mut S,
}
impl<S> SerializeCollection for CollectionSerializer<'_, S>
where
    for<'a> &'a mut S: XTypesSerializer,
{
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XcdrError> {
        value.serialize(&mut *self.serializer)
    }
}

impl XTypesSerializer for &mut Xcdr1BeSerializer<'_> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XcdrError> {
        Ok(PlainCdrBeEncoder {
            serializer: &mut *self,
        })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XcdrError> {
        Ok(PlainCdrEncoder { serializer: self })
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XcdrError> {
        Ok(PlCdrBeEncoder { serializer: self })
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XcdrError> {
        write_with_padding_v1(&mut self.writer, &into_u32(len)?.to_be_bytes())?;
        Ok(CollectionSerializer { serializer: self })
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XcdrError> {
        Ok(CollectionSerializer { serializer: self })
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XcdrError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &[into_u8(v)?])
    }
    fn serialize_string(self, v: &str) -> Result<(), XcdrError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes())?;
        self.writer.write_slice(&[0])
    }
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XcdrError> {
        self.writer.write_slice(&into_u32(v.len())?.to_be_bytes())?;
        self.writer.write_slice(v)
    }
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XcdrError> {
        self.writer.write_slice(v)
    }
}

impl<'a> XTypesSerializer for &'a mut Xcdr1LeSerializer<'_> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XcdrError> {
        Ok(PlainCdrLeEncoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XcdrError> {
        Ok(PlainCdrEncoder { serializer: self })
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XcdrError> {
        Ok(PlCdrLeEncoder { serializer: self })
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XcdrError> {
        write_with_padding_v1(&mut self.writer, &into_u32(len)?.to_le_bytes())?;
        Ok(CollectionSerializer { serializer: self })
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XcdrError> {
        Ok(CollectionSerializer { serializer: self })
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XcdrError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XcdrError> {
        write_with_padding_v1(&mut self.writer, &[into_u8(v)?])
    }
    fn serialize_string(self, v: &str) -> Result<(), XcdrError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes())?;
        self.writer.write_slice(&[0])
    }
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XcdrError> {
        self.writer.write_slice(&into_u32(v.len())?.to_le_bytes())?;
        self.writer.write_slice(v)
    }
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XcdrError> {
        self.writer.write_slice(v)
    }
}

impl XTypesSerializer for &mut Xcdr2BeSerializer<'_> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XcdrError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XcdrError> {
        Ok(DelimitedCdrBeEncoder {
            serializer: Xcdr2BeSerializer::new(&mut self.writer.data),
        })
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XcdrError> {
        Ok(PlCdr2BeEncoder { serializer: self })
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XcdrError> {
        write_with_padding_v2(&mut self.writer, &into_u32(len)?.to_be_bytes())?;
        Ok(CollectionSerializer { serializer: self })
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XcdrError> {
        Ok(CollectionSerializer { serializer: self })
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XcdrError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &[into_u8(v)?])
    }
    fn serialize_string(self, v: &str) -> Result<(), XcdrError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes())?;
        self.writer.write_slice(&[0])
    }
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XcdrError> {
        self.writer.write_slice(&into_u32(v.len())?.to_be_bytes())?;
        self.writer.write_slice(v)
    }
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XcdrError> {
        self.writer.write_slice(v)
    }
}

impl<'a> XTypesSerializer for &'a mut Xcdr2LeSerializer<'_> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XcdrError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XcdrError> {
        Ok(DelimitedCdrLeEncoder {
            serializer: Xcdr2LeSerializer::new(&mut self.writer.data),
        })
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XcdrError> {
        Ok(PlCdr2LeEncoder { serializer: self })
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XcdrError> {
        write_with_padding_v2(&mut self.writer, &into_u32(len)?.to_le_bytes())?;
        Ok(CollectionSerializer { serializer: self })
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XcdrError> {
        Ok(CollectionSerializer { serializer: self })
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XcdrError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XcdrError> {
        write_with_padding_v2(&mut self.writer, &[into_u8(v)?])
    }
    fn serialize_string(self, v: &str) -> Result<(), XcdrError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes())?;
        self.writer.write_slice(&[0])
    }
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XcdrError> {
        self.writer.write_slice(&into_u32(v.len())?.to_le_bytes())?;
        self.writer.write_slice(v)
    }
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XcdrError> {
        self.writer.write_slice(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize_v1_be<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr1BeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v1_le<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v2_be<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr2BeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    fn serialize_v2_le<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr2LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    #[test]
    fn serialize_octet() {
        let v = 0x20u8;
        assert_eq!(serialize_v1_be(&v), [0x20]);
        assert_eq!(serialize_v1_le(&v), [0x20]);
        assert_eq!(serialize_v2_be(&v), [0x20]);
        assert_eq!(serialize_v2_le(&v), [0x20]);
    }

    #[test]
    fn serialize_char() {
        let v = 'Z';
        assert_eq!(serialize_v1_be(&v), [0x5a]);
        assert_eq!(serialize_v1_le(&v), [0x5a]);
        assert_eq!(serialize_v2_be(&v), [0x5a]);
        assert_eq!(serialize_v2_le(&v), [0x5a]);
    }

    #[test]
    fn serialize_ushort() {
        let v = 65500u16;
        assert_eq!(serialize_v1_be(&v), [0xff, 0xdc]);
        assert_eq!(serialize_v1_le(&v), [0xdc, 0xff]);
        assert_eq!(serialize_v2_be(&v), [0xff, 0xdc]);
        assert_eq!(serialize_v2_le(&v), [0xdc, 0xff]);
    }

    #[test]
    fn serialize_short() {
        let v = -32700i16;
        assert_eq!(serialize_v1_be(&v), [0x80, 0x44]);
        assert_eq!(serialize_v1_le(&v), [0x44, 0x80]);
        assert_eq!(serialize_v2_be(&v), [0x80, 0x44]);
        assert_eq!(serialize_v2_le(&v), [0x44, 0x80]);
    }

    #[test]
    fn serialize_ulong() {
        let v = 4294967200u32;
        assert_eq!(serialize_v1_be(&v), [0xff, 0xff, 0xff, 0xa0]);
        assert_eq!(serialize_v1_le(&v), [0xa0, 0xff, 0xff, 0xff]);
        assert_eq!(serialize_v2_be(&v), [0xff, 0xff, 0xff, 0xa0]);
        assert_eq!(serialize_v2_le(&v), [0xa0, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn serialize_long() {
        let v = -2147483600i32;
        assert_eq!(serialize_v1_be(&v), [0x80, 0x00, 0x00, 0x30]);
        assert_eq!(serialize_v1_le(&v), [0x30, 0x00, 0x00, 0x80]);
        assert_eq!(serialize_v2_be(&v), [0x80, 0x00, 0x00, 0x30]);
        assert_eq!(serialize_v2_le(&v), [0x30, 0x00, 0x00, 0x80]);
    }

    #[test]
    fn serialize_ulonglong() {
        let v = 18446744073709551600u64;
        assert_eq!(
            serialize_v1_be(&v),
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
        assert_eq!(
            serialize_v2_be(&v),
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_longlong() {
        let v = -9223372036800i64;
        assert_eq!(
            serialize_v1_be(&v),
            [0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]
        );
        assert_eq!(
            serialize_v2_be(&v),
            [0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_float() {
        let v = core::f32::MIN_POSITIVE;
        assert_eq!(serialize_v1_be(&v), [0x00, 0x80, 0x00, 0x00]);
        assert_eq!(serialize_v1_le(&v), [0x00, 0x00, 0x80, 0x00]);
        assert_eq!(serialize_v2_be(&v), [0x00, 0x80, 0x00, 0x00]);
        assert_eq!(serialize_v2_le(&v), [0x00, 0x00, 0x80, 0x00]);
    }

    #[test]
    fn serialize_double() {
        let v = core::f64::MIN_POSITIVE;
        assert_eq!(
            serialize_v1_be(&v),
            [0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
        );
        assert_eq!(
            serialize_v2_be(&v),
            [0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
        );
    }

    #[test]
    fn serialize_bool() {
        let v = true;
        assert_eq!(serialize_v1_be(&v), [0x01]);
        assert_eq!(serialize_v1_le(&v), [0x01]);
        assert_eq!(serialize_v2_be(&v), [0x01]);
        assert_eq!(serialize_v2_le(&v), [0x01]);
    }

    #[test]
    fn serialize_string() {
        let v = "Hola";
        assert_eq!(
            serialize_v1_be(&v),
            [
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        assert_eq!(
            serialize_v2_be(&v),
            [
                0, 0, 0, 5, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [
                5, 0, 0, 0, //length
                b'H', b'o', b'l', b'a', // str
                0x00, // terminating 0
            ]
        );
    }

    #[test]
    fn serialize_empty_string() {
        let v = "";
        assert_eq!(serialize_v1_be(&v), [0x00, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(serialize_v1_le(&v), [0x01, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(serialize_v2_be(&v), [0x00, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(serialize_v2_le(&v), [0x01, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn serialize_byte_slice() {
        let v = &[1u8, 2, 3, 4, 5][..];
        assert_eq!(
            serialize_v1_be(&v),
            [
                0, 0, 0, 5, // length
                1, 2, 3, 4, 5 // data
            ]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [
                5, 0, 0, 0, // length
                1, 2, 3, 4, 5 // data
            ]
        );
        assert_eq!(
            serialize_v2_be(&v),
            [
                0, 0, 0, 5, // length
                1, 2, 3, 4, 5 // data
            ]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [
                5, 0, 0, 0, // length
                1, 2, 3, 4, 5 // data
            ]
        );
    }

    #[test]
    fn serialize_byte_array() {
        let v = [1u8, 2, 3, 4, 5];
        assert_eq!(serialize_v1_be(&v), [1, 2, 3, 4, 5]);
        assert_eq!(serialize_v1_le(&v), [1, 2, 3, 4, 5]);
        assert_eq!(serialize_v2_be(&v), [1, 2, 3, 4, 5]);
        assert_eq!(serialize_v2_le(&v), [1, 2, 3, 4, 5]);
    }

    //@extensibility(FINAL)
    struct FinalType {
        field_u16: u16,
        field_u64: u64,
    }

    impl XTypesSerialize for FinalType {
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
            let mut serializer = serializer.serialize_final_struct()?;
            serializer.serialize_field(&self.field_u16, "field_u16")?;
            serializer.serialize_field(&self.field_u64, "field_u64")
        }
    }

    #[test]
    fn serialize_final_struct() {
        let v = FinalType {
            field_u16: 7,
            field_u64: 9,
        };
        // PLAIN_CDR:
        assert_eq!(
            serialize_v1_be(&v),
            [
                0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [
                7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_v2_be(&v),
            [
                0, 7, 0, 0, // field_u16 | padding (2 bytes)
                0, 0, 0, 0, 0, 0, 0, 9, // field_u64
            ]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [
                7, 0, 0, 0, // field_u16 | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0, // field_u64
            ]
        );
    }

    //@extensibility(FINAL)
    struct FinalOptionalType {
        field: u8,
        optional_field: Option<u16>,
    }

    impl XTypesSerialize for FinalOptionalType {
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
            let mut serializer = serializer.serialize_final_struct()?;
            serializer.serialize_field(&self.field, "field")?;
            serializer.serialize_optional_field(&self.optional_field, "optional_field")
        }
    }
    #[test]
    fn serialize_final_optional_struct() {
        let some = FinalOptionalType {
            field: 6,
            optional_field: Some(7),
        };
        // PLAIN_CDR:
        assert_eq!(
            serialize_v1_be(&some),
            [
                6, 0, 0, 0, // u8 | padding
                0, 0, 0, 2, // HEADER (FLAGS+ID | length)
                0, 7, // optional_field value
            ]
        );
        assert_eq!(
            serialize_v1_le(&some),
            [
                6, 0, 0, 0, // u8 | padding
                0, 0, 2, 0, // HEADER (FLAGS+ID | length)
                7, 0 // optional_field value
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_v2_be(&some),
            [
                6, 1, // u8 | boolean for option
                0, 7 // optional_field value
            ]
        );
        assert_eq!(
            serialize_v2_le(&some),
            [
                6, 1, // u8 | boolean for option
                7, 0 // optional_field value
            ]
        );

        let none = FinalOptionalType {
            field: 6,
            optional_field: None,
        };
        // PLAIN_CDR:
        assert_eq!(
            serialize_v1_be(&none),
            [
                6, 0, 0, 0, // u8 | padding
                0, 0, 0, 0, // HEADER (FLAGS+ID | length)
            ]
        );
        assert_eq!(
            serialize_v1_le(&none),
            [
                6, 0, 0, 0, // u8 | padding
                0, 0, 0, 0, // HEADER (FLAGS+ID | length)
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_v2_be(&none),
            [
            6, 0, // u8 | boolean for option
        ]
        );
        assert_eq!(
            serialize_v2_le(&none),
            [
            6, 0, // u8 | boolean for option
        ]
        );
    }

    //@extensibility(FINAL)
    struct NestedFinalType {
        field_nested: FinalType,
        field_u8: u8,
    }
    impl XTypesSerialize for NestedFinalType {
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
            let mut serializer = serializer.serialize_final_struct()?;
            serializer.serialize_field(&self.field_nested, "field_nested")?;
            serializer.serialize_field(&self.field_u8, "field_u8")
        }
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
            serialize_v1_be(&v),
            [
                0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                0, 0, 0, 0, 0, 0, 0, 9,  // u64
                10, //u8
            ]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [
                7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                9, 0, 0, 0, 0, 0, 0, 0,  // u64
                10, //u8
            ]
        );
        // PLAIN_CDR2:
        assert_eq!(
            serialize_v2_le(&v),
            [
                7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
                9, 0, 0, 0, 0, 0, 0, 0,  // u64
                10, //u8
            ]
        );
        assert_eq!(
            serialize_v2_be(&v),
            [
                0, 7, 0, 0, // nested FinalType (u16) | padding
                0, 0, 0, 0, 0, 0, 0, 9,  // u64
                10, //u8
            ]
        );
    }

    // @extensibility(APPENDABLE) @nested
    struct AppendableType {
        value: u16,
    }
    impl XTypesSerialize for AppendableType {
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
            let mut serializer = serializer.serialize_appendable_struct()?;
            serializer.serialize_field(&self.value, "value")
        }
    }

    #[test]
    fn serialize_appendable_struct() {
        let v = AppendableType { value: 7 };
        // PLAIN_CDR:
        assert_eq!(serialize_v1_be(&v), [0, 7]);
        assert_eq!(serialize_v1_le(&v), [7, 0]);
        // DELIMITED_CDR:
        assert_eq!(
            serialize_v2_be(&v),
            [
                0, 0, 0, 2, // DHEADER
                0, 7 // value
            ]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [
                2, 0, 0, 0, // DHEADER
                7, 0 // value
            ]
        );
    }

    //@extensibility(MUTABLE)
    struct MutableType {
        // @id(0x005A) @key
        key: u8,
        // @id(0x0050)
        participant_key: u32,
    }
    impl XTypesSerialize for MutableType {
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
            let mut s = serializer.serialize_mutable_struct()?;
            s.serialize_field(&self.key, 0x005A, "key")?;
            s.serialize_field(&self.participant_key, 0x0050, "participant_key")?;
            s.end()
        }
    }

    #[test]
    fn serialize_mutable_struct() {
        let v = MutableType {
            key: 7,
            participant_key: 8,
        };
        // PL_CDR:
        assert_eq!(
            serialize_v1_be(&v),
            [
                0x00, 0x05A, 0, 1, // PID | length
                7, 0, 0, 0, // key | padding
                0x00, 0x050, 0, 4, // PID | length
                0, 0, 0, 8, // participant_key
                1, 0, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_v1_le(&v),
            [
                0x05A, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // key | padding
                0x050, 0x00, 4, 0, // PID | length
                8, 0, 0, 0, // participant_key
                1, 0, 0, 0, // Sentinel
            ]
        );
        // PL_CDR2:
        assert_eq!(
            serialize_v2_be(&v),
            [
                0x00, 0x05A, 0, 1, // PID | length
                7, 0, 0, 0, // key | padding
                0x00, 0x050, 0, 4, // PID | length
                0, 0, 0, 8, // participant_key
                1, 0, 0, 0, // Sentinel
            ]
        );
        assert_eq!(
            serialize_v2_le(&v),
            [
                0x05A, 0x00, 1, 0, // PID | length
                7, 0, 0, 0, // key | padding
                0x050, 0x00, 4, 0, // PID | length
                8, 0, 0, 0, // participant_key
                1, 0, 0, 0, // Sentinel
            ]
        );
    }
}
