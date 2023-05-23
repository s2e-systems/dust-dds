use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::implementation::{
    rtps::messages::{
        submessage_elements::{Parameter, ParameterList},
        types::ParameterId,
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for Parameter {
    fn mapping_write_byte_ordered<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), std::io::Error> {
        let padding: &[u8] = match self.value().len() % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        let length = self.value().len() + padding.len();
        writer.write_u16::<B>(self.parameter_id().into())?;
        writer.write_i16::<B>(length as i16)?;
        writer.write_all(self.value().as_ref())?;

        writer.write_all(padding)?;
        Ok(())
    }
}

impl<'de> MappingReadByteOrdered<'de> for Parameter {
    fn mapping_read_byte_ordered<B: byteorder::ByteOrder>(
        buf: &mut &'de [u8],
    ) -> Result<Self, std::io::Error> {
        let parameter_id = buf.read_u16::<B>()?;
        let length = buf.read_i16::<B>()?;

        let value = if parameter_id == PID_SENTINEL {
            &[]
        } else {
            let (value, following) = buf.split_at(length as usize);
            *buf = following;
            value
        };

        Ok(Self::new(ParameterId(parameter_id), value.to_vec()))
    }
}

impl NumberOfBytes for Parameter {
    fn number_of_bytes(&self) -> usize {
        let padding_length = match self.value().len() % 4 {
            1 => 3,
            2 => 2,
            3 => 1,
            _ => 0,
        };
        4 /* parameter_id and length */ + self.value().len() + padding_length
    }
}

impl NumberOfBytes for ParameterList {
    fn number_of_bytes(&self) -> usize {
        self.parameter().number_of_bytes() + 4 /* Sentinel */
    }
}

const PID_SENTINEL: u16 = 1;

impl MappingWriteByteOrdered for ParameterList {
    fn mapping_write_byte_ordered<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), std::io::Error> {
        for parameter in self.parameter().iter() {
            parameter.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        PID_SENTINEL.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        [0u8; 2].mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ParameterList {
    fn mapping_read_byte_ordered<B: byteorder::ByteOrder>(
        buf: &mut &'de [u8],
    ) -> Result<Self, std::io::Error> {
        const MAX_PARAMETERS: usize = 2_usize.pow(16);

        let mut parameter = vec![];

        for _ in 0..MAX_PARAMETERS {
            let parameter_i: Parameter =
                MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;

            if parameter_i.parameter_id() == ParameterId(PID_SENTINEL) {
                break;
            } else {
                parameter.push(parameter_i);
            }
        }
        Ok(Self::new(parameter))
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le};

    use super::*;

    #[test]
    fn serialize_parameter() {
        let parameter = Parameter::new(ParameterId(2), vec![5, 6, 7, 8]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
        assert_eq!(parameter.number_of_bytes(), 8)
    }

    #[test]
    fn serialize_parameter_non_multiple_4() {
        let parameter = Parameter::new(ParameterId(2), vec![5, 6, 7]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 0,       // value
        ]);
        assert_eq!(parameter.number_of_bytes(), 8)
    }

    #[test]
    fn serialize_parameter_zero_size() {
        let parameter = Parameter::new(ParameterId(2), vec![]);
        assert_eq!(
            to_bytes_le(&parameter).unwrap(),
            vec![
            0x02, 0x00, 0, 0, // Parameter | length
        ]
        );
        assert_eq!(parameter.number_of_bytes(), 4)
    }

    #[test]
    fn deserialize_parameter_non_multiple_of_4() {
        let expected = Parameter::new(ParameterId(2), vec![5, 6, 7, 8, 9, 10, 11, 0]);
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
        let expected = Parameter::new(ParameterId(2), vec![5, 6, 7, 8, 9, 10, 11, 12]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            0x02, 0x00, 8, 0, // Parameter | length
            5, 6, 7, 8,       // value
            9, 10, 11, 12,       // value
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter_1 = Parameter::new(ParameterId(2), vec![51, 61, 71, 81]);
        let parameter_2 = Parameter::new(ParameterId(3), vec![52, 62, 0, 0]);
        let parameter_list_submessage_element = ParameterList::new(vec![parameter_1, parameter_2]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter_list_submessage_element).unwrap(), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 0, 0,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
        assert_eq!(parameter_list_submessage_element.number_of_bytes(), 20);
    }

    #[test]
    fn serialize_parameter_list_empty() {
        let parameter = ParameterList::empty();
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&parameter).unwrap(), vec![
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
        assert_eq!(parameter.number_of_bytes(), 4);
    }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterList::new(vec![
            Parameter::new(ParameterId(2), vec![15, 16, 17, 18]),
            Parameter::new(ParameterId(3), vec![25, 26, 27, 28]),
        ]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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
        let parameter_value_expected = vec![
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];

        let expected = ParameterList::new(vec![Parameter::new(
            ParameterId(0x32),
            parameter_value_expected,
        )]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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
        let parameter_value_expected1 = vec![
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];
        #[rustfmt::skip]
        let parameter_value_expected2 = vec![
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
        ];

        let expected = ParameterList::new(vec![
            Parameter::new(ParameterId(0x32), parameter_value_expected1),
            Parameter::new(ParameterId(0x32), parameter_value_expected2),
        ]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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
