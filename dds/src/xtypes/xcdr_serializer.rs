use super::{
    error::XTypesError,
    serialize::XTypesSerialize,
    serializer::{
        SerializeAppendableStruct, SerializeCollection, SerializeFinalStruct,
        SerializeMutableStruct, XTypesSerializer,
    },
};

const PID_SENTINEL: u16 = 1;

pub struct VirtualCdr1Serializer {
    count: usize,
}

impl VirtualCdr1Serializer {
    fn add<T>(&mut self, v: T) -> Result<(), XTypesError> {
        self.count += core::mem::size_of_val(&v);
        Ok(())
    }
}

impl SerializeFinalStruct for &mut VirtualCdr1Serializer {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        self.count += 4; // header: pid + length
        if let Some(value) = value {
            XTypesSerialize::serialize(value, &mut **self)
        } else {
            Ok(())
        }
    }
}
impl SerializeAppendableStruct for &mut VirtualCdr1Serializer {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }
}
impl SerializeMutableStruct for &mut VirtualCdr1Serializer {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _pid: u16,
        _name: &str,
    ) -> Result<(), XTypesError> {
        self.count += 4; // header: pid + length
        XTypesSerialize::serialize(value, &mut **self)
        // todo: padding?
    }

    fn end(self) -> Result<(), XTypesError> {
        self.count += 4; // sentinel
        Ok(())
    }
}
impl SerializeCollection for &mut VirtualCdr1Serializer {
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }
}

impl XTypesSerializer for &mut VirtualCdr1Serializer {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }

    fn serialize_sequence(self, _len: usize) -> Result<impl SerializeCollection, XTypesError> {
        Ok(self)
    }

    fn serialize_array(self) -> Result<impl SerializeCollection, XTypesError> {
        Ok(self)
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_char8(self, v: char) -> Result<(), XTypesError> {
        self.add(v)
    }

    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.count += 4 + v.len() + 1;
        Ok(())
    }

    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XTypesError> {
        self.count += 4 + v.len();
        Ok(())
    }

    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XTypesError> {
        self.count += v.len();
        Ok(())
    }
}


struct CollectionWriter<'a, C> {
    collection: &'a mut C,
    position: usize,
}

impl<'a, C> CollectionWriter<'a, C>
where
    for<'b> C: Extend<&'b u8>,
{
    fn new(collection: &'a mut C) -> Self {
        Self {
            collection,
            position: 0,
        }
    }

    fn write_slice(&mut self, data: &[u8]) {
        self.collection.extend(data.iter());
        self.position += data.len();
    }

    fn pad(&mut self, alignment: usize) {
        const ZEROS: [u8; 8] = [0; 8];
        let over_alignment = self.position & (alignment - 1);
        if over_alignment > 0 {
            self.write_slice(&ZEROS[..alignment - over_alignment]);
        };
    }
}

fn extend_with_padding_v1<const N: usize, C>(
    writer: &mut CollectionWriter<'_, C>,
    data: &[u8; N],
) -> Result<(), XTypesError>
where
    for<'a> C: Extend<&'a u8>,
{
    writer.pad(N);
    writer.write_slice(data);
    Ok(())
}

pub struct NewXcdr1BeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C> NewXcdr1BeSerializer<'a, C>
where
    for<'b> C: Extend<&'b u8>,
{
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
        }
    }
}

