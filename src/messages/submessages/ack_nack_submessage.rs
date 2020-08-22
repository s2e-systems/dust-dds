use std::collections::BTreeSet;

use super::{SubmessageKind, SubmessageFlag, UdpPsmMapping, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::types;
use crate::messages;
use crate::messages::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };

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
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        available_changes_max: types::SequenceNumber,
        missing_changes: BTreeSet<types::SequenceNumber>,
        count: messages::types::Count,
        final_flag: bool,
        endianness_flag: Endianness) -> Self {
            AckNack {
                reader_id: submessage_elements::EntityId(reader_id),
                writer_id: submessage_elements::EntityId(writer_id),
                reader_sn_state: submessage_elements::SequenceNumberSet::new(available_changes_max, missing_changes),
                count: submessage_elements::Count(count),
                final_flag,
                endianness_flag: endianness_flag.into(),
            }
        }

        pub fn reader_id(&self) -> types::EntityId {
            self.reader_id.0
        }

        pub fn writer_id(&self) -> types::EntityId {
            self.writer_id.0
        }

        pub fn reader_sn_state(&self) -> &submessage_elements::SequenceNumberSet {
            &self.reader_sn_state
        }

        pub fn count(&self) -> messages::types::Count {
            self.count.0
        }
}

impl Submessage for AckNack {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        const X : SubmessageFlag = false;
        let e = self.endianness_flag; 
        let f = self.final_flag; 
        [e, f, X, X, X, X, X, X]
    }

    fn is_valid(&self) -> bool {
        self.reader_sn_state.is_valid()
    }
}