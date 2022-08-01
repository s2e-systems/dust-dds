use dds_transport::{
    messages::submessages::{
        AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage,
    },
    types::Locator,
};

use crate::{
    dcps_psm::{Duration, InstanceHandle},
    implementation::rtps::utils::clock::{Timer, TimerConstructor},
};

use super::{
    endpoint::RtpsEndpointImpl,
    history_cache::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl, RtpsParameter},
    reader_proxy::{
        BestEffortStatefulWriterSendSubmessage, ChangeForReaderStatusKind,
        ReliableStatefulWriterSendSubmessage, RtpsChangeForReader, RtpsReaderProxy,
    },
    types::{ChangeKind, Count, Guid, GuidPrefix, ReliabilityKind, SequenceNumber, TopicKind},
    writer::RtpsWriterImpl,
};

pub struct RtpsStatefulWriterImpl<T> {
    writer: RtpsWriterImpl,
    matched_readers: Vec<RtpsReaderProxy>,
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
    pub fn matched_readers(&mut self) -> &mut Vec<RtpsReaderProxy> {
        &mut self.matched_readers
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
    pub fn matched_reader_add(&mut self, mut a_reader_proxy: RtpsReaderProxy) {
        let status = if self.push_mode() {
            ChangeForReaderStatusKind::Unsent
        } else {
            ChangeForReaderStatusKind::Unacknowledged
        };
        for change in self.writer.writer_cache().changes() {
            a_reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReader::new(
                    status,
                    true,
                    change.sequence_number(),
                ));
        }

        self.matched_readers.push(a_reader_proxy)
    }

    pub fn matched_reader_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderProxy) -> bool,
    {
        self.matched_readers.retain(|x| !f(x));
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: Guid) -> Option<&RtpsReaderProxy> {
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
                ChangeForReaderStatusKind::Unsent
            } else {
                ChangeForReaderStatusKind::Unacknowledged
            };
            reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReader::new(status, true, sequence_number))
        }
    }

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsCacheChangeImpl) -> bool,
    {
        self.writer.writer_cache().remove_change(f)
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_max()
    }
}

impl<'a, T> RtpsStatefulWriterImpl<T>
where
    T: Timer,
{
    pub fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(&RtpsReaderProxy, DataSubmessage<'a>),
        mut send_gap: impl FnMut(&RtpsReaderProxy, GapSubmessage),
        mut send_heartbeat: impl FnMut(&RtpsReaderProxy, HeartbeatSubmessage),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.writer.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period().nanosec() as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
            self.heartbeat_count = Count(self.heartbeat_count.0.wrapping_add(1));
        }

        let reliability_level = self.reliability_level();
        let writer_id = self.guid().entity_id;
        let heartbeat_count = self.heartbeat_count;

        for reader_proxy in self.matched_readers.iter_mut() {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    while let Some(send_submessage) =
                        reader_proxy.best_effort_send_unsent_changes(&self.writer.writer_cache)
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
                            reader_proxy.send_heartbeat(writer_id, &self.writer.writer_cache);
                        heartbeat.count.value = heartbeat_count.0;
                        send_heartbeat(reader_proxy, heartbeat)
                    }

                    while let Some(send_submessage) =
                        reader_proxy.reliable_send_unsent_changes(&self.writer.writer_cache)
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
                        reader_proxy.send_requested_changes(&self.writer.writer_cache)
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
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reliability_level() == ReliabilityKind::Reliable {
            let reader_guid = Guid::new(
                source_guid_prefix,
                acknack_submessage.reader_id.value.into(),
            );

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                reader_proxy.reliable_receive_acknack(acknack_submessage);
            }
        }
    }
}
