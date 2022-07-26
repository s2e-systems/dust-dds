use rtps_pim::{
    behavior::{
        stateless_writer_behavior::{
            BestEffortReaderLocatorUnsentChangesBehavior, BestEffortStatelessWriterSendSubmessage,
            ChangeInHistoryCache, ReliableReaderLocatorReceiveAcknackBehavior,
            RtpsStatelessWriterReceiveAckNackSubmessage, RtpsStatelessWriterSendSubmessages,
        },
        types::Duration,
        writer::{
            reader_locator::{
                RtpsReaderLocatorAttributes, RtpsReaderLocatorConstructor,
                RtpsReaderLocatorOperations,
            },
            stateless_writer::{
                RtpsStatelessWriterAttributes, RtpsStatelessWriterConstructor,
                RtpsStatelessWriterOperations,
            },
            RtpsWriterAttributes, RtpsWriterOperations,
        },
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage},
        types::Count,
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
            ENTITYID_UNKNOWN,
        },
    },
};

use crate::implementation::rtps_impl::utils::clock::{Timer, TimerConstructor};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_writer_impl::RtpsWriterImpl,
};

pub struct RtpsReaderLocatorAttributesImpl {
    requested_changes: Vec<SequenceNumber>,
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    expects_inline_qos: bool,
    last_received_acknack_count: Count,
}

impl RtpsReaderLocatorAttributesImpl {
    pub fn unsent_changes_reset(&mut self) {
        self.unsent_changes = vec![];
    }
}

impl RtpsReaderLocatorConstructor for RtpsReaderLocatorAttributesImpl {
    type CacheChangeType = SequenceNumber;
    fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: vec![],
            unsent_changes: vec![],
            last_received_acknack_count: Count(0),
        }
    }
}

impl RtpsReaderLocatorAttributes for RtpsReaderLocatorAttributesImpl {
    type CacheChangeListType = Vec<SequenceNumber>;

    fn unsent_changes_mut(&mut self) -> &mut Self::CacheChangeListType {
        &mut self.unsent_changes
    }
    fn requested_changes_mut(&mut self) -> &mut Self::CacheChangeListType {
        &mut self.requested_changes
    }

    fn locator(&self) -> Locator {
        self.locator
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}

pub struct RtpsReaderLocatorOperationsImpl<'a> {
    reader_locator_attributes: &'a mut RtpsReaderLocatorAttributesImpl,
    writer_cache: &'a RtpsHistoryCacheImpl,
}

impl<'a> RtpsReaderLocatorOperationsImpl<'a> {
    pub fn new(
        reader_locator_attributes: &'a mut RtpsReaderLocatorAttributesImpl,
        writer_cache: &'a RtpsHistoryCacheImpl,
    ) -> Self {
        Self {
            reader_locator_attributes,
            writer_cache,
        }
    }
}

impl RtpsReaderLocatorAttributes for RtpsReaderLocatorOperationsImpl<'_> {
    type CacheChangeListType = Vec<SequenceNumber>;

    fn unsent_changes_mut(&mut self) -> &mut Self::CacheChangeListType {
        self.reader_locator_attributes.unsent_changes_mut()
    }

    fn requested_changes_mut(&mut self) -> &mut Self::CacheChangeListType {
        self.reader_locator_attributes.requested_changes_mut()
    }

    fn locator(&self) -> Locator {
        self.reader_locator_attributes.locator()
    }

    fn expects_inline_qos(&self) -> bool {
        self.reader_locator_attributes.expects_inline_qos()
    }
}

pub struct RtpsReaderLocatorCacheChange<'a> {
    cache_change: Option<&'a RtpsCacheChangeImpl>,
}

