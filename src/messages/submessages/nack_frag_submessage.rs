use crate::messages::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, UdpPsmMapping, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;


#[derive(PartialEq, Debug)]
pub struct NackFrag {
    endianness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    writer_sn: submessage_elements::SequenceNumber,
    fragment_number_state: submessage_elements::FragmentNumberSet,
    count: submessage_elements::Count,
}


impl Submessage for NackFrag {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag; 
        [e, X, X, X, X, X, X, X]
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn.0 <= 0 ||
        !self.fragment_number_state.is_valid() {
            false
        } else {
            true
        }
    }
}