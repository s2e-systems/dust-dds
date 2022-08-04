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
    endpoint::RtpsEndpoint,
    history_cache::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl, RtpsParameter},
    reader_locator::{BestEffortStatelessWriterSendSubmessage, RtpsReaderLocator},
    types::{
        ChangeKind, Count, EntityId, Guid, ReliabilityKind, SequenceNumber, TopicKind,
        ENTITYID_UNKNOWN,
    },
    writer::RtpsWriterImpl,
};

pub struct RtpsStatelessWriterImpl<T> {
    writer: RtpsWriterImpl,
    reader_locators: Vec<RtpsReaderLocator>,
    heartbeat_timer: T,
    _heartbeat_count: Count,
}

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn guid(&self) -> Guid {
        self.writer.guid()
    }
}

impl<T> RtpsStatelessWriterImpl<T> {
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

impl<T> RtpsStatelessWriterImpl<T> {
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

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn reader_locators(&'_ mut self) -> &mut Vec<RtpsReaderLocator> {
        &mut self.reader_locators
    }
}

impl<T: TimerConstructor> RtpsStatelessWriterImpl<T> {
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
                RtpsEndpoint::new(
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
            reader_locators: Vec::new(),
            heartbeat_timer: T::new(),
            _heartbeat_count: Count(0),
        }
    }
}

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn reader_locator_add(&mut self, mut a_locator: RtpsReaderLocator) {
        *a_locator.unsent_changes_mut() = self
            .writer_cache()
            .changes()
            .iter()
            .map(|c| c.sequence_number())
            .collect();
        self.reader_locators.push(a_locator);
    }

    pub fn reader_locator_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderLocator) -> bool,
    {
        self.reader_locators.retain(|x| !f(x))
    }

    pub fn reader_locator_lookup(
        &mut self,
        locator: &Locator,
    ) -> Option<&mut RtpsReaderLocator> {
        self.reader_locators
            .iter_mut()
            .find(|x| x.locator() == *locator)
    }

    pub fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl<T> RtpsStatelessWriterImpl<T> {
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

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn add_change(&mut self, change: RtpsCacheChangeImpl) {
        for reader_locator in &mut self.reader_locators {
            reader_locator
                .unsent_changes_mut()
                .push(change.sequence_number());
        }
        self.writer.writer_cache().add_change(change);
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

impl<'a, T> RtpsStatelessWriterImpl<T>
where
    T: Timer,
{
    pub fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(&RtpsReaderLocator, DataSubmessage<'a>),
        mut send_gap: impl FnMut(&RtpsReaderLocator, GapSubmessage),
        _send_heartbeat: impl FnMut(&RtpsReaderLocator, HeartbeatSubmessage),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.heartbeat_period().nanosec() as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
        }
        let reliability_level = self.reliability_level();
        for reader_locator in &mut self.reader_locators {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    while let Some(send_submessage) =
                        reader_locator.send_unsent_changes(&self.writer.writer_cache)
                    {
                        match send_submessage {
                            BestEffortStatelessWriterSendSubmessage::Data(data) => {
                                send_data(reader_locator, data)
                            }
                            BestEffortStatelessWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_locator, gap)
                            }
                        }
                    }
                }
                ReliabilityKind::Reliable => todo!(),
            }
        }
    }
}

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn on_acknack_submessage_received(&mut self, acknack_submessage: &AckNackSubmessage) {
        let acknack_reader_id: EntityId = acknack_submessage.reader_id.value.into();
        let message_is_for_me =
            acknack_reader_id == ENTITYID_UNKNOWN || acknack_reader_id == self.guid().entity_id();

        if self.reliability_level() == ReliabilityKind::Reliable && message_is_for_me {
            for reader_locator in self.reader_locators.iter_mut() {
                reader_locator.receive_acknack(acknack_submessage);
            }
        }
    }
}
