use crate::implementation::{
    rtps::messages::submessage_elements::{Parameter, ParameterList},
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::WriteBytesExt;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::{rtps_udp_psm::mapping_traits::to_bytes_le, rtps::messages::types::ParameterId};

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
}
