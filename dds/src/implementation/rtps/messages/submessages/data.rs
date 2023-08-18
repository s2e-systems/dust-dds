use byteorder::ReadBytesExt;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::{
            overall_structure::{
                RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
            },
            submessage_elements::{Data, ParameterList, SubmessageElement},
            types::{SubmessageFlag, SubmessageKind},
        },
        types::{EntityId, SequenceNumber},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for DataSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> DataSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn octets_to_inline_qos(&self) -> usize {
        (&self.data[6..])
            .read_u16::<byteorder::LittleEndian>()
            .unwrap() as usize
    }

    fn inline_qos_len(&self) -> usize {
        let mut parameter_list_buf = &self.data[8 + self.octets_to_inline_qos()..];
        let parameter_list_buf_length = parameter_list_buf.len();

        if self.inline_qos_flag() {
            loop {
                let pid = parameter_list_buf
                    .read_i16::<byteorder::LittleEndian>()
                    .expect("pid read failed");
                let length = parameter_list_buf
                    .read_i16::<byteorder::LittleEndian>()
                    .expect("length read failed");
                if pid == PID_SENTINEL {
                    break;
                } else {
                    (_, parameter_list_buf) = parameter_list_buf.split_at(length as usize);
                }
            }
            parameter_list_buf_length - parameter_list_buf.len()
        } else {
            0
        }
    }

    pub fn inline_qos_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn data_flag(&self) -> bool {
        self.submessage_header().flags()[2]
    }

    pub fn key_flag(&self) -> bool {
        self.submessage_header().flags()[3]
    }

    pub fn non_standard_payload_flag(&self) -> bool {
        self.submessage_header().flags()[4]
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

    pub fn inline_qos(&self) -> ParameterList {
        if self.inline_qos_flag() {
            self.map(&self.data[self.octets_to_inline_qos() + 8..])
        } else {
            ParameterList::empty()
        }
    }

    pub fn serialized_payload(&self) -> Data {
        self.map(&self.data[8 + self.octets_to_inline_qos() + self.inline_qos_len()..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageWrite<'a> {
    inline_qos_flag: SubmessageFlag,
    data_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    submessage_elements: Vec<SubmessageElement<'a>>,
}

impl<'a> DataSubmessageWrite<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: &'a ParameterList,
        serialized_payload: &'a Data,
    ) -> Self {
        const EXTRA_FLAGS: u16 = 0;
        const OCTETS_TO_INLINE_QOS: u16 = 16;
        let mut submessage_elements = vec![
            SubmessageElement::UShort(EXTRA_FLAGS),
            SubmessageElement::UShort(OCTETS_TO_INLINE_QOS),
            SubmessageElement::EntityId(reader_id),
            SubmessageElement::EntityId(writer_id),
            SubmessageElement::SequenceNumber(writer_sn),
        ];
        if inline_qos_flag {
            submessage_elements.push(SubmessageElement::ParameterList(inline_qos));
        }
        if data_flag || key_flag {
            submessage_elements.push(SubmessageElement::SerializedData(serialized_payload));
        }
        Self {
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            submessage_elements,
        }
    }
}

impl Submessage for DataSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::DATA,
            &[
                self.inline_qos_flag,
                self.data_flag,
                self.key_flag,
                self.non_standard_payload_flag,
            ],
            octets_to_next_header,
        )
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{overall_structure::into_bytes_vec, submessage_elements::Parameter},
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::from(5);
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![]);
        let submessage = DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x15_u8, 0b_0000_0001, 20, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
            ]
        );
    }

    #[test]
    fn serialize_with_inline_qos_no_serialized_payload() {
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::from(5);
        let parameter_1 = Parameter::new(6, vec![10, 11, 12, 13]);
        let parameter_2 = Parameter::new(7, vec![20, 21, 22, 23]);
        let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = &Data::new(vec![]);

        let submessage = DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x15, 0b_0000_0011, 40, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                6, 0, 4, 0, // inlineQos: parameterId_1, length_1
                10, 11, 12, 13, // inlineQos: value_1[length_1]
                7, 0, 4, 0, // inlineQos: parameterId_2, length_2
                20, 21, 22, 23, // inlineQos: value_2[length_2]
                1, 0, 0, 0, // inlineQos: Sentinel
            ]
        );
    }

    #[test]
    fn serialize_no_inline_qos_with_serialized_payload() {
        let inline_qos_flag = false;
        let data_flag = true;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::from(5);
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![1, 2, 3, 4]);
        let submessage = DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x15, 0b_0000_0101, 24, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                1, 2, 3, 4, // serialized payload
            ]
        );
    }

    #[test]
    fn serialize_no_inline_qos_with_serialized_payload_non_multiple_of_4() {
        let inline_qos_flag = false;
        let data_flag = true;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::from(5);
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![1, 2, 3]);
        let submessage = DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x15, 0b_0000_0101, 24, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                1, 2, 3, 0, // serialized payload
            ]
        );
    }

    #[test]
    fn deserialize_no_inline_qos_no_serialized_payload() {
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::from(5);
        let inline_qos = ParameterList::empty();
        let serialized_payload = Data::new(vec![]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]);

        assert_eq!(inline_qos_flag, data_submessage.inline_qos_flag());
        assert_eq!(data_flag, data_submessage.data_flag());
        assert_eq!(key_flag, data_submessage.key_flag());
        assert_eq!(
            non_standard_payload_flag,
            data_submessage.non_standard_payload_flag()
        );
        assert_eq!(reader_id, data_submessage.reader_id());
        assert_eq!(writer_id, data_submessage.writer_id());
        assert_eq!(writer_sn, data_submessage.writer_sn());
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_no_inline_qos_with_serialized_payload() {
        let inline_qos = ParameterList::empty();
        let serialized_payload = Data::new(vec![1, 2, 3, 4]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0101, 24, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            1, 2, 3, 4, // SerializedPayload
        ]);
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_with_inline_qos_no_serialized_payload() {
        let inline_qos = ParameterList::new(vec![
            Parameter::new(6, vec![10, 11, 12, 13]),
            Parameter::new(7, vec![20, 21, 22, 23]),
        ]);
        let serialized_payload = Data::new(vec![]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
        ]);
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_with_inline_qos_with_serialized_payload() {
        let inline_qos = ParameterList::new(vec![
            Parameter::new(6, vec![10, 11, 12, 13]),
            Parameter::new(7, vec![20, 21, 22, 23]),
        ]);
        let serialized_payload = Data::new(vec![1, 2, 3, 4]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0111, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
            1, 2, 3, 4, // SerializedPayload
        ]);
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }
}
