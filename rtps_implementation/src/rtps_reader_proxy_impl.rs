use rtps_pim::{
    behavior::{
        types::ChangeForReaderStatusKind::{
            self, Acknowledged, Requested, Unacknowledged, Underway, Unsent,
        },
        writer::{
            change_for_reader::RtpsChangeForReaderAttributes,
            reader_proxy::{
                RtpsReaderProxyAttributes, RtpsReaderProxyConstructor, RtpsReaderProxyOperations,
            },
        },
    },
    messages::{submessage_elements::{Parameter, EntityIdSubmessageElement, SequenceNumberSubmessageElement, ParameterListSubmessageElement, SerializedDataSubmessageElement}, submessages::{DataSubmessage, GapSubmessage}, types::Count},
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::RtpsHistoryCacheAttributes,
        types::{EntityId, Guid, Locator, SequenceNumber, ChangeKind, ENTITYID_UNKNOWN},
    },
};

use crate::rtps_history_cache_impl::RtpsCacheChangeImpl;

use super::rtps_history_cache_impl::RtpsHistoryCacheImpl;

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxyImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Vec<RtpsChangeForReaderImpl>,
    expects_inline_qos: bool,
    is_active: bool,
    pub last_received_acknack_count: Count,
}

impl RtpsReaderProxyImpl {
    pub fn changes_for_reader_mut(&mut self) -> &mut Vec<RtpsChangeForReaderImpl> {
        &mut self.changes_for_reader
    }
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
            last_received_acknack_count: Count(0),
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

pub struct RtpsReaderProxyOperationsImpl<'a> {
    pub sequence_number: SequenceNumber,
    pub reader_proxy: &'a mut RtpsReaderProxyImpl,
    pub writer_cache: &'a RtpsHistoryCacheImpl,
}

impl<'a> RtpsReaderProxyOperationsImpl<'a> {
    pub fn new(
        reader_proxy: &'a mut RtpsReaderProxyImpl,
        writer_cache: &'a RtpsHistoryCacheImpl,
    ) -> Self {
        Self {
            sequence_number: 0,
            reader_proxy,
            writer_cache,
        }
    }
}

impl From<RtpsChangeForReaderCacheChange<'_>> for SequenceNumber {
    fn from(v: RtpsChangeForReaderCacheChange<'_>) -> Self {
        v.change_for_reader.sequence_number
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RtpsChangeForReaderImpl {
    pub status: ChangeForReaderStatusKind,
    pub is_relevant: bool,
    pub sequence_number: SequenceNumber,
}

pub struct RtpsChangeForReaderCacheChange<'a> {
    change_for_reader: RtpsChangeForReaderImpl,
    cache_change: &'a RtpsCacheChangeImpl,
}

impl<'a> RtpsChangeForReaderAttributes for RtpsChangeForReaderCacheChange<'a> {
    fn status(&self) -> ChangeForReaderStatusKind {
        self.change_for_reader.status
    }

    fn is_relevant(&self) -> bool {
        self.change_for_reader.is_relevant
    }
}

impl<'a> RtpsCacheChangeAttributes for RtpsChangeForReaderCacheChange<'a> {
    type DataType = <RtpsCacheChangeImpl as RtpsCacheChangeAttributes>::DataType;
    type ParameterListType = <RtpsCacheChangeImpl as RtpsCacheChangeAttributes>::ParameterListType;

    fn kind(&self) -> rtps_pim::structure::types::ChangeKind {
        self.cache_change.kind()
    }

    fn writer_guid(&self) -> Guid {
        self.cache_change.writer_guid()
    }

    fn instance_handle(&self) -> rtps_pim::structure::types::InstanceHandle {
        self.cache_change.instance_handle()
    }

    fn sequence_number(&self) -> SequenceNumber {
        self.cache_change.sequence_number()
    }

    fn data_value(&self) -> &Self::DataType {
        self.cache_change.data_value()
    }

    fn inline_qos(&self) -> &Self::ParameterListType {
        self.cache_change.inline_qos()
    }
}

impl<'a> RtpsChangeForReaderCacheChange<'a> {
    pub fn new(
        change_for_reader: RtpsChangeForReaderImpl,
        writer_cache: &'a RtpsHistoryCacheImpl,
    ) -> Option<Self> {
        let cache_change = writer_cache
            .changes()
            .iter()
            .find(|cc| cc.sequence_number == change_for_reader.sequence_number)?;
        Some(RtpsChangeForReaderCacheChange {
            change_for_reader,
            cache_change,
        })
    }
}
impl<'a> Into<GapSubmessage<Vec<SequenceNumber>>> for RtpsChangeForReaderCacheChange<'a> {
    fn into(self) -> GapSubmessage<Vec<SequenceNumber>> {
        todo!()
    }
}
impl<'a> Into<DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>> for RtpsChangeForReaderCacheChange<'a> {
    fn into(self) -> DataSubmessage<Vec<Parameter<'a>>, &'a [u8]> {
        let endianness_flag = true;
        let inline_qos_flag = true;
        let (data_flag, key_flag) = match self.cache_change.kind() {
            ChangeKind::Alive => (true, false),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
            _ => todo!(),
        };
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement { value: ENTITYID_UNKNOWN };
        let writer_id = EntityIdSubmessageElement {
            value: self.cache_change.writer_guid().entity_id(),
        };
        let writer_sn = SequenceNumberSubmessageElement {
            value: self.cache_change.sequence_number(),
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: self.cache_change.inline_qos().into(),
        };
        let serialized_payload = SerializedDataSubmessageElement {
            value: self.cache_change.data_value().into(),
        };
        DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }
}

