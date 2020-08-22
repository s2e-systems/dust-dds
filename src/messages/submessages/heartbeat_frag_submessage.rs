use crate::messages::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, UdpPsmMapping, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;

#[derive(PartialEq, Debug)]
pub struct HeartbeatFrag {
    endianness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    writer_sn: submessage_elements::SequenceNumber,
    last_fragment_num: submessage_elements::FragmentNumber,
    count: submessage_elements::Count,
}

impl Submessage for HeartbeatFrag {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        [e, X, X, X, X, X, X, X]
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn.0 <= 0 ||
        self.last_fragment_num.0 <= 0 {
            false
        } else {
            true
        }
    }
}