use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{Count, FragmentNumber, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};

pub struct HeartbeatFrag {
    endianness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId<EntityId>,
    writer_id: submessage_elements::EntityId<EntityId>,
    writer_sn: submessage_elements::SequenceNumber<SequenceNumber>,
    last_fragment_num: submessage_elements::FragmentNumber<FragmentNumber>,
    count: submessage_elements::Count<Count>,
}

impl rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag
    for HeartbeatFrag
{
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type FragmentNumber = FragmentNumber;
    type Count = Count;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        last_fragment_num: submessage_elements::FragmentNumber<Self::FragmentNumber>,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
            count,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.reader_id
    }

    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.writer_id
    }

    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
        &self.writer_sn
    }

    fn last_fragment_num(&self) -> &submessage_elements::FragmentNumber<Self::FragmentNumber> {
        &self.last_fragment_num
    }

    fn count(&self) -> &submessage_elements::Count<Self::Count> {
        &self.count
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for HeartbeatFrag {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
