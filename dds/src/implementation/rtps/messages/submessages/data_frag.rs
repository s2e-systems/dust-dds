use std::io::BufRead;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::{
            overall_structure::{
                RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
            },
            submessage_elements::{ParameterList, SubmessageElement},
            types::{
                FragmentNumber, SubmessageFlag, SubmessageKind,
            },
        },
        types::{EntityId, SequenceNumber}, history_cache::Data,
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

    pub fn serialized_payload(&self) -> Data {
        self.map(&self.data[8 + self.octets_to_inline_qos() as usize + self.inline_qos_len()..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageWrite<'a> {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    submessage_elements: Vec<SubmessageElement<'a>>,
}

impl<'a> DataFragSubmessageWrite<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        inline_qos_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        fragment_starting_num: FragmentNumber,
        fragments_in_submessage: u16,
        data_size: u32,
        fragment_size: u16,
        inline_qos: &'a ParameterList,
        serialized_payload: &'a Data,
    ) -> Self {
        const EXTRA_FLAGS: u16 = 0;
        const OCTETS_TO_INLINE_QOS: u16 = 28;
        let mut submessage_elements = vec![
            SubmessageElement::UShort(EXTRA_FLAGS),
            SubmessageElement::UShort(OCTETS_TO_INLINE_QOS),
            SubmessageElement::EntityId(reader_id),
            SubmessageElement::EntityId(writer_id),
            SubmessageElement::SequenceNumber(writer_sn),
            SubmessageElement::FragmentNumber(fragment_starting_num),
            SubmessageElement::UShort(fragments_in_submessage),
            SubmessageElement::UShort(fragment_size),
            SubmessageElement::ULong(data_size),
        ];
        if inline_qos_flag {
            submessage_elements.push(SubmessageElement::ParameterList(inline_qos));
        }
        submessage_elements.push(SubmessageElement::SerializedData(serialized_payload));

        Self {
            endianness_flag: true,
            inline_qos_flag,
            non_standard_payload_flag,
            key_flag,
            submessage_elements,
        }
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        match self.submessage_elements[4] {
            SubmessageElement::SequenceNumber(e) => e,
            _ => todo!(),
        }
    }

    pub fn fragment_starting_num(&self) -> FragmentNumber {
        match self.submessage_elements[5] {
            SubmessageElement::FragmentNumber(e) => e,
            _ => todo!(),
        }
    }

    pub fn fragments_in_submessage(&self) -> u16 {
        match self.submessage_elements[6] {
            SubmessageElement::UShort(e) => e,
            _ => todo!(),
        }
    }
}

impl Submessage for DataFragSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::DATA_FRAG,
            &[
                self.endianness_flag,
                self.inline_qos_flag,
                self.key_flag,
                self.non_standard_payload_flag,
            ],
            octets_to_next_header,
        )
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }

    fn endianness_flag(&self) -> bool {
        self.endianness_flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{
            overall_structure::into_bytes_vec, submessage_elements::Parameter, types::ParameterId,
        },
        types::{EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![]);
        let submessage = DataFragSubmessageWrite::new(
            false,
            false,
            false,
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            SequenceNumber::new(5),
            FragmentNumber::new(2),
            3,
            4,
            5,
            inline_qos,
            serialized_payload,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x16_u8, 0b_0000_0001, 32, 0, // Submessage header
                0, 0, 28, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                2, 0, 0, 0, // fragmentStartingNum
                3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
                4, 0, 0, 0, // sampleSize
            ]
        );
    }

    #[test]
    fn serialize_with_inline_qos_with_serialized_payload() {
        let inline_qos =
            ParameterList::new(vec![Parameter::new(ParameterId(8), vec![71, 72, 73, 74])]);
        let serialized_payload = Data::new(vec![1, 2, 3]);
        let submessage = DataFragSubmessageWrite::new(
            true,
            false,
            false,
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            SequenceNumber::new(6),
            FragmentNumber::new(2),
            3,
            8,
            5,
            &inline_qos,
            &serialized_payload,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
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
            ]
        );
    }

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
        let expected_serialized_payload = Data::new(vec![]);

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
        let expected_serialized_payload = Data::new(vec![1, 2, 3, 0]);

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
