use super::submessage_elements;
use super::SubmessageFlag;
use super::{Submessage, SubmessageHeader};

use crate::messages::types::constants;

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
    fn submessage_header(&self) -> SubmessageHeader {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
                                      // X|X|X|X|X|X|X|E
        let flags = [e, x, x, x, x, x, x, x];

        SubmessageHeader::new(constants::SUBMESSAGE_KIND_GAP, flags, 0)
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

impl serde::Serialize for Gap {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
