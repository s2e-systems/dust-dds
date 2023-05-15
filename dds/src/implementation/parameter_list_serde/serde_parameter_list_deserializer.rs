use std::{self, io::Read, marker::PhantomData};

use byteorder::{ByteOrder, ReadBytesExt};
use serde::de::{self};

use cdr::Error;

use super::parameter::{
    RepresentationOptions, RepresentationType, CDR_BE, CDR_LE, PL_CDR_BE, PL_CDR_LE,
};

pub struct ParameterListDeserializer<'a, E> {
    data: &'a [u8],
    pos: u64,
    endianness: PhantomData<E>,
}

impl<'a, E> ParameterListDeserializer<'a, E> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            endianness: PhantomData,
        }
    }

    fn read_size(&mut self, size: u64) -> Result<(), Error> {
        self.pos += size;
        Ok(())
    }

    fn read_size_of<T>(&mut self) -> Result<(), Error> {
        self.read_size(std::mem::size_of::<T>() as u64)
    }

    fn read_padding_of<T>(&mut self) -> Result<(), Error> {
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
                self.data
                    .read_exact(&mut padding[..amt])
                    .map_err(Into::into)
            }
            _ => unreachable!(),
        }
    }
}

impl<'de, 'b, E> de::Deserializer<'de> for &'b mut ParameterListDeserializer<'de, E>
where
    E: byteorder::ByteOrder,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(serde::de::Error::custom("deserialize_any not implemented"))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value: u8 = de::Deserialize::deserialize(self)?;
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            _ => Err(serde::de::Error::custom("invalid bool encoding")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.read_size_of::<u8>()?;
        visitor.visit_u8(self.data.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.read_padding_of::<u16>()?;
        self.read_size_of::<u16>()?;
        visitor.visit_u16(self.data.read_u16::<E>()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.read_padding_of::<u32>()?;
        self.read_size_of::<u32>()?;
        visitor.visit_u32(self.data.read_u32::<E>()?)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.read_padding_of::<i16>()?;
        self.read_size_of::<i16>()?;
        visitor.visit_i16(self.data.read_i16::<E>()?)
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.data)
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(PidSeparated::new(self))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
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
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        struct Access<'c, 'de: 'c, E> {
            deserializer: &'c mut ParameterListDeserializer<'de, E>,
            len: usize,
        }

        impl<'de, 'd, E> de::SeqAccess<'de> for Access<'d, 'de, E>
        where
            E: ByteOrder,
        {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value = seed.deserialize(&mut *self.deserializer)?;
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
            len: fields.len(),
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }
}

struct PidSeparated<'de, E> {
    de: ParameterListDeserializer<'de, E>,
    skip_length: usize,
}

impl<'a, E> PidSeparated<'a, E> {
    fn new(de: &ParameterListDeserializer<'a, E>) -> Self {
        PidSeparated {
            de: ParameterListDeserializer::new(de.data),
            skip_length: 0,
        }
    }
}

impl<'de, E> serde::de::MapAccess<'de> for PidSeparated<'de, E>
where
    E: byteorder::ByteOrder,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.de.data = &self.de.data[self.skip_length as usize..];

        let mut cdr_deserializer = cdr::Deserializer::<_,_,E>::new(self.de.data, cdr::Infinite);
        let pid = seed.deserialize(&mut cdr_deserializer).map(Some);
        let length: i16 = serde::Deserialize::deserialize(&mut cdr_deserializer)?;
        self.skip_length = length as usize;
        self.de.data = &self.de.data[4 as usize..];
        pid
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let mut cdr_deserializer = cdr::Deserializer::<_,_,E>::new(self.de.data, cdr::Infinite);

        let length_before_deserialization = self.de.data.len();
        let value = seed.deserialize(&mut cdr_deserializer);
        let length_deserialized = length_before_deserialization - self.de.data.len();

        let padding_length = self.skip_length - length_deserialized;
        self.de.data = &self.de.data[padding_length as usize..];
        self.skip_length = 0;
        value
    }
}

