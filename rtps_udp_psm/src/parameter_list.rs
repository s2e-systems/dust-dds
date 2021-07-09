use rust_rtps_pim::messages::types::ParameterId;
use serde::ser::SerializeStruct;

#[derive(Debug, PartialEq, Clone)]
pub struct VectorUdp(Vec<u8>);
impl serde::Serialize for VectorUdp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

impl From<Vec<u8>> for VectorUdp {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterUdp {
    pub parameter_id: u16,
    pub length: i16,
    pub value: VectorUdp,
}

impl ParameterUdp {
    pub fn new(parameter_id: u16, value: VectorUdp) -> Self {
        Self {
            parameter_id,
            length: value.0.len() as i16,
            value,
        }
    }

    pub fn len(&self) -> u16 {
        4 + self.value.0.len() as u16
    }
}

impl serde::Serialize for ParameterUdp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Parameter", 3)?;
        state.serialize_field("ParameterId", &self.parameter_id)?;
        state.serialize_field("length", &self.length)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

struct ParameterVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterVisitor {
    type Value = ParameterUdp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Parameter of the ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let parameter_id: u16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let length: i16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let mut data = vec![];
        for _ in 0..length {
            data.push(
                seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?,
            );
        }
        Ok(ParameterUdp {
            parameter_id,
            length,
            value: data.into(),
        })
    }
}

impl<'de> serde::Deserialize<'de> for ParameterUdp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_BYTES: usize = 2 ^ 16;
        deserializer.deserialize_tuple(MAX_BYTES, ParameterVisitor {})
    }
}
const PID_SENTINEL: u16 = 1;
static SENTINEL: ParameterUdp = ParameterUdp {
    parameter_id: PID_SENTINEL,
    length: 0,
    value: VectorUdp(vec![]),
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterListUdp {
    pub(crate) parameter: Vec<ParameterUdp>,
}
impl serde::Serialize for ParameterListUdp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.parameter.len();
        let mut state = serializer.serialize_struct("ParameterList", len)?;
        for parameter in &self.parameter {
            state.serialize_field("parameter", &parameter)?;
        }
        state.serialize_field("sentinel", &SENTINEL)?;
        state.end()
    }
}

struct ParameterListVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterListVisitor {
    type Value = ParameterListUdp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut parameters = vec![];
        for _ in 0..seq.size_hint().unwrap() {
            let parameter: ParameterUdp = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            if parameter == SENTINEL {
                return Ok(ParameterListUdp {
                    parameter: parameters.into(),
                });
            } else {
                parameters.push(parameter);
            }
        }
        todo!()
    }
}

impl<'de, 'a> serde::Deserialize<'de> for ParameterListUdp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_PARAMETERS: usize = 2 ^ 16;
        deserializer.deserialize_tuple(MAX_PARAMETERS, ParameterListVisitor {})
    }
}

impl ParameterListUdp {
    pub fn len(&self) -> u16 {
        self.parameter.iter().map(|p| p.len()).sum()
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementType<'a>
    for ParameterListUdp
{
    type IntoIter = Vec<rust_rtps_pim::messages::submessage_elements::Parameter<'a>>;

    fn new(parameter: &[rust_rtps_pim::messages::submessage_elements::Parameter]) -> Self {
        let mut parameter_list = vec![];
        for parameter_i in parameter {
            let parameter_i_udp = ParameterUdp {
                parameter_id: parameter_i.parameter_id.0,
                length: parameter_i.length,
                value: VectorUdp(parameter_i.value.iter().cloned().collect()),
            };
            parameter_list.push(parameter_i_udp);
        }
        Self {
            parameter: parameter_list,
        }
    }

    fn parameter(&'a self) -> Self::IntoIter {
        self.parameter
            .iter()
            .map(
                |x| rust_rtps_pim::messages::submessage_elements::Parameter {
                    parameter_id: ParameterId(x.parameter_id),
                    length: x.value.0.len() as i16,
                    value: x.value.0.as_ref(),
                },
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_serde_cdr::{
        deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer,
    };

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    fn deserialize<'de, T: serde::Deserialize<'de>>(buffer: &'de [u8]) -> T {
        let mut de = RtpsMessageDeserializer { reader: buffer };
        serde::de::Deserialize::deserialize(&mut de).unwrap()
    }

    #[test]
    fn serialize_parameter() {
        let parameter = ParameterUdp::new(2, vec![5, 6, 7, 8].into());
        #[rustfmt::skip]
        assert_eq!(serialize(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(2, vec![51, 61, 71, 81].into()),
                ParameterUdp::new(3, vec![52, 62, 72, 82].into()),
            ]
            .into(),
        };
        #[rustfmt::skip]
        assert_eq!(serialize(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 72, 82,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn deserialize_parameter() {
        let expected = ParameterUdp::new(0x02, vec![5, 6, 7, 8].into());
        #[rustfmt::skip]
        let result = deserialize(&[
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(0x02, vec![15, 16, 17, 18].into()),
                ParameterUdp::new(0x03, vec![25, 26, 27, 28].into()),
            ]
            .into(),
        };
        #[rustfmt::skip]
        let result: ParameterListUdp = deserialize(&[
            0x02, 0x00, 4, 0, // Parameter ID | length
            15, 16, 17, 18,        // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            25, 26, 27, 28,        // value
            0x01, 0x00, 0, 0, // Sentinel: Parameter ID | length
            9, 9, 9,    // Following data
        ]);
        assert_eq!(expected, result);
    }
}
