use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};

pub struct Gap {
    endianness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId<EntityId>,
    writer_id: submessage_elements::EntityId<EntityId>,
    gap_start: submessage_elements::SequenceNumber<SequenceNumber>,
    gap_list: submessage_elements::SequenceNumberSet<SequenceNumber, Vec<SequenceNumber>>,
}

impl rust_rtps_pim::messages::submessages::gap_submessage::Gap for Gap {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type SequenceNumberList = Vec<SequenceNumber>;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        gap_start: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        gap_list: submessage_elements::SequenceNumberSet<
            Self::SequenceNumber,
            Self::SequenceNumberList,
        >,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
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

    fn gap_start(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
        &self.gap_start
    }

    fn gap_list(
        &self,
    ) -> &submessage_elements::SequenceNumberSet<Self::SequenceNumber, Self::SequenceNumberList>
    {
        &self.gap_list
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for Gap {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
