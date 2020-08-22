use super::submessage_elements;
use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use crate::messages;
use crate::types;

#[derive(PartialEq, Debug)]
pub struct Heartbeat {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    first_sn: submessage_elements::SequenceNumber,
    last_sn: submessage_elements::SequenceNumber,
    count: submessage_elements::Count,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

impl Heartbeat {
    const FINAL_FLAG_MASK: u8 = 0x02;
    const LIVELINESS_FLAG_MASK: u8 = 0x04;

    pub fn new(
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        first_sn: types::SequenceNumber,
        last_sn: types::SequenceNumber,
        count: messages::types::Count,
        final_flag: bool,
        manual_liveliness: bool) -> Self {
            Heartbeat {
                reader_id: submessage_elements::EntityId(reader_id),
                writer_id: submessage_elements::EntityId(writer_id),
                first_sn: submessage_elements::SequenceNumber(first_sn),
                last_sn: submessage_elements::SequenceNumber(last_sn),
                count: submessage_elements::Count(count),
                final_flag,
                liveliness_flag: manual_liveliness,
                endianness_flag: false,
            }
        }

    pub fn is_valid(&self) -> bool{
        if self.first_sn.0 < 1 {
            return false;
        };

        if self.last_sn.0 < 0 {
            return false;
        }

        if self.last_sn.0 < self.first_sn.0 - 1 {
            return false;
        }

        true
    }

    pub fn reader_id(&self) -> types::EntityId {
        self.reader_id.0
    }

    pub fn writer_id(&self) -> types::EntityId {
        self.writer_id.0
    }

    pub fn first_sn(&self) -> types::SequenceNumber {
        self.first_sn.0
    }

    pub fn last_sn(&self) -> types::SequenceNumber {
        self.last_sn.0
    }

    pub fn count(&self) -> messages::types::Count {
        self.count.0
    }

    pub fn is_final(&self) -> bool {
        self.final_flag
    }
}

impl Submessage for Heartbeat {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let f = self.final_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let l = self.liveliness_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // X|X|X|X|X|L|F|E
        [e, f, l, x, x, x, x, x]
    }

    fn is_valid(&self) -> bool {
        if self.first_sn.0 <= 0 ||
           self.last_sn.0 < 0 ||
           self.last_sn.0 < self.first_sn.0 - 1 {
            false
        } else {
            true
        }
    }
}