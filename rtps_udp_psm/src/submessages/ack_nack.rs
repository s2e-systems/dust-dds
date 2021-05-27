use rust_rtps_pim::messages::Submessage;

use crate::{Count, EntityId, RtpsUdpPsm, SequenceNumberSet, SubmessageFlag};

use super::SubmessageHeader;

pub struct AckNack {}

impl rust_rtps_pim::messages::submessages::AckNack<RtpsUdpPsm> for AckNack {
    type EntityId = EntityId;
    type SequenceNumberSet = SequenceNumberSet;
    type Count = Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn writer_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn reader_sn_state(&self) -> &Self::SequenceNumberSet {
        todo!()
    }

    fn count(&self) -> &Self::Count {
        todo!()
    }
}

impl Submessage<RtpsUdpPsm> for AckNack {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