impl<'a> RtpsReaderProxyAttributes for RtpsReaderProxyOperationsImpl<'a> {
    type ChangeForReaderType = <RtpsReaderProxyImpl as RtpsReaderProxyAttributes>::ChangeForReaderType;

    fn remote_reader_guid(&self) -> Guid {
        self.reader_proxy.remote_reader_guid()
    }

    fn remote_group_entity_id(&self) -> EntityId {
        self.reader_proxy.remote_group_entity_id()
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy.unicast_locator_list()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy.multicast_locator_list()
    }

    fn changes_for_reader(&self) -> &[Self::ChangeForReaderType] {
        self.reader_proxy.changes_for_reader()
    }

    fn expects_inline_qos(&self) -> bool {
        self.reader_proxy.expects_inline_qos()
    }

    fn is_active(&self) -> bool {
        self.reader_proxy.is_active()
    }
}

impl<'a> RtpsReaderProxyOperations for RtpsReaderProxyOperationsImpl<'a> {
    type ChangeForReaderType = RtpsChangeForReaderCacheChange<'a>;
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

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //  return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.requested_changes().iter().min().cloned()?;

        let change = self
            .reader_proxy
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)?;

        // Following 8.4.9.2.12 Transition T12 of Reliable Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_requested_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_requested_change() as such
        // the modification of the status is done always.
        change.status = Underway;

        RtpsChangeForReaderCacheChange::new(change.clone(), self.writer_cache)
    }

    fn next_unsent_change(&mut self) -> Option<Self::ChangeForReaderType> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.unsent_changes().iter().min().cloned()?;

        let change = self
            .reader_proxy
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)?;

        // Following 8.4.9.1.4 Transition T14 of BestEffort Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_unsent_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_unsent_change() as such
        // the modification of the status is done always.
        change.status = Underway;

        RtpsChangeForReaderCacheChange::new(change.clone(), self.writer_cache)
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

    fn add_new_change_push_mode_true(
        writer_cache: &mut RtpsHistoryCacheImpl,
        reader_proxy: &mut RtpsReaderProxyImpl,
        sequence_number: SequenceNumber,
    ) {
        writer_cache.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            sequence_number,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        ));
        reader_proxy
            .changes_for_reader
            .push(RtpsChangeForReaderImpl {
                status: Unsent,
                is_relevant: true,
                sequence_number,
            });
    }

    fn add_new_change_push_mode_false(
        writer_cache: &mut RtpsHistoryCacheImpl,
        reader_proxy: &mut RtpsReaderProxyImpl,
        sequence_number: SequenceNumber,
    ) {
        writer_cache.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            sequence_number,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        ));
        reader_proxy
            .changes_for_reader
            .push(RtpsChangeForReaderImpl {
                status: Unacknowledged,
                is_relevant: true,
                sequence_number,
            })
    }

    #[test]
    fn next_requested_change() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);

        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 1);
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 2);
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 4);
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 6);

        let mut reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        reader_proxy.requested_changes_set(&[2, 4]);

        let result = reader_proxy.next_requested_change();
        assert_eq!(result.unwrap().change_for_reader.sequence_number, 2);

        let result = reader_proxy.next_requested_change();
        assert_eq!(result.unwrap().change_for_reader.sequence_number, 4);

        assert!(reader_proxy.next_requested_change().is_none());
    }

    #[test]
    fn unsent_changes() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy_attributes, 1);
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy_attributes, 3);
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy_attributes, 4);

        let reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        assert_eq!(reader_proxy.unsent_changes(), vec![1, 3, 4]);
    }

    #[test]
    fn next_unsent_change() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy_attributes, 1);
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy_attributes, 2);

        let mut reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);

        let result = reader_proxy.next_unsent_change();
        assert_eq!(result.unwrap().change_for_reader.sequence_number, 1);

        let result = reader_proxy.next_unsent_change();
        assert_eq!(result.unwrap().change_for_reader.sequence_number, 2);

        assert!(reader_proxy.next_unsent_change().is_none());
    }

    #[test]
    fn unacked_changes() {
        let mut reader_proxy_attributes =
            RtpsReaderProxyImpl::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = RtpsHistoryCacheImpl::new();
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 1);
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 2);
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 4);
        add_new_change_push_mode_false(&mut writer_cache, &mut reader_proxy_attributes, 6);

        let mut reader_proxy =
            RtpsReaderProxyOperationsImpl::new(&mut reader_proxy_attributes, &writer_cache);
        reader_proxy.acked_changes_set(2);

        assert_eq!(reader_proxy.unacked_changes(), vec![4, 6]);
    }
}
