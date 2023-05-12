use std::{self};


use serde::ser::{SerializeTuple};
use serde::ser::SerializeSeq;

// pub struct ParameterListSerializer<W, E> {
//     writer: W,
//     phantom: PhantomData<E>,
// }

// impl<W, E> ParameterListSerializer<W, E>
// where
//     W: Write,
//     E: ByteOrder,
// {
//     pub fn new(writer: W) -> Self {
//         Self {
//             writer,
//             phantom: PhantomData,
//         }
//     }
// }
// impl<'a, W, E> serde::Serializer for &'a mut ParameterListSerializer<W, E>
// where
//     W: Write,
//     E: ByteOrder,
// {
//     type Ok = ();
//     type Error = std::fmt::Error;
//     type SerializeSeq = Compound<'a, W, E>;
//     type SerializeTuple = Compound<'a, W, E>;
//     type SerializeTupleStruct = Compound<'a, W, E>;
//     type SerializeTupleVariant = Compound<'a, W, E>;
//     type SerializeMap = Compound<'a, W, E>;
//     type SerializeStruct = Compound<'a, W, E>;
//     type SerializeStructVariant = Compound<'a, W, E>;
//     fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_str(self, _v: &str) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_bytes(self, v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
//         self.writer.write_all(v).map_err(|_err| std::fmt::Error)
//     }

//     fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_some<T: ?Sized>(self, _value: &T) -> std::result::Result<Self::Ok, Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_unit_struct(
//         self,
//         _name: &'static str,
//     ) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_unit_variant(
//         self,
//         _name: &'static str,
//         _variant_index: u32,
//         _variant: &'static str,
//     ) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }

//     fn serialize_newtype_struct<T: ?Sized>(
//         self,
//         _name: &'static str,
//         value: &T,
//     ) -> std::result::Result<Self::Ok, Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         let mut cdr_serializer = cdr::Serializer::<_, E>::new(&mut self.writer);
//         serde::Serialize::serialize(value, &mut cdr_serializer).unwrap();
//         Ok(())
//     }

//     fn serialize_newtype_variant<T: ?Sized>(
//         self,
//         _name: &'static str,
//         _variant_index: u32,
//         _variant: &'static str,
//         _value: &T,
//     ) -> std::result::Result<Self::Ok, Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn serialize_seq(
//         self,
//         _len: Option<usize>,
//     ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
//         Ok(Compound { ser: self })
//     }

//     fn serialize_tuple(
//         self,
//         _len: usize,
//     ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
//         Ok(Compound { ser: self })
//     }

//     fn serialize_tuple_struct(
//         self,
//         _name: &'static str,
//         _len: usize,
//     ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
//         todo!()
//     }

//     fn serialize_tuple_variant(
//         self,
//         _name: &'static str,
//         _variant_index: u32,
//         _variant: &'static str,
//         _len: usize,
//     ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
//         todo!()
//     }

//     fn serialize_map(
//         self,
//         _len: Option<usize>,
//     ) -> std::result::Result<Self::SerializeMap, Self::Error> {
//         todo!()
//     }

//     fn serialize_struct(
//         self,
//         _name: &'static str,
//         _len: usize,
//     ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
//         Ok(Compound { ser: self })
//     }

//     fn serialize_struct_variant(
//         self,
//         _name: &'static str,
//         _variant_index: u32,
//         _variant: &'static str,
//         _len: usize,
//     ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
//         todo!()
//     }
// }

// pub struct Compound<'a, W: 'a, E: 'a> {
//     ser: &'a mut ParameterListSerializer<W, E>,
// }
// impl<'a, W, E> serde::ser::SerializeSeq for Compound<'a, W, E>
// where
//     W: Write,
//     E: ByteOrder,
// {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         value.serialize(&mut *self.ser)
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

// impl<'a, W, E> serde::ser::SerializeTuple for Compound<'a, W, E> {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }
// }

// impl<'a, W, E> serde::ser::SerializeTupleStruct for Compound<'a, W, E> {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }
// }

// impl<'a, W, E> serde::ser::SerializeTupleVariant for Compound<'a, W, E> {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }
// }

// impl<'a, W, E> serde::ser::SerializeMap for Compound<'a, W, E> {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }
// }

// impl<'a, W, E> serde::ser::SerializeStruct for Compound<'a, W, E>
// where
//     W: Write,
//     E: ByteOrder,
// {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_field<T: ?Sized>(
//         &mut self,
//         _key: &'static str,
//         value: &T,
//     ) -> std::result::Result<Self::Ok, Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         value.serialize(&mut *self.ser)
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

// impl<'a, W, E> serde::ser::SerializeStructVariant for Compound<'a, W, E> {
//     type Ok = ();
//     type Error = std::fmt::Error;

//     fn serialize_field<T: ?Sized>(
//         &mut self,
//         _key: &'static str,
//         _value: &T,
//     ) -> std::result::Result<(), Self::Error>
//     where
//         T: serde::Serialize,
//     {
//         todo!()
//     }

//     fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
//         todo!()
//     }
// }

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

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, PartialEq, serde::Serialize)]
    struct InnerWithDefault {
        id: Parameter<71, u8>,
        n: ParameterWithDefault<72, u8>,
    }

    #[test]
    fn serialize_with_default() {
        let data = InnerWithDefault {
            id: Parameter(21),
            n: ParameterWithDefault(0),
        };
        let expected = &[
            //0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
               //1, 0, 0, 0, // Sentinel
        ][..];
        let mut result = vec![];
        let mut serializer = cdr::Serializer::<_, byteorder::LittleEndian>::new(&mut result);
        serde::Serialize::serialize(&data, &mut serializer).unwrap();
        assert_eq!(result, expected)
    }

    #[derive(Debug, PartialEq, serde::Serialize)]
    struct Inner {
        id: Parameter<71, u8>,
        n: Parameter<72, u16>,
    }
    #[test]
    fn serialize_simple() {
        let data = Inner {
            id: Parameter(21),
            n: Parameter(34),
        };
        let expected = &[
            //0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u16
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
               //1, 0, 0, 0, // Sentinel
        ][..];
        let mut result = vec![];
        let mut serializer = cdr::Serializer::<_, byteorder::LittleEndian>::new(&mut result);
        serde::Serialize::serialize(&data, &mut serializer).unwrap();
        assert_eq!(result, expected)
    }
}
