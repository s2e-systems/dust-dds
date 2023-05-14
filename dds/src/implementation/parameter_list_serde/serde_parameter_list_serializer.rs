use std::io::Write;
use std::marker::PhantomData;
use std::{self};

use byteorder::{ByteOrder, WriteBytesExt};
use serde::ser::SerializeSeq;
use serde::ser::SerializeTuple;

use cdr::Error;

use crate::topic_definition::type_support::DdsSerialize;

pub struct ParameterListSerializer<W, E> {
    ser: cdr::Serializer<W, E>,
    phantom: PhantomData<E>,
}

impl<W, E> ParameterListSerializer<W, E>
where
    W: Write,
    E: ByteOrder,
{
    pub fn new(writer: W) -> Self {
        Self {
            ser: cdr::Serializer::new(writer),
            phantom: PhantomData,
        }
    }
}

impl<'a, W, E> serde::Serializer for &'a mut ParameterListSerializer<W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = cdr::Error;
    type SerializeSeq = Compound<'a, W, E>;
    type SerializeTuple = Compound<'a, W, E>;
    type SerializeTupleStruct = Compound<'a, W, E>;
    type SerializeTupleVariant = Compound<'a, W, E>;
    type SerializeMap = Compound<'a, W, E>;
    type SerializeStruct = Compound<'a, W, E>;
    type SerializeStructVariant = Compound<'a, W, E>;
    fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.serialize_i16(v)
    }

    fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.serialize_u16(v)
    }

    fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, _v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(Compound { ser: &mut self.ser })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

pub struct Compound<'a, W: 'a, E: 'a> {
    ser: &'a mut cdr::Serializer<W, E>,
}
impl<'a, W, E> serde::ser::SerializeSeq for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeTuple for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W, E> serde::ser::SerializeTupleStruct for Compound<'a, W, E> {
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W, E> serde::ser::SerializeTupleVariant for Compound<'a, W, E> {
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W, E> serde::ser::SerializeMap for Compound<'a, W, E> {
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W, E> serde::ser::SerializeStruct for Compound<'a, W, E>
where
    W: Write,
    E: ByteOrder,
{
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeStructVariant for Compound<'a, W, E> {
    type Ok = ();
    type Error = cdr::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
struct XParameter<const PID: u16, T>(T);

#[derive(Debug, PartialEq)]
struct Parameter<const PID: u16, T>(T);

impl<const PID: u16, T> serde::Serialize for Parameter<PID, T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let length_without_padding = cdr::size::calc_serialized_data_size(&self.0) as i16;
        let padding_length = (4 - length_without_padding) & 3;
        let length = length_without_padding + padding_length;

        let mut s = serializer.serialize_tuple(4)?;
        s.serialize_element(&PID)?;
        s.serialize_element(&length)?;
        s.serialize_element(&self.0)?;
        match padding_length {
            1 => s.serialize_element(&[0u8; 1])?,
            2 => s.serialize_element(&[0u8; 2])?,
            3 => s.serialize_element(&[0u8; 3])?,
            _ => s.serialize_element(&[0u8; 0])?,
        }
        s.end()
    }
}

#[derive(Debug, PartialEq)]
struct ParameterWithDefault<const PID: u16, T>(T);

impl<const PID: u16, T> serde::Serialize for ParameterWithDefault<PID, T>
where
    T: serde::Serialize + Default + PartialEq,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0 == T::default() {
            serializer.serialize_tuple(0)?.end()
        } else {
            let length_without_padding = cdr::size::calc_serialized_data_size(&self.0) as i16;
            let padding_length = (4 - length_without_padding) & 3;
            let length = length_without_padding + padding_length;

            let mut s = serializer.serialize_tuple(4)?;
            s.serialize_element(&PID)?;
            s.serialize_element(&length)?;
            s.serialize_element(&self.0)?;
            match padding_length {
                1 => s.serialize_element(&[0u8; 1])?,
                2 => s.serialize_element(&[0u8; 2])?,
                3 => s.serialize_element(&[0u8; 3])?,
                _ => s.serialize_element(&[0u8; 0])?,
            }
            s.end()
        }
    }
}

type RepresentationType = [u8; 2];
type RepresentationOptions = [u8; 2];
pub const CDR_BE: RepresentationType = [0x00, 0x00];
pub const CDR_LE: RepresentationType = [0x00, 0x01];
pub const PL_CDR_BE: RepresentationType = [0x00, 0x02];
pub const PL_CDR_LE: RepresentationType = [0x00, 0x03];
pub const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];
trait RepresentationFormat {
    const REPRESENTATION_IDENTIFIER: RepresentationType;
}
impl<T: serde::Serialize + DdsSerde> RepresentationFormat for T {
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;
}

trait DdsSerde {}

fn dds_serialize<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize + RepresentationFormat,
{
    let mut writer = vec![];
    match T::REPRESENTATION_IDENTIFIER {
        CDR_LE => {
            writer.write_all(&CDR_LE)?;
            writer.write_all(&REPRESENTATION_OPTIONS)?;
            let mut serializer =
                cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer)?;
        }
        PL_CDR_LE => {
            writer.write_all(&PL_CDR_LE)?;
            writer.write_all(&REPRESENTATION_OPTIONS)?;
            let mut serializer =
                ParameterListSerializer::<_, byteorder::LittleEndian>::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer).unwrap();
            writer.write_u16::<byteorder::LittleEndian>(1)?;
            writer.write_i16::<byteorder::LittleEndian>(0)?;
        }
        _ => todo!(),
    };
    Ok(writer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Serialize)]
    struct Inner {
        id: Parameter<71, u8>,
        n: Parameter<72, u16>,
    }
    impl RepresentationFormat for Inner {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
    }
    #[test]
    fn serialize_simple() {
        let data = Inner {
            id: Parameter(21),
            n: Parameter(34),
        };
        let expected = &[
            0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u16
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let result = dds_serialize(&data).unwrap();
        assert_eq!(result, expected)
    }
    #[derive(Debug, PartialEq, serde::Serialize)]
    struct UserData {
        id: u8,
        n: i32,
    }
    impl DdsSerde for UserData {}

    #[test]
    fn cdr_simple() {
        let data = UserData { id: 3, n: 4 };
        let expected = &[
            0x00, 0x01, 0, 0, // representation identifier
            3, 0, 0, 0, // id
            4, 0, 0, 0, // n
        ][..];
        let result = dds_serialize(&data).unwrap();
        assert_eq!(expected, result)
    }
    #[derive(Debug, PartialEq, serde::Serialize)]
    struct InnerWithDefault {
        id: Parameter<71, u8>,
        n: ParameterWithDefault<72, u8>,
    }
    impl RepresentationFormat for InnerWithDefault {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
    }

    #[test]
    fn serialize_with_default() {
        let data = InnerWithDefault {
            id: Parameter(21),
            n: ParameterWithDefault(0),
        };
        let expected = &[
            0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let result = dds_serialize(&data).unwrap();
        assert_eq!(result, expected)
    }
}