pub fn dds_deserialize<'de, T>(mut data: &'de [u8]) -> Result<T, Error>
where
    T: serde::Deserialize<'de>,
{
    let mut representation_identifier: RepresentationType = [0, 0];
    data.read_exact(&mut representation_identifier)?;

    let mut representation_option: RepresentationOptions = [0, 0];
    data.read_exact(&mut representation_option)?;

    match representation_identifier {
        CDR_BE => {
            let mut deserializer =
                cdr::Deserializer::<_, _, byteorder::BigEndian>::new(data, cdr::Infinite);
            serde::Deserialize::deserialize(&mut deserializer)
        }
        CDR_LE => {
            let mut deserializer =
                cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(data, cdr::Infinite);
            serde::Deserialize::deserialize(&mut deserializer)
        }
        PL_CDR_BE => {
            let mut deserializer = ParameterListDeserializer::<byteorder::BigEndian>::new(data);
            serde::Deserialize::deserialize(&mut deserializer)
        }
        PL_CDR_LE => {
            let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
            serde::Deserialize::deserialize(&mut deserializer)
        }
        _ => Err(Error::InvalidEncapsulation),
    }
}
#[cfg(test)]
mod tests {
    use crate::implementation::parameter_list_serde::parameter::{
        Parameter, ParameterVector, ParameterWithDefault,
    };

    use super::*;

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct CdrParameterType {
        i: u32
    }

    #[test]
    fn deserialize_one_parameter_le() {
        let data = &[
            71, 0x00, 4, 0, // pid | Length (incl padding)
            21, 0, 0, 0, // CdrParameterType
        ][..];
        let expected: Parameter<71, CdrParameterType> = Parameter(CdrParameterType{i:21});
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: Parameter<71, CdrParameterType> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }


    #[test]
    fn deserialize_one_parameter_be() {
        let data = &[
            0, 71, 0, 4, // pid | Length (incl padding)
            0, 0, 0, 21, // u32
        ][..];
        let expected: Parameter<71, u32> = Parameter(21);
        let mut deserializer = ParameterListDeserializer::<byteorder::BigEndian>::new(data);
        let result: Parameter<71, u32> =
            serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct PlInner {
        id: Parameter<71, u8>,
        n: Parameter<72, u8>,
    }
    #[test]
    fn deserialize_pl_simple() {
        let expected = PlInner {
            id: Parameter(21),
            n: Parameter(34),
        };
        let data = &[
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: PlInner = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct PlWithDefault {
        id: Parameter<71, u8>,
        n: ParameterWithDefault<72, u8>,
    }
    #[test]
    fn deserialize_pl_with_default() {
        let expected = PlWithDefault {
            id: Parameter(21),
            n: ParameterWithDefault(0),
        };
        let data = &[
            71, 0x00, 4, 0, // n | Length (incl padding)
            21, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: PlWithDefault = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct PlWithList {
        id: Parameter<71, u8>,
        values: ParameterVector<93, u16>,
    }

    #[test]
    fn deserialize_pl_vec_le() {
        let expected = PlWithList {
            id: Parameter(21),
            values: ParameterVector(vec![34, 35]),
        };
        let data = &[
            0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            93, 0x00, 4, 0, // values | Length (incl padding)
            34, 0, 0, 0, // u16
            93, 0x00, 4, 0, // values | Length (incl padding)
            35, 0, 0, 0, // u16
            1, 0, 0, 0, // Sentinel
        ][..];
        let result: PlWithList = dds_deserialize(&data).unwrap();
        assert_eq!(result, expected)
    }

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct PlOuter {
        outer: Parameter<2, u8>,
        inner: PlInner,
    }
    #[test]
    fn deserialize_dds_deserialize_compound() {
        let expected = PlOuter {
            outer: Parameter(7),
            inner: PlInner {
                id: Parameter(21),
                n: Parameter(34),
            },
        };
        let data = &[
            0x00, 0x03, 0, 0, // representation identifier
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
            2, 0x00, 4, 0, // outer | Length (incl padding)
            7, 0, 0, 0, // u8
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];

        let result: PlOuter = dds_deserialize(data).unwrap();
        assert_eq!(result, expected)
    }

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct UserInner {
        id: u8,
        n: u32,
    }
    #[test]
    fn deserialize_cdr_simple() {
        let expected = UserInner { id: 21, n: 34 };
        let data = &[
            0x00, 0x01, 0, 0, // representation identifier
            21, 0, 0, 0, // id
            34, 0, 0, 0, // n
        ][..];
        let result: UserInner = dds_deserialize(data).unwrap();
        assert_eq!(result, expected)
    }
}
