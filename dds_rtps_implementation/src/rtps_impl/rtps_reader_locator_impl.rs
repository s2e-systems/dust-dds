use rust_rtps_pim::structure::types::{LocatorType, SequenceNumberType};

pub trait RTPSReaderLocatorImplTrait: LocatorType + SequenceNumberType {}

impl<T: LocatorType + SequenceNumberType> RTPSReaderLocatorImplTrait for T {}

pub struct RTPSReaderLocatorImpl<PSM: RTPSReaderLocatorImplTrait> {
    locator: PSM::Locator,
}

impl<PSM: RTPSReaderLocatorImplTrait>
    rust_rtps_pim::behavior::stateless_writer::RTPSReaderLocator<PSM>
    for RTPSReaderLocatorImpl<PSM>
{
    type SequenceNumberVector = Vec<PSM::SequenceNumber>;

    fn new(_locator: PSM::Locator, _expects_inline_qos: bool) -> Self {
        todo!()
    }

    fn locator(&self) -> &PSM::Locator {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn next_unsent_change(
        &mut self,
        _highest_sequence_number: PSM::SequenceNumber,
    ) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes_set(
        &mut self,
        _req_seq_num_set: Self::SequenceNumberVector,
        _highest_sequence_number: PSM::SequenceNumber,
    ) {
        todo!()
    }

    fn unsent_changes(
        &self,
        _highest_sequence_number: PSM::SequenceNumber,
    ) -> Self::SequenceNumberVector {
        todo!()
    }
}
