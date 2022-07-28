use rtps_pim::{
    behavior::types::{
        ChangeForReaderStatusKind::{self, Unacknowledged, Unsent},
        Duration,
    },
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, Parameter,
            SequenceNumberSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::types::{
        ChangeKind, EntityId, Guid, GuidPrefix, InstanceHandle, Locator, ReliabilityKind,
        SequenceNumber, TopicKind, ENTITYID_UNKNOWN,
    },
};

use crate::implementation::rtps::utils::clock::{Timer, TimerConstructor};

use super::{
    endpoint::RtpsEndpointImpl,
    history_cache::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl, RtpsParameter},
    writer::RtpsWriterImpl,
};

pub enum BestEffortStatefulWriterSendSubmessage<'a> {
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
}

pub enum ReliableStatefulWriterSendSubmessage<'a> {
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
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

impl RtpsReaderProxyImpl {
    pub fn new(
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

impl RtpsReaderProxyImpl {
    pub fn remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    pub fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_slice()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_slice()
    }

    pub fn changes_for_reader(&self) -> &[RtpsChangeForReaderImpl] {
        self.changes_for_reader.as_slice()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn is_active(&self) -> bool {
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

    pub fn best_effort_send_unsent_changes(
        &mut self,
    ) -> Option<BestEffortStatefulWriterSendSubmessage<'a>> {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.1.4 Transition T4
        let reader_id = self.remote_reader_guid().entity_id();

        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<Vec<Parameter<'a>>, &'a [u8]> =
                    change.into();
                data_submessage.reader_id.value = reader_id;
                Some(BestEffortStatefulWriterSendSubmessage::Data(
                    data_submessage,
                ))
            } else {
                let mut gap_submessage: GapSubmessage<Vec<SequenceNumber>> = change.into();
                gap_submessage.reader_id.value = reader_id;
                Some(BestEffortStatefulWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }

    pub fn reliable_send_unsent_changes(
        &mut self,
    ) -> Option<ReliableStatefulWriterSendSubmessage<'a>> {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
        let reader_id = self.remote_reader_guid().entity_id();

        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<Vec<Parameter<'_>>, &[u8]> = change.into();
                data_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Data(data_submessage))
            } else {
                let mut gap_submessage: GapSubmessage<Vec<SequenceNumber>> = change.into();
                gap_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }

    pub fn send_heartbeat(&self, writer_id: EntityId) -> HeartbeatSubmessage {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement { value: writer_id };
        let first_sn = SequenceNumberSubmessageElement {
            value: self.get_seq_num_min().unwrap_or(1),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: self.get_seq_num_max().unwrap_or(0),
        };
        let count = CountSubmessageElement { value: Count(0) };
        HeartbeatSubmessage {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }

    pub fn reliable_receive_acknack(&mut self, acknack: &AckNackSubmessage<Vec<SequenceNumber>>) {
        self.acked_changes_set(acknack.reader_sn_state.base - 1);
        self.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }

    pub fn send_requested_changes(&mut self) -> Option<ReliableStatefulWriterSendSubmessage<'a>> {
        let reader_id = self.remote_reader_guid().entity_id();

        if self.requested_changes().into_iter().next().is_some() {
            let change_for_reader = self.next_requested_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            if change_for_reader.is_relevant() {
                let mut data_submessage: DataSubmessage<Vec<Parameter<'_>>, &[u8]> =
                    change_for_reader.into();
                data_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Data(data_submessage))
            } else {
                let mut gap_submessage: GapSubmessage<Vec<SequenceNumber>> =
                    change_for_reader.into();
                gap_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }
}

impl<'a> RtpsReaderProxyOperationsImpl<'a> {
    pub fn add_change(&mut self, _change: RtpsCacheChangeImpl) {
        unimplemented!()
    }

    pub fn remove_change<F>(&mut self, _f: F)
    where
        F: FnMut(&RtpsCacheChangeImpl) -> bool,
    {
        unimplemented!()
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.writer_cache.get_seq_num_min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
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

impl<'a> RtpsChangeForReaderCacheChange<'a> {
    pub fn status(&self) -> ChangeForReaderStatusKind {
        self.change_for_reader.status
    }

    pub fn is_relevant(&self) -> bool {
        self.change_for_reader.is_relevant
    }
}

impl<'a> RtpsChangeForReaderCacheChange<'a> {
    pub fn kind(&self) -> rtps_pim::structure::types::ChangeKind {
        self.cache_change.kind()
    }

    pub fn writer_guid(&self) -> Guid {
        self.cache_change.writer_guid()
    }

    pub fn instance_handle(&self) -> rtps_pim::structure::types::InstanceHandle {
        self.cache_change.instance_handle()
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.cache_change.sequence_number()
    }

    pub fn data_value(&self) -> &[u8] {
        self.cache_change.data_value()
    }

    pub fn inline_qos(&self) -> &[RtpsParameter] {
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

impl<'a> RtpsReaderProxyOperationsImpl<'a> {
    pub fn remote_reader_guid(&self) -> Guid {
        self.reader_proxy.remote_reader_guid()
    }

    pub fn remote_group_entity_id(&self) -> EntityId {
        self.reader_proxy.remote_group_entity_id()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy.multicast_locator_list()
    }

    pub fn changes_for_reader(&self) -> &[RtpsChangeForReaderImpl] {
        self.reader_proxy.changes_for_reader()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.reader_proxy.expects_inline_qos()
    }

    pub fn is_active(&self) -> bool {
        self.reader_proxy.is_active()
    }
}

impl<'a> RtpsReaderProxyOperationsImpl<'a> {
    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // "FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;"
        for change in &mut self.reader_proxy.changes_for_reader {
            if change.sequence_number <= committed_seq_num {
                change.status = ChangeForReaderStatusKind::Acknowledged;
            }
        }
    }

    pub fn next_requested_change(&mut self) -> RtpsChangeForReaderCacheChange<'a> {
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

    pub fn next_unsent_change(&mut self) -> RtpsChangeForReaderCacheChange<'a> {
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

    pub fn unsent_changes(&self) -> Vec<SequenceNumber> {
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

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
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

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
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

    pub fn unacked_changes(&self) -> Vec<SequenceNumber> {
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

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn guid(&self) -> Guid {
        self.writer.guid()
    }
}

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn topic_kind(&self) -> TopicKind {
        self.writer.topic_kind()
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.writer.reliability_level()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.writer.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.writer.multicast_locator_list()
    }
}

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn push_mode(&self) -> bool {
        self.writer.push_mode()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.writer.heartbeat_period()
    }

    pub fn nack_response_delay(&self) -> Duration {
        self.writer.nack_response_delay()
    }

    pub fn nack_suppression_duration(&self) -> Duration {
        self.writer.nack_suppression_duration()
    }

    pub fn last_change_sequence_number(&self) -> SequenceNumber {
        self.writer.last_change_sequence_number()
    }

    pub fn data_max_size_serialized(&self) -> Option<i32> {
        self.writer.data_max_size_serialized()
    }

    pub fn writer_cache(&mut self) -> &mut RtpsHistoryCacheImpl {
        self.writer.writer_cache()
    }
}

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn matched_readers(&mut self) -> Vec<RtpsReaderProxyOperationsImpl<'_>> {
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

impl<T: TimerConstructor> RtpsStatefulWriterImpl<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn matched_reader_add(&mut self, mut a_reader_proxy: RtpsReaderProxyImpl) {
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

    pub fn matched_reader_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderProxyImpl) -> bool,
    {
        self.matched_readers.retain(|x| !f(x));
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: Guid) -> Option<&RtpsReaderProxyImpl> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
    ) -> RtpsCacheChangeImpl {
        self.writer.new_change(kind, data, inline_qos, handle)
    }
}

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn add_change(&mut self, change: RtpsCacheChangeImpl) {
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

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsCacheChangeImpl) -> bool,
    {
        self.writer.writer_cache().remove_change(f)
    }

    pub fn get_seq_num_min(&self) -> Option<rtps_pim::structure::types::SequenceNumber> {
        self.writer.writer_cache.get_seq_num_min()
    }

    pub fn get_seq_num_max(&self) -> Option<rtps_pim::structure::types::SequenceNumber> {
        self.writer.writer_cache.get_seq_num_max()
    }
}

impl<'a, T> RtpsStatefulWriterImpl<T>
where
    T: Timer,
{
    pub fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(
            &RtpsReaderProxyOperationsImpl<'a>,
            DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>,
        ),
        mut send_gap: impl FnMut(&RtpsReaderProxyOperationsImpl<'a>, GapSubmessage<Vec<SequenceNumber>>),
        mut send_heartbeat: impl FnMut(&RtpsReaderProxyOperationsImpl<'a>, HeartbeatSubmessage),
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
                    while let Some(send_submessage) = reader_proxy.best_effort_send_unsent_changes()
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
                        let mut heartbeat = reader_proxy.send_heartbeat(writer_id);
                        heartbeat.count.value = heartbeat_count;
                        send_heartbeat(reader_proxy, heartbeat)
                    }

                    while let Some(send_submessage) = reader_proxy.reliable_send_unsent_changes() {
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
                        reader_proxy.send_requested_changes()
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

impl<T> RtpsStatefulWriterImpl<T> {
    pub fn on_acknack_submessage_received(
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
                    RtpsReaderProxyOperationsImpl::new(reader_proxy, &self.writer.writer_cache)
                        .reliable_receive_acknack(acknack_submessage);

                    reader_proxy.last_received_acknack_count = acknack_submessage.count.value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::structure::types::{ChangeKind, ENTITYID_UNKNOWN, GUID_UNKNOWN};

    use crate::implementation::rtps::history_cache::RtpsCacheChangeImpl;

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
