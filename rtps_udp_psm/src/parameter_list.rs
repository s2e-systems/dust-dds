use std::{collections::HashMap, io::BufRead, marker::PhantomData};

use rust_rtps_pim::messages::types::ParameterId;
use rust_serde_cdr::deserializer::RtpsMessageDeserializer;
use serde::ser::SerializeStruct;

#[derive(Debug, PartialEq)]
pub struct VectorUdp(pub Vec<u8>);
impl serde::Serialize for VectorUdp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ParameterUdpRef<'a> {
    pub parameter_id: u16,
    pub length: i16,
    #[serde(with = "serde_bytes")]
    pub value: &'a[u8],
}

impl<'a> ParameterUdpRef<'a> {
    pub fn new(parameter_id: u16, value: &'a [u8]) -> Self {
        let length = value.len() as i16;
        Self {
            parameter_id,
            length,
            value,
        }
    }
    pub fn len(&self) -> usize {
        4 + self.value.len()
    }
}

struct ParameterVisitorRef<'a>(PhantomData<&'a()>);

impl<'a, 'de:'a> serde::de::Visitor<'de> for ParameterVisitorRef<'a> {
    type Value = ParameterUdpRef<'a>;

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
        let data: &[u8] =
                seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
        Ok(ParameterUdpRef {
            parameter_id,
            length,
            value: &data[..length as usize],
        })
    }
}

impl<'a,'de:'a> serde::Deserialize<'de> for ParameterUdpRef<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            const FIELDS: &[&str] = &["parameter_id", "length", "value"];
            deserializer.deserialize_struct("Parameter", FIELDS, ParameterVisitorRef(PhantomData))
    }
}

const PID_SENTINEL: u16 = 1;
static SENTINEL_REF: ParameterUdpRef = ParameterUdpRef {
    parameter_id: PID_SENTINEL,
    length: 0,
    value: &[],
};


impl<'a> rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementType<'a>
    for ParameterListUdpRef<'a>
{
    type IntoIter = Vec<rust_rtps_pim::messages::submessage_elements::Parameter<'a>>;

    fn new(parameter: &[rust_rtps_pim::messages::submessage_elements::Parameter]) -> Self {
        // let mut parameter_list = vec![];
        // for parameter_i in parameter {
        //     let parameter_i_udp = ParameterUdpRef {
        //         parameter_id: parameter_i.parameter_id.0,
        //         length: parameter_i.length,
        //         value: parameter_i.value,
        //     };
        //     parameter_list.push(parameter_i_udp);
        // }
        // Self {
        //     parameter: parameter_list,
        // };
        todo!()
    }

    fn parameter(&'a self) -> Self::IntoIter {
        self.parameter
            .iter()
            .map(
                |x| rust_rtps_pim::messages::submessage_elements::Parameter {
                    parameter_id: ParameterId(x.parameter_id),
                    length: x.value.len() as i16,
                    value: x.value,
                },
            )
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListUdpRef<'a> {
    pub parameter: Vec<ParameterUdpRef<'a>>,
}

impl<'a> ParameterListUdpRef<'a> {
    pub fn len(&self) -> usize {
        self.parameter.iter().map(|p| p.len()).sum::<usize>() + 4
    }
    pub fn get<'b: 'de, 'de, T: serde::Deserialize<'de>>(&'b self, parameter_id: u16) -> Option<T> {
        let map = self.as_map();
        let bytes = map.get(&parameter_id)?;
        rust_serde_cdr::deserializer::from_bytes(bytes).ok()
    }

    pub fn get_list<'b: 'de, 'de, T: serde::Deserialize<'de>>(
        &'b self,
        parameter_id: u16,
    ) -> Vec<T> {
        let mut list = Vec::new();
        for parameter in &self.parameter {
            if parameter.parameter_id == parameter_id {
                let value =
                    rust_serde_cdr::deserializer::from_bytes(parameter.value).unwrap();
                list.push(value);
            }
        }
        list
    }

    fn as_map(&self) -> HashMap<u16, &[u8]> {
        let mut map = HashMap::new();
        for parameter_i in &self.parameter {
            map.insert(parameter_i.parameter_id, parameter_i.value);
        }
        map
    }
}

impl<'a> serde::Serialize for ParameterListUdpRef<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.parameter.len() + 1;
        let mut state = serializer.serialize_struct("ParameterList", len)?;
        for parameter in &self.parameter {
            state.serialize_field("parameter", &parameter)?;
        }
        state.serialize_field("sentinel", &SENTINEL_REF)?;
        state.end()
    }
}


struct ParameterListVisitorRef<'a>(PhantomData<&'a()>);

impl<'a, 'de:'a> serde::de::Visitor<'de> for ParameterListVisitorRef<'a> {
    type Value = ParameterListUdpRef<'a>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        const MAX_PARAMETERS: usize = 2_usize.pow(16);

        let buf: &[u8] = seq
        .next_element()?
        .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        let mut parameter = vec![];

        let mut de = RtpsMessageDeserializer { reader: buf };

        for _ in 0..MAX_PARAMETERS {
            let parameter_i: ParameterUdpRef = serde::de::Deserialize::deserialize(&mut de).unwrap();
            // de.reader.consume(parameter)
            if parameter_i == SENTINEL_REF {
                return Ok(ParameterListUdpRef {
                    parameter,
                });
            } else {
                parameter.push(parameter_i);
            }
        }
        Ok(ParameterListUdpRef{parameter})
    }
}

