use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{Count, FragmentNumber, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};

pub struct NackFrag {
    endianness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId<EntityId>,
    writer_id: submessage_elements::EntityId<EntityId>,
    writer_sn: submessage_elements::SequenceNumber<SequenceNumber>,
    fragment_number_state:
        submessage_elements::FragmentNumberSet<FragmentNumber, Vec<FragmentNumber>>,
    count: submessage_elements::Count<Count>,
}

impl rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag for NackFrag {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type FragmentNumber = FragmentNumber;
    type FragmentNumberSet = Vec<FragmentNumber>;
    type Count = Count;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        fragment_number_state: submessage_elements::FragmentNumberSet<
            Self::FragmentNumber,
            Self::FragmentNumberSet,
        >,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_number_state,
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

    fn fragment_number_state(
        &self,
    ) -> &submessage_elements::FragmentNumberSet<Self::FragmentNumber, Self::FragmentNumberSet>
    {
        &self.fragment_number_state
    }

    fn count(&self) -> &submessage_elements::Count<Self::Count> {
        &self.count
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for NackFrag {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
