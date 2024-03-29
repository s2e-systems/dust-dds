use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::{ArcSlice, Data, ParameterList, SubmessageElement},
            types::{SubmessageFlag, SubmessageKind},
        },
        types::{EntityId, SequenceNumber, TryReadFromBytes, WriteIntoBytes},
    },
    infrastructure::error::DdsResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageRead {
    inline_qos_flag: bool,
    data_flag: bool,
    key_flag: bool,
    non_standard_payload_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    inline_qos: ParameterList,
    serialized_payload: Data,
}

impl DataSubmessageRead {
    pub fn try_from_arc_slice(
        submessage_header: &SubmessageHeaderRead,
        data: ArcSlice,
    ) -> DdsResult<Self> {
        let mut slice = data.as_ref();
        let endianness = submessage_header.endianness();
        let inline_qos_flag = submessage_header.flags()[1];
        let data_flag = submessage_header.flags()[2];
        let key_flag = submessage_header.flags()[3];
        let non_standard_payload_flag = submessage_header.flags()[4];

        let _extra_flags = u16::try_read_from_bytes(&mut slice, endianness)?;
        let octets_to_inline_qos = u16::try_read_from_bytes(&mut slice, endianness)? as usize + 4;
        let reader_id = EntityId::try_read_from_bytes(&mut slice, endianness)?;
        let writer_id = EntityId::try_read_from_bytes(&mut slice, endianness)?;
        let writer_sn = SequenceNumber::try_read_from_bytes(&mut slice, endianness)?;

        let mut data_starting_at_inline_qos =
            data.sub_slice(octets_to_inline_qos..submessage_header.submessage_length() as usize)?;
        let inline_qos = if inline_qos_flag {
            ParameterList::try_read_from_arc_slice(
                &mut data_starting_at_inline_qos,
                submessage_header.endianness(),
            )?
        } else {
            ParameterList::empty()
        };

        let serialized_payload = if data_flag || key_flag {
            Data::new(data_starting_at_inline_qos)
        } else {
            Data::empty()
        };

        Ok(Self {
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: ParameterList,
        serialized_payload: Data,
    ) -> Self {
        Self {
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }

    pub fn _inline_qos_flag(&self) -> bool {
        self.inline_qos_flag
    }

    pub fn _data_flag(&self) -> bool {
        self.data_flag
    }

    pub fn _key_flag(&self) -> bool {
        self.key_flag
    }

    pub fn _non_standard_payload_flag(&self) -> bool {
        self.non_standard_payload_flag
    }

    pub fn reader_id(&self) -> EntityId {
        self.reader_id
    }

    pub fn writer_id(&self) -> EntityId {
        self.writer_id
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.writer_sn
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }

    pub fn serialized_payload(&self) -> &Data {
        &self.serialized_payload
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageWrite<'a> {
    inline_qos_flag: SubmessageFlag,
    data_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 5],
    inline_qos_submessage_element: Option<SubmessageElement<'a>>,
    serialized_payload_submessage_element: Option<SubmessageElement<'a>>,
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
        let submessage_elements = [
            SubmessageElement::UShort(EXTRA_FLAGS),
            SubmessageElement::UShort(OCTETS_TO_INLINE_QOS),
            SubmessageElement::EntityId(reader_id),
            SubmessageElement::EntityId(writer_id),
            SubmessageElement::SequenceNumber(writer_sn),
        ];
        let inline_qos_submessage_element = if inline_qos_flag {
            Some(SubmessageElement::ParameterList(inline_qos))
        } else {
            None
        };

        let serialized_payload_submessage_element = if data_flag || key_flag {
            Some(SubmessageElement::SerializedData(serialized_payload))
        } else {
            None
        };
        Self {
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            submessage_elements,
            inline_qos_submessage_element,
            serialized_payload_submessage_element,
        }
    }
}

impl SubmessageHeader for &DataSubmessageWrite<'_> {
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
}

impl WriteIntoBytes for &DataSubmessageWrite<'_> {
    fn write_into_bytes(&self, buf: &mut &mut [u8]) {
        for submessage_element in &self.submessage_elements {
            submessage_element.write_into_bytes(buf);
        }
        if let Some(submessage_element) = &self.inline_qos_submessage_element {
            submessage_element.write_into_bytes(buf);
        }
        if let Some(submessage_element) = &self.serialized_payload_submessage_element {
            submessage_element.write_into_bytes(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{
            overall_structure::{
                write_into_bytes_vec, write_submessage_into_bytes_vec, RtpsSubmessageWriteKind,
            },
            submessage_elements::Parameter,
        },
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
        let writer_sn = 5;
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![].into());
        let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
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
        let writer_sn = 5;
        let parameter_1 = Parameter::new(6, vec![10, 11, 12, 13].into());
        let parameter_2 = Parameter::new(7, vec![20, 21, 22, 23].into());
        let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = &Data::new(vec![].into());

        let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
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
        let writer_sn = 5;
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![1, 2, 3, 4].into());
        let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
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
        let writer_sn = 5;
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![1, 2, 3].into());
        let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
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
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = 5;
        let inline_qos = ParameterList::empty();
        let serialized_payload = Data::new(vec![].into());

