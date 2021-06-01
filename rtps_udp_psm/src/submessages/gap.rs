use rust_rtps_pim::messages::types::SubmessageKindType;

use crate::{EntityId, RtpsUdpPsm, SequenceNumber, SequenceNumberSet, SubmessageFlag};

use super::SubmessageHeader;

pub struct GapSubmessage {
    header: SubmessageHeader,
    reader_id: EntityId,
    writer_id: EntityId,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,
}

impl GapSubmessage {
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
    ) -> Self {
        let flags = [endianness_flag].into();

        let submessage_length = 4 + gap_list.len();

        let header = SubmessageHeader {
            submessage_id: <RtpsUdpPsm as SubmessageKindType>::GAP.into(),
            flags,
            submessage_length,
        };
        Self {
            header,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        }
    }
}

impl rust_rtps_pim::messages::submessages::Gap<RtpsUdpPsm> for GapSubmessage {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type SequenceNumberSet = SequenceNumberSet;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(0)
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn gap_start(&self) -> &Self::SequenceNumber {
        &self.gap_start
    }

    fn gap_list(&self) -> &Self::SequenceNumberSet {
        &self.gap_list
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for GapSubmessage {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
