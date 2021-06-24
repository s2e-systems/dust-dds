use rust_rtps_pim::structure::types::{EntityId, LocatorPIM, SequenceNumber, GUIDPIM};

pub struct RTPSReaderProxyImpl<PSM>
where
    PSM: GUIDPIM + LocatorPIM,
{
    remote_reader_guid: PSM::GUIDType,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<PSM::LocatorType>,
    multicast_locator_list: Vec<PSM::LocatorType>,
    expects_inline_qos: bool,
    is_active: bool,
    last_sent_sequence_number: SequenceNumber,
}

impl<PSM> RTPSReaderProxyImpl<PSM>
where
    PSM: GUIDPIM + LocatorPIM,
{
    pub fn new(
        remote_reader_guid: PSM::GUIDType,
        remote_group_entity_id: EntityId,
        unicast_locator_list: Vec<PSM::LocatorType>,
        multicast_locator_list: Vec<PSM::LocatorType>,
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

impl<PSM> rust_rtps_pim::behavior::stateful_writer::RTPSReaderProxy<PSM>
    for RTPSReaderProxyImpl<PSM>
where
    PSM: GUIDPIM + LocatorPIM,
{
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn remote_reader_guid(&self) -> &PSM::GUIDType {
        &self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> &EntityId {
        &self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[PSM::LocatorType] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[PSM::LocatorType] {
        &self.multicast_locator_list
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn acked_changes_set(&mut self, _committed_seq_num: SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<SequenceNumber> {
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
