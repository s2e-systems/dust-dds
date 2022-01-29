use rust_rtps_pim::{
    behavior::writer::reader_proxy::{
        RtpsReaderProxyAttributes, RtpsReaderProxyConstructor, RtpsReaderProxyOperations,
    },
    structure::{
        history_cache::RtpsHistoryCacheAttributes,
        types::{EntityId, Guid, Locator, SequenceNumber},
    },
};

use super::rtps_writer_history_cache_impl::WriterHistoryCache;

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxyAttributesImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,
    highest_seq_num_sent: SequenceNumber,
    lowest_requested_change: SequenceNumber,
}

impl RtpsReaderProxyConstructor for RtpsReaderProxyAttributesImpl {
    fn new(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            expects_inline_qos,
            is_active,
            highest_seq_num_sent: 0,
            lowest_requested_change: 0,
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
        &self.is_active
    }
}

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

impl RtpsReaderProxyAttributes for RtpsReaderProxyOperationsImpl<'_> {
    fn remote_reader_guid(&self) -> &Guid {
        &self.reader_proxy_attributes.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> &EntityId {
        &self.reader_proxy_attributes.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy_attributes.unicast_locator_list.as_slice()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy_attributes
            .multicast_locator_list
            .as_slice()
    }

    fn expects_inline_qos(&self) -> &bool {
        &self.reader_proxy_attributes.expects_inline_qos
    }

    fn is_active(&self) -> &bool {
        todo!()
    }
}

impl RtpsReaderProxyOperations for RtpsReaderProxyOperationsImpl<'_> {
    type ChangeForReaderType = SequenceNumber;
    type ChangeForReaderListType = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        // next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //     return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);
        if let Some(requested_change) = self.requested_changes().iter().min().cloned() {
            self.requested_changes().retain(|x| x != &requested_change);
            Some(requested_change.clone())
        } else {
            None
        }
    }

    fn next_unsent_change(&mut self) -> Option<Self::ChangeForReaderType> {
        // next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);
        // "The implementation of next_unsent_change() would also look at
        //  the HistoryCache and return the CacheChange that has the
        //  next-highest sequence number greater than highestSeqNumSent."
        if self.reader_proxy_attributes.highest_seq_num_sent > 0 {
            self.reader_proxy_attributes.highest_seq_num_sent =
                self.reader_proxy_attributes.highest_seq_num_sent + 1;
            Some(self.reader_proxy_attributes.highest_seq_num_sent)
        } else {
            None
        }
    }

    fn unsent_changes(&self) -> Self::ChangeForReaderListType {
        //return change IN this.changes_for_reader SUCH-THAT (change.status == UNSENT);
        // "the operation unsent_changes() could be implemented by looking
        //  up all changes in the HistoryCache and selecting the ones with
        //  sequenceNumber greater than highestSeqNumSent."
        self.writer_cache
            .changes()
            .iter()
            .filter_map(|cc| {
                if cc.sequence_number > self.reader_proxy_attributes.highest_seq_num_sent {
                    Some(cc.sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }

    fn requested_changes(&self) -> Self::ChangeForReaderListType {
        // return change IN this.changes_for_reader
        //      SUCH-THAT (change.status == REQUESTED);
        todo!()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END
        todo!()
        // self.reader_proxy_attributes.requested_changes = req_seq_num_set.to_vec();
    }

    fn unacked_changes(&self) -> Self::ChangeForReaderListType {
        //return change IN this.changes_for_reader
        //    SUCH-THAT (change.status == UNACKNOWLEDGED);
        todo!()
    }

    fn changes_for_reader(&self) -> Self::ChangeForReaderListType {
        // List of CacheChange changes as they relate to the matched RTPS Reader.
        self.writer_cache
            .changes()
            .iter()
            .map(|cc| cc.sequence_number)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ChangeKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
    };

    use crate::rtps_impl::rtps_writer_history_cache_impl::WriterCacheChange;

    use super::*;

    #[test]
    fn next_unsent_change() {
        let mut reader_proxy_attributes = RtpsReaderProxyAttributesImpl::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[],
            &[],
            false,
            true,
        );
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
        let mut reader_proxy_attributes = RtpsReaderProxyAttributesImpl::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[],
            &[],
            false,
            true,
        );
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
        let mut reader_proxy_attributes = RtpsReaderProxyAttributesImpl::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[],
            &[],
            false,
            true,
        );
        let writer_cache = WriterHistoryCache::new();

        let reader_proxy_operations_impl =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        let unsent_changes = reader_proxy_operations_impl.unsent_changes();
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unsent_changes_after_next_unsent_change() {
        let mut reader_proxy_attributes = RtpsReaderProxyAttributesImpl::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[],
            &[],
            false,
            true,
        );
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
        let mut reader_proxy_attributes = RtpsReaderProxyAttributesImpl::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[],
            &[],
            false,
            true,
        );
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
