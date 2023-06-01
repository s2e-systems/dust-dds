use crate::implementation::rtps::{
    messages::{
        overall_structure::{
            RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
        },
        submessage_elements::{SequenceNumberSet, SubmessageElement},
        types::{SubmessageFlag, SubmessageKind},
    },
    types::{Count, EntityId},
};

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for AckNackSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> AckNackSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn final_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn reader_sn_state(&self) -> SequenceNumberSet {
        self.map(&self.data[12..])
    }

    pub fn count(&self) -> Count {
        self.map(&self.data[self.data.len() - 4..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessageWrite<'a> {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 4],
}

impl AckNackSubmessageWrite<'_> {
    pub fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        reader_sn_state: SequenceNumberSet,
        count: Count,
    ) -> Self {
        Self {
            endianness_flag,
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

impl Submessage for AckNackSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::ACKNACK,
            &[self.endianness_flag, self.final_flag],
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
        messages::overall_structure::into_bytes_vec,
        types::{EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_acknack() {
        let endianness_flag = true;
        let final_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let submessage = AckNackSubmessageWrite::new(
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            SequenceNumberSet {
                base: SequenceNumber::new(10),
                set: vec![],
            },
            Count::new(14),
        );
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
        let submessage = AckNackSubmessageRead::new(&[
                0x06_u8, 0b_0000_0001, 24, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // reader_sn_state.base
               10, 0, 0, 0, // reader_sn_state.base
                0, 0, 0, 0, // reader_sn_state.set: numBits (ULong)
                2, 0, 0, 0, // count
        ]);

        let expected_final_flag = false;
        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_reader_sn_state = SequenceNumberSet {
            base: SequenceNumber::new(10),
            set: vec![],
        };
        let expected_count = Count::new(2);

        assert_eq!(expected_final_flag, submessage.final_flag());
        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_reader_sn_state, submessage.reader_sn_state());
        assert_eq!(expected_count, submessage.count());
    }
}
