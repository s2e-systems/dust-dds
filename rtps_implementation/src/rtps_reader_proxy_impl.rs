use rtps_pim::{
    behavior::{
        types::ChangeForReaderStatusKind::{
            Acknowledged, Requested, Unacknowledged, Underway, Unsent,
        },
        writer::reader_proxy::{
            RtpsReaderProxyAttributes, RtpsReaderProxyConstructor, RtpsReaderProxyOperations,
        },
    },
    structure::{
        history_cache::RtpsHistoryCacheAttributes,
        types::{EntityId, Guid, Locator, SequenceNumber},
    },
};

use super::{
    rtps_change_for_reader_impl::RtpsChangeForReaderImpl,
    rtps_history_cache_impl::RtpsHistoryCacheImpl,
};

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxyImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Vec<RtpsChangeForReaderImpl>,
    expects_inline_qos: bool,
    is_active: bool,
}

impl RtpsReaderProxyConstructor for RtpsReaderProxyImpl {
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
            changes_for_reader: vec![],
            expects_inline_qos,
            is_active,
        }
    }
}

impl RtpsReaderProxyAttributes for RtpsReaderProxyImpl {
    type ChangeForReaderType = RtpsChangeForReaderImpl;

    fn remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_slice()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_slice()
    }

    fn changes_for_reader(&self) -> &[Self::ChangeForReaderType] {
        self.changes_for_reader.as_slice()
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

pub struct ChangeForReader<'a> {
    pub sequence_number: SequenceNumber,
    pub reader_proxy: &'a mut RtpsReaderProxyImpl,
    pub writer_cache: &'a RtpsHistoryCacheImpl,
}

impl<'a> ChangeForReader<'a> {
    pub fn new(
        reader_proxy: &'a mut RtpsReaderProxyImpl,
        writer_cache: &'a RtpsHistoryCacheImpl,
        push_mode: bool,
    ) -> Self {
        // Todo: move the following code to where add_change is called
        // add changes from writer_cache_changes that are not in changes_for_reader
        for writer_cache_change_seq_num in writer_cache.changes().iter().map(|c| c.sequence_number)
        {
            if !reader_proxy
                .changes_for_reader
                .iter()
                .any(|c| c.sequence_number == writer_cache_change_seq_num)
            {
                let status = if push_mode { Unsent } else { Unacknowledged };
                reader_proxy
                    .changes_for_reader
                    .push(RtpsChangeForReaderImpl {
                        status,
                        is_relevant: true,
                        sequence_number: writer_cache_change_seq_num,
                    })
            }
        }

        Self {
            sequence_number: 0,
            reader_proxy,
            writer_cache,
        }
    }
}

impl From<&'_ ChangeForReader<'_>> for SequenceNumber {
    fn from(cfr: &'_ ChangeForReader<'_>) -> Self {
        cfr.sequence_number
    }
}

impl<'a> RtpsReaderProxyOperations for ChangeForReader<'a> {
    type ChangeForReaderType = ChangeForReader<'a>;
    type ChangeForReaderListType = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // "FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;"
        for change in &mut self.reader_proxy.changes_for_reader {
            if change.sequence_number <= committed_seq_num {
                change.status = Acknowledged;
            }
        }
    }

    fn next_requested_change(&mut self) -> Option<&mut Self::ChangeForReaderType> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //  return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.requested_changes().iter().min().cloned()?;

        // Following 8.4.9.2.12 Transition T12 of Reliable Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_requested_change();
        // a_change.status := UNDERWAY;
        self.reader_proxy
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap()
            .status = Underway;

        self.sequence_number = next_seq_num;
        Some(self)
    }

    fn next_unsent_change(&mut self) -> Option<&mut Self::ChangeForReaderType> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.unsent_changes().iter().min().cloned()?;

        // Following 8.4.9.1.4 Transition T14 of BestEffort Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_unsent_change();
        // a_change.status := UNDERWAY;
        self.reader_proxy
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap()
            .status = Underway;

        self.sequence_number = next_seq_num;
        Some(self)
    }

    fn unsent_changes(&self) -> Self::ChangeForReaderListType {
        // "return change IN this.changes_for_reader SUCH-THAT (change.status == UNSENT);"
        self.reader_proxy
            .changes_for_reader
            .iter()
            .filter_map(|cc| {
                if cc.status == Unsent {
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
        let requested_changes_for_reader: Vec<_> = self
            .reader_proxy
            .changes_for_reader
            .iter()
            .filter(|&change_for_reader| change_for_reader.status == Requested)
            .collect();
        requested_changes_for_reader
            .iter()
            .map(|change_for_reader| change_for_reader.sequence_number)
            .collect()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        for &seq_num in req_seq_num_set {
            for change_for_reader in &mut self
                .reader_proxy
                .changes_for_reader
                .iter_mut()
                .filter(|change_for_reader| change_for_reader.sequence_number == seq_num)
            {
                change_for_reader.status = Requested;
            }
        }
    }

    fn unacked_changes(&self) -> Self::ChangeForReaderListType {
        //"return change IN this.changes_for_reader
        //    SUCH-THAT (change.status == UNACKNOWLEDGED);"
        self.reader_proxy
            .changes_for_reader
            .iter()
            .filter_map(|cc| {
                if cc.status == Unacknowledged {
                    Some(cc.sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ChangeKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
    };

    use crate::rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsData, RtpsParameterList};

    use super::*;

    fn add_new_change(writer_cache: &mut RtpsHistoryCacheImpl, sequence_number: SequenceNumber) {
        writer_cache.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            sequence_number,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        ));
    }

    #[test]
    fn next_requested_change() {
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 2);
        add_new_change(&mut writer_cache, 4);
        add_new_change(&mut writer_cache, 6);

        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);

        let mut reader_proxy =
            ChangeForReader::new(&mut reader_proxy_attributes, &writer_cache, true);

        reader_proxy.requested_changes_set(&[2, 4]);

        let result = reader_proxy.next_requested_change().unwrap();
        assert_eq!(result.sequence_number, 2);

        let result = reader_proxy.next_requested_change().unwrap();
        assert_eq!(result.sequence_number, 4);

        assert!(reader_proxy.next_requested_change().is_none());
    }

    #[test]
    fn unsent_changes() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 3);
        add_new_change(&mut writer_cache, 4);

        let reader_proxy = ChangeForReader::new(&mut reader_proxy_attributes, &writer_cache, true);

        assert_eq!(reader_proxy.unsent_changes(), vec![1, 3, 4]);
    }

    #[test]
    fn next_unsent_change() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 2);

        let mut reader_proxy =
            ChangeForReader::new(&mut reader_proxy_attributes, &writer_cache, true);

        let result = reader_proxy.next_unsent_change().unwrap();
        assert_eq!(result.sequence_number, 1);

        let result = reader_proxy.next_unsent_change().unwrap();
        assert_eq!(result.sequence_number, 2);

        assert!(reader_proxy.next_unsent_change().is_none());
    }

    #[test]
    fn unacked_changes() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change(&mut writer_cache, 1);
        add_new_change(&mut writer_cache, 2);
        add_new_change(&mut writer_cache, 4);
        add_new_change(&mut writer_cache, 6);

        let mut reader_proxy =
            ChangeForReader::new(&mut reader_proxy_attributes, &writer_cache, false);

        reader_proxy.acked_changes_set(2);

        assert_eq!(reader_proxy.unacked_changes(), vec![4, 6]);
    }
}
