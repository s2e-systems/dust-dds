use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{Count, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};
pub struct Heartbeat {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId<EntityId>,
    writer_id: submessage_elements::EntityId<EntityId>,
    first_sn: submessage_elements::SequenceNumber<SequenceNumber>,
    last_sn: submessage_elements::SequenceNumber<SequenceNumber>,
    count: submessage_elements::Count<Count>,
}

impl rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat for Heartbeat {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type Count = Count;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        first_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        last_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self {
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

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn final_flag(&self) -> SubmessageFlag {
        self.final_flag
    }

    fn liveliness_flag(&self) -> SubmessageFlag {
        self.liveliness_flag
    }

    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.reader_id
    }

    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.writer_id
    }

    fn first_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
        &self.first_sn
    }

    fn last_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
        &self.last_sn
    }

    fn count(&self) -> &submessage_elements::Count<Self::Count> {
        &self.count
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for Heartbeat {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
