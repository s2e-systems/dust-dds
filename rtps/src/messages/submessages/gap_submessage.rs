use std::collections::BTreeSet;

use crate::messages::types::Endianness;
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
        endianness: Endianness,
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        gap_start: rust_dds_types::SequenceNumber,
        gap_list: BTreeSet<rust_dds_types::SequenceNumber>) -> Self {

            Gap {
                endianness_flag: endianness.into(),
                reader_id,
                writer_id,
                gap_start,
                gap_list: submessage_elements::SequenceNumberSet::from_set(gap_list),
            }
    }

    pub fn from_raw_parts(
        endianness_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId,
        writer_id: submessage_elements::EntityId,
        gap_start: submessage_elements::SequenceNumber,
        gap_list: submessage_elements::SequenceNumberSet) -> Self {

        Self {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        }
    }

    pub fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    pub fn reader_id(&self) -> submessage_elements::EntityId {
        self.reader_id
    }

    pub fn writer_id(&self) -> submessage_elements::EntityId {
        self.writer_id
    }

    pub fn gap_start(&self) -> submessage_elements::SequenceNumber {
        self.gap_start
    }

    pub fn gap_list(&self) -> &submessage_elements::SequenceNumberSet {
        &self.gap_list
    }
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
        if self.gap_start <= 0 ||
           !self.gap_list.is_valid()
        {
            false
        } else {
            true
        }
    }
}
