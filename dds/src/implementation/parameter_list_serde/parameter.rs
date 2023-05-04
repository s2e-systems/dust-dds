use std::marker::PhantomData;

use serde::ser::SerializeStruct;

type Pid = u16;
pub struct Parameter<T> {
    pid: Pid,
    value: T,
}

impl<T> Parameter<T> {
    pub fn new(pid: Pid, value: T) -> Self {
        Self { pid, value }
    }
}

impl<T: serde::Serialize> serde::Serialize for Parameter<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let length_without_padding = cdr::size::calc_serialized_data_size(&self.value) as i16;
        let padding_length = (4 - length_without_padding) & 3;
        let length = length_without_padding + padding_length;

        let mut s = serializer.serialize_struct("parameter", 4)?;
        s.serialize_field("pid", &self.pid)?;
        s.serialize_field("length", &length)?;
        s.serialize_field("value", &self.value)?;
        match padding_length {
            1 => s.serialize_field("padding", &[0u8; 1])?,
            2 => s.serialize_field("padding", &[0u8; 2])?,
            3 => s.serialize_field("padding", &[0u8; 3])?,
            _ => s.serialize_field("padding", &[0u8; 0])?,
        }
        s.end()
    }
}

struct ParameterVisitor<T> {
    phantom: PhantomData<T>,
}

impl<T> ParameterVisitor<T> {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<'de, T: serde::Deserialize<'de> + serde::Serialize> serde::de::Visitor<'de>
    for ParameterVisitor<T>
{
    type Value = Parameter<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Parameter")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let pid = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let length: i16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let value = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

        let length_without_padding = cdr::size::calc_serialized_data_size(&value) as i16;
        let padding_length = length - length_without_padding;
        match padding_length {
            1 => {
                let _padding: [u8; 1] = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
            }
            2 => {
                let _padding: [u8; 2] = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
            }
            3 => {
                let _padding: [u8; 3] = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
            }
            _ => (),
        };
        Ok(Parameter::new(pid, value))
    }
}

impl<'de, T: serde::Deserialize<'de> + serde::Serialize> serde::Deserialize<'de> for Parameter<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "parameter",
            &["pid", "length", "value", "padding"],
            ParameterVisitor::new(),
        )
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for ParameterWithDefault<T> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

pub struct ParameterWithDefault<T> {
    parameter: Parameter<T>,
    default: T,
}

impl<T> ParameterWithDefault<T> {
    pub fn new(pid: Pid, value: T) -> Self
    where
        T: Default,
    {
        Self {
            parameter: Parameter { pid, value },
            default: T::default(),
        }
    }
}
impl<T: serde::Serialize + PartialEq> serde::Serialize for ParameterWithDefault<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.parameter.value == self.default {
            serializer.serialize_struct("", 0)?.end()
        } else {
            self.parameter.serialize(serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;
    use crate::{
        implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
        topic_definition::type_support::{PL_CDR_LE, REPRESENTATION_OPTIONS},
    };

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Inner {
        id: Parameter<u8>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Outer {
        inner: Inner,
        qos: ParameterWithDefault<Qos>,
        text: Parameter<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq)]
    struct Qos {
        is: bool,
    }
    impl Default for Qos {
        fn default() -> Self {
            Self { is: false }
        }
    }

    #[test]
    fn serialize_simple() {
        let data = Outer {
            inner: Inner {
                id: Parameter::new(71, 21),
            },
            qos: ParameterWithDefault::new(72, Qos { is: false }),
            text: Parameter::new(73, "ab".to_string()),
        };
        let mut result = vec![];
        let mut serializer = cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut result);
        PL_CDR_LE.serialize(&mut serializer).unwrap();
        REPRESENTATION_OPTIONS.serialize(&mut serializer).unwrap();
        data.serialize(&mut serializer).unwrap();
        PID_SENTINEL.serialize(&mut serializer).unwrap();
        [0_u8, 0].serialize(&mut serializer).unwrap();

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE | OPTIONS
            71, 0x00, 4, 0, // id | Length (incl padding)
            21, 0, 0, 0, // u8
            73, 0x00, 8, 0x00, // text | Length  (incl padding)
            3, 0x00, 0x00, 0x00, // String length (incl. termination)
            b'a', b'b', 0, 0x00, // String + termination 0 + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        assert_eq!(result, expected)
    }
}
