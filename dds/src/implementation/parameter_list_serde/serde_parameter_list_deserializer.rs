//! Deserializing CDR into Rust data types.

use std::{self, io::Read, marker::PhantomData};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use serde::de::{self, IntoDeserializer};

use cdr::size::{Infinite, SizeLimit};
use cdr::{Error, Result};

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

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
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
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        struct Access<'a, E> {
            de: &'a mut ParameterListDeserializer<E>,
        }
        impl<'de, 'a, E> de::MapAccess<'de> for Access<'a, E>
        where
            E: ByteOrder,
        {
            type Error = Error;

            fn next_key_seed<K>(
                &mut self,
                seed: K,
            ) -> std::result::Result<Option<K::Value>, Self::Error>
            where
                K: de::DeserializeSeed<'de>,
            {
                let res = seed.deserialize(&mut *self.de).map(Some);
                res
            }

            fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
            where
                V: de::DeserializeSeed<'de>,
            {
                let res = seed.deserialize(&mut *self.de);
                res
            }
        }
        visitor.visit_map(Access { de: self })
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

#[derive(Debug, PartialEq)]
struct Parameter<const PID: u16, T> {
    value: T,
}

#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[allow(unused_macros)]
    macro_rules! r#try {
    ($__expr:expr) => {
      match$__expr {
        _serde::__private::Ok(__val) => __val,_serde::__private::Err(__err) => {
          return _serde::__private::Err(__err);
        }
      }
    }
  }
    #[automatically_derived]
    impl<'de, const PID: u16, T> _serde::Deserialize<'de> for Parameter<PID, T>
    where
        T: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum Field {
                __field0,
                __ignore,
            }
            struct __FieldVisitor;

            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(Field::__field0),
                        _ => _serde::__private::Ok(Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "value" => _serde::__private::Ok(Field::__field0),
                        _ => _serde::__private::Ok(Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"value" => _serde::__private::Ok(Field::__field0),
                        _ => _serde::__private::Ok(Field::__ignore),
                    }
                }
            }


            impl<'de> _serde::Deserialize<'de> for Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct Visitor<'de, const PID: u16, T>
            where
                T: serde::Deserialize<'de>,
            {
                marker: _serde::__private::PhantomData<Parameter<PID, T>>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de, const PID: u16, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
            where
                T: serde::Deserialize<'de>,
            {
                type Value = Parameter<PID, T>;
                fn expecting(
                    &self,
                    formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(formatter, "struct Parameter")
                }

                fn visit_map<A>(
                    self,
                    mut map: A,
                ) -> _serde::__private::Result<Self::Value, A::Error>
                where
                    A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: Option<T> = None;

                    while let Some(key) = map.next_key::<Field>()? {
                        match key {
                            Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <A::Error as _serde::de::Error>::duplicate_field("value"),
                                    );
                                }
                                __field0 = _serde::__private::Some(r#try!(
                                    _serde::de::MapAccess::next_value::<T>(&mut map)
                                ));
                            }
                            _ => {
                                let _ = r#try!(_serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut map));
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => {
                            r#try!(_serde::__private::de::missing_field("value"))
                        }
                    };
                    _serde::__private::Ok(Parameter { value: __field0 })
                }
            }
            const FIELDS: &'static [&'static str] = &["value"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Parameter",
                FIELDS,
                Visitor {
                    marker: _serde::__private::PhantomData::<Parameter<PID, T>>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

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
        let data = vec![71];
        let expected = Parameter::<71, _> { value: 21u8 };
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: Parameter<71, u8> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }
}
