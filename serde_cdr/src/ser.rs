//! Serializing Rust data types into CDR.

use std::{self, io::Write, marker::PhantomData};

use byteorder::{ByteOrder, WriteBytesExt};
use serde::ser;

use crate::error::{Error, Result};
use crate::size::{
    calc_serialized_data_size, calc_serialized_data_size_bounded, Infinite, SizeLimit,
};

/// A serializer that writes values into a buffer.
pub struct Serializer<W, E> {
    writer: W,
    pos: u64,
    phantom: PhantomData<E>,
}

impl<W, E> Serializer<W, E>
where
    W: Write,
    E: ByteOrder,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            pos: 0,
            phantom: PhantomData,
        }
    }

    fn add_pos(&mut self, size: u64) {
        self.pos += size;
    }

    pub(crate) fn reset_pos(&mut self) {
        self.pos = 0;
    }

    fn set_pos_of<T>(&mut self) -> Result<()> {
        self.write_padding_of::<T>()?;
        self.add_pos(std::mem::size_of::<T>() as u64);
        Ok(())
    }

    fn write_padding_of<T>(&mut self) -> Result<()> {
        // Calculate the required padding to align with 1-byte, 2-byte, 4-byte, 8-byte boundaries
        // Instead of using the slow modulo operation '%', the faster bit-masking is used
        const PADDING: [u8; 8] = [0; 8];
        let alignment = std::mem::size_of::<T>();
        let rem_mask = alignment - 1; // mask like 0x0, 0x1, 0x3, 0x7
        match (self.pos as usize) & rem_mask {
            0 => Ok(()),
            n @ 1..=7 => {
                let amt = alignment - n;
                self.pos += amt as u64;
                self.writer.write_all(&PADDING[..amt]).map_err(Into::into)
            }
            _ => unreachable!(),
        }
    }

    fn write_usize_as_u32(&mut self, v: usize) -> Result<()> {
        if v > std::u32::MAX as usize {
            return Err(Error::NumberOutOfRange);
        }

        ser::Serializer::serialize_u32(self, v as u32)
    }
}

macro_rules! impl_serialize_value {
    ($ser_method:ident($ty:ty) = $writer_method:ident()) => {
        fn $ser_method(self, v: $ty) -> Result<Self::Ok> {
            self.set_pos_of::<$ty>()?;
            self.writer.$writer_method::<E>(v).map_err(Into::into)
        }
    };
}

