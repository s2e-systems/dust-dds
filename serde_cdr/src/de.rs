//! Deserializing CDR into Rust data types.

use std::{self, io::Read, marker::PhantomData};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use serde::de::{self, IntoDeserializer};

use crate::error::{Error, Result};
use crate::size::{Infinite, SizeLimit};

/// A deserializer that reads bytes from a buffer.
pub struct Deserializer<R, S, E> {
    reader: R,
    size_limit: S,
    pos: u64,
    phantom: PhantomData<E>,
}

impl<R, S, E> Deserializer<R, S, E>
where
    R: Read,
    S: SizeLimit,
    E: ByteOrder,
{
    pub fn new(reader: R, size_limit: S) -> Self {
        Self {
            reader,
            size_limit,
            pos: 0,
            phantom: PhantomData,
        }
    }

    fn read_padding_of<T>(&mut self) -> Result<()> {
        // Calculate the required padding to align with 1-byte, 2-byte, 4-byte, 8-byte boundaries
        // Instead of using the slow modulo operation '%', the faster bit-masking is used
        let alignment = std::mem::size_of::<T>();
        let rem_mask = alignment - 1; // mask like 0x0, 0x1, 0x3, 0x7
        let mut padding: [u8; 8] = [0; 8];
        match (self.pos as usize) & rem_mask {
            0 => Ok(()),
            n @ 1..=7 => {
                let amt = alignment - n;
                self.read_size(amt as u64)?;
                self.reader
                    .read_exact(&mut padding[..amt])
                    .map_err(Into::into)
            }
            _ => unreachable!(),
        }
    }

    fn read_size(&mut self, size: u64) -> Result<()> {
        self.pos += size;
        self.size_limit.add(size)
    }

    fn read_size_of<T>(&mut self) -> Result<()> {
        self.read_size(std::mem::size_of::<T>() as u64)
    }

    fn read_string(&mut self) -> Result<String> {
        String::from_utf8(self.read_bytes().map(|mut v| {
            v.pop(); // removes a terminating null character
            v
        })?)
        .map_err(|e| Error::InvalidUtf8Encoding(e.utf8_error()))
    }

    fn read_bytes(&mut self) -> Result<Vec<u8>> {
        let len: u32 = de::Deserialize::deserialize(&mut *self)?;
        let mut buf = vec![0_u8; len as usize];
        self.read_size(u64::from(len))?;
        self.reader.read_exact(&mut buf[..])?;
        Ok(buf)
    }

    pub(crate) fn reset_pos(&mut self) {
        self.pos = 0;
    }
}

macro_rules! impl_deserialize_value {
    ($de_method:ident<$ty:ty> = $visitor_method:ident ($reader_method:ident)) => {
        fn $de_method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.read_padding_of::<$ty>()?;
            self.read_size_of::<$ty>()?;
            visitor.$visitor_method(self.reader.$reader_method::<E>()?)
        }
    };
}

impl<'de, 'a, R, S, E> de::Deserializer<'de> for &'a mut Deserializer<R, S, E>
where
    R: Read,
    S: SizeLimit,
    E: ByteOrder,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::DeserializeAnyNotSupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let value: u8 = de::Deserialize::deserialize(self)?;
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            value => Err(Error::InvalidBoolEncoding(value)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.read_size_of::<i8>()?;
        visitor.visit_i8(self.reader.read_i8()?)
    }

    impl_deserialize_value!(deserialize_i16<i16> = visit_i16(read_i16));
    impl_deserialize_value!(deserialize_i32<i32> = visit_i32(read_i32));
    impl_deserialize_value!(deserialize_i64<i64> = visit_i64(read_i64));

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.read_size_of::<u8>()?;
        visitor.visit_u8(self.reader.read_u8()?)
    }

    impl_deserialize_value!(deserialize_u16<u16> = visit_u16(read_u16));
    impl_deserialize_value!(deserialize_u32<u32> = visit_u32(read_u32));
    impl_deserialize_value!(deserialize_u64<u64> = visit_u64(read_u64));

    impl_deserialize_value!(deserialize_f32<f32> = visit_f32(read_f32));
    impl_deserialize_value!(deserialize_f64<f64> = visit_f64(read_f64));

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf[..1])?;

        if utf8_char_width(buf[0]) != 1 {
            Err(Error::InvalidCharEncoding)
        } else {
            self.read_size(1)?;
            visitor.visit_char(buf[0] as char)
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(&self.read_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.read_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(&self.read_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.read_bytes()?)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len: u32 = de::Deserialize::deserialize(&mut *self)?;
        self.deserialize_tuple(len as usize, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        struct Access<'a, R: 'a, S: 'a, E: 'a>
        where
            R: Read,
            S: SizeLimit,
            E: ByteOrder,
        {
            deserializer: &'a mut Deserializer<R, S, E>,
            len: usize,
        }

        impl<'de, 'a, R: 'a, S, E> de::SeqAccess<'de> for Access<'a, R, S, E>
        where
            R: Read,
            S: SizeLimit,
            E: ByteOrder,
        {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value = de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        impl<'de, 'a, R: 'a, S, E> de::EnumAccess<'de> for &'a mut Deserializer<R, S, E>
        where
            R: Read,
            S: SizeLimit,
            E: ByteOrder,
        {
            type Error = Error;
            type Variant = Self;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
            where
                V: de::DeserializeSeed<'de>,
            {
                let idx: u32 = de::Deserialize::deserialize(&mut *self)?;
                let val: Result<_> = seed.deserialize(idx.into_deserializer());
                Ok((val?, self))
            }
        }

        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de, 'a, R, S, E> de::VariantAccess<'de> for &'a mut Deserializer<R, S, E>
where
    R: Read,
    S: SizeLimit,
    E: ByteOrder,
{
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        de::DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

impl<R, S> From<Deserializer<R, S, BigEndian>> for Deserializer<R, S, LittleEndian> {
    fn from(t: Deserializer<R, S, BigEndian>) -> Self {
        Deserializer::<R, S, LittleEndian> {
            reader: t.reader,
            size_limit: t.size_limit,
            pos: t.pos,
            phantom: PhantomData,
        }
    }
}

#[inline]
fn utf8_char_width(first_byte: u8) -> usize {
    UTF8_CHAR_WIDTH[first_byte as usize] as usize
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, //
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

/// Deserializes a slice of bytes into an object.
pub fn deserialize_data<'de, T, E>(bytes: &[u8]) -> Result<T>
where
    T: de::Deserialize<'de>,
    E: ByteOrder,
{
    deserialize_data_from::<_, _, _, E>(bytes, Infinite)
}

/// Deserializes an object directly from a `Read`.
pub fn deserialize_data_from<'de, R, T, S, E>(reader: R, size_limit: S) -> Result<T>
where
    R: Read,
    T: de::Deserialize<'de>,
    S: SizeLimit,
    E: ByteOrder,
{
    let mut deserializer = Deserializer::<_, S, E>::new(reader, size_limit);
    de::Deserialize::deserialize(&mut deserializer)
}
