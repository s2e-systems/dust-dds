use super::submessage_elements;
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag, SubmessageKind};

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
}

impl Submessage for Heartbeat {
    fn submessage_header(&self) -> SubmessageHeader {
        let submessage_id = SubmessageKind::Heartbeat;

        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let f = self.final_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let l = self.liveliness_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
                                      // X|X|X|X|X|L|F|E
        let flags = [e, f, l, x, x, x, x, x];

        SubmessageHeader::new(submessage_id, flags, 0)
    }

    fn is_valid(&self) -> bool {
        if self.first_sn <= 0 || self.last_sn < 0 || self.last_sn < self.first_sn - 1 {
            false
        } else {
            true
        }
    }
}

impl serde::Serialize for Heartbeat {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