impl<'a, W, E> ser::Serializer for &'a mut Serializer<W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W, E>;
    type SerializeTuple = Compound<'a, W, E>;
    type SerializeTupleStruct = Compound<'a, W, E>;
    type SerializeTupleVariant = Compound<'a, W, E>;
    type SerializeMap = Compound<'a, W, E>;
    type SerializeStruct = Compound<'a, W, E>;
    type SerializeStructVariant = Compound<'a, W, E>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.set_pos_of::<bool>()?;
        self.writer.write_u8(v as u8).map_err(Into::into)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.set_pos_of::<i8>()?;
        self.writer.write_i8(v).map_err(Into::into)
    }

    impl_serialize_value! { serialize_i16(i16) = write_i16() }
    impl_serialize_value! { serialize_i32(i32) = write_i32() }
    impl_serialize_value! { serialize_i64(i64) = write_i64() }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.set_pos_of::<u8>()?;
        self.writer.write_u8(v).map_err(Into::into)
    }

    impl_serialize_value! { serialize_u16(u16) = write_u16() }
    impl_serialize_value! { serialize_u32(u32) = write_u32() }
    impl_serialize_value! { serialize_u64(u64) = write_u64() }

    impl_serialize_value! { serialize_f32(f32) = write_f32() }
    impl_serialize_value! { serialize_f64(f64) = write_f64() }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        if !v.is_ascii() {
            Err(Error::InvalidChar(v))
        } else {
            let mut buf = [0u8; 1];
            v.encode_utf8(&mut buf);
            self.add_pos(1);
            self.writer.write_all(&buf).map_err(Into::into)
        }
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        if !v.is_ascii() {
            Err(Error::InvalidString(v.into()))
        } else {
            let terminating_char = [0u8];
            let l = v.len() + terminating_char.len();
            self.write_usize_as_u32(l)?;
            self.add_pos(l as u64);
            self.writer.write_all(v.as_bytes())?;
            self.writer.write_all(&terminating_char).map_err(Into::into)
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let l = v.len();
        self.write_usize_as_u32(l)?;
        self.add_pos(l as u64);
        self.writer.write_all(v).map_err(Into::into)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::TypeNotSupported)
    }

    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        Err(Error::TypeNotSupported)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(Error::SequenceMustHaveLength)?;
        self.write_usize_as_u32(len)?;
        Ok(Compound { ser: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_u32(variant_index)?;
        Ok(Compound { ser: self })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::TypeNotSupported)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(Compound { ser: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_u32(variant_index)?;
        Ok(Compound { ser: self })
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[doc(hidden)]
pub struct Compound<'a, W: 'a, E: 'a> {
    ser: &'a mut Serializer<W, E>,
}

impl<'a, W, E> ser::SerializeSeq for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> ser::SerializeTuple for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> ser::SerializeTupleStruct for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> ser::SerializeTupleVariant for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> ser::SerializeMap for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> ser::SerializeStruct for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> ser::SerializeStructVariant for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serializes a serializable object into a `Vec` of bytes.
pub fn serialize_data<T: ?Sized, S, E>(value: &T, size_limit: S) -> Result<Vec<u8>>
where
    T: ser::Serialize,
    S: SizeLimit,
    E: ByteOrder,
{
    let mut writer = match size_limit.limit() {
        Some(limit) => {
            let actual_size = calc_serialized_data_size_bounded(value, limit)?;
            Vec::with_capacity(actual_size as usize)
        }
        None => {
            let size = calc_serialized_data_size(value) as usize;
            Vec::with_capacity(size)
        }
    };

    serialize_data_into::<_, _, _, E>(&mut writer, value, Infinite)?;
    Ok(writer)
}

/// Serializes an object directly into a `Write`.
pub fn serialize_data_into<W, T: ?Sized, S, E>(writer: W, value: &T, size_limit: S) -> Result<()>
where
    W: Write,
    T: ser::Serialize,
    S: SizeLimit,
    E: ByteOrder,
{
    if let Some(limit) = size_limit.limit() {
        calc_serialized_data_size_bounded(value, limit)?;
    }

    let mut serializer = Serializer::<_, E>::new(writer);
    ser::Serialize::serialize(value, &mut serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BigEndian, LittleEndian};

    #[test]
    fn serialize_octet() {
        let v = 32u8;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x20]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x20]
        );
    }

    #[test]
    fn serialize_char() {
        let v = 'Z';
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x5a]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x5a]
        );
    }

    #[test]
    fn serialize_wchar() {
        let v = 'Å';
        assert!(serialize_data::<_, _, BigEndian>(&v, Infinite).is_err());
        assert!(serialize_data::<_, _, LittleEndian>(&v, Infinite).is_err());
    }

    #[test]
    fn serialize_ushort() {
        let v = 65500u16;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0xff, 0xdc]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0xdc, 0xff]
        );
    }

    #[test]
    fn serialize_short() {
        let v = -32700i16;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x80, 0x44]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x44, 0x80]
        );
    }

    #[test]
    fn serialize_ulong() {
        let v = 4294967200u32;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0xff, 0xff, 0xff, 0xa0]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0xa0, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_long() {
        let v = -2147483600i32;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x80, 0x00, 0x00, 0x30]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x30, 0x00, 0x00, 0x80]
        );
    }

    #[test]
    fn serialize_ulonglong() {
        let v = 18446744073709551600u64;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_longlong() {
        let v = -9223372036800i64;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_float() {
        let v = std::f32::MIN_POSITIVE;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x00, 0x80, 0x00, 0x00]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x00, 0x00, 0x80, 0x00]
        );
    }

    #[test]
    fn serialize_double() {
        let v = std::f64::MIN_POSITIVE;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
        );
    }

    #[test]
    fn serialize_bool() {
        let v = true;
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x01]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x01]
        );
    }

    #[test]
    fn serialize_string() {
        let v = "Hola a todos, esto es un test";
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x1e, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x1e, 0x00, 0x00, 0x00, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ]
        );
    }

    #[test]
    fn serialize_wstring() {
        let v = "みなさんこんにちは。これはテストです。";
        assert!(serialize_data::<_, _, BigEndian>(&v, Infinite).is_err());
        assert!(serialize_data::<_, _, LittleEndian>(&v, Infinite).is_err());
    }

    #[test]
    fn serialize_empty_string() {
        let v = "";
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x00, 0x00, 0x00, 0x01, 0x00]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x01, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn serialize_octet_array() {
        let v = [1u8, 2, 3, 4, 5];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x01, 0x02, 0x03, 0x04, 0x05]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x01, 0x02, 0x03, 0x04, 0x05]
        );
    }

    #[test]
    fn serialize_char_array() {
        let v = ['A', 'B', 'C', 'D', 'E'];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x41, 0x42, 0x43, 0x44, 0x45]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x41, 0x42, 0x43, 0x44, 0x45]
        );
    }

    #[test]
    fn serialize_ushort_array() {
        let v = [65500u16, 65501, 65502, 65503, 65504];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0xff, 0xdc, //
                0xff, 0xdd, //
                0xff, 0xde, //
                0xff, 0xdf, //
                0xff, 0xe0
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0xdc, 0xff, //
                0xdd, 0xff, //
                0xde, 0xff, //
                0xdf, 0xff, //
                0xe0, 0xff
            ]
        );
    }

    #[test]
    fn serialize_short_array() {
        let v = [-32700i16, -32701, -32702, -32703, -32704];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x80, 0x44, //
                0x80, 0x43, //
                0x80, 0x42, //
                0x80, 0x41, //
                0x80, 0x40
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x44, 0x80, //
                0x43, 0x80, //
                0x42, 0x80, //
                0x41, 0x80, //
                0x40, 0x80
            ]
        );
    }

    #[test]
    fn serialize_ulong_array() {
        let v = [
            4294967200u32,
            4294967201,
            4294967202,
            4294967203,
            4294967204,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0xff, 0xff, 0xff, 0xa0, //
                0xff, 0xff, 0xff, 0xa1, //
                0xff, 0xff, 0xff, 0xa2, //
                0xff, 0xff, 0xff, 0xa3, //
                0xff, 0xff, 0xff, 0xa4,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0xa0, 0xff, 0xff, 0xff, //
                0xa1, 0xff, 0xff, 0xff, //
                0xa2, 0xff, 0xff, 0xff, //
                0xa3, 0xff, 0xff, 0xff, //
                0xa4, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_long_array() {
        let v = [
            -2147483600,
            -2147483601,
            -2147483602,
            -2147483603,
            -2147483604,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x80, 0x00, 0x00, 0x30, //
                0x80, 0x00, 0x00, 0x2f, //
                0x80, 0x00, 0x00, 0x2e, //
                0x80, 0x00, 0x00, 0x2d, //
                0x80, 0x00, 0x00, 0x2c,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x30, 0x00, 0x00, 0x80, //
                0x2f, 0x00, 0x00, 0x80, //
                0x2e, 0x00, 0x00, 0x80, //
                0x2d, 0x00, 0x00, 0x80, //
                0x2c, 0x00, 0x00, 0x80,
            ]
        );
    }

    #[test]
    fn serialize_ulonglong_array() {
        let v = [
            18446744073709551600u64,
            18446744073709551601,
            18446744073709551602,
            18446744073709551603,
            18446744073709551604,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf1, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf2, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf4,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf4, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_longlong_array() {
        let v = [
            -9223372036800i64,
            -9223372036801,
            -9223372036802,
            -9223372036803,
            -9223372036804,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3f, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3e, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3d, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3c,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3f, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3e, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3d, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3c, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_float_array() {
        let f = std::f32::MIN_POSITIVE;

        let v = [f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x80, 0x00, 0x00, //
                0x3f, 0x80, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, //
                0x40, 0x40, 0x00, 0x00, //
                0x40, 0x80, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x80, 0x00, //
                0x00, 0x00, 0x80, 0x3f, //
                0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x40, 0x40, //
                0x00, 0x00, 0x80, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_double_array() {
        let f = std::f64::MIN_POSITIVE;

        let v = [f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_bool_array() {
        let v = [true, false, true, false, true];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![0x01, 0x00, 0x01, 0x00, 0x01]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![0x01, 0x00, 0x01, 0x00, 0x01]
        );
    }

    #[test]
    fn serialize_string_array() {
        let v = ["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x04, //
                0x42, 0x59, 0x45, 0x00, //
                0x00, 0x00, 0x00, 0x08, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x04, 0x00, 0x00, 0x00, //
                0x42, 0x59, 0x45, 0x00, //
                0x08, 0x00, 0x00, 0x00, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ]
        );
    }

    #[test]
    fn serialize_octet_sequence() {
        let v = vec![1u8, 2, 3, 4, 5];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x01, 0x02, 0x03, 0x04, 0x05
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x01, 0x02, 0x03, 0x04, 0x05
            ]
        );
    }

    #[test]
    fn serialize_char_sequence() {
        let v = vec!['A', 'B', 'C', 'D', 'E'];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x41, 0x42, 0x43, 0x44, 0x45
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x41, 0x42, 0x43, 0x44, 0x45
            ]
        );
    }

    #[test]
    fn serialize_ushort_sequence() {
        let v = vec![65500u16, 65501, 65502, 65503, 65504];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0xff, 0xdc, //
                0xff, 0xdd, //
                0xff, 0xde, //
                0xff, 0xdf, //
                0xff, 0xe0
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0xdc, 0xff, //
                0xdd, 0xff, //
                0xde, 0xff, //
                0xdf, 0xff, //
                0xe0, 0xff
            ]
        );
    }

    #[test]
    fn serialize_short_sequence() {
        let v = vec![-32700i16, -32701, -32702, -32703, -32704];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x80, 0x44, //
                0x80, 0x43, //
                0x80, 0x42, //
                0x80, 0x41, //
                0x80, 0x40
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x44, 0x80, //
                0x43, 0x80, //
                0x42, 0x80, //
                0x41, 0x80, //
                0x40, 0x80
            ]
        );
    }

    #[test]
    fn serialize_ulong_sequence() {
        let v = vec![
            4294967200u32,
            4294967201,
            4294967202,
            4294967203,
            4294967204,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0xff, 0xff, 0xff, 0xa0, //
                0xff, 0xff, 0xff, 0xa1, //
                0xff, 0xff, 0xff, 0xa2, //
                0xff, 0xff, 0xff, 0xa3, //
                0xff, 0xff, 0xff, 0xa4,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0xa0, 0xff, 0xff, 0xff, //
                0xa1, 0xff, 0xff, 0xff, //
                0xa2, 0xff, 0xff, 0xff, //
                0xa3, 0xff, 0xff, 0xff, //
                0xa4, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_long_sequence() {
        let v = vec![
            -2147483600,
            -2147483601,
            -2147483602,
            -2147483603,
            -2147483604,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x80, 0x00, 0x00, 0x30, //
                0x80, 0x00, 0x00, 0x2f, //
                0x80, 0x00, 0x00, 0x2e, //
                0x80, 0x00, 0x00, 0x2d, //
                0x80, 0x00, 0x00, 0x2c,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x30, 0x00, 0x00, 0x80, //
                0x2f, 0x00, 0x00, 0x80, //
                0x2e, 0x00, 0x00, 0x80, //
                0x2d, 0x00, 0x00, 0x80, //
                0x2c, 0x00, 0x00, 0x80,
            ]
        );
    }

    #[test]
    fn serialize_ulonglong_sequence() {
        let v = vec![
            18446744073709551600u64,
            18446744073709551601,
            18446744073709551602,
            18446744073709551603,
            18446744073709551604,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x00, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf1, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf2, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf4,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf4, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_longlong_sequence() {
        let v = vec![
            -9223372036800i64,
            -9223372036801,
            -9223372036802,
            -9223372036803,
            -9223372036804,
        ];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x00, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3f, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3e, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3d, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3c,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3f, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3e, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3d, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3c, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_float_sequence() {
        let f = std::f32::MIN_POSITIVE;

        let v = vec![f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x80, 0x00, 0x00, //
                0x3f, 0x80, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, //
                0x40, 0x40, 0x00, 0x00, //
                0x40, 0x80, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x80, 0x00, //
                0x00, 0x00, 0x80, 0x3f, //
                0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x40, 0x40, //
                0x00, 0x00, 0x80, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_double_sequence() {
        let f = std::f64::MIN_POSITIVE;

        let v = vec![f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_bool_sequence() {
        let v = vec![true, false, true, false, true];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x01, 0x00, 0x01, 0x00, 0x01
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x01, 0x00, 0x01, 0x00, 0x01
            ]
        );
    }

    #[test]
    fn serialize_string_sequence() {
        let v = vec!["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            serialize_data::<_, _, BigEndian>(&v, Infinite).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x05, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x04, //
                0x42, 0x59, 0x45, 0x00, //
                0x00, 0x00, 0x00, 0x08, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ]
        );
        assert_eq!(
            serialize_data::<_, _, LittleEndian>(&v, Infinite).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x05, 0x00, 0x00, 0x00, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x04, 0x00, 0x00, 0x00, //
                0x42, 0x59, 0x45, 0x00, //
                0x08, 0x00, 0x00, 0x00, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ]
        );
    }
}
