use std::ops::{Deref, DerefMut};

use rust_dds_api::{
    infrastructure::{qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind},
    return_type::DDSResult,
};
use rust_rtps_pim::behavior::stateless_writer::RTPSStatelessWriterBehavior;
use rust_rtps_pim::messages::submessages::Data;
use rust_rtps_pim::messages::submessages::Gap;
use rust_rtps_pim::{
    behavior::{
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    structure,
};
use rust_rtps_udp_psm::types::ChangeKind;
use rust_rtps_udp_psm::{
    submessages,
    types::{Duration, EntityId, Locator, TopicKind},
    RtpsUdpPsm,
};
use structure::{types::GUID, RTPSHistoryCache};

use super::history_cache_impl::HistoryCacheImpl;

pub struct StatelessDataWriterImpl {
    pub writer: RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>,
    pub reader_locators: Vec<RTPSReaderLocator<RtpsUdpPsm>>,
}

impl StatelessDataWriterImpl {
    pub fn new(qos: DataWriterQos) -> Self {
        let guid = GUID::new(
            [1; 12],
            EntityId {
                entity_key: [1; 3],
                entity_kind: 1,
            },
        );
        let topic_kind = TopicKind::WithKey;

        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => {
                <RtpsUdpPsm as structure::Types>::BEST_EFFORT
            }
            ReliabilityQosPolicyKind::ReliableReliabilityQos => {
                <RtpsUdpPsm as structure::Types>::RELIABLE
            }
        };
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let push_mode = true;
        let heartbeat_period = Duration {
            seconds: 1,
            fraction: 0,
        };
        let nack_response_delay = Duration {
            seconds: 0,
            fraction: 0,
        };
        let nack_suppression_duration = Duration {
            seconds: 0,
            fraction: 0,
        };
        let data_max_size_serialized = i32::MAX;

        let writer = RTPSWriter::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        Self {
            writer,
            reader_locators: Vec::new(),
        }
    }

    pub fn write_w_timestamp(&mut self) -> DDSResult<()> {
        let kind = ChangeKind::Alive;
        let data = vec![0, 1, 2];
        let inline_qos = vec![];
        let handle = 1;
        let change = self.new_change(kind, data, inline_qos, handle);
        self.writer_cache.add_change(change);
        Ok(())
    }

    pub fn produce_messages(
        &mut self,
        send_data_to: &mut impl FnMut(&Locator, submessages::Data),
        send_gap_to: &mut impl FnMut(&Locator, submessages::Gap),
    ) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.produce_messages(&self.writer, send_data_to, send_gap_to);
        }
    }
}

impl Deref for StatelessDataWriterImpl {
    type Target = RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for StatelessDataWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl RTPSStatelessWriter<RtpsUdpPsm, HistoryCacheImpl> for StatelessDataWriterImpl {
    fn reader_locator_add(
        &mut self,
        a_locator: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator,
    ) {
        let expects_inline_qos = false;
        self.reader_locators
            .push(RTPSReaderLocator::new(a_locator, expects_inline_qos));
    }

    fn reader_locator_remove(
        &mut self,
        a_locator: &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator,
    ) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn reader_locators(
        &mut self,
    ) -> &mut [rust_rtps_pim::behavior::stateless_writer::RTPSReaderLocator<RtpsUdpPsm>] {
        &mut self.reader_locators
    }

    fn reader_locators_and_writer(
        &mut self,
    ) -> (
        &mut [RTPSReaderLocator<RtpsUdpPsm>],
        &RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>,
    ) {
        (&mut self.reader_locators, &self.writer)
    }
}
