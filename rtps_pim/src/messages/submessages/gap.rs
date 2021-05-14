use crate::{messages::submessage_elements, PIM};

pub struct Gap<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub gap_start: submessage_elements::SequenceNumber<PSM>,
    pub gap_list: submessage_elements::SequenceNumberSet<PSM>,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl<PSM: PIM> Gap<PSM> {
    pub fn new(
        endianness_flag: PSM::SubmessageFlag,
        reader_id: PSM::EntityId,
        writer_id: PSM::EntityId,
        gap_start: PSM::SequenceNumber,
        gap_list: PSM::SequenceNumberVector,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id: submessage_elements::EntityId { value: reader_id },
            writer_id: submessage_elements::EntityId { value: writer_id },
            gap_start: submessage_elements::SequenceNumber { value: gap_start },
            gap_list: submessage_elements::SequenceNumberSet {
                base: gap_start,
                set: gap_list,
            },
        }
    }
}
