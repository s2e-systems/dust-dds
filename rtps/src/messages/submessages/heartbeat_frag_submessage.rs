use super::submessage_elements;
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag, SubmessageKind};

#[derive(PartialEq, Debug)]
pub struct HeartbeatFrag {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub writer_sn: submessage_elements::SequenceNumber,
    pub last_fragment_num: submessage_elements::FragmentNumber,
    pub count: submessage_elements::Count,
}

impl Submessage for HeartbeatFrag {
    fn submessage_header(&self) -> SubmessageHeader {
        let submessage_id = SubmessageKind::HeartbeatFrag;

        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let flags = [e, X, X, X, X, X, X, X];

        SubmessageHeader::new(submessage_id, flags, 0)
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn <= 0 || self.last_fragment_num <= 0 {
            false
        } else {
            true
        }
    }
}

impl serde::Serialize for HeartbeatFrag {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
