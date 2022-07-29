use rtps_pim::{
    messages::submessages::{
        AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage,
    },
    structure::types::Locator,
};

use crate::{
    dcps_psm::{Duration, InstanceHandle},
    implementation::rtps::utils::clock::{Timer, TimerConstructor},
};

use super::{
    endpoint::RtpsEndpointImpl,
    history_cache::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl, RtpsParameter},
    types::{
        ChangeKind, Count, EntityId, Guid, ReliabilityKind, SequenceNumber, TopicKind,
        ENTITYID_UNKNOWN,
    },
    writer::RtpsWriterImpl,
};

pub enum BestEffortStatelessWriterSendSubmessage<'a> {
    Data(DataSubmessage<'a>),
    Gap(GapSubmessage),
}

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

impl RtpsReaderLocatorAttributesImpl {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: vec![],
            unsent_changes: vec![],
            last_received_acknack_count: Count(0),
        }
    }
}

impl RtpsReaderLocatorAttributesImpl {
    pub fn unsent_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        &mut self.unsent_changes
    }

    pub fn requested_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        &mut self.requested_changes
    }

    pub fn locator(&self) -> Locator {
        self.locator
    }

    pub fn expects_inline_qos(&self) -> bool {
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

    /// 8.4.8.1.4 Transition T4
    pub fn send_unsent_changes(&mut self) -> Option<BestEffortStatelessWriterSendSubmessage<'a>> {
        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // The post-condition:
            // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_in_cache() {
                let data_submessage = change.into();
                Some(BestEffortStatelessWriterSendSubmessage::Data(
                    data_submessage,
                ))
            } else {
                let gap_submessage = change.into();
                Some(BestEffortStatelessWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }

    /// 8.4.8.2.5 Transition T6
    /// Implementation does not include the part corresponding to searching the reader locator
    /// on the stateless writer
    pub fn receive_acknack(&mut self, acknack: &AckNackSubmessage) {
        self.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }
}

impl RtpsReaderLocatorOperationsImpl<'_> {
    pub fn unsent_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        self.reader_locator_attributes.unsent_changes_mut()
    }

    pub fn requested_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        self.reader_locator_attributes.requested_changes_mut()
    }

    pub fn locator(&self) -> Locator {
        self.reader_locator_attributes.locator()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.reader_locator_attributes.expects_inline_qos()
    }
}

pub struct RtpsReaderLocatorCacheChange<'a> {
    cache_change: Option<&'a RtpsCacheChangeImpl>,
}

impl RtpsReaderLocatorCacheChange<'_> {
    fn is_in_cache(&self) -> bool {
        self.cache_change.is_some()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for GapSubmessage {
    fn from(_val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        todo!()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for DataSubmessage<'a> {
    fn from(val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        let cache_change = val
            .cache_change
            .expect("Can only convert to data if it exists in the writer cache");
        cache_change.into()
    }
}

impl<'a> RtpsReaderLocatorOperationsImpl<'a> {
    pub fn next_requested_change(&mut self) -> RtpsReaderLocatorCacheChange<'a> {
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

    pub fn next_unsent_change(&mut self) -> RtpsReaderLocatorCacheChange<'a> {
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

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        self.reader_locator_attributes.requested_changes = req_seq_num_set.to_vec();
    }

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
        self.reader_locator_attributes.requested_changes.clone()
    }

    pub fn unsent_changes(&self) -> Vec<SequenceNumber> {
        self.reader_locator_attributes.unsent_changes.clone()
    }
}

pub struct RtpsStatelessWriterImpl<T> {
    writer: RtpsWriterImpl,
    reader_locators: Vec<RtpsReaderLocatorAttributesImpl>,
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
    pub fn reader_locators(&'_ mut self) -> Vec<RtpsReaderLocatorOperationsImpl<'_>> {
        let writer_cache = &self.writer.writer_cache;
        self.reader_locators
            .iter_mut()
            .map(|reader_locator_attributes| {
                RtpsReaderLocatorOperationsImpl::new(reader_locator_attributes, writer_cache)
            })
            .collect()
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

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn reader_locator_add(&mut self, mut a_locator: RtpsReaderLocatorAttributesImpl) {
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
        F: FnMut(&RtpsReaderLocatorAttributesImpl) -> bool,
    {
        self.reader_locators.retain(|x| !f(x))
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
        mut send_data: impl FnMut(&RtpsReaderLocatorOperationsImpl<'a>, DataSubmessage<'a>),
        mut send_gap: impl FnMut(&RtpsReaderLocatorOperationsImpl<'a>, GapSubmessage),
        _send_heartbeat: impl FnMut(&RtpsReaderLocatorOperationsImpl<'a>, HeartbeatSubmessage),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.heartbeat_period().nanosec() as u64);
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

impl<T> RtpsStatelessWriterImpl<T> {
    pub fn on_acknack_submessage_received(&mut self, acknack_submessage: &AckNackSubmessage) {
        let acknack_reader_id: EntityId = acknack_submessage.reader_id.value.into();
        let message_is_for_me =
            acknack_reader_id == ENTITYID_UNKNOWN || acknack_reader_id == self.guid().entity_id();

        if self.reliability_level() == ReliabilityKind::Reliable && message_is_for_me {
            for reader_locator in self.reader_locators.iter_mut() {
                if reader_locator.last_received_acknack_count.0 != acknack_submessage.count.value {
                    RtpsReaderLocatorOperationsImpl::new(reader_locator, &self.writer.writer_cache)
                        .receive_acknack(acknack_submessage);

                    reader_locator.last_received_acknack_count.0 = acknack_submessage.count.value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::structure::types::LOCATOR_INVALID;

    use crate::implementation::rtps::{history_cache::RtpsCacheChangeImpl, types::GUID_UNKNOWN};

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
