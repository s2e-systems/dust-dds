use std::ops::{Deref, DerefMut};

use rust_dds_api::{
    infrastructure::{qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind},
    return_type::DDSResult,
};
use rust_rtps_pim::{
    behavior::{
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    structure,
};
use rust_rtps_udp_psm::{RtpsUdpPsm, submessages, types::{Duration, EntityId, Guid, Locator, TopicKind}};
use rust_rtps_pim::messages::submessages::Data;
use rust_rtps_pim::messages::submessages::Gap;
use rust_rtps_udp_psm::types::ChangeKind;
use structure::RTPSHistoryCache;

use super::history_cache_impl::HistoryCacheImpl;

pub struct StatelessDataWriterImpl {
    pub writer: RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>,
    pub reader_locators: Vec<RTPSReaderLocator<RtpsUdpPsm>>,
}

impl StatelessDataWriterImpl {
    pub fn new(qos: DataWriterQos) -> Self {
        let guid = Guid {
            prefix: [1; 12],
            entity_id: EntityId {
                entity_key: [1; 3],
                entity_kind: 1,
            },
        };
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
        mut send_data_to: impl FnMut(&Locator, submessages::Data),
        mut send_gap_to: impl FnMut(&Locator, submessages::Gap),
    ) {
        if self.writer.endpoint.reliability_level == <RtpsUdpPsm as structure::Types>::BEST_EFFORT {
            for reader_locator in &mut self.reader_locators {
                while reader_locator
                    .unsent_changes(&self.writer)
                    .into_iter()
                    .next()
                    .is_some()
                {
                    // Pushing state
                    if let Some(seq_num) = reader_locator.next_unsent_change(&self.writer) {
                        // Transition T4
                        if let Some(change) = self.writer.writer_cache.get_change(&seq_num) {
                            // Send Data submessage
                            let endianness_flag = true.into();
                            let inline_qos_flag = false.into();
                            let data_flag = true.into();
                            let key_flag = false.into();
                            let non_standard_payload_flag = false.into();
                            let reader_id = <<RtpsUdpPsm as structure::Types>::Guid as structure::types::Guid>::ENTITYID_UNKNOWN;
                            let writer_id = <<RtpsUdpPsm as structure::Types>::Guid as structure::types::Guid>::ENTITYID_UNKNOWN;
                            let writer_sn = change.sequence_number;
                            let inline_qos = change.inline_qos.clone();
                            let serialized_payload = &change.data_value;
                            let data = submessages::Data::new(
                                endianness_flag,
                                inline_qos_flag,
                                data_flag,
                                key_flag,
                                non_standard_payload_flag,
                                reader_id,
                                writer_id,
                                writer_sn,
                                inline_qos,
                                serialized_payload,
                            );
                            send_data_to(reader_locator.locator(), data)
                        } else {
                            // Send Gap submessage
                            let endianness_flag = true.into();
                            let reader_id = <<RtpsUdpPsm as structure::Types>::Guid as structure::types::Guid>::ENTITYID_UNKNOWN;
                            let writer_id = <<RtpsUdpPsm as structure::Types>::Guid as structure::types::Guid>::ENTITYID_UNKNOWN;
                            let gap_start = seq_num;
                            let gap_list = core::iter::empty().collect();
                            let gap = Gap::new(
                                endianness_flag,
                                reader_id,
                                writer_id,
                                gap_start,
                                gap_list,
                            );
                            send_gap_to(reader_locator.locator(), gap)
                        }
                    }
                }
            }
        } else {
            unimplemented!()
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
}
