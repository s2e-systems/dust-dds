//! Deserializing CDR into Rust data types.

use std::{self, marker::PhantomData, io::Read};

use byteorder::ByteOrder;
use serde::de::{self};

use cdr::{Error, Result, SizeLimit};

/// A deserializer that reads bytes from a buffer.
pub struct ParameterListDeserializer<E> {
    reader: Vec<u8>,
    phantom: PhantomData<E>,
}

impl<E> ParameterListDeserializer<E>
where
    E: ByteOrder,
{
    pub fn new(reader: Vec<u8>) -> Self {
        Self {
            reader,
            phantom: PhantomData,
        }
    }
}

impl<'de, 'a, E> de::Deserializer<'de> for &'a mut ParameterListDeserializer<E>
where
    E: ByteOrder,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::DeserializeAnyNotSupported)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(self.reader.as_slice())
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
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
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
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
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        struct Access<R, S, E> {
            de: cdr::Deserializer<R,S,E>,
        }
        impl<'de, R, S, E> de::MapAccess<'de> for Access<R, S, E>
        where
            E: ByteOrder,
            S: SizeLimit,
            R: Read
        {
            type Error = Error;

            fn next_key_seed<K>(
                &mut self,
                seed: K,
            ) -> std::result::Result<Option<K::Value>, Self::Error>
            where
                K: de::DeserializeSeed<'de>,
            {
                let res = seed.deserialize(&mut self.de).map(Some);
                res
            }

            fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
            where
                V: de::DeserializeSeed<'de>,
            {
                let _length: u16 = serde::Deserialize::deserialize(&mut self.de)?;
                let res = seed.deserialize(&mut self.de);
                res
            }
        }
        let reader = self.reader.as_slice();
        let cdr_deserializer = cdr::Deserializer::<_,_,E>::new(reader, cdr::Infinite);
        visitor.visit_map(Access { de: cdr_deserializer })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::TypeNotSupported)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(&self.reader)
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

#[derive(serde::Deserialize)]
struct ParamHeader {
    pid: u16,
    length: i16,
}

#[derive(Debug, PartialEq)]
struct Parameter<const PID: u16, T> {
    value: T,
}

impl<'de, const PID: u16, T> serde::Deserialize<'de> for Parameter<PID, T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Field0,
            Ignore,
        }
        struct FieldVisitor;

        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("field identifier")
            }
            fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0u64 => Ok(Field::Field0),
                    _ => Ok(Field::Ignore),
                }
            }
            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "value" => Ok(Field::Field0),
                    _ => Ok(Field::Ignore),
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"value" => Ok(Field::Field0),
                    _ => Ok(Field::Ignore),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }

        struct Visitor<'de, const PID: u16, T>
        where
            T: serde::Deserialize<'de>,
        {
            marker: PhantomData<Parameter<PID, T>>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de, const PID: u16, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
        where
            T: serde::Deserialize<'de>,
        {
            type Value = Parameter<PID, T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Parameter")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut value: Option<T> = None;
                while let Some(pid) = map.next_key::<u16>()? {
                    if pid == PID {
                        value = Some(map.next_value::<T>()?);
                    } else {
                        let _ = map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                let value = match value {
                    Some(value) => value,
                    None => serde::__private::de::missing_field("value")?,
                };
                Ok(Parameter { value })
            }
        }
        const FIELDS: &'static [&'static str] = &["value"];
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Parameter",
            FIELDS,
            Visitor {
                marker: PhantomData::<Parameter<PID, T>>,
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Inner {
        id: Parameter<71, u8>,
        n: Parameter<72, u8>,
    }

    #[test]
    fn deserialize_simple() {
        // let expected = Inner {
        //     id: Parameter(21),
        //     n: Parameter(34),
        // };
        // let mut data = &[
        //     71, 0x00, 4, 0, // id | Length (incl padding)
        //     21, 0, 0, 0, // u8
        //     72, 0x00, 4, 0, // n | Length (incl padding)
        //     34, 0, 0, 0, // u8
        // ][..];
        let data = vec![
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
        ];
        let expected = Parameter::<71, _> { value: 21u8 };
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: Parameter<71, u8> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }
}
