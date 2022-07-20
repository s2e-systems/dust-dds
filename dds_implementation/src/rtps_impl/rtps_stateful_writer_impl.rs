use rtps_pim::{
    behavior::{
        stateful_writer_behavior::{
            BestEffortReaderProxyUnsentChangesBehavior, BestEffortStatefulWriterSendSubmessage,
            ReliableReaderProxyReceiveAcknackBehavior, ReliableReaderProxyRequestedChangesBehavior,
            ReliableReaderProxySendHeartbeatBehavior, ReliableReaderProxyUnsentChangesBehavior,
            ReliableStatefulWriterSendSubmessage, RtpsStatefulWriterReceiveAckNackSubmessage,
            RtpsStatefulWriterSendSubmessages,
        },
        types::{
            ChangeForReaderStatusKind::{self, Unacknowledged, Unsent},
            Duration,
        },
        writer::{
            change_for_reader::RtpsChangeForReaderAttributes,
            reader_proxy::{
                RtpsReaderProxyAttributes, RtpsReaderProxyConstructor, RtpsReaderProxyOperations,
            },
            stateful_writer::{
                RtpsStatefulWriterAttributes, RtpsStatefulWriterConstructor,
                RtpsStatefulWriterOperations,
            },
            RtpsWriterAttributes, RtpsWriterOperations,
        },
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{
            ChangeKind, EntityId, Guid, GuidPrefix, InstanceHandle, Locator, ReliabilityKind,
            SequenceNumber, TopicKind,
        },
    },
};

use crate::rtps_impl::utils::clock::{Timer, TimerConstructor};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_writer_impl::RtpsWriterImpl,
};

pub enum RtpsStatefulSubmessage<'a> {
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
    Heartbeat(HeartbeatSubmessage),
}

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxyImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Vec<RtpsChangeForReaderImpl>,
    expects_inline_qos: bool,
    is_active: bool,
    last_received_acknack_count: Count,
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
    reader_proxy: &'a mut RtpsReaderProxyImpl,
    writer_cache: &'a RtpsHistoryCacheImpl,
}

impl<'a> RtpsReaderProxyOperationsImpl<'a> {
    pub fn new(
        reader_proxy: &'a mut RtpsReaderProxyImpl,
        writer_cache: &'a RtpsHistoryCacheImpl,
    ) -> Self {
        Self {
            reader_proxy,
            writer_cache,
        }
    }
}

impl<'a> RtpsHistoryCacheOperations for RtpsReaderProxyOperationsImpl<'a> {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn add_change(&mut self, _change: Self::CacheChangeType) {
        unimplemented!()
    }

    fn remove_change<F>(&mut self, _f: F)
    where
        F: FnMut(&Self::CacheChangeType) -> bool,
    {
        unimplemented!()
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.writer_cache.get_seq_num_min()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.writer_cache.get_seq_num_max()
    }
}

impl From<RtpsChangeForReaderCacheChange<'_>> for SequenceNumber {
    fn from(v: RtpsChangeForReaderCacheChange<'_>) -> Self {
        v.change_for_reader.sequence_number
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RtpsChangeForReaderImpl {
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
    sequence_number: SequenceNumber,
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
    ) -> Self {
        let cache_change = writer_cache
            .changes()
            .iter()
            .find(|cc| cc.sequence_number() == change_for_reader.sequence_number)
            .unwrap();
        RtpsChangeForReaderCacheChange {
            change_for_reader,
            cache_change,
        }
    }
}

impl<'a> From<RtpsChangeForReaderCacheChange<'a>> for GapSubmessage<Vec<SequenceNumber>> {
    fn from(_val: RtpsChangeForReaderCacheChange<'a>) -> Self {
        todo!()
    }
}

impl<'a> From<RtpsChangeForReaderCacheChange<'a>> for DataSubmessage<Vec<Parameter<'a>>, &'a [u8]> {
    fn from(val: RtpsChangeForReaderCacheChange<'a>) -> Self {
        val.cache_change.into()
    }
}

