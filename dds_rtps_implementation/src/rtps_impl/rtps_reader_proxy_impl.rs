use rust_rtps_pim::structure::types::{
    EntityIdType, GUIDType, GuidPrefixType, LocatorType, SequenceNumberType,
};

pub trait RTPSReaderProxyImplTrait:
    SequenceNumberType + GuidPrefixType + EntityIdType + GUIDType<Self> + LocatorType + Sized
{
}

impl<
        T: SequenceNumberType + GuidPrefixType + EntityIdType + GUIDType<Self> + LocatorType + Sized,
    > RTPSReaderProxyImplTrait for T
{
}

pub struct RTPSReaderProxyImpl<PSM: RTPSReaderProxyImplTrait> {
    last_sent_sequence_number: PSM::SequenceNumber,
}

impl<PSM: RTPSReaderProxyImplTrait> rust_rtps_pim::behavior::stateful_writer::RTPSReaderProxy<PSM>
    for RTPSReaderProxyImpl<PSM>
{
    type SequenceNumberVector = Vec<PSM::SequenceNumber>;

    fn remote_reader_guid(&self) -> &PSM::GUID {
        todo!()
    }

    fn remote_group_entity_id(&self) -> &PSM::EntityId {
        todo!()
    }

    fn unicast_locator_list(&self) -> &[PSM::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[PSM::Locator] {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }

    fn is_active(&self) -> bool {
        todo!()
    }

    fn acked_changes_set(&mut self, committed_seq_num: PSM::SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn unsent_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector) {
        todo!()
    }

    fn unacked_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }
}
