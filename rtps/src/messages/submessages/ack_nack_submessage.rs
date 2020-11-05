use std::collections::BTreeSet;

use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::types;
use crate::messages;
use crate::messages::types::Endianness;

#[derive(PartialEq, Debug)]
pub struct AckNack {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    reader_sn_state: submessage_elements::SequenceNumberSet,
    count: submessage_elements::Count,
}

impl AckNack {
    pub fn new(
        endianness: Endianness,
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        available_changes_max: rust_dds_interface::types::SequenceNumber,
        missing_changes: BTreeSet<rust_dds_interface::types::SequenceNumber>,
        count: messages::types::Count,
        final_flag: bool) -> Self {
            
        AckNack {
            endianness_flag: endianness.into(),
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state: submessage_elements::SequenceNumberSet::new(available_changes_max, missing_changes),
            count,
        }
    }

    pub fn from_raw_parts(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId,
        writer_id: submessage_elements::EntityId,
        reader_sn_state: submessage_elements::SequenceNumberSet,
        count: submessage_elements::Count,
    ) -> Self {

        Self {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state,
            count,
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

    pub fn reader_sn_state(&self) -> &submessage_elements::SequenceNumberSet {
        &self.reader_sn_state
    }

    pub fn count(&self) -> &submessage_elements::Count {
        &self.count
    }
}

impl Submessage for AckNack {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::AckNack;

        const X : SubmessageFlag = false;
        let e = self.endianness_flag; 
        let f = self.final_flag; 
        let flags = [e, f, X, X, X, X, X, X];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        self.reader_sn_state.is_valid()
    }
}