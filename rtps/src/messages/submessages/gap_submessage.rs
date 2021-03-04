use super::submessage_elements;
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag, SubmessageKind};

#[derive(PartialEq, Debug)]
pub struct Gap {
    pub endianness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub gap_start: submessage_elements::SequenceNumber,
    pub gap_list: submessage_elements::SequenceNumberSet,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl Submessage for Gap {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::Gap;

        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
                                      // X|X|X|X|X|X|X|E
        let flags = [e, x, x, x, x, x, x, x];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        todo!()
        // if self.gap_start <= 0 || !self.gap_list.is_valid() {
        //     false
        // } else {
        //     true
        // }
    }
}
