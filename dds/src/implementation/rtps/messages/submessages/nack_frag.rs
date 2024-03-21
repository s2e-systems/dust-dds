use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderWrite},
            submessage_elements::{FragmentNumberSet, SubmessageElement},
            types::{Count, SubmessageKind},
        },
        types::{Endianness, EntityId, FromBytesE, SequenceNumber, TryFromBytes},
    },
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessageRead {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_number_state: FragmentNumberSet,
    count: Count,
}

impl NackFragSubmessageRead {
    pub fn try_from_bytes(data: &[u8]) -> DdsResult<Self> {
        if data.len() >= 32 {
            let flags = data[1];
            let endianness = &Endianness::from_flags(flags);
            let mut buf = &data[20..];
            Ok(Self {
                reader_id: EntityId::try_from_bytes(&data[4..], endianness)?,
                writer_id: EntityId::try_from_bytes(&data[8..], endianness)?,
                writer_sn: SequenceNumber::try_from_bytes(&data[12..], endianness)?,
                fragment_number_state: FragmentNumberSet::try_from_bytes(&mut buf, endianness)?,
                count: Count::from_bytes_e(buf, endianness),
            })
        } else {
            Err(DdsError::Error("NackFrag submessage invalid".to_string()))
        }
    }

    pub fn reader_id(&self) -> EntityId {
        self.reader_id
    }

    pub fn _writer_id(&self) -> EntityId {
        self.writer_id
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.writer_sn
    }

    pub fn _fragment_number_state(&self) -> &FragmentNumberSet {
        &self.fragment_number_state
    }

    pub fn count(&self) -> Count {
        self.count
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessageWrite<'a> {
    submessage_elements: [SubmessageElement<'a>; 5],
}

impl NackFragSubmessageWrite<'_> {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        fragment_number_state: FragmentNumberSet,
        count: Count,
    ) -> Self {
        Self {
            submessage_elements: [
                SubmessageElement::EntityId(reader_id),
                SubmessageElement::EntityId(writer_id),
                SubmessageElement::SequenceNumber(writer_sn),
                SubmessageElement::FragmentNumberSet(fragment_number_state),
                SubmessageElement::Count(count),
            ],
        }
    }
}

impl<'a> Submessage<'a> for NackFragSubmessageWrite<'a> {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::NACK_FRAG, &[], octets_to_next_header)
    }

    fn submessage_elements(&'a self) -> Self::SubmessageList {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::{into_bytes_vec, RtpsSubmessageWriteKind},
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_nack_frag() {
        let submessage = RtpsSubmessageWriteKind::NackFrag(NackFragSubmessageWrite::new(
            EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
            SequenceNumber::from(4),
            FragmentNumberSet::new(10, []),
            6,
        ));
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x12_u8, 0b_0000_0001, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN
                4, 0, 0, 0, // writerSN
               10, 0, 0, 0, // fragmentNumberState.base
                0, 0, 0, 0, // fragmentNumberState.numBits
                6, 0, 0, 0, // count
            ]
        );
    }

    #[test]
    fn deserialize_nack_frag() {
        #[rustfmt::skip]
        let submessage = NackFragSubmessageRead::try_from_bytes(&[
            0x12_u8, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN
            4, 0, 0, 0, // writerSN
           10, 0, 0, 0, // fragmentNumberState.base
            0, 0, 0, 0, // fragmentNumberState.numBits
            6, 0, 0, 0, // count
        ]).unwrap();

        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_writer_sn = SequenceNumber::from(4);
        let expected_fragment_number_state = FragmentNumberSet::new(10, []);
        let expected_count = 6;

        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage._writer_id());
        assert_eq!(expected_writer_sn, submessage.writer_sn());
        assert_eq!(
            &expected_fragment_number_state,
            submessage._fragment_number_state()
        );
        assert_eq!(expected_count, submessage.count());
    }
}
