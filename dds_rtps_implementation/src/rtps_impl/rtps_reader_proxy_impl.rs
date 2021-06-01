use rust_rtps_pim::structure::types::{
    EntityIdPIM, GUIDType, GuidPrefixPIM, LocatorType, SequenceNumberType,
};

pub trait RTPSReaderProxyImplTrait:
    SequenceNumberType + GuidPrefixPIM + EntityIdPIM + GUIDType<Self> + LocatorType + Sized
{
}

impl<
        T: SequenceNumberType + GuidPrefixPIM + EntityIdPIM + GUIDType<Self> + LocatorType + Sized,
    > RTPSReaderProxyImplTrait for T
{
}

pub struct RTPSReaderProxyImpl<PSM: RTPSReaderProxyImplTrait> {
    remote_reader_guid: PSM::GUID,
    remote_group_entity_id: PSM::EntityIdType,
    unicast_locator_list: Vec<PSM::Locator>,
    multicast_locator_list: Vec<PSM::Locator>,
    expects_inline_qos: bool,
    is_active: bool,
    last_sent_sequence_number: PSM::SequenceNumber,
}

impl<PSM: RTPSReaderProxyImplTrait> RTPSReaderProxyImpl<PSM> {
    pub fn new(
        remote_reader_guid: PSM::GUID,
        remote_group_entity_id: PSM::EntityIdType,
        unicast_locator_list: Vec<PSM::Locator>,
        multicast_locator_list: Vec<PSM::Locator>,
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
            last_sent_sequence_number: 0.into(),
        }
    }
}

impl<PSM: RTPSReaderProxyImplTrait> rust_rtps_pim::behavior::stateful_writer::RTPSReaderProxy<PSM>
    for RTPSReaderProxyImpl<PSM>
{
    type SequenceNumberVector = Vec<PSM::SequenceNumber>;

    fn remote_reader_guid(&self) -> &PSM::GUID {
        &self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> &PSM::EntityIdType {
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
