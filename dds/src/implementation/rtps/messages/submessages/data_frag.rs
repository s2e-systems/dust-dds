use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::{ArcSlice, Data, ParameterList, SubmessageElement},
            types::{FragmentNumber, SubmessageFlag, SubmessageKind},
        },
        types::{EntityId, SequenceNumber, TryReadFromBytes},
    },
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataFragSubmessageRead {
    inline_qos_flag: bool,
    non_standard_payload_flag: SubmessageFlag,
    key_flag: bool,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_starting_num: FragmentNumber,
    fragments_in_submessage: u16,
    fragment_size: u16,
    data_size: u32,
    inline_qos: ParameterList,
    serialized_payload: Data,
}

impl DataFragSubmessageRead {
    pub fn try_from_arc_slice(
        submessage_header: &SubmessageHeaderRead,
        data: ArcSlice,
    ) -> DdsResult<Self> {
        let mut slice = data.as_ref();
        if data.len() >= 32 {
            let endianness = submessage_header.endianness();
            let inline_qos_flag = submessage_header.flags()[1];
            let key_flag = submessage_header.flags()[2];
            let non_standard_payload_flag = submessage_header.flags()[3];

            let _extra_flags = u16::try_read_from_bytes(&mut slice, endianness)?;
            let octets_to_inline_qos = u16::try_read_from_bytes(&mut slice, endianness)? as usize + 4;
            let reader_id = EntityId::try_read_from_bytes(&mut slice, endianness)?;
            let writer_id = EntityId::try_read_from_bytes(&mut slice, endianness)?;
            let writer_sn = SequenceNumber::try_read_from_bytes(&mut slice, endianness)?;
            let fragment_starting_num = FragmentNumber::try_read_from_bytes(&mut slice, endianness)?;
            let fragments_in_submessage = u16::try_read_from_bytes(&mut slice, endianness)?;
            let fragment_size = u16::try_read_from_bytes(&mut slice, endianness)?;
            let data_size = u32::try_read_from_bytes(&mut slice, endianness)?;

            let mut data_starting_at_inline_qos = data.sub_slice(octets_to_inline_qos..submessage_header.submessage_length() as usize)?;

            let inline_qos = if inline_qos_flag {
                ParameterList::try_read_from_arc_slice(&mut data_starting_at_inline_qos, endianness)?
            } else {
                ParameterList::empty()
            };
            let serialized_payload = Data::new(data_starting_at_inline_qos);

            Ok(Self {
                inline_qos_flag,
                non_standard_payload_flag,
                key_flag,
                reader_id,
                writer_id,
                writer_sn,
                fragment_starting_num,
                fragments_in_submessage,
                fragment_size,
                data_size,
                inline_qos,
                serialized_payload,
            })
        } else {
            Err(DdsError::Error("DataFrag submessage invalid".to_string()))
        }
    }

    pub fn inline_qos_flag(&self) -> bool {
        self.inline_qos_flag
    }

