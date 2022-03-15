use std::cell::RefCell;

use rtps_pim::{
    behavior::{
        stateful_writer_behavior::{
            BestEffortStatefulWriterBehavior, ReliableStatefulWriterBehavior,
        },
        types::Duration,
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
        submessages::{DataSubmessage, GapSubmessage},
        types::Count,
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};

use crate::{rtps_reader_proxy_impl::RtpsReaderProxyOperationsImpl, utils::clock::StdTimer};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_writer_impl::RtpsWriterImpl,
};

pub enum RtpsStatefulSubmessage<'a> {
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
}

pub struct RtpsStatefulWriterImpl {
    pub writer: RtpsWriterImpl,
    pub matched_readers: Vec<RtpsReaderProxyImpl>,
    pub heartbeat_timer: StdTimer,
    pub heartbeat_count: Count,
}

impl RtpsStatefulWriterImpl {
    pub fn produce_destined_submessages<'a>(
        &'a mut self,
    ) -> Vec<(&mut RtpsReaderProxyImpl, Vec<RtpsStatefulSubmessage<'a>>)> {
        let mut destined_submessages = Vec::new();

        for reader_proxy in &mut self.matched_readers {
            match self.writer.endpoint.reliability_level {
                ReliabilityKind::BestEffort => {
                    let submessages = RefCell::new(Vec::new());
                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                    BestEffortStatefulWriterBehavior::send_unsent_changes(
                        &mut RtpsReaderProxyOperationsImpl::new(
                            reader_proxy,
                            &self.writer.writer_cache,
                            self.writer.push_mode,
                        ),
                        &self.writer.writer_cache,
                        reader_id,
                        |data| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatefulSubmessage::Data(data))
                        },
                        |gap| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatefulSubmessage::Gap(gap))
                        },
                    );

                    let submessages = submessages.take();

                    if !submessages.is_empty() {
                        destined_submessages.push((reader_proxy, submessages));
                    }
                }

                ReliabilityKind::Reliable => {
                    let submessages = RefCell::new(Vec::new());
                    // ReliableStatefulWriterBehavior::send_heartbeat(
                    //     &self.writer.writer_cache,
                    //     self.writer.endpoint.entity.guid.entity_id,
                    //     self.heartbeat_count,
                    //     &mut |heartbeat| {
                    //         submessages
                    //             .borrow_mut()
                    //             .push(RtpsSubmessageType::Heartbeat(heartbeat));
                    //     },
                    // );
                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                    ReliableStatefulWriterBehavior::send_unsent_changes(
                        &mut RtpsReaderProxyOperationsImpl::new(
                            reader_proxy,
                            &self.writer.writer_cache,
                            self.writer.push_mode,
                        ),
                        &self.writer.writer_cache,
                        reader_id,
                        |data| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatefulSubmessage::Data(data))
                        },
                        |gap| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatefulSubmessage::Gap(gap))
                        },
                    );

                    let submessages = submessages.take();

                    if !submessages.is_empty() {
                        destined_submessages.push((reader_proxy, submessages));
                    }
                }
            }
        }

        destined_submessages
    }
}