        #[rustfmt::skip]
        let mut data = &[
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let data_submessage =
            DataSubmessageRead::try_from_arc_slice(&submessage_header, data.into()).unwrap();

        assert_eq!(inline_qos_flag, data_submessage._inline_qos_flag());
        assert_eq!(data_flag, data_submessage._data_flag());
        assert_eq!(key_flag, data_submessage._key_flag());
        assert_eq!(reader_id, data_submessage.reader_id());
        assert_eq!(writer_id, data_submessage.writer_id());
        assert_eq!(writer_sn, data_submessage.writer_sn());
        assert_eq!(&inline_qos, data_submessage.inline_qos());
        assert_eq!(&serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_no_inline_qos_with_serialized_payload() {
        let expected_inline_qos = ParameterList::empty();
        let expected_serialized_payload = Data::new(vec![1, 2, 3, 4].into());

        #[rustfmt::skip]
        let mut data = &[
            0x15, 0b_0000_0101, 24, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            1, 2, 3, 4, // SerializedPayload
            123, 123, 123 // Following data
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let data_submessage =
            DataSubmessageRead::try_from_arc_slice(&submessage_header, data.into()).unwrap();
        assert_eq!(&expected_inline_qos, data_submessage.inline_qos());
        assert_eq!(
            &expected_serialized_payload,
            data_submessage.serialized_payload()
        );
    }

    #[test]
    fn deserialize_with_inline_qos_no_serialized_payload() {
        let inline_qos = ParameterList::new(vec![
            Parameter::new(6, vec![10, 11, 12, 13].into()),
            Parameter::new(7, vec![20, 21, 22, 23].into()),
        ]);
        let serialized_payload = Data::new(vec![].into());

        #[rustfmt::skip]
        let mut data = &[
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
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let data_submessage =
            DataSubmessageRead::try_from_arc_slice(&submessage_header, data.into()).unwrap();
        assert_eq!(&inline_qos, data_submessage.inline_qos());
        assert_eq!(&serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_with_inline_qos_with_serialized_payload() {
        let expected_inline_qos = ParameterList::new(vec![
            Parameter::new(6, vec![10, 11, 12, 13].into()),
            Parameter::new(7, vec![20, 21, 22, 23].into()),
        ]);
        let expected_serialized_payload = Data::new(vec![1, 2, 3, 4].into());

        #[rustfmt::skip]
        let mut data = &[
            0x15, 0b_0000_0111, 44, 0, // Submessage header
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
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let data_submessage =
            DataSubmessageRead::try_from_arc_slice(&submessage_header, data.into()).unwrap();

        assert_eq!(&expected_inline_qos, data_submessage.inline_qos());
        assert_eq!(
            &expected_serialized_payload,
            data_submessage.serialized_payload()
        );
    }

    #[test]
    fn deserialize_octets_to_inline_qos_non_16() {
        let expected_inline_qos = ParameterList::new(vec![
            Parameter::new(6, vec![10, 11, 12, 13].into()),
            Parameter::new(7, vec![20, 21, 22, 23].into()),
        ]);
        #[rustfmt::skip]
        let mut data = &[
            0x15, 0b_0000_0011, 44, 0, // Submessage header
            123, 123, 20, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            123, 123, 123, 123, // Unknown data
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let data_submessage =
            DataSubmessageRead::try_from_arc_slice(&submessage_header, data.into()).unwrap();

        assert_eq!(&expected_inline_qos, data_submessage.inline_qos());
    }
}
