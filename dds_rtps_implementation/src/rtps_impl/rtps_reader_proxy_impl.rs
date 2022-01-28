use rust_rtps_pim::{
    behavior::writer::reader_proxy::{
        RtpsReaderProxyAttributes, RtpsReaderProxyConstructor, RtpsReaderProxyOperations,
    },
    structure::types::{EntityId, Guid, Locator, SequenceNumber},
};

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxyImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    last_sent_sequence_number: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
    highest_acknowledge_change_sequence_number: SequenceNumber,
}

impl RtpsReaderProxyConstructor for RtpsReaderProxyImpl {
    fn new(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            expects_inline_qos,
            last_sent_sequence_number: 0,
            requested_changes: Vec::new(),
            highest_acknowledge_change_sequence_number: 0,
        }
    }
}

impl RtpsReaderProxyAttributes for RtpsReaderProxyImpl {
    fn remote_reader_guid(&self) -> &Guid {
        &self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> &EntityId {
        &self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_slice()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_slice()
    }

    fn expects_inline_qos(&self) -> &bool {
        &self.expects_inline_qos
    }

    fn is_active(&self) -> &bool {
        todo!()
    }
}

impl RtpsReaderProxyOperations for RtpsReaderProxyImpl {
    type ChangeForReaderType = SequenceNumber;
    type ChangeForReaderListType = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_acknowledge_change_sequence_number = committed_seq_num;
    }

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        if let Some(requested_change) = self.requested_changes.iter().min().cloned() {
            self.requested_changes.retain(|x| x != &requested_change);
            Some(requested_change.clone())
        } else {
            None
        }
    }

    fn next_unsent_change(&mut self) -> Option<Self::ChangeForReaderType> {
        todo!()
        // if &self.last_sent_sequence_number < last_change_sequence_number {
        //     self.last_sent_sequence_number = self.last_sent_sequence_number + 1;
        //     Some(self.last_sent_sequence_number.clone())
        // } else {
        //     None
        // }
    }

    fn unsent_changes(&self) -> Self::ChangeForReaderListType {
        todo!()
        // let mut unsent_changes = Vec::new();
        // for unsent_change_seq_num in
        //     self.last_sent_sequence_number + 1..=*last_change_sequence_number
        // {
        //     unsent_changes.push(unsent_change_seq_num)
        // }
        // unsent_changes
    }

    fn requested_changes(&self) -> Self::ChangeForReaderListType {
        self.requested_changes.clone()
    }

    fn requested_changes_set(&mut self, _req_seq_num_set: &[SequenceNumber]) {
        todo!()
        // let mut requested_changes: Self::SequenceNumberVector =
        //     req_seq_num_set.iter().cloned().collect();
        // requested_changes.retain(|x| x <= last_change_sequence_number);
        // self.requested_changes = requested_changes;
    }

    fn unacked_changes(&self) -> Self::ChangeForReaderListType {
        todo!()
        // let mut unacked_changes = Vec::new();
        // for unacked_changes_seq_num in
        //     self.highest_acknowledge_change_sequence_number + 1..=*last_change_sequence_number
        // {
        //     unacked_changes.push(unacked_changes_seq_num)
        // }
        // unacked_changes
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::{ENTITYID_UNKNOWN, GUID_UNKNOWN};

    use super::*;

    #[test]
    fn next_unsent_change() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        assert_eq!(reader_proxy_impl.next_unsent_change(), Some(1));
        assert_eq!(reader_proxy_impl.next_unsent_change(), Some(2));
        assert_eq!(reader_proxy_impl.next_unsent_change(), None);
    }

    #[test]
    fn next_unsent_change_non_compliant_last_change_sequence_number() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );

        assert_eq!(reader_proxy_impl.next_unsent_change(), Some(1));
        assert_eq!(reader_proxy_impl.next_unsent_change(), Some(2));
        assert_eq!(reader_proxy_impl.next_unsent_change(), None);
        assert_eq!(reader_proxy_impl.next_unsent_change(), None);
        assert_eq!(reader_proxy_impl.next_unsent_change(), None);
        assert_eq!(reader_proxy_impl.next_unsent_change(), Some(3));
    }

    #[test]
    fn requested_changes_set() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;

        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );

        let req_seq_num_set = vec![1, 2, 3];
        reader_proxy_impl.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_proxy_impl.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn requested_changes_set_above_last_change_sequence_number() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;

        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );

        let req_seq_num_set = vec![1, 2, 3];
        reader_proxy_impl.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1];
        assert_eq!(
            reader_proxy_impl.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn unsent_changes() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;

        let reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );

        let unsent_changes = reader_proxy_impl.unsent_changes();
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unsent_changes_after_next_unsent_change() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );

        let last_change_sequence_number = 3;
        reader_proxy_impl.next_unsent_change();
        let unsent_changes = reader_proxy_impl.unsent_changes();

        let expected_unsent_changes = vec![2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unacked_changes() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let expects_inline_qos = false;

        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );

        reader_proxy_impl.acked_changes_set(2);
        let last_change_sequence_number = 4;

        let expected_unacked_changes = vec![3, 4];

        assert_eq!(
            reader_proxy_impl.unacked_changes(),
            expected_unacked_changes
        );
    }
}