impl RtpsEntityAttributes for RtpsStatefulWriterImpl {
    fn guid(&self) -> Guid {
        self.writer.endpoint.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatefulWriterImpl {
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

impl RtpsWriterAttributes for RtpsStatefulWriterImpl {
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

impl RtpsStatefulWriterAttributes for RtpsStatefulWriterImpl {
    type ReaderProxyType = RtpsReaderProxyImpl;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        &self.matched_readers
    }
}

impl RtpsStatefulWriterConstructor for RtpsStatefulWriterImpl {
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
            heartbeat_timer: StdTimer::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl RtpsStatefulWriterOperations for RtpsStatefulWriterImpl {
    type ReaderProxyType = RtpsReaderProxyImpl;

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType) {
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

impl RtpsWriterOperations for RtpsStatefulWriterImpl {
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

#[cfg(test)]
mod tests {
    use rtps_pim::{
        behavior::writer::reader_proxy::{RtpsReaderProxyConstructor, RtpsReaderProxyOperations},
        messages::{
            submessage_elements::{
                EntityIdSubmessageElement, ParameterListSubmessageElement,
                SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
            },
            types::ParameterId,
        },
        structure::{
            cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
            history_cache::RtpsHistoryCacheOperations,
            types::{EntityId, GuidPrefix, USER_DEFINED_READER_NO_KEY, USER_DEFINED_WRITER_NO_KEY},
        },
    };

    use crate::rtps_history_cache_impl::{RtpsData, RtpsParameter, RtpsParameterList};

    use super::*;

    #[test]
    fn produce_destined_submessages_one_locator_one_submessage() {
        let guid = Guid::new(
            GuidPrefix([0; 12]),
            EntityId::new([1, 2, 3], USER_DEFINED_WRITER_NO_KEY),
        );

        let mut writer = RtpsStatefulWriterImpl::new(
            guid,
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            false,
            Duration::new(0, 0),
            Duration::new(0, 0),
            Duration::new(0, 0),
            None,
        );

        let mut matched_reader_proxy = RtpsReaderProxyImpl::new(
            Guid::new(
                GuidPrefix([1; 12]),
                EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            ),
            EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            &[],
            &[],
            false,
            false,
        );

        let change = RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            guid,
            0,
            1,
            RtpsData(vec![4, 1, 3]),
            RtpsParameterList(vec![RtpsParameter {
                parameter_id: ParameterId(8),
                value: vec![6, 1, 2],
            }]),
        );

        writer.writer.writer_cache.add_change(change);
        RtpsReaderProxyOperationsImpl::new(
            &mut matched_reader_proxy,
            &writer.writer.writer_cache,
            true,
        );
        writer.matched_readers.push(matched_reader_proxy);

        let mut matched_reader_proxy = RtpsReaderProxyImpl::new(
            Guid::new(
                GuidPrefix([1; 12]),
                EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            ),
            EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            &[],
            &[],
            false,
            false,
        );

        let change = RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            guid,
            0,
            1,
            RtpsData(vec![4, 1, 3]),
            RtpsParameterList(vec![RtpsParameter {
                parameter_id: ParameterId(8),
                value: vec![6, 1, 2],
            }]),
        );

        {
            let mut operations = RtpsReaderProxyOperationsImpl::new(
                &mut matched_reader_proxy,
                &writer.writer.writer_cache,
                true,
            );
            operations.requested_changes_set(&[change.sequence_number]);
            operations.next_requested_change();
        }

        let destined_submessages = writer.produce_destined_submessages();
        assert_eq!(1, destined_submessages.len());
        let (reader_proxy, submessages) = &destined_submessages[0];
        assert_eq!(&&matched_reader_proxy, reader_proxy);
        assert_eq!(1, submessages.len());

        if let RtpsStatefulSubmessage::Data(data) = &submessages[0] {
            assert_eq!(true, data.endianness_flag);
            assert_eq!(
                &DataSubmessage {
                    endianness_flag: true,
                    inline_qos_flag: true,
                    data_flag: true,
                    key_flag: false,
                    non_standard_payload_flag: false,
                    reader_id: EntityIdSubmessageElement {
                        value: EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
                    },
                    writer_id: EntityIdSubmessageElement {
                        value: change.writer_guid.entity_id,
                    },
                    writer_sn: SequenceNumberSubmessageElement {
                        value: change.sequence_number
                    },
                    inline_qos: ParameterListSubmessageElement {
                        parameter: change.inline_qos().into()
                    },
                    serialized_payload: SerializedDataSubmessageElement {
                        value: change.data_value().into()
                    }
                },
                data
            )
        } else {
            panic!("Should be Data");
        }
    }
}
