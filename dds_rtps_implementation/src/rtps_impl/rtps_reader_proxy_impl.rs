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
    requested_changes: Vec<SequenceNumber>,
    committed_seq_num: SequenceNumber,
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
            requested_changes: vec![],
            committed_seq_num: 0,
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
        &self.reader_proxy_attributes.is_active
    }
}

impl RtpsReaderProxyOperations for RtpsReaderProxyOperationsImpl<'_> {
    type ChangeForReaderType = SequenceNumber;
    type ChangeForReaderListType = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // "FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;"
        self.reader_proxy_attributes.committed_seq_num = committed_seq_num;
    }

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //     return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self.reader_proxy_attributes.requested_changes.iter().min().cloned();

        //remove from requested_changes
        if let Some(next_seq_num) = next_seq_num {
            self.reader_proxy_attributes
                .requested_changes
                .retain(|x| x != &next_seq_num);
        };

        next_seq_num
    }

    fn next_unsent_change(&mut self) -> Option<Self::ChangeForReaderType> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        // "The implementation of next_unsent_change() would also look at
        //  the HistoryCache and return the CacheChange that has the
        //  next-highest sequence number greater than highestSeqNumSent."

        let next_change_seq_num = self
            .unsent_changes()
            .iter()
            .filter(|&c| c > &self.reader_proxy_attributes.highest_seq_num_sent)
            .next()
            .cloned();
        if let Some(next_change_seq_num) = next_change_seq_num {
            self.reader_proxy_attributes.highest_seq_num_sent = next_change_seq_num;
        }
        next_change_seq_num
    }

    fn unsent_changes(&self) -> Self::ChangeForReaderListType {
        // "return change IN this.changes_for_reader SUCH-THAT (change.status == UNSENT);"
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
        // "return change IN this.changes_for_reader
        //      SUCH-THAT (change.status == REQUESTED);"
        self.reader_proxy_attributes.requested_changes.clone()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        self.reader_proxy_attributes.requested_changes = req_seq_num_set.to_vec();
    }

    fn unacked_changes(&self) -> Self::ChangeForReaderListType {
        //"return change IN this.changes_for_reader
        //    SUCH-THAT (change.status == UNACKNOWLEDGED);"
        self.changes_for_reader()
            .iter()
            .filter(|&cc| cc > &self.reader_proxy_attributes.committed_seq_num)
            .cloned()
            .collect()
    }

    fn changes_for_reader(&self) -> Self::ChangeForReaderListType {
        // "List of CacheChange changes as they relate to the matched RTPS Reader."
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

    fn add_new_change(writer_cache: &mut WriterHistoryCache, sequence_number: SequenceNumber) {
        writer_cache.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &sequence_number,
            &[],
            &[],
        ));
    }

    #[test]
    fn next_requested_change() {
        let mut reader_proxy_attributes = RtpsReaderProxyAttributesImpl::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[],
            &[],
            false,
            true,
        );
        let mut writer_cache = WriterHistoryCache::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 2);
        add_new_change(&mut writer_cache, 4);
        add_new_change(&mut writer_cache, 6);

        let mut reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        reader_proxy.requested_changes_set(&[2, 4]);

        assert_eq!(reader_proxy.next_requested_change(), Some(2));
        assert_eq!(reader_proxy.next_requested_change(), Some(4));
        assert_eq!(reader_proxy.next_requested_change(), None);
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
        let mut writer_cache = WriterHistoryCache::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 3);
        add_new_change(&mut writer_cache, 4);

        let reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        let result = reader_proxy.unsent_changes();
        assert_eq!(result, vec![1, 3, 4]);
    }

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
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 2);

        let mut reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);
        assert_eq!(reader_proxy.next_unsent_change(), Some(1));
        assert_eq!(reader_proxy.next_unsent_change(), Some(2));
        assert_eq!(reader_proxy.next_unsent_change(), None);
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
        let mut writer_cache = WriterHistoryCache::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 2);
        add_new_change(&mut writer_cache, 4);
        add_new_change(&mut writer_cache, 6);

        let mut reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);
        reader_proxy.acked_changes_set(2);
        let result = reader_proxy.unacked_changes();
        assert_eq!(result, vec![4, 6]);
    }

}
