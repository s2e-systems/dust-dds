use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::reader_proxy::{RtpsReaderProxy, RtpsReaderProxyOperations},
    structure::types::{Locator, SequenceNumber},
};

pub struct RtpsReaderProxyImpl {
    pub reader_proxy: RtpsReaderProxy<Vec<Locator>>,
    last_sent_sequence_number: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
    highest_acknowledge_change_sequence_number: SequenceNumber,
}

impl RtpsReaderProxyImpl {
    pub fn new(reader_proxy: RtpsReaderProxy<Vec<Locator>>) -> Self {
        Self {
            reader_proxy,
            last_sent_sequence_number: 0,
            requested_changes: Vec::new(),
            highest_acknowledge_change_sequence_number: 0,
        }
    }
}

impl Deref for RtpsReaderProxyImpl {
    type Target = RtpsReaderProxy<Vec<Locator>>;

    fn deref(&self) -> &Self::Target {
        &self.reader_proxy
    }
}

impl DerefMut for RtpsReaderProxyImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader_proxy
    }
}

impl RtpsReaderProxyOperations for RtpsReaderProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_acknowledge_change_sequence_number = committed_seq_num;
    }

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        if let Some(requested_change) = self.requested_changes.iter().min().cloned() {
            self.requested_changes.retain(|x| x != &requested_change);
            Some(requested_change.clone())
        } else {
            None
        }
    }

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Option<SequenceNumber> {
        if &self.last_sent_sequence_number < last_change_sequence_number {
            self.last_sent_sequence_number = self.last_sent_sequence_number + 1;
            Some(self.last_sent_sequence_number.clone())
        } else {
            None
        }
    }

    fn unsent_changes(
        &self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Self::SequenceNumberVector {
        let mut unsent_changes = Vec::new();
        for unsent_change_seq_num in
            self.last_sent_sequence_number + 1..=*last_change_sequence_number
        {
            unsent_changes.push(unsent_change_seq_num)
        }
        unsent_changes
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        self.requested_changes.clone()
    }

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        last_change_sequence_number: &SequenceNumber,
    ) {
        let mut requested_changes: Self::SequenceNumberVector =
            req_seq_num_set.iter().cloned().collect();
        requested_changes.retain(|x| x <= last_change_sequence_number);
        self.requested_changes = requested_changes;
    }

    fn unacked_changes(
        &self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Self::SequenceNumberVector {
        let mut unacked_changes = Vec::new();
        for unacked_changes_seq_num in
            self.highest_acknowledge_change_sequence_number + 1..=*last_change_sequence_number
        {
            unacked_changes.push(unacked_changes_seq_num)
        }
        unacked_changes
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
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);
        assert_eq!(reader_proxy_impl.next_unsent_change(&2), Some(1));
        assert_eq!(reader_proxy_impl.next_unsent_change(&2), Some(2));
        assert_eq!(reader_proxy_impl.next_unsent_change(&2), None);
    }

    #[test]
    fn next_unsent_change_non_compliant_last_change_sequence_number() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);

        assert_eq!(reader_proxy_impl.next_unsent_change(&2), Some(1));
        assert_eq!(reader_proxy_impl.next_unsent_change(&2), Some(2));
        assert_eq!(reader_proxy_impl.next_unsent_change(&2), None);
        assert_eq!(reader_proxy_impl.next_unsent_change(&0), None);
        assert_eq!(reader_proxy_impl.next_unsent_change(&-10), None);
        assert_eq!(reader_proxy_impl.next_unsent_change(&3), Some(3));
    }

    #[test]
    fn requested_changes_set() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);

        let req_seq_num_set = vec![1, 2, 3];
        reader_proxy_impl.requested_changes_set(&req_seq_num_set, &3);

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
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);

        let req_seq_num_set = vec![1, 2, 3];
        reader_proxy_impl.requested_changes_set(&req_seq_num_set, &1);

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
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);

        let unsent_changes = reader_proxy_impl.unsent_changes(&3);
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unsent_changes_after_next_unsent_change() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);

        let last_change_sequence_number = 3;
        reader_proxy_impl.next_unsent_change(&last_change_sequence_number);
        let unsent_changes = reader_proxy_impl.unsent_changes(&last_change_sequence_number);

        let expected_unsent_changes = vec![2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unacked_changes() {
        let remote_reader_guid = GUID_UNKNOWN;
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let reader_proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        );
        let mut reader_proxy_impl = RtpsReaderProxyImpl::new(reader_proxy);

        reader_proxy_impl.acked_changes_set(2);
        let last_change_sequence_number = 4;

        let expected_unacked_changes = vec![3, 4];

        assert_eq!(
            reader_proxy_impl.unacked_changes(&last_change_sequence_number),
            expected_unacked_changes
        );
    }
}
