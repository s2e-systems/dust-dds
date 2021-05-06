use rust_dds_api::infrastructure::qos::DataWriterQos;
use rust_rtps_pim::{
    behavior::{
        stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter},
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    structure::{
        types::{ChangeKind, Locator, ReliabilityKind, TopicKind, GUID},
        RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

use super::rtps_history_cache_impl::RTPSHistoryCacheImpl;

pub struct RTPSWriterImpl<PSM: rust_rtps_pim::structure::Types> {
    guid: GUID<PSM>,
    pub reader_locators: Vec<RTPSReaderLocator<PSM>>,
    pub reader_proxies: Vec<RTPSReaderProxy<PSM>>,
    last_change_sequence_number: PSM::SequenceNumber,
    pub writer_cache: RTPSHistoryCacheImpl<PSM>,
}

impl<PSM: rust_rtps_pim::structure::Types> RTPSWriterImpl<PSM> {
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
            reader_proxies: Vec::new(),
            last_change_sequence_number: 0.into(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        }
    }

    pub fn produce_messages<'a, Data, Gap, SendDataTo, SendGapTo>(
        &'a mut self,
        send_data_to: &mut SendDataTo,
        send_gap_to: &mut SendGapTo,
    ) where
        PSM: rust_rtps_pim::behavior::Types + rust_rtps_pim::messages::Types,
        PSM::ParameterVector: Clone,
        Data: rust_rtps_pim::messages::submessages::Data<
                SerializedData = &'a <PSM as rust_rtps_pim::structure::types::Types>::Data,
            > + rust_rtps_pim::messages::Submessage<PSM = PSM>,
        Gap: rust_rtps_pim::messages::submessages::Gap
            + rust_rtps_pim::messages::Submessage<PSM = PSM>,
        SendDataTo: FnMut(&Locator<PSM>, Data),
        SendGapTo: FnMut(&Locator<PSM>, Gap),
    {
        for reader_locator in &mut self.reader_locators {
            reader_locator.produce_messages::<Data, Gap, _, _>(
                &self.writer_cache,
                send_data_to,
                send_gap_to,
            )
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

impl<PSM: rust_rtps_pim::structure::Types> RTPSEntity<PSM> for RTPSWriterImpl<PSM> {
    fn guid(&self) -> GUID<PSM> {
        self.guid
    }
}

impl<PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types> RTPSWriter<PSM>
    for RTPSWriterImpl<PSM>
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
        &self.writer_cache
    }

    fn writer_cache_mut(&mut self) -> &mut dyn RTPSHistoryCache<PSM> {
        &mut self.writer_cache
    }

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <PSM as rust_rtps_pim::structure::Types>::Data,
        inline_qos: <PSM as rust_rtps_pim::structure::Types>::ParameterVector,
        handle: <PSM as rust_rtps_pim::structure::Types>::InstanceHandle,
    ) -> rust_rtps_pim::structure::RTPSCacheChange<PSM> {
        self.last_change_sequence_number = (self.last_change_sequence_number.into() + 1i64).into();
        RTPSCacheChange {
            kind,
            writer_guid: self.guid,
            instance_handle: handle,
            sequence_number: self.last_change_sequence_number,
            data_value: data,
            inline_qos,
        }
    }
}

impl<PSM: rust_rtps_pim::structure::Types> RTPSEndpoint<PSM> for RTPSWriterImpl<PSM> {
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

impl<PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types> RTPSStatelessWriter<PSM>
    for RTPSWriterImpl<PSM>
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

impl<PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types> RTPSStatefulWriter<PSM>
    for RTPSWriterImpl<PSM>
{
    fn matched_readers(&self) -> &[rust_rtps_pim::behavior::stateful_writer::RTPSReaderProxy<PSM>] {
        todo!()
    }

    fn matched_reader_add(&mut self, _guid: GUID<PSM>) {
        todo!()
    }

    fn matched_reader_remove(&mut self, _reader_proxy_guid: &GUID<PSM>) {
        todo!()
    }

    fn matched_reader_lookup(
        &self,
        _a_reader_guid: GUID<PSM>,
    ) -> Option<&rust_rtps_pim::behavior::stateful_writer::RTPSReaderProxy<PSM>> {
        todo!()
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_udp_psm::submessages::{Data, Gap};
    use rust_rtps_udp_psm::{types::EntityId, RtpsUdpPsm};

    use super::*;

    #[test]
    fn send_data() {
        let mut rtps_writer_impl: RTPSWriterImpl<RtpsUdpPsm> = RTPSWriterImpl::new(
            DataWriterQos::default(),
            GUID::new(
                [1; 12],
                EntityId {
                    entity_key: [1; 3],
                    entity_kind: 1,
                },
            ),
        );

        let cc = rtps_writer_impl.new_change(ChangeKind::Alive, vec![0, 1, 2, 3], vec![], 0);
        rtps_writer_impl.writer_cache_mut().add_change(cc);
        rtps_writer_impl.reader_locator_add(Locator::new(1, 2, [0; 16]));
        println!("First");
        rtps_writer_impl.produce_messages::<Data, Gap, _, _>(
            &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y),
            &mut |_, _| (),
        );
        println!("Second");
        rtps_writer_impl.reader_locator_add(Locator::new(1, 3, [1; 16]));
        rtps_writer_impl.produce_messages::<Data, Gap, _, _>(
            &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y),
            &mut |_, _| (),
        );
        println!("After reset");
        rtps_writer_impl.unsent_changes_reset();
        rtps_writer_impl.produce_messages::<Data, Gap, _, _>(
            &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y),
            &mut |_, _| (),
        );
    }
}
