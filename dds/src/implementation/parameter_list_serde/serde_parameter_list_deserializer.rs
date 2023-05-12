use std::{self, io::Read, marker::PhantomData};

use byteorder::ReadBytesExt;
use serde::de::{self};

use cdr::Error;

//use serde::de::value::Error;
// use std::io::Error;

pub struct ParameterListDeserializer<'a> {
    data: &'a [u8],
    pos: u64,
    is_big_endian: bool,
    is_pl: bool,
}

impl<'a> ParameterListDeserializer<'a> {
    pub fn new(reader: &'a [u8]) -> Self {
        let is_big_endian = match reader[1] {
            0 | 2 => true,
            1 | 3 => false,
            _ => todo!(),
        };
        let is_pl = match reader[1] {
            2 | 3 => true,
            0 | 1 => false,
            _ => todo!(),
        };
        Self {
            data: &reader[4..],
            pos: 0,
            is_big_endian,
            is_pl,
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

impl<'de, 'b> de::Deserializer<'de> for &'b mut ParameterListDeserializer<'de> {
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
        let v = if self.is_big_endian {
            self.data.read_u16::<byteorder::BigEndian>()?
        } else {
            self.data.read_u16::<byteorder::LittleEndian>()?
        };
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.read_padding_of::<u32>()?;
        self.read_size_of::<u32>()?;
        let v = if self.is_big_endian {
            self.data.read_u32::<byteorder::BigEndian>()?
        } else {
            self.data.read_u32::<byteorder::LittleEndian>()?
        };
        visitor.visit_u32(v)
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
        let v = if self.is_big_endian {
            self.data.read_i16::<byteorder::BigEndian>()
        } else {
            self.data.read_i16::<byteorder::LittleEndian>()
        }?;
        visitor.visit_i16(v)
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
        if self.is_pl {
            visitor.visit_map(PidSeparated::new(self))
        } else {
            visitor.visit_newtype_struct(self)
        }
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
        struct Access<'c, 'de: 'c> {
            deserializer: &'c mut ParameterListDeserializer<'de>,
            len: usize,
        }

        impl<'de, 'd> de::SeqAccess<'de> for Access<'d, 'de> {
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

struct PidSeparated<'de> {
    de: ParameterListDeserializer<'de>,
    skip_length: usize,
}

impl<'a> PidSeparated<'a> {
    fn new(de: &ParameterListDeserializer<'a>) -> Self {
        PidSeparated {
            de: ParameterListDeserializer {
                data: de.data,
                pos: 0,
                is_big_endian: de.is_big_endian,
                is_pl: de.is_pl,
            },
            skip_length: 0,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for PidSeparated<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.de.data = &self.de.data[self.skip_length as usize..];

        let pid = seed.deserialize(&mut self.de).map(Some);
        let length: i16 = serde::Deserialize::deserialize(&mut self.de)?;
        self.skip_length = length as usize;
        pid
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let length_before_deserialization = self.de.data.len();
        let value = seed.deserialize(&mut self.de);
        let length_deserialized = length_before_deserialization - self.de.data.len();

        let padding_length = self.skip_length - length_deserialized;
        self.de.data = &self.de.data[padding_length as usize..];
        self.skip_length = 0;
        value
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
            fn visit_newtype_struct<E>(
                self,
                deserializer: E,
            ) -> std::result::Result<Self::Value, E::Error>
            where
                E: serde::Deserializer<'de>,
            {
                Ok(Parameter(serde::Deserialize::deserialize(deserializer)?))
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                loop {
                    if let Some(key) = map.next_key::<u16>()? {
                        if key == PID {
                            return Ok(Parameter(map.next_value()?));
                        }
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

    #[test]
    fn deserialize_one_parameter_le() {
        let data = &[
            0x00, 0x03, 0, 0, // representation identifier
            71, 0x00, 4, 0, // pid | Length (incl padding)
            21, 0, 0, 0, // u8
        ][..];
        let expected: Parameter<71, u8> = Parameter(21);
        let mut deserializer = ParameterListDeserializer::new(data);
        let result: Parameter<71, u8> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }

    #[test]
    fn deserialize_one_parameter_be() {
        let data = &[
            0x00, 0x02, 0, 0, // representation identifier
            0, 71, 0, 4, // pid | Length (incl padding)
            0, 0, 0, 21, // u32
        ][..];
        let expected: Parameter<71, u32> = Parameter(21);
        let mut deserializer = ParameterListDeserializer::new(data);
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
            0x00, 0x03, 0, 0, // representation identifier
            72, 0x00, 4, 0, // n | Length (incl padding)
            34, 0, 0, 0, // u8
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            1, 0, 0, 0, // Sentinel
        ][..];
        let mut deserializer = ParameterListDeserializer::new(data);
        let result: PlInner = serde::Deserialize::deserialize(&mut deserializer).unwrap();
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
        let mut deserializer = ParameterListDeserializer::new(data);
        let result: UserInner = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(result, expected)
    }
}