impl<'a> RtpsReaderProxyAttributes for RtpsReaderProxyOperationsImpl<'a> {
    type ChangeForReaderType =
        <RtpsReaderProxyImpl as RtpsReaderProxyAttributes>::ChangeForReaderType;

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
                change.status = ChangeForReaderStatusKind::Acknowledged;
            }
        }
    }

    fn next_requested_change(&mut self) -> Self::ChangeForReaderType {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //  return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.requested_changes().iter().min().cloned().unwrap();

        let change = self
            .reader_proxy
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap();

        // Following 8.4.9.2.12 Transition T12 of Reliable Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_requested_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_requested_change() as such
        // the modification of the status is done always.
        change.status = ChangeForReaderStatusKind::Underway;

        RtpsChangeForReaderCacheChange::new(change.clone(), self.writer_cache)
    }

    fn next_unsent_change(&mut self) -> Self::ChangeForReaderType {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.unsent_changes().iter().min().cloned().unwrap();

        let change = self
            .reader_proxy
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap();

        // Following 8.4.9.1.4 Transition T14 of BestEffort Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_unsent_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_unsent_change() as such
        // the modification of the status is done always.
        change.status = ChangeForReaderStatusKind::Underway;

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
            .filter(|&change_for_reader| {
                change_for_reader.status == ChangeForReaderStatusKind::Requested
            })
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
                change_for_reader.status = ChangeForReaderStatusKind::Requested;
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

pub struct RtpsStatefulWriterImpl<T> {
    writer: RtpsWriterImpl,
    matched_readers: Vec<RtpsReaderProxyImpl>,
    heartbeat_timer: T,
    heartbeat_count: Count,
}

impl<T> RtpsEntityAttributes for RtpsStatefulWriterImpl<T> {
    fn guid(&self) -> Guid {
        self.writer.guid()
    }
}

impl<T> RtpsEndpointAttributes for RtpsStatefulWriterImpl<T> {
    fn topic_kind(&self) -> TopicKind {
        self.writer.topic_kind()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.writer.reliability_level()
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.writer.unicast_locator_list()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.writer.multicast_locator_list()
    }
}

impl<T> RtpsWriterAttributes for RtpsStatefulWriterImpl<T> {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn push_mode(&self) -> bool {
        self.writer.push_mode()
    }

    fn heartbeat_period(&self) -> Duration {
        self.writer.heartbeat_period()
    }

    fn nack_response_delay(&self) -> Duration {
        self.writer.nack_response_delay()
    }

    fn nack_suppression_duration(&self) -> Duration {
        self.writer.nack_suppression_duration()
    }

    fn last_change_sequence_number(&self) -> SequenceNumber {
        self.writer.last_change_sequence_number()
    }

    fn data_max_size_serialized(&self) -> Option<i32> {
        self.writer.data_max_size_serialized()
    }

    fn writer_cache(&mut self) -> &mut Self::HistoryCacheType {
        self.writer.writer_cache()
    }
}

impl<'a, T> RtpsStatefulWriterAttributes<'a> for RtpsStatefulWriterImpl<T> {
    type ReaderProxyListType = Vec<RtpsReaderProxyOperationsImpl<'a>>;

    fn matched_readers(&'a mut self) -> Self::ReaderProxyListType {
        let writer_cache = &self.writer.writer_cache;
        self.matched_readers
            .iter_mut()
            .map(|reader_proxy| RtpsReaderProxyOperationsImpl {
                reader_proxy,
                writer_cache,
            })
            .collect()
    }
}

impl<T: TimerConstructor> RtpsStatefulWriterConstructor for RtpsStatefulWriterImpl<T> {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            writer: RtpsWriterImpl::new(
                RtpsEndpointImpl::new(
                    guid,
                    topic_kind,
                    reliability_level,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                data_max_size_serialized,
            ),
            matched_readers: Vec::new(),
            heartbeat_timer: T::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl<T> RtpsStatefulWriterOperations for RtpsStatefulWriterImpl<T> {
    type ReaderProxyType = RtpsReaderProxyImpl;

    fn matched_reader_add(&mut self, mut a_reader_proxy: Self::ReaderProxyType) {
        let status = if self.push_mode() {
            ChangeForReaderStatusKind::Unsent
        } else {
            ChangeForReaderStatusKind::Unacknowledged
        };
        for change in self.writer.writer_cache().changes() {
            a_reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReaderImpl {
                    status,
                    is_relevant: true,
                    sequence_number: change.sequence_number(),
                });
        }

        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::ReaderProxyType) -> bool,
    {
        self.matched_readers.retain(|x| !f(x));
    }

    fn matched_reader_lookup(&self, a_reader_guid: Guid) -> Option<&Self::ReaderProxyType> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl<T> RtpsWriterOperations for RtpsStatefulWriterImpl<T> {
    type CacheChangeType = RtpsCacheChangeImpl;
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <Self::CacheChangeType as RtpsCacheChangeConstructor>::DataType,
        inline_qos: <Self::CacheChangeType as RtpsCacheChangeConstructor>::ParameterListType,
        handle: InstanceHandle,
    ) -> Self::CacheChangeType {
        self.writer.new_change(kind, data, inline_qos, handle)
    }
}

impl<T> RtpsHistoryCacheOperations for RtpsStatefulWriterImpl<T> {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        let sequence_number = change.sequence_number();
        self.writer.writer_cache().add_change(change);

        for reader_proxy in &mut self.matched_readers {
            let status = if self.writer.push_mode() {
                Unsent
            } else {
                Unacknowledged
            };
            reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReaderImpl {
                    status,
                    is_relevant: true,
                    sequence_number,
                })
        }
    }

    fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&Self::CacheChangeType) -> bool,
    {
        self.writer.writer_cache().remove_change(f)
    }

    fn get_seq_num_min(&self) -> Option<rtps_pim::structure::types::SequenceNumber> {
        self.writer.writer_cache.get_seq_num_min()
    }

    fn get_seq_num_max(&self) -> Option<rtps_pim::structure::types::SequenceNumber> {
        self.writer.writer_cache.get_seq_num_max()
    }
}

