use super::submessage_elements;
use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use crate::messages;
use crate::types;
use crate::messages::types::Endianness;

#[derive(PartialEq, Debug)]
pub struct Heartbeat {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub first_sn: submessage_elements::SequenceNumber,
    pub last_sn: submessage_elements::SequenceNumber,
    pub count: submessage_elements::Count,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

impl Heartbeat {
    pub const FINAL_FLAG_MASK: u8 = 0x02;
    pub const LIVELINESS_FLAG_MASK: u8 = 0x04;

    pub fn new(
        endianness: Endianness,
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        first_sn: rust_dds_types::SequenceNumber,
        last_sn: rust_dds_types::SequenceNumber,
        count: messages::types::Count,
        final_flag: bool,
        manual_liveliness: bool) -> Self {

        Self {
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            final_flag,
            liveliness_flag: manual_liveliness,
            endianness_flag: endianness.into(),
        }
    }

    pub fn from_raw_parts(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId,
        writer_id: submessage_elements::EntityId,
        first_sn: submessage_elements::SequenceNumber,
        last_sn: submessage_elements::SequenceNumber,
        count: submessage_elements::Count,) -> Self {

        Self {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
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

    pub fn first_sn(&self) -> submessage_elements::SequenceNumber {
        self.first_sn
    }

    pub fn last_sn(&self) -> submessage_elements::SequenceNumber {
        self.last_sn
    }

    pub fn count(&self) -> submessage_elements::Count {
        self.count
    }

    pub fn is_final(&self) -> bool {
        self.final_flag
    }
}

impl Submessage for Heartbeat {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::Heartbeat;
        
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let f = self.final_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let l = self.liveliness_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // X|X|X|X|X|L|F|E
        let flags = [e, f, l, x, x, x, x, x];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        if self.first_sn <= 0 ||
           self.last_sn < 0 ||
           self.last_sn < self.first_sn - 1 {
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::ENTITYID_UNKNOWN;

    #[test]
    fn test_heartbeat_validity_function() {
        let valid_heartbeat = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: 2,
            last_sn: 5, 
            count: 0,
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat.is_valid(), true);

        let valid_heartbeat_first_message = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: 1,
            last_sn: 0, 
            count: 2,
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat_first_message.is_valid(), true);

        let invalid_heartbeat_zero_first_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: 0,
            last_sn: 1, 
            count: 2,
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_zero_first_value.is_valid(), false);

        let invalid_heartbeat_negative_last_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: 5,
            last_sn: -6, 
            count: 2,
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_negative_last_value.is_valid(), false);

        let invalid_heartbeat_wrong_first_last_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: 6,
            last_sn: 4, 
            count: 2,
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_wrong_first_last_value.is_valid(), false);
    }
}