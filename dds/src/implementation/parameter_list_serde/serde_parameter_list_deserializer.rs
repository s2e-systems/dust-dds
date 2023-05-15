use std::{self, marker::PhantomData};

use byteorder::ByteOrder;
use serde::de::{self};

use cdr::Error;

pub struct ParameterListDeserializer<'a, E> {
    data: &'a [u8],
    endianness: PhantomData<E>,
}

impl<'a, E> ParameterListDeserializer<'a, E> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            endianness: PhantomData,
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
        Err(Error::TypeNotSupported)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
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

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
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

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
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
    data: &'de [u8],
    skip_length: usize,
    byteorder: PhantomData<E>,
}

impl<'a, E> PidSeparated<'a, E> {
    fn new(de: &ParameterListDeserializer<'a, E>) -> Self
    where
        E: byteorder::ByteOrder,
    {
        PidSeparated {
            data: de.data,
            skip_length: 0,
            byteorder: PhantomData,
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
        self.data = &self.data[self.skip_length as usize..];

        let mut cdr_deserializer = cdr::Deserializer::<_, _, E>::new(self.data, cdr::Infinite);
        let pid = seed.deserialize(&mut cdr_deserializer).map(Some);
        let length: i16 = serde::Deserialize::deserialize(&mut cdr_deserializer)?;
        self.skip_length = length as usize;
        self.data = &self.data[4 as usize..];
        pid
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let mut cdr_deserializer = cdr::Deserializer::<_, _, E>::new(self.data, cdr::Infinite);
        let value = seed.deserialize(&mut cdr_deserializer);
        self.data = &self.data[self.skip_length..];
        self.skip_length = 0;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::parameter_list_serde::parameter::{
        Parameter, ParameterVector, ParameterWithDefault,
    };

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct CdrParameterType {
        i: u32,
    }

    #[test]
    fn deserialize_one_parameter_le() {
        let data = &[
            71, 0x00, 4, 0, // pid | Length (incl padding)
            21, 0, 0, 0, // CdrParameterType
        ][..];
        let expected: Parameter<71, CdrParameterType> = Parameter(CdrParameterType { i: 21 });
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: Parameter<71, CdrParameterType> =
            serde::Deserialize::deserialize(&mut deserializer).unwrap();
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
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            93, 0x00, 4, 0, // values | Length (incl padding)
            34, 0, 0, 0, // u16
            93, 0x00, 4, 0, // values | Length (incl padding)
            35, 0, 0, 0, // u16
            1, 0, 0, 0, // Sentinel
        ][..];
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: PlWithList = serde::Deserialize::deserialize(&mut deserializer).unwrap();
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
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
            2, 0x00, 4, 0, // outer | Length (incl padding)
            7, 0, 0, 0, // u8
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: PlOuter = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }
}
