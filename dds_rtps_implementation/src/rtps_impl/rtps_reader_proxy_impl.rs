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
    remote_reader_guid: PSM::GUID,
    remote_group_entity_id: PSM::EntityId,
    unicast_locator_list: Vec<PSM::Locator>,
    multicast_locator_list: Vec<PSM::Locator>,
    expects_inline_qos: bool,
    is_active: bool,
    last_sent_sequence_number: PSM::SequenceNumber,
}

impl<PSM: RTPSReaderProxyImplTrait> RTPSReaderProxyImpl<PSM> {}

impl<PSM: RTPSReaderProxyImplTrait> rust_rtps_pim::behavior::stateful_writer::RTPSReaderProxy<PSM>
    for RTPSReaderProxyImpl<PSM>
{
    type SequenceNumberVector = Vec<PSM::SequenceNumber>;

    fn remote_reader_guid(&self) -> &PSM::GUID {
        &self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> &PSM::EntityId {
        &self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[PSM::Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[PSM::Locator] {
        &self.multicast_locator_list
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn acked_changes_set(&mut self, _committed_seq_num: PSM::SequenceNumber) {
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

    fn requested_changes_set(&mut self, _req_seq_num_set: Self::SequenceNumberVector) {
        todo!()
    }

    fn unacked_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }
}
