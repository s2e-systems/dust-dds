use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use rust_rtps_pim::messages::{
    submessage_elements::{Parameter, ParameterListSubmessageElementType},
    types::ParameterId,
};

const PID_SENTINEL: u16 = 1;
const SENTINEL: ParameterUdp = ParameterUdp {
    parameter_id: PID_SENTINEL,
    length: 0,
    value: &[],
};

#[derive(Debug, PartialEq)]
pub struct ParameterUdp<'a> {
    parameter_id: u16,
    length: i16,
    value: &'a [u8],
}

impl<'a> ParameterUdp<'a> {
    pub fn new(parameter_id: u16, value: &'a [u8]) -> Self {
        let length = ((value.len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value,
        }
    }
    pub fn number_of_bytes(&self) -> usize {
        4 + self.length as usize
    }
}

impl<'a> From<&'a Parameter<'a>> for ParameterUdp<'a> {
    fn from(value: &'a Parameter) -> Self {
        ParameterUdp::new(value.parameter_id.0, value.value)
    }
}

impl<'a> From<ParameterUdp<'a>> for Parameter<'a> {
    fn from(value: ParameterUdp<'a>) -> Self {
        Parameter {
            parameter_id: ParameterId(value.parameter_id),
            length: value.value.len() as i16,
            value: value.value,
        }
    }
}

impl crate::serialize::Serialize for u16 {
    fn serialize<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        writer.write_u16::<B>(*self)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for u16 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: byteorder::ByteOrder,
    {
        buf.read_u16::<B>()
    }
}
impl crate::serialize::Serialize for i16 {
    fn serialize<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        writer.write_i16::<B>(*self)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for i16 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: byteorder::ByteOrder,
    {
        buf.read_i16::<B>()
    }
}

impl<'a> crate::serialize::Serialize for &'a [u8] {
    fn serialize<W: std::io::Write, B: ByteOrder>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(self)
    }
}

impl<'a> crate::serialize::Serialize for ParameterUdp<'a> {
    fn serialize<W: std::io::Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        self.parameter_id.serialize::<_, B>(&mut writer)?;
        self.length.serialize::<_, B>(&mut writer)?;
        self.value.serialize::<_, B>(&mut writer)?;
        let padding: &[u8] = match self.value.len() % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        padding.serialize::<_, B>(&mut writer)
    }
}

impl<'a, 'de: 'a> crate::deserialize::Deserialize<'de> for ParameterUdp<'a> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let parameter_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let length = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let (value, following) = buf.split_at(length as usize);
        *buf = following;
        Ok(Self {
            parameter_id,
            length,
            value,
        })
    }
}

impl<'a> crate::serialize::Serialize for ParameterListUdp<'a> {
    fn serialize<W: std::io::Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        for parameter in &self.parameter {
            ParameterUdp::from(parameter).serialize::<_, B>(&mut writer)?;
        }
        SENTINEL.serialize::<_, B>(&mut writer)
    }
}

impl<'a, 'de: 'a> crate::deserialize::Deserialize<'de> for ParameterListUdp<'a> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        const MAX_PARAMETERS: usize = 2_usize.pow(16);

        let mut parameter = vec![];

        for _ in 0..MAX_PARAMETERS {
            let parameter_i: ParameterUdp =
                crate::deserialize::Deserialize::deserialize::<B>(buf).unwrap();

            if parameter_i == SENTINEL {
                return Ok(ParameterListUdp { parameter });
            } else {
                parameter.push(Parameter::from(parameter_i));
            }
        }
        Ok(ParameterListUdp { parameter })
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListUdp<'a> {
    pub parameter: Vec<Parameter<'a>>,
}