impl<C> SerializeFinalStruct for &mut NewXcdr1BeSerializer<'_, C>
where
    for<'b> C: Extend<&'b u8>,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        if let Some(value) = value {
            let mut virtual_serializer = VirtualCdr1Serializer { count: 0 };
            XTypesSerialize::serialize(value, &mut virtual_serializer)?;
            let length = virtual_serializer.count as u16;
            self.writer.pad(4);
            self.writer.write_slice(&0_u16.to_be_bytes());
            self.writer.write_slice(&length.to_be_bytes());
            XTypesSerialize::serialize(value, &mut **self)
        } else {
            self.writer.pad(4);
            self.writer.write_slice(&0_u16.to_be_bytes());
            self.writer.write_slice(&0_u16.to_be_bytes());
            Ok(())
        }
    }
}
impl<C> SerializeAppendableStruct for &mut NewXcdr1BeSerializer<'_, C>
where
    for<'b> C: Extend<&'b u8>,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }
}
impl<C> SerializeMutableStruct for &mut NewXcdr1BeSerializer<'_, C>
where
    for<'a> C: Extend<&'a u8>,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        _name: &str,
    ) -> Result<(), XTypesError> {
        let mut virtual_serializer = VirtualCdr1Serializer { count: 0 };
        XTypesSerialize::serialize(value, &mut virtual_serializer)?;
        let length = virtual_serializer.count as u16;
        self.writer.write_slice(&pid.to_be_bytes());
        self.writer.write_slice(&length.to_be_bytes());
        XTypesSerialize::serialize(value, &mut **self)?;
        self.writer.pad(4);
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_be_bytes());
        self.writer.write_slice(&0u16.to_be_bytes());
        Ok(())
    }
}
impl<C> SerializeCollection for &mut NewXcdr1BeSerializer<'_, C>
where
    for<'a> C: Extend<&'a u8>,
{
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }
}

impl<C> XTypesSerializer for &mut NewXcdr1BeSerializer<'_, C>
where
    for<'a> C: Extend<&'a u8>,
{
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XTypesError> {
        self.serialize_uint32(into_u32(len)?)?;
        Ok(self)
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XTypesError> {
        Ok(self)
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        self.serialize_uint8(v as u8)
    }

    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &[v as u8])
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
        self.serialize_uint8(into_u8(v)?)
    }

    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes());
        self.writer.write_slice(&[0]);
        Ok(())
    }

    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XTypesError> {
        self.serialize_uint32(into_u32(v.len())?)?;
        self.writer.write_slice(v);
        Ok(())
    }

    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XTypesError> {
        self.writer.write_slice(v);
        Ok(())
    }
}

pub struct NewXcdr1LeSerializer<'a, C> {
    writer: CollectionWriter<'a, C>,
}

impl<'a, C> NewXcdr1LeSerializer<'a, C>
where
    for<'b> C: Extend<&'b u8>,
{
    pub fn new(collection: &'a mut C) -> Self {
        Self {
            writer: CollectionWriter::new(collection),
        }
    }
}

impl<C> SerializeFinalStruct for &mut NewXcdr1LeSerializer<'_, C>
where
    for<'b> C: Extend<&'b u8>,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XTypesError> {
        if let Some(value) = value {
            let mut virtual_serializer = VirtualCdr1Serializer { count: 0 };
            XTypesSerialize::serialize(value, &mut virtual_serializer)?;
            let length = virtual_serializer.count as u16;
            self.writer.pad(4);
            self.writer.write_slice(&0_u16.to_le_bytes());
            self.writer.write_slice(&length.to_le_bytes());
            XTypesSerialize::serialize(value, &mut **self)
        } else {
            self.writer.pad(4);
            self.writer.write_slice(&0_u16.to_le_bytes());
            self.writer.write_slice(&0_u16.to_le_bytes());
            Ok(())
        }
    }
}
impl<C> SerializeAppendableStruct for &mut NewXcdr1LeSerializer<'_, C>
where
    for<'b> C: Extend<&'b u8>,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        _name: &str,
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }
}
impl<C> SerializeMutableStruct for &mut NewXcdr1LeSerializer<'_, C>
where
    for<'a> C: Extend<&'a u8>,
{
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        _name: &str,
    ) -> Result<(), XTypesError> {
        let mut virtual_serializer = VirtualCdr1Serializer { count: 0 };
        XTypesSerialize::serialize(value, &mut virtual_serializer)?;
        let length = virtual_serializer.count as u16;
        self.writer.write_slice(&pid.to_le_bytes());
        self.writer.write_slice(&length.to_le_bytes());
        XTypesSerialize::serialize(value, &mut **self)?;
        self.writer.pad(4);
        Ok(())
    }

    fn end(self) -> Result<(), XTypesError> {
        self.writer.write_slice(&PID_SENTINEL.to_le_bytes());
        self.writer.write_slice(&0u16.to_le_bytes());
        Ok(())
    }
}
impl<C> SerializeCollection for &mut NewXcdr1LeSerializer<'_, C>
where
    for<'a> C: Extend<&'a u8>,
{
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut **self)
    }
}

