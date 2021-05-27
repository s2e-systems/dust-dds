use crate::{Count, EntityId, RtpsUdpPsm, SequenceNumber, SubmessageFlag};

pub struct Heartbeat;

impl rust_rtps_pim::messages::submessages::Heartbeat<RtpsUdpPsm> for Heartbeat {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type Count = Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn liveliness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn writer_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn first_sn(&self) -> &Self::SequenceNumber {
        todo!()
    }

    fn last_sn(&self) -> &Self::SequenceNumber {
        todo!()
    }

    fn count(&self) -> &Self::Count {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for Heartbeat {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
