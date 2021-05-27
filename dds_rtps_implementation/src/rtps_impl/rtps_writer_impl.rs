use rust_dds_api::infrastructure::qos::DataWriterQos;
use rust_rtps_pim::{
    behavior::{
        stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter},
        types::DurationType,
        RTPSWriter,
    },
    messages::types::ParameterIdType,
    structure::{
        types::{
            ChangeKind, DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType,
            LocatorType, ParameterListType, ReliabilityKind, SequenceNumberType, TopicKind,
        },
        RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

use super::{
    rtps_history_cache_impl::RTPSHistoryCacheImpl, rtps_reader_locator_impl::RTPSReaderLocatorImpl,
};

pub trait RTPSWriterImplTrait:
    SequenceNumberType
    + GuidPrefixType
    + EntityIdType
    + DurationType
    + DataType
    + LocatorType
    + InstanceHandleType
    + ParameterIdType
    + GUIDType<Self>
    + ParameterListType<Self>
    + Sized
{
}

impl<
        T: SequenceNumberType
            + GuidPrefixType
            + EntityIdType
            + DurationType
            + DataType
            + LocatorType
            + InstanceHandleType
            + ParameterIdType
            + GUIDType<Self>
            + ParameterListType<Self>
            + Sized,
    > RTPSWriterImplTrait for T
{
}

pub struct RTPSWriterImpl<PSM: RTPSWriterImplTrait> {
    guid: PSM::GUID,
    reader_locators: Vec<RTPSReaderLocatorImpl<PSM>>,
    reader_proxies: Vec<RTPSReaderProxy<PSM>>,
    last_change_sequence_number: PSM::SequenceNumber,
    writer_cache: RTPSHistoryCacheImpl<PSM>,
}

impl<PSM: RTPSWriterImplTrait> RTPSWriterImpl<PSM> {
    pub fn new(_qos: DataWriterQos, guid: PSM::GUID) -> Self {
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

    // pub fn locators_and_writer_cache(
    //     &mut self,
    // ) -> (&mut [RTPSReaderLocator<PSM>], &RTPSHistoryCacheImpl<PSM>) {
    //     (&mut self.reader_locators, &self.writer_cache)
    // }

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

impl<PSM: RTPSWriterImplTrait> RTPSEntity<PSM> for RTPSWriterImpl<PSM> {
    fn guid(&self) -> PSM::GUID {
        self.guid
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSWriter<PSM, RTPSHistoryCacheImpl<PSM>> for RTPSWriterImpl<PSM> {
    fn push_mode(&self) -> bool {
        todo!()
    }

    fn heartbeat_period(&self) -> PSM::Duration {
        todo!()
    }

    fn nack_response_delay(&self) -> PSM::Duration {
        todo!()
    }

    fn nack_suppression_duration(&self) -> PSM::Duration {
        todo!()
    }

    fn last_change_sequence_number(&self) -> PSM::SequenceNumber {
        todo!()
    }

    fn data_max_size_serialized(&self) -> i32 {
        todo!()
    }

    fn writer_cache(&self) -> &RTPSHistoryCacheImpl<PSM> {
        todo!()
    }

    fn writer_cache_mut(&mut self) -> &mut RTPSHistoryCacheImpl<PSM> {
        todo!()
    }

    fn new_change(
        &mut self,
        _kind: ChangeKind,
        _data: PSM::Data,
        _inline_qos: PSM::ParameterList,
        _handle: PSM::InstanceHandle,
    ) -> <RTPSHistoryCacheImpl<PSM> as RTPSHistoryCache<PSM>>::CacheChange {
        todo!()
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSEndpoint<PSM> for RTPSWriterImpl<PSM> {
    fn topic_kind(&self) -> TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        todo!()
    }

    fn unicast_locator_list(&self) -> &[PSM::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[PSM::Locator] {
        todo!()
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSStatefulWriter<PSM, RTPSHistoryCacheImpl<PSM>>
    for RTPSWriterImpl<PSM>
{
    fn matched_readers(&self) -> &[RTPSReaderProxy<PSM>] {
        todo!()
    }

    fn matched_reader_add(&mut self, _guid: PSM::GUID) {
        todo!()
    }

    fn matched_reader_remove(&mut self, _reader_proxy_guid: &PSM::GUID) {
        todo!()
    }

    fn matched_reader_lookup(&self, _a_reader_guid: PSM::GUID) -> Option<&RTPSReaderProxy<PSM>> {
        todo!()
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use rust_rtps_udp_psm::{types::EntityId, RtpsUdpPsm};

    // use super::*;

    // #[test]
    // fn send_data() {
    //     let mut rtps_writer_impl: RTPSWriterImpl<RtpsUdpPsm> = RTPSWriterImpl::new(
    //         DataWriterQos::default(),
    //         GUID::new(
    //             [1; 12],
    //             EntityId {
    //                 entity_key: [1; 3],
    //                 entity_kind: 1,
    //             },
    //         ),
    //     );

    //     let cc = rtps_writer_impl.new_change(ChangeKind::Alive, vec![0, 1, 2, 3], &[], 0);
    //     rtps_writer_impl.writer_cache_mut().add_change(cc);
    //     rtps_writer_impl.reader_locator_add(Locator::new(1, 2, [0; 16]));
    //     {
    //         println!("First");
    //         // let mut data_vec = Vec::new();
    //         // rtps_writer_impl.produce_messages(
    //         //     &mut |x, y| {
    //         //         println!("Locator: {:?}, Data: {}", x.address(), y.endianness_flag);
    //         //         data_vec.push(y);
    //         //     },
    //         //     &mut |_, _| (),
    //         // );
    //         // for data in &data_vec {
    //         //     println!("{}", data.endianness_flag);
    //         // }
    //     }
    //     println!("Second");
    //     // rtps_writer_impl.reader_locator_add(Locator::new(1, 3, [1; 16]));
    //     // rtps_writer_impl.produce_messages(
    //     //     &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y.endianness_flag),
    //     //     &mut |_, _| (),
    //     // );
    //     // println!("After reset");
    //     // rtps_writer_impl.unsent_changes_reset();
    //     // rtps_writer_impl.produce_messages(
    //     //     &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y.endianness_flag),
    //     //     &mut |_, _| (),
    //     // );
    // }
}