impl<C> XTypesSerializer for &mut NewXcdr1LeSerializer<'_, C>
where
    for<'a> C: Extend<&'a u8>,
{
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(self)
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XTypesError> {
        self.serialize_uint32(into_u32(len)?)?;
        Ok(self)
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XTypesError> {
        Ok(self)
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        self.serialize_uint8(v as u8)
    }

    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        extend_with_padding_v1(&mut self.writer, &[v as u8])
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
        self.serialize_uint8(into_u8(v)?)
    }

    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes());
        self.writer.write_slice(&[0]);
        Ok(())
    }

    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XTypesError> {
        self.serialize_uint32(into_u32(v.len())?)?;
        self.writer.write_slice(v);
        Ok(())
    }

    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XTypesError> {
        self.writer.write_slice(v);
        Ok(())
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

    fn write_slice(&mut self, data: &[u8]) -> Result<(), XTypesError> {
        let end = self.pos + data.len();
        if end > self.data.len() {
            Err(XTypesError::OutOfMemory)
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

    fn pad(&mut self, alignment: usize) -> Result<(), XTypesError> {
        const ZEROS: [u8; 8] = [0; 8];
        let over_alignment = self.pos & (alignment - 1);
        if over_alignment > 0 {
            self.write_slice(&ZEROS[..alignment - over_alignment])?;
        };
        Ok(())
    }
}

fn write_with_padding_v2<const N: usize>(
    writer: &mut Writer,
    data: &[u8; N],
) -> Result<(), XTypesError> {
    writer.pad(core::cmp::min(N, 4))?;
    writer.write_slice(data)
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
const SENTINEL: [u8; 4] = [1u8, 0, 0, 0];

fn write_pl_header_be(writer: &mut Writer, header_pos: usize, pid: u16) -> Result<(), XTypesError> {
    let pos = writer.position();
    let length = (pos - header_pos - 4) as u16;
    writer.set_position(header_pos);
    writer.write_slice(&pid.to_be_bytes())?;
    writer.write_slice(&length.to_be_bytes())?;
    writer.set_position(pos);
    Ok(())
}
fn write_pl_header_le(writer: &mut Writer, header_pos: usize, pid: u16) -> Result<(), XTypesError> {
    let pos = writer.position();
    let length = (pos - header_pos - 4) as u16;
    writer.set_position(header_pos);
    writer.write_slice(&pid.to_le_bytes())?;
    writer.write_slice(&length.to_le_bytes())?;
    writer.set_position(pos);
    Ok(())
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
    ) -> Result<(), XTypesError> {
        let header_pos = self.serializer.writer.position();
        self.serializer.writer.set_position(header_pos + 4);
        value.serialize(&mut *self.serializer)?;
        write_pl_header_be(&mut self.serializer.writer, header_pos, pid)?;
        self.serializer.writer.pad(4)
    }
    fn end(self) -> Result<(), XTypesError> {
        SENTINEL.serialize(self.serializer)
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
    ) -> Result<(), XTypesError> {
        let header_pos = self.serializer.writer.position();
        self.serializer.writer.set_position(header_pos + 4);
        value.serialize(&mut *self.serializer)?;
        write_pl_header_le(&mut self.serializer.writer, header_pos, pid)?;
        self.serializer.writer.pad(4)
    }
    fn end(self) -> Result<(), XTypesError> {
        SENTINEL.serialize(self.serializer)
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
    ) -> Result<(), XTypesError> {
        XTypesSerialize::serialize(value, &mut *self.serializer)
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        _name: &str,
    ) -> Result<(), XTypesError> {
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
    ) -> Result<(), XTypesError> {
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
    ) -> Result<(), XTypesError> {
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
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XTypesError> {
        value.serialize(&mut *self.serializer)
    }
}

impl XTypesSerializer for &mut Xcdr2BeSerializer<'_> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(DelimitedCdrBeEncoder {
            serializer: Xcdr2BeSerializer::new(self.writer.data),
        })
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(PlCdr2BeEncoder { serializer: self })
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XTypesError> {
        write_with_padding_v2(&mut self.writer, &into_u32(len)?.to_be_bytes())?;
        Ok(CollectionSerializer { serializer: self })
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XTypesError> {
        Ok(CollectionSerializer { serializer: self })
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_be_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &[into_u8(v)?])
    }
    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes())?;
        self.writer.write_slice(&[0])
    }
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XTypesError> {
        self.writer.write_slice(&into_u32(v.len())?.to_be_bytes())?;
        self.writer.write_slice(v)
    }
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XTypesError> {
        self.writer.write_slice(v)
    }
}

