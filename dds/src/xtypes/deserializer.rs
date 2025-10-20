use crate::xtypes::{dynamic_type::DynamicData, error::XTypesResult};

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
    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32>;
    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32>;
    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64>;
    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64>;
    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32>;
    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64>;
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

    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32> {
        Ok(i32::from_be_bytes(*reader.read_exact_array::<4>()?))
    }

    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32> {
        Ok(u32::from_be_bytes(*reader.read_exact_array::<4>()?))
    }

    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64> {
        Ok(i64::from_be_bytes(*reader.read_exact_array::<8>()?))
    }

    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64> {
        Ok(u64::from_be_bytes(*reader.read_exact_array::<8>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_be_bytes(*reader.read_exact_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_be_bytes(*reader.read_exact_array::<8>()?))
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

    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32> {
        Ok(i32::from_le_bytes(*reader.read_exact_array::<4>()?))
    }

    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32> {
        Ok(u32::from_le_bytes(*reader.read_exact_array::<4>()?))
    }

    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64> {
        Ok(i64::from_le_bytes(*reader.read_exact_array::<8>()?))
    }

    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64> {
        Ok(u64::from_le_bytes(*reader.read_exact_array::<8>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_le_bytes(*reader.read_exact_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_le_bytes(*reader.read_exact_array::<8>()?))
    }
}

/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait XTypesDeserializer<'de>: Sized {
    fn deserialize_final_struct(&mut self, v: &mut DynamicData) -> Result<(), XTypesError>;
    fn deserialize_appendable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_mutable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_array(&mut self) -> Result<(), XTypesError>;
    fn deserialize_sequence(&mut self) -> Result<(), XTypesError>;

    /// Deserialize a [`&str`] value.
    fn deserialize_string(&mut self) -> Result<&'de str, XTypesError>;

    /// Deserialize a variable sized sequence of bytes by borrowing.
    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], XTypesError>;

    /// Deserialize an array of bytes by borrowing.
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], XTypesError>;
}
