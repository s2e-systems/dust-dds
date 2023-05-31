use std::io::Read;

use super::{overall_structure::EndianWriteBytes, submessage_elements::{ParameterList, Parameter}};
use crate::implementation::{rtps::types::{EntityId, EntityKey, EntityKind, SequenceNumber}, data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL};


impl EndianWriteBytes for EntityKey {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        <[u8; 3]>::from(*self).as_slice().read(buf).unwrap()
    }
}

impl EndianWriteBytes for EntityKind {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        buf[0] = u8::from(*self);
        1
    }
}

impl EndianWriteBytes for EntityId {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        self.entity_key().endian_write_bytes::<E>(&mut buf[0..]);
        self.entity_kind().endian_write_bytes::<E>(&mut buf[3..]);
        4
    }
}

impl EndianWriteBytes for SequenceNumber {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        let high = (<i64>::from(*self) >> 32) as i32;
        let low = <i64>::from(*self) as i32;
        E::write_i32(&mut buf[0..], high);
        E::write_i32(&mut buf[4..], low);
        8
    }
}

impl EndianWriteBytes for Parameter  {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, mut buf: &mut [u8]) -> usize {
        let padding_len= match self.value().len() % 4 {
            1 => 3,
            2 => 2,
            3 => 1,
            _ => 0,
        };
        let length = self.value().len() + padding_len;
        E::write_u16(&mut buf[0..], self.parameter_id().into());
        E::write_i16(&mut buf[2..], length as i16);
        buf = &mut buf[4..];
        buf[..self.value().len()].copy_from_slice(self.value().as_ref());
        buf[self.value().len()..length].fill(0);
        4+length
    }
}

impl EndianWriteBytes for &ParameterList {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        let mut length = 0;
        for parameter in self.parameter().iter() {
            length += parameter.endian_write_bytes::<E>(&mut buf[length..]);
        }
        E::write_u16(&mut buf[length..], PID_SENTINEL);
        buf[length+2..length+4].fill(0);
        length+4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{
            overall_structure::into_bytes_le_vec, submessage_elements::Parameter, types::ParameterId,
        },
        types::{EntityKey, EntityKind},
    };

    #[test]
    fn serialize_parameter() {
        let parameter = Parameter::new(ParameterId(2), vec![5, 6, 7, 8]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_le_vec(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_non_multiple_4() {
        let parameter = Parameter::new(ParameterId(2), vec![5, 6, 7]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_le_vec(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 0,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_zero_size() {
        let parameter = Parameter::new(ParameterId(2), vec![]);
        assert_eq!(
            into_bytes_le_vec(parameter),
            vec![
            0x02, 0x00, 0, 0, // Parameter | length
        ]
        );
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter_1 = Parameter::new(ParameterId(2), vec![51, 61, 71, 81]);
        let parameter_2 = Parameter::new(ParameterId(3), vec![52, 62, 0, 0]);
        let parameter_list_submessage_element = &ParameterList::new(vec![parameter_1, parameter_2]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_le_vec(parameter_list_submessage_element), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 0, 0,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn serialize_parameter_list_empty() {
        let parameter = &ParameterList::empty();
        #[rustfmt::skip]
        assert_eq!(into_bytes_le_vec(parameter), vec![
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn serialize_sequence_number() {
        let data = SequenceNumber::new(7);
        let result = into_bytes_le_vec(data);
        assert_eq!(
            result,
            vec![
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ]
        );
    }

    #[test]
    fn serialize_entity_id() {
        let data = EntityId::new(EntityKey::new([1, 2, 3]), EntityKind::new(0x04));
        assert_eq!(
            into_bytes_le_vec(data),
            vec![
            1, 2, 3, 0x04, //value (long)
        ]
        );
    }
}