impl ChangeInHistoryCache for RtpsReaderLocatorCacheChange<'_> {
    fn is_in_cache(&self) -> bool {
        self.cache_change.is_some()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for GapSubmessage<Vec<SequenceNumber>> {
    fn from(_val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        todo!()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for DataSubmessage<Vec<Parameter<'a>>, &'a [u8]> {
    fn from(val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        let cache_change = val
            .cache_change
            .expect("Can only convert to data if it exists in the writer cache");
        cache_change.into()
    }
}

impl<'a> RtpsReaderLocatorOperations for RtpsReaderLocatorOperationsImpl<'a> {
    type CacheChangeType = RtpsReaderLocatorCacheChange<'a>;
    type CacheChangeListType = Vec<SequenceNumber>;

    fn next_requested_change(&mut self) -> Self::CacheChangeType {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()};
        // return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self
            .reader_locator_attributes
            .requested_changes
            .iter()
            .min()
            .cloned()
            .unwrap();

        // 8.4.8.2.4 Transition T4
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.reader_locator_attributes
            .unsent_changes
            .retain(|c| *c != next_seq_num);

        let cache_change = self
            .writer_cache
            .changes()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        RtpsReaderLocatorCacheChange { cache_change }
    }

    fn next_unsent_change(&mut self) -> Self::CacheChangeType {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self
            .reader_locator_attributes
            .unsent_changes
            .iter()
            .min()
            .cloned()
            .unwrap();

        // 8.4.8.2.10 Transition T10
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.reader_locator_attributes
            .unsent_changes
            .retain(|c| *c != next_seq_num);

        let cache_change = self
            .writer_cache
            .changes()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        RtpsReaderLocatorCacheChange { cache_change }
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        self.reader_locator_attributes.requested_changes = req_seq_num_set.to_vec();
    }

    fn requested_changes(&self) -> Self::CacheChangeListType {
        self.reader_locator_attributes.requested_changes.clone()
    }
    fn unsent_changes(&self) -> Self::CacheChangeListType {
        self.reader_locator_attributes.unsent_changes.clone()
    }
}

pub struct RtpsStatelessWriterImpl<T> {
    writer: RtpsWriterImpl,
    reader_locators: Vec<RtpsReaderLocatorAttributesImpl>,
    heartbeat_timer: T,
    _heartbeat_count: Count,
}

impl<T> RtpsEntityAttributes for RtpsStatelessWriterImpl<T> {
    fn guid(&self) -> Guid {
        self.writer.guid()
    }
}

impl<T> RtpsEndpointAttributes for RtpsStatelessWriterImpl<T> {
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

impl<T> RtpsWriterAttributes for RtpsStatelessWriterImpl<T> {
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

impl<'a, T> RtpsStatelessWriterAttributes<'a> for RtpsStatelessWriterImpl<T> {
    type ReaderLocatorListType = Vec<RtpsReaderLocatorOperationsImpl<'a>>;

    fn reader_locators(&'a mut self) -> Self::ReaderLocatorListType {
        let writer_cache = &self.writer.writer_cache;
        self.reader_locators
            .iter_mut()
            .map(|reader_locator_attributes| {
                RtpsReaderLocatorOperationsImpl::new(reader_locator_attributes, writer_cache)
            })
            .collect()
    }
}

impl<T: TimerConstructor> RtpsStatelessWriterConstructor for RtpsStatelessWriterImpl<T> {
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
            heartbeat_timer: T::new(),
            _heartbeat_count: Count(0),
        }
    }
}

impl<T> RtpsStatelessWriterOperations for RtpsStatelessWriterImpl<T> {
    type ReaderLocatorType = RtpsReaderLocatorAttributesImpl;

    fn reader_locator_add(&mut self, mut a_locator: Self::ReaderLocatorType) {
        *a_locator.unsent_changes_mut() = self
            .writer_cache()
            .changes()
            .iter()
            .map(|c| c.sequence_number())
            .collect();
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

impl<T> RtpsWriterOperations for RtpsStatelessWriterImpl<T> {
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

impl<T> RtpsHistoryCacheOperations for RtpsStatelessWriterImpl<T> {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        for reader_locator in &mut self.reader_locators {
            reader_locator
                .unsent_changes_mut()
                .push(change.sequence_number());
        }
        self.writer.writer_cache().add_change(change);
    }

    fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&Self::CacheChangeType) -> bool,
    {
        self.writer.writer_cache().remove_change(f)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_min()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_max()
    }
}

impl<'a, T>
    RtpsStatelessWriterSendSubmessages<'a, Vec<Parameter<'a>>, &'a [u8], Vec<SequenceNumber>>
    for RtpsStatelessWriterImpl<T>
where
    T: Timer,
{
    type ReaderLocatorType = RtpsReaderLocatorOperationsImpl<'a>;

    fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(
            &Self::ReaderLocatorType,
            rtps_pim::messages::submessages::DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>,
        ),
        mut send_gap: impl FnMut(
            &Self::ReaderLocatorType,
            rtps_pim::messages::submessages::GapSubmessage<Vec<SequenceNumber>>,
        ),
        _send_heartbeat: impl FnMut(
            &Self::ReaderLocatorType,
            rtps_pim::messages::submessages::HeartbeatSubmessage,
        ),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.heartbeat_period().seconds as u64)
                + std::time::Duration::from_nanos(self.heartbeat_period().fraction as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
        }
        let reliability_level = self.reliability_level();
        for reader_locator in &mut self.reader_locators() {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    while let Some(send_submessage) = reader_locator.send_unsent_changes() {
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

impl<T> RtpsStatelessWriterReceiveAckNackSubmessage<Vec<SequenceNumber>>
    for RtpsStatelessWriterImpl<T>
{
    fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage<Vec<SequenceNumber>>,
    ) {
        let message_is_for_me = acknack_submessage.reader_id.value == ENTITYID_UNKNOWN
            || acknack_submessage.reader_id.value == self.guid().entity_id();

        if self.reliability_level() == ReliabilityKind::Reliable && message_is_for_me {
            for reader_locator in self.reader_locators.iter_mut() {
                if reader_locator.last_received_acknack_count != acknack_submessage.count.value {
                    ReliableReaderLocatorReceiveAcknackBehavior::receive_acknack(
                        &mut RtpsReaderLocatorOperationsImpl::new(
                            reader_locator,
                            &self.writer.writer_cache,
                        ),
                        acknack_submessage,
                    );

                    reader_locator.last_received_acknack_count = acknack_submessage.count.value;
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
        types::{ChangeKind, GUID_UNKNOWN, LOCATOR_INVALID},
    };

    use crate::implementation::rtps_impl::rtps_history_cache_impl::RtpsCacheChangeImpl;

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut hc = RtpsHistoryCacheImpl::new();
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            1,
            vec![],
            vec![],
        ));
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            2,
            vec![],
            vec![],
        ));
        let mut reader_locator_attributes =
            RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
        reader_locator_attributes.unsent_changes = vec![1, 2];
        let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
            reader_locator_attributes: &mut reader_locator_attributes,
            writer_cache: &hc,
        };

        assert_eq!(
            reader_locator_operations
                .next_unsent_change()
                .cache_change
                .unwrap()
                .sequence_number(),
            1
        );
        assert_eq!(
            reader_locator_operations
                .next_unsent_change()
                .cache_change
                .unwrap()
                .sequence_number(),
            2
        );
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let hc = RtpsHistoryCacheImpl::new();
        let mut reader_locator_attributes =
            RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
        let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
            reader_locator_attributes: &mut reader_locator_attributes,
            writer_cache: &hc,
        };
        let req_seq_num_set = vec![1, 2, 3];
        reader_locator_operations.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator_operations.requested_changes(),
            expected_requested_changes
        )
    }
}