impl<'a> ParameterListUdp<'a> {
    pub fn number_of_bytes(&self) -> usize {
        self.parameter
            .iter()
            .map(|p| ParameterUdp::from(p).number_of_bytes())
            .sum::<usize>()
            + 4
    }
    pub fn get<'b: 'de, 'de, T: serde::Deserialize<'de>>(&'b self, parameter_id: u16) -> Option<T> {
        for parameter in &self.parameter {
            if parameter.parameter_id.0 == parameter_id {
                return Some(rust_serde_cdr::deserializer::from_bytes(parameter.value).unwrap());
            }
        }
        None
    }

    pub fn get_list<'b: 'de, 'de, T: serde::Deserialize<'de>>(
        &'b self,
        parameter_id: u16,
    ) -> Vec<T> {
        let mut list = Vec::new();
        for parameter in &self.parameter {
            if parameter.parameter_id.0 == parameter_id {
                let value = rust_serde_cdr::deserializer::from_bytes(parameter.value).unwrap();
                list.push(value);
            }
        }
        list
    }
}

impl<'a> ParameterListSubmessageElementType<'a> for ParameterListUdp<'a> {
    fn new(parameter: &'a [Parameter]) -> Self {
        Self {
            parameter: parameter.to_vec(),
        }
    }

    fn parameter(&self) -> &[Parameter<'a>] {
        self.parameter.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_parameter() {
        let parameter = ParameterUdp::new(2, &[5, 6, 7, 8]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_non_multiple_4() {
        let parameter = ParameterUdp::new(2, &[5, 6, 7]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 0,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_zero_size() {
        let parameter = ParameterUdp::new(2, &[]);
        assert_eq!(
            to_bytes_le(&parameter).unwrap(),
            vec![
            0x02, 0x00, 0, 0, // Parameter | length
        ]
        );
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(2, &[51, 61, 71, 81]).into(),
                ParameterUdp::new(3, &[52, 62, 72, 82]).into(),
            ],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 72, 82,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn deserialize_parameter_non_multiple_of_4() {
        let expected = ParameterUdp::new(0x02, &[5, 6, 7, 8, 9, 10, 11, 0]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 0,     // value
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter() {
        let expected = ParameterUdp::new(0x02, &[5, 6, 7, 8, 9, 10, 11, 12]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 12,       // value
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(0x02, &[15, 16, 17, 18]).into(),
                ParameterUdp::new(0x03, &[25, 26, 27, 28]).into(),
            ],
        };
        #[rustfmt::skip]
        let result: ParameterListUdp = from_bytes_le(&[
            0x02, 0x00, 4, 0, // Parameter ID | length
            15, 16, 17, 18,        // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            25, 26, 27, 28,        // value
            0x01, 0x00, 0, 0, // Sentinel: Parameter ID | length
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list_with_long_parameter_including_sentinel() {
        #[rustfmt::skip]
        let parameter_value_expected = &[
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];

        let expected = ParameterListUdp {
            parameter: vec![ParameterUdp::new(0x32, parameter_value_expected).into()],
        };
        #[rustfmt::skip]
        let result: ParameterListUdp = from_bytes_le(&[
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

        let expected = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(0x32, parameter_value_expected1).into(),
                ParameterUdp::new(0x32, parameter_value_expected2).into(),
            ],
        };
        #[rustfmt::skip]
        let result: ParameterListUdp = from_bytes_le(&[
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

    #[test]
    fn paramter_submessage_element_roundtrip() {
        let parameter_value1 = [1, 2];
        let parameter_value2 = [3, 4, 5];
        let parameter1 = Parameter {
            parameter_id: ParameterId(1),
            length: parameter_value1.len() as i16,
            value: &parameter_value1,
        };
        let parameter2 = Parameter {
            parameter_id: ParameterId(2),
            length: parameter_value2.len() as i16,
            value: &parameter_value2,
        };
        let parameter = &[parameter1, parameter2];
        let submessage_element: ParameterListUdp =
            ParameterListSubmessageElementType::new(parameter);
        let result = submessage_element.parameter();
        assert_eq!(parameter, result);
    }
}
