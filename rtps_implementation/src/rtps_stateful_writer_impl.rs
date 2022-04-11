use rtps_pim::{
    behavior::{
        stateful_writer_behavior::{
            BestEffortReaderProxyUnsentChangesBehavior,
            ReliableReaderProxyRequestedChangesBehavior, ReliableReaderProxySendHeartbeatBehavior,
            ReliableReaderProxyUnsentChangesBehavior, RtpsStatefulWriterSendSubmessages,
        },
        types::{
            ChangeForReaderStatusKind::{self, Unacknowledged, Unsent},
            Duration,
        },
        writer::{
            reader_proxy::RtpsReaderProxyAttributes,
            stateful_writer::{
                RtpsStatefulWriterAttributes, RtpsStatefulWriterConstructor,
                RtpsStatefulWriterOperations,
            },
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};

use crate::{
    rtps_reader_proxy_impl::{RtpsChangeForReaderImpl, RtpsReaderProxyOperationsImpl},
    utils::clock::{Timer, TimerConstructor},
};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_writer_impl::RtpsWriterImpl,
};

pub enum RtpsStatefulSubmessage<'a> {
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
    Heartbeat(HeartbeatSubmessage),
}

pub struct RtpsStatefulWriterImpl<T> {
    pub writer: RtpsWriterImpl,
    pub matched_readers: Vec<RtpsReaderProxyImpl>,
    pub heartbeat_timer: T,
    pub heartbeat_count: Count,
}

impl<T> RtpsEntityAttributes for RtpsStatefulWriterImpl<T> {
    fn guid(&self) -> Guid {
        self.writer.endpoint.entity.guid
    }
}

impl<T> RtpsEndpointAttributes for RtpsStatefulWriterImpl<T> {
    fn topic_kind(&self) -> TopicKind {
        self.writer.endpoint.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.writer.endpoint.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.writer.endpoint.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.writer.endpoint.multicast_locator_list
    }
}

impl<T> RtpsWriterAttributes for RtpsStatefulWriterImpl<T> {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn push_mode(&self) -> bool {
        self.writer.push_mode
    }

    fn heartbeat_period(&self) -> Duration {
        self.writer.heartbeat_period
    }

    fn nack_response_delay(&self) -> Duration {
        self.writer.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> Duration {
        self.writer.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> SequenceNumber {
        self.writer.last_change_sequence_number
    }

    fn data_max_size_serialized(&self) -> Option<i32> {
        self.writer.data_max_size_serialized
    }

    fn writer_cache(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.writer.writer_cache
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
                    sequence_number: change.sequence_number,
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
    type DataType = Vec<u8>;
    type ParameterListType = Vec<u8>;
    type CacheChangeType = RtpsCacheChangeImpl;
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Self::DataType,
        _inline_qos: Self::ParameterListType,
        handle: InstanceHandle,
    ) -> Self::CacheChangeType {
        self.writer.new_change(kind, data, _inline_qos, handle)
    }
}

impl<T> RtpsHistoryCacheOperations for RtpsStatefulWriterImpl<T> {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        let sequence_number = change.sequence_number;
        self.writer.writer_cache.add_change(change);

        for reader_proxy in &mut self.matched_readers {
            let status = if self.writer.push_mode {
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
        self.writer.writer_cache.remove_change(f)
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
            >= std::time::Duration::from_secs(self.writer.heartbeat_period.seconds as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period.fraction as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
            self.heartbeat_count = Count(self.heartbeat_count.0.wrapping_add(1));
        }

        let reliability_level = self.reliability_level();
        let writer_id = self.writer.endpoint.entity.guid.entity_id;
        let heartbeat_count = self.heartbeat_count;

        for reader_proxy in &mut self.matched_readers() {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    BestEffortReaderProxyUnsentChangesBehavior::send_unsent_changes(
                        reader_proxy,
                        |rp, data| send_data(rp, data),
                        |rp, gap| send_gap(rp, gap),
                    )
                }
                ReliabilityKind::Reliable => {
                    if time_for_heartbeat {
                        ReliableReaderProxySendHeartbeatBehavior::send_heartbeat(
                            reader_proxy,
                            writer_id,
                            heartbeat_count,
                            |rp, heartbeat| send_heartbeat(rp, heartbeat),
                        );
                    }

                    ReliableReaderProxyUnsentChangesBehavior::send_unsent_changes(
                        reader_proxy,
                        |rp, data| send_data(rp, data),
                        |rp, gap| send_gap(rp, gap),
                    );

                    ReliableReaderProxyRequestedChangesBehavior::send_requested_changes(
                        reader_proxy,
                        |rp, data| send_data(rp, data),
                        |rp, gap| send_gap(rp, gap),
                    );
                }
            }
        }
    }
}
