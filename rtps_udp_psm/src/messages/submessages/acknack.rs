use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{Count, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};

struct AckNack {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<EntityId>,
    pub writer_id: submessage_elements::EntityId<EntityId>,
    pub reader_sn_state:
        submessage_elements::SequenceNumberSet<SequenceNumber, Vec<SequenceNumber>>,
    pub count: submessage_elements::Count<Count>,
}

impl rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack for AckNack {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type SequenceNumberList = Vec<SequenceNumber>;
    type Count = Count;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        reader_sn_state: submessage_elements::SequenceNumberSet<
            Self::SequenceNumber,
            Self::SequenceNumberList,
        >,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self {
        Self {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state,
            count,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn final_flag(&self) -> SubmessageFlag {
        self.final_flag
    }

    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.reader_id
    }

    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.writer_id
    }

    fn reader_sn_state(
        &self,
    ) -> &submessage_elements::SequenceNumberSet<Self::SequenceNumber, Self::SequenceNumberList>
    {
        &self.reader_sn_state
    }

    fn count(&self) -> &submessage_elements::Count<Self::Count> {
        &self.count
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for AckNack {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        let x = SubmessageFlag(false);
        SubmessageHeader {
            submessage_id:
                <SubmessageKind as rust_rtps_pim::messages::types::SubmessageKind>::ACKNACK,
            flags: [self.endianness_flag, self.final_flag, x, x, x, x, x, x],
            submessage_length: 0,
        }
    }
}
