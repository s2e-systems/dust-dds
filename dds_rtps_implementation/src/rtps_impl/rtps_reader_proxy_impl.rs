use rust_rtps_pim::{
    behavior::writer::reader_proxy::{
        RtpsReaderProxyAttributes, RtpsReaderProxyConstructor, RtpsReaderProxyOperations,
    },
    structure::types::{EntityId, Guid, Locator, SequenceNumber},
};

use super::rtps_writer_history_cache_impl::WriterHistoryCache;

pub struct RtpsReaderProxyOperationsImpl<'a> {
    pub reader_proxy_attributes: &'a mut RtpsReaderProxyAttributesImpl,
    writer_cache: &'a WriterHistoryCache,
}

impl<'a> RtpsReaderProxyOperationsImpl<'a> {
    pub fn new(
        reader_proxy_attributes: &'a mut RtpsReaderProxyAttributesImpl,
        writer_cache: &'a WriterHistoryCache,
    ) -> Self {
        Self {
            reader_proxy_attributes,
            writer_cache,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxyAttributesImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    last_sent_sequence_number: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
    highest_acknowledge_change_sequence_number: SequenceNumber,
}

impl RtpsReaderProxyConstructor for RtpsReaderProxyAttributesImpl {
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

impl RtpsReaderProxyAttributes for RtpsReaderProxyAttributesImpl {
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

impl RtpsReaderProxyOperations for RtpsReaderProxyOperationsImpl<'_> {
    type ChangeForReaderType = SequenceNumber;
    type ChangeForReaderListType = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.reader_proxy_attributes
            .highest_acknowledge_change_sequence_number = committed_seq_num;
    }

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        if let Some(requested_change) = self
            .reader_proxy_attributes
            .requested_changes
            .iter()
            .min()
            .cloned()
        {
            self.reader_proxy_attributes
                .requested_changes
                .retain(|x| x != &requested_change);
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
        self.reader_proxy_attributes.requested_changes.clone()
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
    use rust_rtps_pim::structure::{
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ENTITYID_UNKNOWN, GUID_UNKNOWN, ChangeKind}, cache_change::RtpsCacheChangeConstructor,
    };

    use crate::rtps_impl::rtps_writer_history_cache_impl::WriterCacheChange;

    use super::*;

    #[test]
    fn next_unsent_change() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyAttributesImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false);
        let mut writer_cache = WriterHistoryCache::new();
        writer_cache.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &[],
            &[],
        ));
        writer_cache.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &[],
            &[],
        ));

        let mut reader_proxy_operations_impl =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);
        assert_eq!(reader_proxy_operations_impl.next_unsent_change(), Some(1));
        assert_eq!(reader_proxy_operations_impl.next_unsent_change(), Some(2));
        assert_eq!(reader_proxy_operations_impl.next_unsent_change(), None);
    }

    #[test]
    fn requested_changes_set() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyAttributesImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false);
        let writer_cache = WriterHistoryCache::new();

        let mut reader_proxy_operations_impl =
        RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        let req_seq_num_set = vec![1, 2, 3];
        reader_proxy_operations_impl.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_proxy_operations_impl.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn unsent_changes() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyAttributesImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false);
        let writer_cache = WriterHistoryCache::new();

        let reader_proxy_operations_impl =
        RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        let unsent_changes = reader_proxy_operations_impl.unsent_changes();
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unsent_changes_after_next_unsent_change() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyAttributesImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false);
        let writer_cache = WriterHistoryCache::new();

        let mut reader_proxy_operations_impl =
        RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        reader_proxy_operations_impl.next_unsent_change();
        let unsent_changes = reader_proxy_operations_impl.unsent_changes();

        let expected_unsent_changes = vec![2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unacked_changes() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyAttributesImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false);
        let writer_cache = WriterHistoryCache::new();

        let mut reader_proxy_operations_impl =
        RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        reader_proxy_operations_impl.acked_changes_set(2);

        let expected_unacked_changes = vec![3, 4];

        assert_eq!(
            reader_proxy_operations_impl.unacked_changes(),
            expected_unacked_changes
        );
    }
}