impl<'a, T> RtpsStatefulWriterSendSubmessages<'a, Vec<Parameter<'a>>, &'a [u8], Vec<SequenceNumber>>
    for RtpsStatefulWriterImpl<T>
where
    T: Timer,
{
    type ReaderProxyType = RtpsReaderProxyOperationsImpl<'a>;

    fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(&Self::ReaderProxyType, DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
        mut send_gap: impl FnMut(&Self::ReaderProxyType, GapSubmessage<Vec<SequenceNumber>>),
        mut send_heartbeat: impl FnMut(&Self::ReaderProxyType, HeartbeatSubmessage),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.writer.heartbeat_period().seconds as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period().fraction as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
            self.heartbeat_count = Count(self.heartbeat_count.0.wrapping_add(1));
        }

        let reliability_level = self.reliability_level();
        let writer_id = self.guid().entity_id;
        let heartbeat_count = self.heartbeat_count;

        for reader_proxy in &mut self.matched_readers() {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    while let Some(send_submessage) =
                        BestEffortReaderProxyUnsentChangesBehavior::send_unsent_changes(
                            reader_proxy,
                        )
                    {
                        match send_submessage {
                            BestEffortStatefulWriterSendSubmessage::Data(data) => {
                                send_data(reader_proxy, data)
                            }
                            BestEffortStatefulWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_proxy, gap)
                            }
                        }
                    }
                }
                ReliabilityKind::Reliable => {
                    if time_for_heartbeat {
                        let mut heartbeat =
                            ReliableReaderProxySendHeartbeatBehavior::send_heartbeat(
                                reader_proxy,
                                writer_id,
                            );
                        heartbeat.count.value = heartbeat_count;
                        send_heartbeat(reader_proxy, heartbeat)
                    }

                    while let Some(send_submessage) =
                        ReliableReaderProxyUnsentChangesBehavior::send_unsent_changes(reader_proxy)
                    {
                        match send_submessage {
                            ReliableStatefulWriterSendSubmessage::Data(data) => {
                                send_data(reader_proxy, data)
                            }
                            ReliableStatefulWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_proxy, gap)
                            }
                        }
                    }
                    while let Some(send_requested_submessage) =
                        ReliableReaderProxyRequestedChangesBehavior::send_requested_changes(
                            reader_proxy,
                        )
                    {
                        match send_requested_submessage {
                            ReliableStatefulWriterSendSubmessage::Data(data) => {
                                send_data(reader_proxy, data)
                            }
                            ReliableStatefulWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_proxy, gap)
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T> RtpsStatefulWriterReceiveAckNackSubmessage<Vec<SequenceNumber>>
    for RtpsStatefulWriterImpl<T>
{
    fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage<Vec<SequenceNumber>>,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reliability_level() == ReliabilityKind::Reliable {
            let reader_guid = Guid::new(source_guid_prefix, acknack_submessage.reader_id.value);

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                if reader_proxy.last_received_acknack_count != acknack_submessage.count.value {
                    ReliableReaderProxyReceiveAcknackBehavior::receive_acknack(
                        &mut RtpsReaderProxyOperationsImpl::new(
                            reader_proxy,
                            &self.writer.writer_cache,
                        ),
                        acknack_submessage,
                    );

                    reader_proxy.last_received_acknack_count = acknack_submessage.count.value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ChangeKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
    };

    use crate::rtps_impl::rtps_history_cache_impl::RtpsCacheChangeImpl;

    use super::*;

    fn add_new_change_push_mode_true(
        writer_cache: &mut RtpsHistoryCacheImpl,
        reader_proxy: &mut RtpsReaderProxyImpl,
        sequence_number: SequenceNumber,
    ) {
        writer_cache.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            sequence_number,
            vec![],
            vec![],
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
            [0; 16],
            sequence_number,
            vec![],
            vec![],
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
        assert_eq!(result.change_for_reader.sequence_number, 2);

        let result = reader_proxy.next_requested_change();
        assert_eq!(result.change_for_reader.sequence_number, 4);
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
        assert_eq!(result.change_for_reader.sequence_number, 1);

        let result = reader_proxy.next_unsent_change();
        assert_eq!(result.change_for_reader.sequence_number, 2);

        // let result = std::panic::catch_unwind(|| reader_proxy.next_unsent_change());
        // assert!(result.is_err());
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
