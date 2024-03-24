use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::{SequenceNumberSet, SubmessageElement},
            types::{Count, SubmessageFlag, SubmessageKind},
        },
        types::{EntityId, TryReadFromBytes},
    },
    infrastructure::error::DdsResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessageRead {
    final_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    reader_sn_state: SequenceNumberSet,
    count: Count,
}

impl AckNackSubmessageRead {
    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> DdsResult<Self> {
            let endianness = submessage_header.endianness();
            Ok(Self {
                final_flag: submessage_header.flags()[1],
                reader_id: EntityId::try_read_from_bytes(&mut data, endianness)?,
                writer_id: EntityId::try_read_from_bytes(&mut data, endianness)?,
                reader_sn_state: SequenceNumberSet::try_read_from_bytes(&mut data, endianness)?,
                count: Count::try_read_from_bytes(&mut data, endianness)?,
            })
    }

    pub fn _final_flag(&self) -> bool {
        self.final_flag
    }

    pub fn reader_id(&self) -> &EntityId {
        &self.reader_id
    }

    pub fn _writer_id(&self) -> &EntityId {
        &self.writer_id
    }

    pub fn reader_sn_state(&self) -> &SequenceNumberSet {
        &self.reader_sn_state
    }

    pub fn count(&self) -> Count {
        self.count
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessageWrite<'a> {
    final_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 4],
}

impl AckNackSubmessageWrite<'_> {
    pub fn new(
        final_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        reader_sn_state: SequenceNumberSet,
        count: Count,
    ) -> Self {
        Self {
            final_flag,
            submessage_elements: [
                SubmessageElement::EntityId(reader_id),
                SubmessageElement::EntityId(writer_id),
                SubmessageElement::SequenceNumberSet(reader_sn_state),
                SubmessageElement::Count(count),
            ],
        }
    }
}

impl<'a> Submessage<'a> for AckNackSubmessageWrite<'a> {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::ACKNACK,
            &[self.final_flag],
            octets_to_next_header,
        )
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
        types::{SequenceNumber, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_acknack() {
        let final_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let submessage = RtpsSubmessageWriteKind::AckNack(AckNackSubmessageWrite::new(
            final_flag,
            reader_id,
            writer_id,
            SequenceNumberSet::new(10, []),
            14,
        ));
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x06_u8, 0b_0000_0001, 24, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // reader_sn_state.base
               10, 0, 0, 0, // reader_sn_state.base
                0, 0, 0, 0, // reader_sn_state.set: numBits (ULong)
                14, 0, 0, 0, // count
            ]
        );
    }

    #[test]
    fn deserialize_acknack() {
        #[rustfmt::skip]
        let mut data = &[
            0x06_u8, 0b_0000_0001, 24, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // reader_sn_state.base
           10, 0, 0, 0, // reader_sn_state.base
            0, 0, 0, 0, // reader_sn_state.set: numBits (ULong)
            2, 0, 0, 0, // count
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            AckNackSubmessageRead::try_from_bytes(&submessage_header, data.as_ref()).unwrap();
        let expected_final_flag = false;
        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_reader_sn_state = SequenceNumberSet::new(10, []);
        let expected_count = 2;

        assert_eq!(expected_final_flag, submessage._final_flag());
        assert_eq!(&expected_reader_id, submessage.reader_id());
        assert_eq!(&expected_writer_id, submessage._writer_id());
        assert_eq!(&expected_reader_sn_state, submessage.reader_sn_state());
        assert_eq!(expected_count, submessage.count());
    }
}
