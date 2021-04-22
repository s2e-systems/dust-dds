use rust_dds_api::infrastructure::qos::DataWriterQos;
use rust_rtps_pim::{
    behavior::{
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    structure::{
        types::{ChangeKind, Locator, ReliabilityKind, TopicKind, GUID},
        RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

pub struct RTPSStatelessWriterImpl<PSM: rust_rtps_pim::structure::Types> {
    guid: GUID<PSM>,
    pub reader_locators: Vec<RTPSReaderLocator<PSM>>,
}

impl<PSM: rust_rtps_pim::structure::Types> RTPSStatelessWriterImpl<PSM> {
    pub fn new(_qos: DataWriterQos, guid: GUID<PSM>) -> Self {
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
            guid,
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

impl<PSM: rust_rtps_pim::structure::Types> RTPSEntity<PSM>
    for RTPSStatelessWriterImpl<PSM>
{
    fn guid(&self) -> GUID<PSM> {
        self.guid
    }
}

impl<PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types> RTPSWriter<PSM>
    for RTPSStatelessWriterImpl<PSM>
{
    fn push_mode(&self) -> bool {
        todo!()
    }

    fn heartbeat_period(&self) -> <PSM as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn nack_response_delay(&self) -> <PSM as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn nack_suppression_duration(&self) -> <PSM as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn last_change_sequence_number(
        &self,
    ) -> <PSM as rust_rtps_pim::structure::Types>::SequenceNumber {
        todo!()
    }

    fn data_max_size_serialized(&self) -> i32 {
        todo!()
    }

    fn writer_cache(&self) -> &dyn RTPSHistoryCache<PSM> {
        todo!()
    }

    fn writer_cache_mut(&mut self) -> &mut dyn RTPSHistoryCache<PSM> {
        todo!()
    }

    fn new_change(
        &mut self,
        _kind: ChangeKind,
        _data: <PSM as rust_rtps_pim::structure::Types>::Data,
        _inline_qos: <PSM as rust_rtps_pim::structure::Types>::ParameterVector,
        _handle: <PSM as rust_rtps_pim::structure::Types>::InstanceHandle,
    ) -> rust_rtps_pim::structure::RTPSCacheChange<PSM> {
        todo!()
    }
}

impl<PSM: rust_rtps_pim::structure::Types> RTPSEndpoint<PSM>
    for RTPSStatelessWriterImpl<PSM>
{
    fn topic_kind(&self) -> TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        todo!()
    }

    fn unicast_locator_list(&self) -> &[<PSM as rust_rtps_pim::structure::Types>::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[<PSM as rust_rtps_pim::structure::Types>::Locator] {
        todo!()
    }
}

impl<
        PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types
    > RTPSStatelessWriter<PSM> for RTPSStatelessWriterImpl<PSM>
{
    fn reader_locator_add(&mut self, a_locator: Locator<PSM>) {
        let expects_inline_qos = false;
        self.reader_locators
            .push(RTPSReaderLocator::new(a_locator, expects_inline_qos));
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator<PSM>) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn reader_locators(
        &mut self,
    ) -> &mut [rust_rtps_pim::behavior::stateless_writer::RTPSReaderLocator<PSM>] {
        &mut self.reader_locators
    }
}
