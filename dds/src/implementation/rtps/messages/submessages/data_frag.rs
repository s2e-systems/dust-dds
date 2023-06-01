use std::io::BufRead;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::{
            overall_structure::{RtpsMap, SubmessageHeader, SubmessageHeaderRead},
            submessage_elements::ParameterList,
            types::{FragmentNumber, SerializedPayload, SubmessageFlag, ULong, UShort},
        },
        types::{EntityId, SequenceNumber},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for DataFragSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> DataFragSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn octets_to_inline_qos(&self) -> u16 {
        self.map(&self.data[6..])
    }

    fn inline_qos_len(&self) -> usize {
        if self.inline_qos_flag() {
            let mut parameter_list_buf = &self.data[8 + self.octets_to_inline_qos() as usize..];
            let parameter_list_buf_length = parameter_list_buf.len();
            loop {
                let pid: u16 = self.map(parameter_list_buf);
                parameter_list_buf.consume(2);
                let length: i16 = self.map(parameter_list_buf);
                parameter_list_buf.consume(2);
                if pid == PID_SENTINEL {
                    break;
                } else {
                    parameter_list_buf.consume(length as usize);
                }
            }
            parameter_list_buf_length - parameter_list_buf.len()
        } else {
            0
        }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn inline_qos_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn key_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0100) != 0
    }

    pub fn non_standard_payload_flag(&self) -> bool {
        (self.data[1] & 0b_0000_1000) != 0
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[12..])
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.map(&self.data[16..])
    }

    pub fn fragment_starting_num(&self) -> FragmentNumber {
        self.map(&self.data[24..])
    }

    pub fn fragments_in_submessage(&self) -> u16 {
        self.map(&self.data[28..])
    }

    pub fn fragment_size(&self) -> u16 {
        self.map(&self.data[30..])
    }

    pub fn data_size(&self) -> u32 {
        self.map(&self.data[32..])
    }

    pub fn inline_qos(&self) -> ParameterList {
        if self.inline_qos_flag() {
            self.map(&self.data[self.octets_to_inline_qos() as usize + 8..])
        } else {
            ParameterList::empty()
        }
    }

    pub fn serialized_payload(&self) -> SerializedPayload<'a> {
        self.map(&self.data[8 + self.octets_to_inline_qos() as usize + self.inline_qos_len()..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_starting_num: FragmentNumber,
    pub fragments_in_submessage: UShort,
    pub data_size: ULong,
    pub fragment_size: UShort,
    pub inline_qos: &'a ParameterList,
    pub serialized_payload: SerializedPayload<'a>,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::{
        messages::{submessage_elements::Parameter, types::ParameterId},
        types::{EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    use super::*;

    #[test]
    fn deserialize_no_inline_qos_no_serialized_payload() {
        #[rustfmt::skip]
        let submessage = DataFragSubmessageRead::new(&[
            0x16_u8, 0b_0000_0001, 32, 0, // Submessage header
            0, 0, 28, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            2, 0, 0, 0, // fragmentStartingNum
            3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0, // sampleSize
        ]);

        let expected_inline_qos_flag = false;
        let expected_non_standard_payload_flag = false;
        let expected_key_flag = false;
        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_writer_sn = SequenceNumber::new(5);
        let expected_fragment_starting_num = FragmentNumber::new(2);
        let expected_fragments_in_submessage = 3;
        let expected_data_size = 4;
        let expected_fragment_size = 5;
        let expected_inline_qos = ParameterList::empty();
        let expected_serialized_payload = SerializedPayload::new(&[]);

        assert_eq!(expected_inline_qos_flag, submessage.inline_qos_flag());
        assert_eq!(
            expected_non_standard_payload_flag,
            submessage.non_standard_payload_flag()
        );
        assert_eq!(expected_key_flag, submessage.key_flag());
        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_writer_sn, submessage.writer_sn());
        assert_eq!(
            expected_fragment_starting_num,
            submessage.fragment_starting_num()
        );
        assert_eq!(
            expected_fragments_in_submessage,
            submessage.fragments_in_submessage()
        );
        assert_eq!(expected_data_size, submessage.data_size());
        assert_eq!(expected_fragment_size, submessage.fragment_size());
        assert_eq!(expected_inline_qos, submessage.inline_qos());
        assert_eq!(expected_serialized_payload, submessage.serialized_payload());
    }

    #[test]
    fn deserialize_with_inline_qos_with_serialized_payload() {
        #[rustfmt::skip]
        let submessage = DataFragSubmessageRead::new(&[
            0x16_u8, 0b_0000_0011, 48, 0, // Submessage header
            0, 0, 28, 0, // extraFlags | octetsToInlineQos
            1, 2, 3, 4, // readerId
            6, 7, 8, 9, // writerId
            0, 0, 0, 0, // writerSN: high
            6, 0, 0, 0, // writerSN: low
            2, 0, 0, 0, // fragmentStartingNum
            3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
            8, 0, 0, 0, // sampleSize
            8, 0, 4, 0, // inlineQos: parameterId, length
            71, 72, 73, 74, // inlineQos: value[length]
            1, 0, 0, 0, // inlineQos: Sentinel
            1, 2, 3, 0, // serializedPayload
        ]);

        let expected_inline_qos_flag = true;
        let expected_non_standard_payload_flag = false;
        let expected_key_flag = false;
        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_writer_sn = SequenceNumber::new(6);
        let expected_fragment_starting_num = FragmentNumber::new(2);
        let expected_fragments_in_submessage = 3;
        let expected_data_size = 8;
        let expected_fragment_size = 5;
        let expected_inline_qos =
            ParameterList::new(vec![Parameter::new(ParameterId(8), vec![71, 72, 73, 74])]);
        let expected_serialized_payload = SerializedPayload::new(&[1, 2, 3, 0]);

        assert_eq!(expected_inline_qos_flag, submessage.inline_qos_flag());
        assert_eq!(
            expected_non_standard_payload_flag,
            submessage.non_standard_payload_flag()
        );
        assert_eq!(expected_key_flag, submessage.key_flag());
        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_writer_sn, submessage.writer_sn());
        assert_eq!(
            expected_fragment_starting_num,
            submessage.fragment_starting_num()
        );
        assert_eq!(
            expected_fragments_in_submessage,
            submessage.fragments_in_submessage()
        );
        assert_eq!(expected_data_size, submessage.data_size());
        assert_eq!(expected_fragment_size, submessage.fragment_size());
        assert_eq!(expected_inline_qos, submessage.inline_qos());
        assert_eq!(expected_serialized_payload, submessage.serialized_payload());
    }
}
