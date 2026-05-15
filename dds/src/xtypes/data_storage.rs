use std::str::FromStr;

use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::DynamicData,
        error::{XTypesError, XTypesResult},
    },
};
use alloc::{string::String, vec::Vec};

#[derive(Clone, Copy, Debug)]
struct U256 {
    hi: u128,
    lo: u128,
}

impl U256 {
    fn from_u128(v: u128) -> Self {
        Self { hi: 0, lo: v }
    }

    fn shl1(self) -> Self {
        Self {
            hi: (self.hi << 1) | (self.lo >> 127),
            lo: self.lo << 1,
        }
    }

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.hi.cmp(&other.hi) {
            std::cmp::Ordering::Equal => self.lo.cmp(&other.lo),
            x => x,
        }
    }

    fn sub(self, other: Self) -> Self {
        let (lo, borrow) = self.lo.overflowing_sub(other.lo);

        Self {
            hi: self.hi - other.hi - (borrow as u128),
            lo,
        }
    }

    fn add(self, other: Self) -> Self {
        let (lo, carry) = self.lo.overflowing_add(other.lo);

        Self {
            hi: self.hi + other.hi + (carry as u128),
            lo,
        }
    }

    fn mul10(self) -> Self {
        // x * 10 = x*8 + x*2
        let x2 = self.shl1();
        let x8 = x2.shl1().shl1();
        x8.add(x2)
    }

    fn pow10(exp: usize) -> Self {
        let mut v = Self::from_u128(1);

        for _ in 0..exp {
            v = v.mul10();
        }

        v
    }

    fn leading_bit(self) -> i32 {
        if self.hi != 0 {
            128 + (127 - self.hi.leading_zeros() as i32)
        } else {
            127 - self.lo.leading_zeros() as i32
        }
    }
}

#[derive(Debug, Clone, PartialEq, TypeSupport)]
pub struct F128(pub [u8; 16]);

impl FromStr for F128 {
    type Err = XTypesError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        const EXP_BIAS: i32 = 16383;

        let input = input.trim();

        if input.is_empty() {
            return Err(XTypesError::InvalidData);
        }

        let negative = input.starts_with('-');
        let input = input.trim_start_matches(&['+', '-']);

        let (base, exp10_part) = match input.find(['e', 'E']) {
            Some(pos) => (&input[..pos], &input[pos + 1..]),
            None => (input, "0"),
        };

        let exp10_extra: i32 = exp10_part.parse().map_err(|_| XTypesError::InvalidData)?;

        let parts: Vec<_> = base.split('.').collect();

        if parts.len() > 2 {
            return Err(XTypesError::InvalidData);
        }

        let int_part = parts[0];
        let frac_part = if parts.len() == 2 { parts[1] } else { "" };

        let scale = frac_part.len() as i32 - exp10_extra;

        let digits = format!("{}{}", int_part, frac_part);

        let mut mantissa: u128 = 0;

        for ch in digits.bytes() {
            if !ch.is_ascii_digit() {
                return Err(XTypesError::InvalidData);
            }

            mantissa = mantissa.checked_mul(10).ok_or(XTypesError::InvalidData)?;

            mantissa += (ch - b'0') as u128;
        }

        if mantissa == 0 {
            return Ok(if negative {
                Self((1u128 << 127).to_be_bytes())
            } else {
                Self([0; 16])
            });
        }

        let mut num = U256::from_u128(mantissa);
        let mut den = U256::from_u128(1);

        if scale > 0 {
            den = U256::pow10(scale as usize);
        } else if scale < 0 {
            num = num.mul10();
            for _ in 1..(-scale as usize) {
                num = num.mul10();
            }
        }

        // Normalize into [1,2)
        let mut exp2 = num.leading_bit() - den.leading_bit();

        if exp2 > 0 {
            for _ in 0..exp2 {
                den = den.shl1();
            }
        } else {
            for _ in 0..(-exp2) {
                num = num.shl1();
            }
        }

        while num.cmp(&den) == std::cmp::Ordering::Less {
            num = num.shl1();
            exp2 -= 1;
        }

        while num.cmp(&den.shl1()) != std::cmp::Ordering::Less {
            den = den.shl1();
            exp2 += 1;
        }

        num = num.sub(den);

        // Generate fraction bits
        let mut frac: u128 = 0;

