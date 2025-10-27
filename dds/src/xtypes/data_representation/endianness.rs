use crate::xtypes::{
    error::{XTypesError, XTypesResult},
    read_write::Read,
};

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
    fn read_char<R: Read>(reader: &mut R) -> XTypesResult<char> {
        Ok(char::from(reader.read_exact(1)?[0]))
    }
}

pub struct BigEndian;

impl EndiannessRead for BigEndian {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Ok(i16::from_be_bytes(*reader.read_array::<2>()?))
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Ok(u16::from_be_bytes(*reader.read_array::<2>()?))
    }

    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32> {
        Ok(i32::from_be_bytes(*reader.read_array::<4>()?))
    }

    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32> {
        Ok(u32::from_be_bytes(*reader.read_array::<4>()?))
    }

    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64> {
        Ok(i64::from_be_bytes(*reader.read_array::<8>()?))
    }

    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64> {
        Ok(u64::from_be_bytes(*reader.read_array::<8>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_be_bytes(*reader.read_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_be_bytes(*reader.read_array::<8>()?))
    }
}

pub struct LittleEndian;

impl EndiannessRead for LittleEndian {
    fn read_i16<R: Read>(reader: &mut R) -> XTypesResult<i16> {
        Ok(i16::from_le_bytes(*reader.read_array::<2>()?))
    }

    fn read_u16<R: Read>(reader: &mut R) -> XTypesResult<u16> {
        Ok(u16::from_le_bytes(*reader.read_array::<2>()?))
    }

    fn read_i32<R: Read>(reader: &mut R) -> XTypesResult<i32> {
        Ok(i32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_u32<R: Read>(reader: &mut R) -> XTypesResult<u32> {
        Ok(u32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_i64<R: Read>(reader: &mut R) -> XTypesResult<i64> {
        Ok(i64::from_le_bytes(*reader.read_array::<8>()?))
    }

    fn read_u64<R: Read>(reader: &mut R) -> XTypesResult<u64> {
        Ok(u64::from_le_bytes(*reader.read_array::<8>()?))
    }

    fn read_f32<R: Read>(reader: &mut R) -> XTypesResult<f32> {
        Ok(f32::from_le_bytes(*reader.read_array::<4>()?))
    }

    fn read_f64<R: Read>(reader: &mut R) -> XTypesResult<f64> {
        Ok(f64::from_le_bytes(*reader.read_array::<8>()?))
    }
}
