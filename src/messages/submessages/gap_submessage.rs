use std::collections::BTreeSet;

use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::types;
// use super::submessage_elements::SequenceNumberSet;

#[derive(PartialEq, Debug)]
pub struct Gap {
    endianness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    gap_start: submessage_elements::SequenceNumber,
    gap_list: submessage_elements::SequenceNumberSet,    
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl Gap {
    pub fn new(
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        gap_start: types::SequenceNumber,) -> Self {

            let mut gap_list_set = BTreeSet::new();
            gap_list_set.insert(gap_start);

            Gap {
                reader_id: submessage_elements::EntityId(reader_id),
                writer_id: submessage_elements::EntityId(writer_id),
                gap_start: submessage_elements::SequenceNumber(gap_start),
                gap_list: submessage_elements::SequenceNumberSet::from_set(gap_list_set),
                endianness_flag: false,
            }
    }

    pub fn reader_id(&self) -> types::EntityId {
        self.reader_id.0
    }

    pub fn writer_id(&self) -> types::EntityId {
        self.writer_id.0
    }

    pub fn gap_start(&self) -> types::SequenceNumber {
        self.gap_start.0
    }

    pub fn gap_list(&self) -> &submessage_elements::SequenceNumberSet {
        &self.gap_list
    }
}

impl Submessage for Gap {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        // X|X|X|X|X|X|X|E
        [e, x, x, x, x, x, x, x]
    }

    fn is_valid(&self) -> bool {
        if self.gap_start.0 <= 0 ||
           !self.gap_list.is_valid()
        {
            false
        } else {
            true
        }
    }
}
