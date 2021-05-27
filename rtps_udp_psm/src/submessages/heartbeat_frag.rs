use crate::{Count, EntityId, FragmentNumber, RtpsUdpPsm, SequenceNumber, SubmessageFlag};

use super::SubmessageHeader;

pub struct HeartbeatFrag;

impl rust_rtps_pim::messages::submessages::HeartbeatFrag<RtpsUdpPsm> for HeartbeatFrag {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type FragmentNumber = FragmentNumber;
    type Count = Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn writer_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn writer_sn(&self) -> &Self::SequenceNumber {
        todo!()
    }

    fn last_fragment_num(&self) -> &Self::FragmentNumber {
        todo!()
    }

    fn count(&self) -> &Self::Count {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for HeartbeatFrag {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
