use crate::xtypes::{
    data_representation::DataKind,
    dynamic_type::{DynamicData, DynamicType},
    error::XTypesResult,
};

use super::error::XTypesError;

pub trait Read {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]>;

    fn pos(&self) -> usize;

    fn read_exact_array<const N: usize>(&mut self) -> XTypesResult<&[u8; N]> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_e| XTypesError::InvalidData)
    }
}

impl Read for &[u8] {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]> {
        todo!()
    }

    fn pos(&self) -> usize {
        todo!()
    }
}

pub trait EndiannessRead {
    fn read_bool<R: Read>(reader: &mut R) -> XTypesResult<bool> {
        let buf = reader.read_exact(1)?;
        match buf[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(XTypesError::InvalidData),
        }
    }

    fn read_i8<R: Read>(reader: &mut R) -> XTypesResult<i8> {
        Ok(reader.read_exact(1)?[0] as i8)
    }

    fn read_u8<R: Read>(reader: &mut R) -> XTypesResult<u8> {
        Ok(reader.read_exact(1)?[0])
    }

    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16>;
    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16>;
    // fn write_i32<C: Write>(v: i32, writer: &mut C);
    // fn write_u32<C: Write>(v: u32, writer: &mut C);
    // fn write_i64<C: Write>(v: i64, writer: &mut C);
    // fn write_u64<C: Write>(v: u64, writer: &mut C);
    // fn write_f32<C: Write>(v: f32, writer: &mut C);
    // fn write_f64<C: Write>(v: f64, writer: &mut C);
    // fn write_char<C: Write>(v: char, writer: &mut C) {
    //     writer.write(v.to_string().as_bytes());
    // }
    // fn write_str<C: Write>(v: &str, writer: &mut C) {
    //     writer.write(v.as_bytes());
    // }
    // fn write_slice_u8<C: Write>(v: &[u8], writer: &mut C) {
    //     writer.write(v);
    // }
}

pub struct BigEndian;

impl EndiannessRead for BigEndian {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Ok(i16::from_be_bytes(*reader.read_exact_array::<2>()?))
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Ok(u16::from_be_bytes(*reader.read_exact_array::<2>()?))
    }
}

pub struct LittleEndian;

impl EndiannessRead for LittleEndian {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Ok(i16::from_le_bytes(*reader.read_exact_array::<2>()?))
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Ok(u16::from_le_bytes(*reader.read_exact_array::<2>()?))
    }
}

pub trait PaddingRead {
    fn pad(alignment: usize, reader: &mut impl Read) -> XTypesResult<()>;
}

pub struct PaddingV1;

impl PaddingRead for PaddingV1 {
    fn pad(alignment: usize, reader: &mut impl Read) -> XTypesResult<()> {
        let mask = alignment - 1;
        let pad_length = ((reader.pos() + mask) & !mask) - reader.pos();
        reader.read_exact(pad_length)?;
        Ok(())
    }
}

pub trait PadEndiannessRead {
    type Endianness: EndiannessRead;
    type Padding: PaddingRead;

    fn read_bool<R: Read>(reader: &mut R) -> XTypesResult<bool> {
        Self::Endianness::read_bool(reader)
    }

    fn read_i8<R: Read>(reader: &mut R) -> XTypesResult<i8> {
        Self::Endianness::read_i8(reader)
    }

    fn read_u8<R: Read>(reader: &mut R) -> XTypesResult<u8> {
        Self::Endianness::read_u8(reader)
    }

    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Self::Padding::pad(2, reader);
        Self::Endianness::read_i16(reader)
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Self::Padding::pad(2, reader);
        Self::Endianness::read_u16(reader)
    }
}

/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait XTypesDeserializer<'de>: Sized {
    fn deserialize_final_struct(&mut self, v: &mut DynamicData) -> Result<(), XTypesError>;
    fn deserialize_appendable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_mutable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_array(&mut self) -> Result<(), XTypesError>;
    fn deserialize_sequence(&mut self) -> Result<(), XTypesError>;

    /// Deserialize a [`bool`] value.
    fn deserialize_boolean(&mut self) -> Result<bool, XTypesError>;

    /// Deserialize an [`i8`] value.
    fn deserialize_int8(&mut self) -> Result<i8, XTypesError>;

    /// Deserialize an [`i16`] value.
    fn deserialize_int16(&mut self) -> Result<i16, XTypesError>;

    /// Deserialize an [`i32`] value.
    fn deserialize_int32(&mut self) -> Result<i32, XTypesError>;

    /// Deserialize an [`i64`] value.
    fn deserialize_int64(&mut self) -> Result<i64, XTypesError>;

    /// Deserialize a [`u8`] value.
    fn deserialize_uint8(&mut self) -> Result<u8, XTypesError>;

    /// Deserialize a [`u16`] value.
    fn deserialize_uint16(&mut self) -> Result<u16, XTypesError>;

    /// Deserialize a [`u32`] value.
    fn deserialize_uint32(&mut self) -> Result<u32, XTypesError>;

    /// Deserialize a [`u64`] value.
    fn deserialize_uint64(&mut self) -> Result<u64, XTypesError>;

    /// Deserialize an [`f32`] value.
    fn deserialize_float32(&mut self) -> Result<f32, XTypesError>;

    /// Deserialize an [`f64`] value.
    fn deserialize_float64(&mut self) -> Result<f64, XTypesError>;

    /// Deserialize a [`char`] value.
    fn deserialize_char8(&mut self) -> Result<char, XTypesError>;

    /// Deserialize a [`&str`] value.
    fn deserialize_string(&mut self) -> Result<&'de str, XTypesError>;

    /// Deserialize a variable sized sequence of bytes by borrowing.
    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], XTypesError>;

    /// Deserialize an array of bytes by borrowing.
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], XTypesError>;

    fn deserialize_data_kind(&mut self, dynamic_type: &DynamicType) -> XTypesResult<DataKind>;
}
