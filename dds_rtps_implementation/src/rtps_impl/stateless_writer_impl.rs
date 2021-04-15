use rust_dds_api::infrastructure::qos::DataWriterQos;
use rust_rtps_pim::behavior::stateless_writer::RTPSReaderLocator;
use rust_rtps_udp_psm::RtpsUdpPsm;

pub struct RTPSStatelessWriterImpl {
    pub reader_locators: Vec<RTPSReaderLocator<RtpsUdpPsm>>,
}

impl RTPSStatelessWriterImpl {
    pub fn new(_qos: DataWriterQos) -> Self {
        // let guid = GUID::new(
        //     [1; 12],
        //     EntityId {
        //         entity_key: [1; 3],
        //         entity_kind: 1,
        //     },
        // );
        // let topic_kind = TopicKind::WithKey;

        // let reliability_level = match qos.reliability.kind {
        //     ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        //     ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        // };
        // let unicast_locator_list = vec![];
        // let multicast_locator_list = vec![];
        // let push_mode = true;
        // let heartbeat_period = Duration {
        //     seconds: 1,
        //     fraction: 0,
        // };
        // let nack_response_delay = Duration {
        //     seconds: 0,
        //     fraction: 0,
        // };
        // let nack_suppression_duration = Duration {
        //     seconds: 0,
        //     fraction: 0,
        // };
        // let data_max_size_serialized = i32::MAX;

        // let writer = RTPSWriter::new(
        //     guid,
        //     topic_kind,
        //     reliability_level,
        //     unicast_locator_list,
        //     multicast_locator_list,
        //     push_mode,
        //     heartbeat_period,
        //     nack_response_delay,
        //     nack_suppression_duration,
        //     data_max_size_serialized,
        // );
        Self {
            reader_locators: Vec::new(),
        }
    }

//     pub fn write_w_timestamp(&mut self) -> DDSResult<()> {
//         let kind = ChangeKind::Alive;
//         let data = vec![0, 1, 2];
//         let inline_qos = vec![];
//         let handle = 1;
//         let change = self.new_change(kind, data, inline_qos, handle);
//         self.writer_cache_mut().add_change(change);
//         Ok(())
//     }

//     pub fn produce_messages(
//         &mut self,
//         _send_data_to: &mut impl FnMut(&Locator<RtpsUdpPsm>, submessages::Data),
//         _send_gap_to: &mut impl FnMut(&Locator<RtpsUdpPsm>, submessages::Gap),
//     ) {
//         todo!()
//         // for reader_locator in &mut self.reader_locators {
//         //     reader_locator.produce_messages(&self, send_data_to, send_gap_to);
//         // }
//     }
}

// impl RTPSEntity<RtpsUdpPsm> for StatelessDataWriterImpl {
//     fn guid(&self) -> GUID<RtpsUdpPsm> {
//         todo!()
//     }
// }

// impl RTPSWriter<RtpsUdpPsm, HistoryCacheImpl> for StatelessDataWriterImpl {
//     fn push_mode(&self) -> bool {
//         todo!()
//     }

//     fn heartbeat_period(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
//         todo!()
//     }

//     fn nack_response_delay(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
//         todo!()
//     }

//     fn nack_suppression_duration(
//         &self,
//     ) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
//         todo!()
//     }

//     fn last_change_sequence_number(
//         &self,
//     ) -> <RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber {
//         todo!()
//     }

//     fn data_max_size_serialized(&self) -> i32 {
//         todo!()
//     }

//     fn writer_cache(&self) -> &HistoryCacheImpl {
//         todo!()
//     }

//     fn writer_cache_mut(&mut self) -> &mut HistoryCacheImpl {
//         todo!()
//     }

//     fn new_change(
//         &mut self,
//         _kind: ChangeKind,
//         _data: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::Data,
//         _inline_qos: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::ParameterVector,
//         _handle: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::InstanceHandle,
//     ) -> structure::RTPSCacheChange<RtpsUdpPsm> {
//         todo!()
//     }
// }

// impl RTPSEndpoint<RtpsUdpPsm> for StatelessDataWriterImpl {
//     fn topic_kind(&self) -> TopicKind {
//         todo!()
//     }

//     fn reliability_level(&self) -> ReliabilityKind {
//         todo!()
//     }

//     fn unicast_locator_list(&self) -> &[<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator] {
//         todo!()
//     }

//     fn multicast_locator_list(
//         &self,
//     ) -> &[<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator] {
//         todo!()
//     }
// }

// impl RTPSStatelessWriter<RtpsUdpPsm, HistoryCacheImpl> for StatelessDataWriterImpl {
//     fn reader_locator_add(
//         &mut self,
//         a_locator: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator,
//     ) {
//         let expects_inline_qos = false;
//         self.reader_locators
//             .push(RTPSReaderLocator::new(a_locator, expects_inline_qos));
//     }

//     fn reader_locator_remove(
//         &mut self,
//         a_locator: &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator,
//     ) {
//         self.reader_locators.retain(|x| x.locator() != a_locator)
//     }

//     fn reader_locators(
//         &mut self,
//     ) -> &mut [rust_rtps_pim::behavior::stateless_writer::RTPSReaderLocator<RtpsUdpPsm>] {
//         &mut self.reader_locators
//     }
// }