impl<'a> XTypesSerializer for &'a mut Xcdr2LeSerializer<'_> {
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError> {
        Ok(PlainCdr2Encoder { serializer: self })
    }
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError> {
        Ok(DelimitedCdrLeEncoder {
            serializer: Xcdr2LeSerializer::new(self.writer.data),
        })
    }
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError> {
        Ok(PlCdr2LeEncoder { serializer: self })
    }
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XTypesError> {
        write_with_padding_v2(&mut self.writer, &into_u32(len)?.to_le_bytes())?;
        Ok(CollectionSerializer { serializer: self })
    }
    fn serialize_array(self) -> Result<impl SerializeCollection, XTypesError> {
        Ok(CollectionSerializer { serializer: self })
    }

    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError> {
        self.serialize_uint8(v as u8)
    }
    fn serialize_int8(self, v: i8) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int16(self, v: i16) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int32(self, v: i32) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_int64(self, v: i64) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float32(self, v: f32) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_float64(self, v: f64) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &v.to_le_bytes())
    }
    fn serialize_char8(self, v: char) -> Result<(), XTypesError> {
        write_with_padding_v2(&mut self.writer, &[into_u8(v)?])
    }
    fn serialize_string(self, v: &str) -> Result<(), XTypesError> {
        self.serialize_uint32(str_len(v)?)?;
        self.writer.write_slice(v.as_bytes())?;
        self.writer.write_slice(&[0])
    }
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XTypesError> {
        self.writer.write_slice(&into_u32(v.len())?.to_le_bytes())?;
        self.writer.write_slice(v)
    }
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XTypesError> {
        self.writer.write_slice(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    fn serialize_v1_be<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = std::vec::Vec::new();
        v.serialize(&mut NewXcdr1BeSerializer::new(&mut buffer))
            .unwrap();
        buffer.try_into().unwrap()
    }

    fn serialize_v1_le<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = std::vec::Vec::new();
        v.serialize(&mut NewXcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        buffer.try_into().unwrap()
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
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
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
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
            let mut serializer = serializer.serialize_final_struct()?;
            serializer.serialize_field(&self.field, "field")?;
            serializer.serialize_optional_field(&self.optional_field, "optional_field")
        }
    }
    #[test]
    fn serialize_final_optional_struct_some() {
        let some = FinalOptionalType {
            field: 6,
            optional_field: Some(7),
        };
        //PLAIN_CDR:
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
        //PLAIN_CDR2:
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
    }
    #[test]
    fn serialize_final_optional_struct_none() {
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
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
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
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
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
        fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XTypesError> {
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
                0, 1, 0, 0, // Sentinel
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
