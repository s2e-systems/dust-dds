use std::{self, marker::PhantomData};

use byteorder::ByteOrder;
use serde::de::{self};

use crate::implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL;
use cdr::{Error, Result};

pub struct ParameterListDeserializer<'a, E> {
    reader: &'a [u8],
    phantom: PhantomData<E>,
}

impl<'a, E> ParameterListDeserializer<'a, E>
where
    E: ByteOrder,
{
    pub fn new(reader: &'a [u8]) -> Self {
        Self {
            reader,
            phantom: PhantomData,
        }
    }
}

impl<'de, 'a: 'b, 'b, E> de::Deserializer<'de> for &'b mut ParameterListDeserializer<'a, E>
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
        visitor.visit_bytes(self.reader)
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
        struct Access<'c, 'd, E>
        where
            E: ByteOrder,
        {
            deserializer: &'c mut ParameterListDeserializer<'d, E>,
            len: usize,
        }

        impl<'de, 'c, 'd, E> de::SeqAccess<'de> for Access<'c, 'd, E>
        where
            E: ByteOrder,
        {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
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
}

#[derive(Debug, PartialEq)]
struct Parameter<const PID: u16, T>(T);

impl<'de, const PID: u16, T> serde::Deserialize<'de> for Parameter<PID, T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                const PL_CDR_BE: &[u8] = &[0x00, 0x02];
                const PL_CDR_LE: &[u8] = &[0x00, 0x03];

                let (representation_identifier, v) = v.split_at(2);
                if representation_identifier != PL_CDR_BE && representation_identifier != PL_CDR_LE
                {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Bytes(representation_identifier),
                        &"PL_CDR_BE or PL_CDR_LE",
                    ));
                }
                let (_representation_options, mut reader) = v.split_at(2);
                loop {
                    let mut deserializer_be =
                        cdr::Deserializer::<_, _, byteorder::BigEndian>::new(reader, cdr::Infinite);
                    let mut deserializer_le =
                        cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(
                            reader,
                            cdr::Infinite,
                        );
                    let pid: u16 = if representation_identifier == PL_CDR_BE {
                        serde::Deserialize::deserialize(&mut deserializer_be)
                    } else {
                        serde::Deserialize::deserialize(&mut deserializer_le)
                    }
                    .map_err(|_err| serde::de::Error::missing_field("PID"))?;
                    let length: u16 = if representation_identifier == PL_CDR_BE {
                        serde::Deserialize::deserialize(&mut deserializer_be)
                    } else {
                        serde::Deserialize::deserialize(&mut deserializer_le)
                    }
                    .map_err(|_err| serde::de::Error::missing_field("length"))?;

                    if pid == PID {
                        let value: T = if representation_identifier == PL_CDR_BE {
                            serde::Deserialize::deserialize(&mut deserializer_be)
                        } else {
                            serde::Deserialize::deserialize(&mut deserializer_le)
                        }
                        .map_err(|_err| serde::de::Error::missing_field("value"))?;
                        return Ok(Parameter(value));
                    } else if pid == PID_SENTINEL {
                        return Err(serde::de::Error::missing_field("PID missing"));
                    } else {
                        let skip_bytes = length as usize + 4 /*number of bytes of pid and length */;
                        reader = &reader[skip_bytes..];
                    }
                }
            }
        }
        deserializer.deserialize_newtype_struct(
            "Parameter",
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

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Inner {
        id: Parameter<71, u8>,
        n: Parameter<72, u8>,
    }

    #[test]
    fn deserialize_simple() {
        let expected = Inner {
            id: Parameter(21),
            n: Parameter(34),
        };
        let data = &[
            0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
        let result: Inner = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }
}