    pub fn key_flag(&self) -> bool {
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

    pub fn fragment_starting_num(&self) -> FragmentNumber {
        self.fragment_starting_num
    }

    pub fn fragments_in_submessage(&self) -> u16 {
        self.fragments_in_submessage
    }

    pub fn fragment_size(&self) -> u16 {
        self.fragment_size
    }

    pub fn data_size(&self) -> u32 {
        self.data_size
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }

    pub fn serialized_payload(&self) -> &Data {
        &self.serialized_payload
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageWrite<'a> {
    inline_qos_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 9],
    inline_qos_submessage_element: Option<SubmessageElement<'a>>,
    serialized_payload_submessage_element: SubmessageElement<'a>,
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
        let submessage_elements = [
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
        let inline_qos_submessage_element = if inline_qos_flag {
            Some(SubmessageElement::ParameterList(inline_qos))
        } else {
            None
        };
        let serialized_payload_submessage_element =
            SubmessageElement::SerializedData(serialized_payload);

        Self {
            inline_qos_flag,
            non_standard_payload_flag,
            key_flag,
            submessage_elements,
            inline_qos_submessage_element,
            serialized_payload_submessage_element,
        }
    }
}

impl<'a> Submessage<'a> for DataFragSubmessageWrite<'a> {
    type SubmessageList = std::iter::Chain<
        std::iter::Chain<
            std::slice::Iter<'a, SubmessageElement<'a>>,
            std::option::Iter<'a, SubmessageElement<'a>>,
        >,
        std::iter::Once<&'a SubmessageElement<'a>>,
    >;

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::DATA_FRAG,
            &[
                self.inline_qos_flag,
                self.key_flag,
                self.non_standard_payload_flag,
            ],
            octets_to_next_header,
        )
    }

    fn submessage_elements(&'a self) -> Self::SubmessageList {
        self.submessage_elements
            .iter()
            .chain(self.inline_qos_submessage_element.iter())
            .chain(std::iter::once(&self.serialized_payload_submessage_element))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{
            overall_structure::{into_bytes_vec, RtpsSubmessageWriteKind},
            submessage_elements::Parameter,
        },
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let inline_qos = &ParameterList::empty();
        let serialized_payload = &Data::new(vec![].into());
        let submessage = RtpsSubmessageWriteKind::DataFrag(DataFragSubmessageWrite::new(
            false,
            false,
            false,
            EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
            5,
            2,
            3,
            4,
            5,
            inline_qos,
            serialized_payload,
        ));
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
        let inline_qos = ParameterList::new(vec![Parameter::new(8, vec![71, 72, 73, 74].into())]);
        let serialized_payload = Data::new(vec![1, 2, 3].into());
        let submessage = RtpsSubmessageWriteKind::DataFrag(DataFragSubmessageWrite::new(
            true,
            false,
            false,
            EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
            6,
            2,
            3,
            8,
            5,
            &inline_qos,
            &serialized_payload,
        ));
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
        let mut data = &[
            0x16_u8, 0b_0000_0001, 32, 0, // Submessage header
            0, 0, 28, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            2, 0, 0, 0, // fragmentStartingNum
            3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0, // sampleSize
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            DataFragSubmessageRead::try_from_arc_slice(&submessage_header, ArcSlice::from(data)).unwrap();

        let expected_inline_qos_flag = false;
        let expected_non_standard_payload_flag = false;
        let expected_key_flag = false;
        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_writer_sn = 5;
        let expected_fragment_starting_num = 2;
        let expected_fragments_in_submessage = 3;
        let expected_data_size = 4;
        let expected_fragment_size = 5;
        let expected_inline_qos = ParameterList::empty();
        let expected_serialized_payload = Data::new(vec![].into());

        assert_eq!(expected_inline_qos_flag, submessage.inline_qos_flag());
        assert_eq!(
            expected_non_standard_payload_flag,
            submessage._non_standard_payload_flag()
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
        assert_eq!(&expected_inline_qos, submessage.inline_qos());
        assert_eq!(
            &expected_serialized_payload,
            submessage.serialized_payload()
        );
    }

    #[test]
    fn deserialize_with_inline_qos_with_serialized_payload() {
        #[rustfmt::skip]
        let mut data = &[
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
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            DataFragSubmessageRead::try_from_arc_slice(&submessage_header, data.into()).unwrap();

        let expected_inline_qos_flag = true;
        let expected_non_standard_payload_flag = false;
        let expected_key_flag = false;
        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_writer_sn = 6;
        let expected_fragment_starting_num = 2;
        let expected_fragments_in_submessage = 3;
        let expected_data_size = 8;
        let expected_fragment_size = 5;
        let expected_inline_qos =
            ParameterList::new(vec![Parameter::new(8, vec![71, 72, 73, 74].into())]);
        let expected_serialized_payload = Data::new(vec![1, 2, 3, 0].into());

        assert_eq!(expected_inline_qos_flag, submessage.inline_qos_flag());
        assert_eq!(
            expected_non_standard_payload_flag,
            submessage._non_standard_payload_flag()
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
        assert_eq!(&expected_inline_qos, submessage.inline_qos());
        assert_eq!(
            &expected_serialized_payload,
            submessage.serialized_payload()
        );
    }
}
