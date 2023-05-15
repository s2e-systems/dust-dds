use std::{self, marker::PhantomData};

use serde::de::{self};
use serde::ser::SerializeTuple;

use crate::implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter<const PID: u16, T>(pub T);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParameterWithDefault<const PID: u16, T>(pub T);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParameterVector<const PID: u16, T>(pub Vec<T>);

impl<const PID: u16, T> From<T> for Parameter<PID, T> {
    fn from(v: T) -> Self {
        Parameter(v)
    }
}

impl<const PID: u16, T> From<T> for ParameterWithDefault<PID, T> {
    fn from(v: T) -> Self {
        ParameterWithDefault(v)
    }
}

impl<const PID: u16, T> From<Vec<T>> for ParameterVector<PID, T> {
    fn from(v: Vec<T>) -> Self {
        ParameterVector(v)
    }
}

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

impl<const PID: u16, T> serde::Serialize for ParameterVector<PID, T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0.is_empty() {
            return serializer.serialize_tuple(0)?.end();
        }
        let length_without_padding = cdr::size::calc_serialized_data_size(&self.0[0]) as i16;
        let padding_length = (4 - length_without_padding) & 3;
        let length = length_without_padding + padding_length;

        let mut s = serializer.serialize_tuple(4)?;
        for value in &self.0 {
            s.serialize_element(&PID)?;
            s.serialize_element(&length)?;
            s.serialize_element(value)?;
            match padding_length {
                1 => s.serialize_element(&[0u8; 1])?,
                2 => s.serialize_element(&[0u8; 2])?,
                3 => s.serialize_element(&[0u8; 3])?,
                _ => s.serialize_element(&[0u8; 0])?,
            }
        }
        s.end()
    }
}

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

impl<'de, const PID: u16, T> serde::Deserialize<'de> for ParameterWithDefault<PID, T>
where
    T: serde::Deserialize<'de> + Default,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de, const PID: u16, T>
        where
            T: serde::Deserialize<'de>,
        {
            marker: PhantomData<ParameterWithDefault<PID, T>>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de, const PID: u16, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
        where
            T: serde::Deserialize<'de> + Default,
        {
            type Value = ParameterWithDefault<PID, T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ParameterWithDefault")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<u16>()? {
                    if key == PID {
                        return Ok(ParameterWithDefault(map.next_value()?));
                    } else if key == PID_SENTINEL {
                        break;
                    }
                }
                Ok(ParameterWithDefault(T::default()))
            }
        }
        deserializer.deserialize_newtype_struct(
            "ParameterWithDefault",
            Visitor {
                marker: PhantomData::<ParameterWithDefault<PID, T>>,
                lifetime: PhantomData,
            },
        )
    }
}

impl<'de, const PID: u16, T> serde::Deserialize<'de> for ParameterVector<PID, T>
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
            marker: PhantomData<ParameterVector<PID, T>>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de, const PID: u16, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
        where
            T: serde::Deserialize<'de>,
        {
            type Value = ParameterVector<PID, T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ParameterVector")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut values = vec![];
                while let Some(key) = map.next_key::<u16>()? {
                    if key == PID {
                        values.push(map.next_value()?);
                    } else if key == PID_SENTINEL {
                        break;
                    }
                }
                Ok(ParameterVector(values))
            }
        }
        deserializer.deserialize_newtype_struct(
            "ParameterVector",
            Visitor {
                marker: PhantomData::<ParameterVector<PID, T>>,
                lifetime: PhantomData,
            },
        )
    }
}
