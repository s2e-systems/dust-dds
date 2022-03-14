use std::cell::RefCell;

use rtps_pim::{
    behavior::{
        stateless_writer_behavior::{
            BestEffortStatelessWriterBehavior, ReliableStatelessWriterBehavior,
        },
        types::Duration,
        writer::{
            reader_locator::RtpsReaderLocatorAttributes,
            stateless_writer::{
                RtpsStatelessWriterAttributes, RtpsStatelessWriterConstructor,
                RtpsStatelessWriterOperations,
            },
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, GapSubmessage},
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};

use crate::rtps_reader_locator_impl::RtpsReaderLocatorOperationsImpl;

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_reader_locator_impl::RtpsReaderLocatorAttributesImpl,
    rtps_writer_impl::RtpsWriterImpl,
};

pub struct RtpsStatelessWriterImpl {
    pub writer: RtpsWriterImpl,
    pub reader_locators: Vec<RtpsReaderLocatorAttributesImpl>,
}

pub enum RtpsStatelessSubmessage<'a> {
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
}

impl RtpsStatelessWriterImpl {
    pub fn get_destined_submessages<'a>(
        &'a mut self,
    ) -> Vec<(Locator, Vec<RtpsStatelessSubmessage<'a>>)> {
        let mut destined_submessages = Vec::new();

        for reader_locator in self.reader_locators.iter_mut() {
            match self.writer.endpoint.reliability_level {
                ReliabilityKind::BestEffort => {
                    let submessages = RefCell::new(Vec::new());
                    let writer_cache = &self.writer.writer_cache;
                    BestEffortStatelessWriterBehavior::send_unsent_changes(
                        &mut RtpsReaderLocatorOperationsImpl::new(reader_locator, writer_cache),
                        writer_cache,
                        |data| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatelessSubmessage::Data(data))
                        },
                        |gap| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatelessSubmessage::Gap(gap))
                        },
                    );

                    let submessages = submessages.take();
                    if !submessages.is_empty() {
                        destined_submessages.push((reader_locator.locator(), submessages));
                    }
                }

                ReliabilityKind::Reliable => {
                    let submessages = RefCell::new(Vec::new());
                    let writer_cache = &self.writer.writer_cache;
                    ReliableStatelessWriterBehavior::send_unsent_changes(
                        &mut RtpsReaderLocatorOperationsImpl::new(reader_locator, writer_cache),
                        writer_cache,
                        |data| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatelessSubmessage::Data(data))
                        },
                        |gap| {
                            submessages
                                .borrow_mut()
                                .push(RtpsStatelessSubmessage::Gap(gap))
                        },
                    );

                    let submessages = submessages.take();
                    if !submessages.is_empty() {
                        destined_submessages.push((reader_locator.locator(), submessages));
                    }
                }
            }
        }

        destined_submessages
    }
}

impl RtpsEntityAttributes for RtpsStatelessWriterImpl {
    fn guid(&self) -> Guid {
        self.writer.guid()
    }
}

impl RtpsEndpointAttributes for RtpsStatelessWriterImpl {
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

impl RtpsWriterAttributes for RtpsStatelessWriterImpl {
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

impl RtpsStatelessWriterAttributes for RtpsStatelessWriterImpl {
    type ReaderLocatorType = RtpsReaderLocatorAttributesImpl;

    fn reader_locators(&self) -> &[Self::ReaderLocatorType] {
        &self.reader_locators
    }
}

impl RtpsStatelessWriterConstructor for RtpsStatelessWriterImpl {
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
            reader_locators: Vec::new(),
        }
    }
}

impl RtpsStatelessWriterOperations for RtpsStatelessWriterImpl {
    type ReaderLocatorType = RtpsReaderLocatorAttributesImpl;

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType) {
        self.reader_locators.push(a_locator);
    }

    fn reader_locator_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::ReaderLocatorType) -> bool,
    {
        self.reader_locators.retain(|x| !f(x))
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl RtpsWriterOperations for RtpsStatelessWriterImpl {
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