impl<'a,'de:'a> serde::Deserialize<'de> for ParameterListUdpRef<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["parameter"];
        deserializer.deserialize_struct("ParameterList", FIELDS, ParameterListVisitorRef(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use rust_serde_cdr::deserializer::{self, RtpsMessageDeserializer};

    use super::*;

    #[test]
    fn serialize_parameter() {
        let parameter = ParameterUdpRef::new(2, &[5, 6, 7, 8]);
        #[rustfmt::skip]
        assert_eq!(rust_serde_cdr::serializer::to_bytes(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }


    #[test]
    fn serialize_parameter_non_multiple_4() {
        let parameter = ParameterUdpRef::new(2, &[5, 6, 7]);
        #[rustfmt::skip]
        assert_eq!(rust_serde_cdr::serializer::to_bytes(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 0,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_zero_size() {
        let parameter = ParameterUdpRef::new(2, &[]);
        assert_eq!(
            rust_serde_cdr::serializer::to_bytes(&parameter).unwrap(),
            vec![
            0x02, 0x00, 0, 0, // Parameter | length
        ]
        );
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter = ParameterListUdpRef {
            parameter: vec![
                ParameterUdpRef::new(2, &[51, 61, 71, 81]),
                ParameterUdpRef::new(3, &[52, 62, 72, 82]),
            ]
            .into(),
        };
        #[rustfmt::skip]
        assert_eq!(rust_serde_cdr::serializer::to_bytes(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 72, 82,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn deserialize_parameter_non_multiple_of_4() {
        let expected = ParameterUdpRef::new(0x02, &[5, 6, 7, 8, 9, 10, 11]);
        #[rustfmt::skip]
        let result = deserializer::from_bytes(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 0,     // value
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter() {
        let expected = ParameterUdpRef::new(0x02, &[5, 6, 7, 8, 9, 10, 11, 12]);
        #[rustfmt::skip]
        let result = deserializer::from_bytes(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 12,       // value
        ]).unwrap();
        assert_eq!(expected, result);
    }


    // #[test]
    // fn deserialize_parameter_ref_multiple() {

    //     #[rustfmt::skip]
    //     let buf = &[
    //         0x02_u8, 0x00, 4, 0, // Parameter | length
    //         5, 6, 7, 8,       // value
    //         0x03_u8, 0x00, 4, 0, // Parameter | length
    //         9, 10, 11, 12,       // value
    //     ][..];
    //     let mut de = RtpsMessageDeserializer { reader: buf };

    //     let expected1 = ParameterUdpRef{parameter_id: 2, length: 4,  value: &[5, 6, 7, 8]};

    //     let expected2 = ParameterUdpRef{parameter_id: 3, length: 4,  value: &[9, 10, 11, 12]};

    //     let result1 = serde::de::Deserialize::deserialize(&mut de).unwrap();
    //     assert_eq!(expected1, result1);
    //     let result2 = serde::de::Deserialize::deserialize(&mut de).unwrap();
    //     assert_eq!(expected2, result2);
    // }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterListUdpRef {
            parameter: vec![
                ParameterUdpRef::new(0x02, &[15, 16, 17, 18]),
                ParameterUdpRef::new(0x03, &[25, 26, 27, 28]),
            ]
            .into(),
        };
        #[rustfmt::skip]
        let result: ParameterListUdpRef = deserializer::from_bytes(&[
            0x02, 0x00, 4, 0, // Parameter ID | length
            15, 16, 17, 18,        // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            25, 26, 27, 28,        // value
            0x01, 0x00, 0, 0, // Sentinel: Parameter ID | length
        ]).unwrap();
        assert_eq!(expected, result);
    }


    #[test]
    fn deserialize_parameter_list_with_long_parameter() {
        #[rustfmt::skip]
        let parameter_value_expected = &[
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];

        let expected = ParameterListUdpRef {
            parameter: vec![ParameterUdpRef::new(0x32, parameter_value_expected)].into(),
        };
        #[rustfmt::skip]
        let result: ParameterListUdpRef = deserializer::from_bytes(&[
            0x32, 0x00, 24, 0x00, // Parameter ID | length
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list_with_multiple_parameters_with_same_id() {
        #[rustfmt::skip]
        let parameter_value_expected1 = &[
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];
        let parameter_value_expected2 = &[
            0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
        ];

        let expected = ParameterListUdpRef {
            parameter: vec![
                ParameterUdpRef::new(0x32, parameter_value_expected1),
                ParameterUdpRef::new(0x32, parameter_value_expected2),
            ]
            .into(),
        };
        #[rustfmt::skip]
        let result: ParameterListUdpRef = deserializer::from_bytes(&[
            0x32, 0x00, 24, 0x00, // Parameter ID | length
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x01, 0x01, 0x01, 0x01, // Parameter value
            0x32, 0x00, 24, 0x00, // Parameter ID | length
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x01, 0x00, 0x00, 0x00, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x02, 0x02, 0x02, 0x02, // Parameter value
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
