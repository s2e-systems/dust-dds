use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::{SequenceNumberSet, SubmessageElement},
            types::SubmessageKind,
        },
        types::{EntityId, SequenceNumber, TryFromBytes},
    },
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageRead {
    reader_id: EntityId,
    writer_id: EntityId,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,
}

impl GapSubmessageRead {
    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        data: &[u8],
    ) -> DdsResult<Self> {
        if data.len() >= 28 {
            let endianness = submessage_header.endianness();
            Ok(Self {
                reader_id: EntityId::try_from_bytes(&data[0..], endianness)?,
                writer_id: EntityId::try_from_bytes(&data[4..], endianness)?,
                gap_start: SequenceNumber::try_from_bytes(&data[8..], endianness)?,
                gap_list: SequenceNumberSet::try_from_bytes(&mut &data[16..], endianness)?,
            })
        } else {
            Err(DdsError::Error("Gap submessage invalid".to_string()))
        }
    }

    pub fn _reader_id(&self) -> EntityId {
        self.reader_id
    }

    pub fn writer_id(&self) -> EntityId {
        self.writer_id
    }

    pub fn gap_start(&self) -> SequenceNumber {
        self.gap_start
    }

    pub fn gap_list(&self) -> &SequenceNumberSet {
        &self.gap_list
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageWrite<'a> {
    submessage_elements: [SubmessageElement<'a>; 4],
}

impl GapSubmessageWrite<'_> {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
    ) -> Self {
        Self {
            submessage_elements: [
                SubmessageElement::EntityId(reader_id),
                SubmessageElement::EntityId(writer_id),
                SubmessageElement::SequenceNumber(gap_start),
                SubmessageElement::SequenceNumberSet(gap_list),
            ],
        }
    }
}

impl<'a> Submessage<'a> for GapSubmessageWrite<'a> {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::GAP, &[], octets_to_next_header)
    }

    fn submessage_elements(&'a self) -> Self::SubmessageList {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::{
        messages::overall_structure::{
            into_bytes_vec, RtpsSubmessageWriteKind, SubmessageHeaderRead,
        },
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_gap() {
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let gap_start = SequenceNumber::from(5);
        let gap_list = SequenceNumberSet::new(SequenceNumber::from(10), []);
        let submessage = RtpsSubmessageWriteKind::Gap(GapSubmessageWrite::new(
            reader_id, writer_id, gap_start, gap_list,
        ));
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // gapStart: SequenceNumber: high
                5, 0, 0, 0, // gapStart: SequenceNumber: low
                0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
               10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
                0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            ]
        );
    }

    use super::*;
    #[test]
    fn deserialize_gap() {
        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_gap_start = SequenceNumber::from(5);
        let expected_gap_list = SequenceNumberSet::new(SequenceNumber::from(10), []);
        #[rustfmt::skip]
        let mut data = &[
            0x08, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            GapSubmessageRead::try_from_bytes(&&submessage_header, data.as_ref()).unwrap();
        assert_eq!(expected_reader_id, submessage._reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_gap_start, submessage.gap_start());
        assert_eq!(&expected_gap_list, submessage.gap_list());
    }
}