        for i in (0..112).rev() {
            num = num.shl1();

            if num.cmp(&den) != std::cmp::Ordering::Less {
                num = num.sub(den);
                frac |= 1u128 << i;
            }
        }

        // Guard bit
        num = num.shl1();

        let guard = num.cmp(&den) != std::cmp::Ordering::Less;

        if guard {
            num = num.sub(den);
        }

        let sticky = num.hi != 0 || num.lo != 0;

        // Round-to-nearest-even
        if guard && (sticky || (frac & 1) != 0) {
            frac += 1;

            if frac >> 112 != 0 {
                frac &= (1u128 << 112) - 1;
                exp2 += 1;
            }
        }

        let biased_exp = exp2 + EXP_BIAS;

        if biased_exp <= 0 {
            return Err(XTypesError::InvalidData);
        }

        if biased_exp >= 0x7fff {
            return Err(XTypesError::InvalidData);
        }

        let bits = ((negative as u128) << 127) | ((biased_exp as u128) << 112) | frac;

        Ok(Self(bits.to_be_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataStorage {
    UInt8(u8),
    Int8(i8),
    UInt16(u16),
    Int16(i16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Float128(F128),
    Char8(char),
    Boolean(bool),
    String(String),
    ComplexValue(DynamicData),
    // Sequence
    SequenceUInt8(Vec<u8>),
    SequenceInt8(Vec<i8>),
    SequenceUInt16(Vec<u16>),
    SequenceInt16(Vec<i16>),
    SequenceInt32(Vec<i32>),
    SequenceUInt32(Vec<u32>),
    SequenceInt64(Vec<i64>),
    SequenceUInt64(Vec<u64>),
    SequenceFloat32(Vec<f32>),
    SequenceFloat64(Vec<f64>),
    SequenceFloat128(Vec<F128>),
    SequenceChar8(Vec<char>),
    SequenceBoolean(Vec<bool>),
    SequenceString(Vec<String>),
    SequenceComplexValue(Vec<DynamicData>),
}

pub trait DataStorageMapping: Sized {
    fn into_storage(self) -> DataStorage;

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self>;
}

impl DataStorageMapping for u8 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i8 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for u16 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i16 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for u32 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i32 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for u64 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i64 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for f32 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Float32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Float32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for f64 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Float64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Float64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for bool {
    fn into_storage(self) -> DataStorage {
        DataStorage::Boolean(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Boolean(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for char {
    fn into_storage(self) -> DataStorage {
        DataStorage::Char8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Char8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for String {
    fn into_storage(self) -> DataStorage {
        DataStorage::String(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::String(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

// SequenceChar8(Vec<char>),
// SequenceBoolean(Vec<bool>),
// SequenceString(Vec<String>),
// SequenceComplexValue(Vec<DynamicData>),

impl DataStorageMapping for Vec<u8> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i8> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<u16> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i16> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<u32> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i32> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<u64> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i64> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<f32> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<f64> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<char> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceChar8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceChar8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<bool> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceBoolean(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceBoolean(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<String> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceString(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceString(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u8; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt8(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt8(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i8; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt8(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt8(x) => Self::try_from(x).map_err(|_| XTypesError::InvalidType),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u16; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt16(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt16(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i16; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt16(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt16(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u32; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt32(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt32(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i32; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt32(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt32(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u64; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt64(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt64(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i64; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt64(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt64(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [f32; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat32(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat32(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [f64; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat64(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat64(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [char; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceChar8(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceChar8(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [bool; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceBoolean(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceBoolean(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [String; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceString(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceString(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: TypeSupport> DataStorageMapping for T {
    fn into_storage(self) -> DataStorage {
        DataStorage::ComplexValue(T::create_dynamic_sample(self))
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::ComplexValue(x) => Ok(T::create_sample(x)),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: TypeSupport> DataStorageMapping for Vec<T> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(self.into_iter().map(T::create_dynamic_sample).collect())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Ok(x.into_iter().map(T::create_sample).collect())
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize, T: TypeSupport> DataStorageMapping for [T; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(self.into_iter().map(T::create_dynamic_sample).collect())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Self::try_from(x.into_iter().map(T::create_sample).collect::<Vec<_>>())
                    .map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}
