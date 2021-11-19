use std::ops::Deref;

use rust_dds_api::{
    dcps_psm::InstanceHandle,
    infrastructure::{entity::Entity, qos::DataWriterQos},
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener, publisher::Publisher,
    },
    return_type::DDSResult,
    topic::topic::Topic,
};
use rust_rtps_pim::{
    behavior::writer::{
        reader_locator::RtpsReaderLocator,
        reader_proxy::RtpsReaderProxy,
        stateful_writer::{
            RtpsStatefulWriter, RtpsStatefulWriterOperations, StatefulWriterBehavior,
        },
        stateless_writer::{
            RtpsStatelessWriter, RtpsStatelessWriterOperations, StatelessWriterBehavior,
        },
        writer::{RtpsWriter, RtpsWriterOperations},
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, GapSubmessage},
    },
    structure::{
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, Guid, GuidPrefix, Locator, SequenceNumber},
    },
};
use rust_rtps_psm::{
    messages::submessages::AckNackSubmessageRead, rtps_reader_locator_impl::RtpsReaderLocatorImpl,
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
};

use crate::{
    dds_type::DdsSerialize, rtps_impl::rtps_writer_history_cache_impl::WriterHistoryCache,
    utils::message_receiver::ProcessAckNackSubmessage,
};

pub type RtpsWriterType = RtpsWriter<Vec<Locator>, WriterHistoryCache>;
pub type RtpsStatelessWriterType =
    RtpsStatelessWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderLocatorImpl>>;
pub type RtpsStatefulWriterType =
    RtpsStatefulWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderProxyImpl>>;

pub trait StatelessWriterBehaviorType:
    for<'a> StatelessWriterBehavior<'a, Vec<SequenceNumber>, Vec<Parameter<Vec<u8>>>, &'a [u8]>
{
}

impl<T> StatelessWriterBehaviorType for T where
    T: for<'a> StatelessWriterBehavior<'a, Vec<SequenceNumber>, Vec<Parameter<Vec<u8>>>, &'a [u8]>
{
}

impl<'a, T> StatelessWriterBehavior<'a, Vec<SequenceNumber>, Vec<Parameter<Vec<u8>>>, &'a [u8]>
    for DataWriterImpl<T>
{
    fn send_unsent_data(
        &'a mut self,
        send_data: &mut dyn FnMut(
            &RtpsReaderLocator,
            DataSubmessage<Vec<Parameter<Vec<u8>>>, &'a [u8]>,
        ),
        send_gap: &mut dyn FnMut(&RtpsReaderLocator, GapSubmessage<Vec<SequenceNumber>>),
    ) {
        // self.rtps_writer_impl.send_unsent_data(send_data, send_gap)
        todo!()
    }
}

pub trait StatefulWriterBehaviorType:
    for<'a> StatefulWriterBehavior<
    'a,
    Vec<SequenceNumber>,
    Vec<Parameter<Vec<u8>>>,
    &'a [u8],
    Vec<Locator>,
>
{
}

impl<T> StatefulWriterBehaviorType for T where
    T: for<'a> StatefulWriterBehavior<
        'a,
        Vec<SequenceNumber>,
        Vec<Parameter<Vec<u8>>>,
        &'a [u8],
        Vec<Locator>,
    >
{
}

impl<'a, T>
    StatefulWriterBehavior<'a, Vec<SequenceNumber>, Vec<Parameter<Vec<u8>>>, &'a [u8], Vec<Locator>>
    for DataWriterImpl<T>
{
    fn send_unsent_data(
        &'a mut self,
        send_data: &mut dyn FnMut(
            &RtpsReaderProxy<Vec<Locator>>,
            DataSubmessage<Vec<Parameter<Vec<u8>>>, &'a [u8]>,
        ),
        send_gap: &mut dyn FnMut(
            &RtpsReaderProxy<Vec<Locator>>,
            GapSubmessage<Vec<SequenceNumber>>,
        ),
    ) {
        // self.rtps_writer_impl.send_unsent_data(send_data, send_gap)
        todo!()
    }

    fn send_heartbeat(
        &mut self,
        send_heartbeat: &mut dyn FnMut(
            &RtpsReaderProxy<Vec<Locator>>,
            rust_rtps_pim::messages::submessages::HeartbeatSubmessage,
        ),
    ) {
        let heartbeat_period_duration = std::time::Duration::new(
            self.rtps_writer_impl.heartbeat_period.seconds as u64,
            self.rtps_writer_impl.heartbeat_period.fraction,
        );
        let since_last_heartbeat_sent = std::time::Instant::now() - self.last_sent_heartbeat;
        if since_last_heartbeat_sent > heartbeat_period_duration {
            // self.rtps_writer_impl.send_heartbeat(send_heartbeat);
            // self.last_sent_heartbeat = std::time::Instant::now();
            todo!()
        }
    }

    fn send_requested_data(
        &'a mut self,
        send_data: &mut dyn FnMut(
            &RtpsReaderProxy<Vec<Locator>>,
            DataSubmessage<Vec<Parameter<Vec<u8>>>, &'a [u8]>,
        ),
        send_gap: &mut dyn FnMut(
            &RtpsReaderProxy<Vec<Locator>>,
            GapSubmessage<Vec<SequenceNumber>>,
        ),
    ) {
        todo!()
        // self.rtps_writer_impl
        // .send_requested_data(send_data, send_gap)
    }

    fn process_acknack_submessage(
        &mut self,
        acknack: &rust_rtps_pim::messages::submessages::AckNackSubmessage<Vec<SequenceNumber>>,
    ) {
        // self.rtps_writer_impl.process_acknack_submessage(acknack)
        todo!()
    }
}

// std::thread::spawn(move || {
//     let mut heartbeat_count = Count(1);
//     let heartbeat_period = stateful_writer_shared.lock().unwrap().heartbeat_period;
//
//     loop {
//         stateful_writer_shared.lock().unwrap().send_heartbeat(
//             heartbeat_count,
//
//                     .unwrap();
//             },
//         );
//         heartbeat_count += Count(1);

//         std::thread::sleep(heartbeat_period_duration);
//     }
// });

pub struct DataWriterImpl<T> {
    _qos: DataWriterQos,
    rtps_writer_impl: RtpsWriterType,
    _listener: Option<Box<dyn DataWriterListener<DataType = T> + Send + Sync>>,
    last_sent_heartbeat: std::time::Instant,
    reader_locators: Vec<RtpsReaderLocatorImpl>,
    matched_readers: Vec<RtpsReaderProxyImpl>,
}

impl<T> DataWriterImpl<T>
where
    T: Send + 'static,
{
    pub fn new(qos: DataWriterQos, rtps_writer_impl: RtpsWriterType) -> Self {
        Self {
            _qos: qos,
            rtps_writer_impl,
            _listener: None,
            last_sent_heartbeat: std::time::Instant::now(),
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
        }
    }
}

impl<T> DataWriter<T> for DataWriterImpl<T>
where
    T: DdsSerialize,
{
    fn register_instance(&mut self, _instance: T) -> DDSResult<Option<InstanceHandle>> {
        unimplemented!()
    }

    fn register_instance_w_timestamp(
        &mut self,
        _instance: T,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn unregister_instance(
        &mut self,
        _instance: T,
        _handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        unimplemented!()
    }

    fn unregister_instance_w_timestamp(
        &mut self,
        _instance: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&mut self, _data: &T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn write_w_timestamp(
        &mut self,
        data: &T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        let change = self
            .rtps_writer_impl
            .new_change(ChangeKind::Alive, data, vec![], 0);
        let time = rust_rtps_pim::messages::types::Time(0);
        self.rtps_writer_impl
            .writer_cache
            .set_source_timestamp(Some(time));
        self.rtps_writer_impl.writer_cache.add_change(change);
        Ok(())
    }

    fn dispose(&mut self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn dispose_w_timestamp(
        &mut self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::LivelinessLostStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_topic(&self) -> &dyn Topic<T> {
        unimplemented!()
    }

    fn get_publisher(&self) -> &dyn Publisher {
        unimplemented!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: rust_dds_api::builtin_topics::SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<T> Entity for DataWriterImpl<T> {
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener<DataType = T>>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // let qos = qos.unwrap_or_default();
        // qos.is_consistent()?;
        // self.qos = qos;
        // Ok(())
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // &self.qos
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(
        &self,
    ) -> DDSResult<rust_dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<rust_dds_api::dcps_psm::StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<T> RtpsStatelessWriterOperations for DataWriterImpl<T> {
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        let reader_locator_impl = RtpsReaderLocatorImpl::new(a_locator);
        self.reader_locators.push(reader_locator_impl);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl<T> RtpsStatefulWriterOperations<Vec<Locator>> for DataWriterImpl<T> {
    fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy<Vec<Locator>>) {
        let reader_proxy = RtpsReaderProxyImpl::new(a_reader_proxy);
        self.matched_readers.push(reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid) {
        self.matched_readers
            .retain(|x| &x.remote_reader_guid != reader_proxy_guid);
    }

    fn matched_reader_lookup(
        &self,
        a_reader_guid: &Guid,
    ) -> Option<&RtpsReaderProxy<Vec<Locator>>> {
        self.matched_readers
            .iter()
            .find(|&x| &x.remote_reader_guid == a_reader_guid)
            .map(|x| x.deref())
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl<T> ProcessAckNackSubmessage for DataWriterImpl<T> {
    fn process_acknack_submessage(
        &self,
        _source_guid_prefix: GuidPrefix,
        _acknack: &AckNackSubmessageRead,
    ) {
        todo!()
    }
}

// impl RtpsSubmessageSender for DataWriterImpl {
//     fn create_submessages(&mut self) -> Vec<(Locator, Vec<RtpsSubmessageTypeWrite>)> {
//         let destined_submessages: Vec<(Locator, Vec<RtpsSubmessageTypeWrite>)> = Vec::new();
//         let destined_submessages = RefCell::new(destined_submessages);
//         match &mut self.rtps_writer_impl {
//             RtpsWriterFlavor::Stateful {
//                 stateful_writer,
//                 heartbeat_sent_instant,
//                 heartbeat_count,
//             } => match stateful_writer.reliability_level {
//                 ReliabilityKind::Reliable => {
//                     stateful_writer.send_unsent_data(
//                         |reader_proxy, data| {
//                             let mut destined_submessages_borrow = destined_submessages.borrow_mut();
//                             match destined_submessages_borrow.iter_mut().find(|(locator, _)| {
//                                 locator == &reader_proxy.unicast_locator_list[0]
//                             }) {
//                                 Some((_, submessages)) => submessages.push(
//                                     RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
//                                         data.endianness_flag,
//                                         data.inline_qos_flag,
//                                         data.data_flag,
//                                         data.key_flag,
//                                         data.non_standard_payload_flag,
//                                         data.reader_id,
//                                         data.writer_id,
//                                         data.writer_sn,
//                                         data.inline_qos,
//                                         data.serialized_payload,
//                                     )),
//                                 ),
//                                 None => destined_submessages_borrow.push((
//                                     reader_proxy.unicast_locator_list[0],
//                                     vec![RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
//                                         data.endianness_flag,
//                                         data.inline_qos_flag,
//                                         data.data_flag,
//                                         data.key_flag,
//                                         data.non_standard_payload_flag,
//                                         data.reader_id,
//                                         data.writer_id,
//                                         data.writer_sn,
//                                         data.inline_qos,
//                                         data.serialized_payload,
//                                     ))],
//                                 )),
//                             }
//                         },
//                         |reader_proxy, gap| {
//                             let mut destined_submessages_borrow = destined_submessages.borrow_mut();
//                             match destined_submessages_borrow.iter_mut().find(|(locator, _)| {
//                                 locator == &reader_proxy.unicast_locator_list[0]
//                             }) {
//                                 Some((_, submessages)) => submessages.push(
//                                     RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
//                                         gap.endianness_flag,
//                                         gap.reader_id,
//                                         gap.writer_id,
//                                         gap.gap_start,
//                                         gap.gap_list,
//                                     )),
//                                 ),
//                                 None => destined_submessages_borrow.push((
//                                     reader_proxy.unicast_locator_list[0],
//                                     vec![RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
//                                         gap.endianness_flag,
//                                         gap.reader_id,
//                                         gap.writer_id,
//                                         gap.gap_start,
//                                         gap.gap_list,
//                                     ))],
//                                 )),
//                             }
//                         },
//                     );
//                     if heartbeat_sent_instant.elapsed()
//                         > Duration::new(
//                             stateful_writer.heartbeat_period.seconds as u64,
//                             stateful_writer.heartbeat_period.fraction,
//                         )
//                     {
//                         stateful_writer.send_heartbeat(
//                             *heartbeat_count,
//                             |reader_proxy, heartbeat| {
//                                 let mut destined_submessages_borrow =
//                                     destined_submessages.borrow_mut();
//                                 destined_submessages_borrow.push((
//                                     reader_proxy.unicast_locator_list[0],
//                                     vec![RtpsSubmessageTypeWrite::Heartbeat(
//                                         HeartbeatSubmessageWrite::new(
//                                             heartbeat.endianness_flag,
//                                             heartbeat.final_flag,
//                                             heartbeat.liveliness_flag,
//                                             heartbeat.reader_id,
//                                             heartbeat.writer_id,
//                                             heartbeat.first_sn,
//                                             heartbeat.last_sn,
//                                             heartbeat.count,
//                                         ),
//                                     )],
//                                 ));
//                             },
//                         );
//                         *heartbeat_sent_instant = Instant::now();
//                         heartbeat_count.0 += 1
//                     }
//                 }
//                 ReliabilityKind::BestEffort => {
//                     stateful_writer.send_unsent_data(
//                         |reader_proxy, data| {
//                             let mut destined_submessages_borrow = destined_submessages.borrow_mut();
//                             match destined_submessages_borrow.iter_mut().find(|(locator, _)| {
//                                 locator == &reader_proxy.unicast_locator_list[0]
//                             }) {
//                                 Some((_, submessages)) => submessages.push(
//                                     RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
//                                         data.endianness_flag,
//                                         data.inline_qos_flag,
//                                         data.data_flag,
//                                         data.key_flag,
//                                         data.non_standard_payload_flag,
//                                         data.reader_id,
//                                         data.writer_id,
//                                         data.writer_sn,
//                                         data.inline_qos,
//                                         data.serialized_payload,
//                                     )),
//                                 ),
//                                 None => destined_submessages_borrow.push((
//                                     reader_proxy.unicast_locator_list[0],
//                                     vec![RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
//                                         data.endianness_flag,
//                                         data.inline_qos_flag,
//                                         data.data_flag,
//                                         data.key_flag,
//                                         data.non_standard_payload_flag,
//                                         data.reader_id,
//                                         data.writer_id,
//                                         data.writer_sn,
//                                         data.inline_qos,
//                                         data.serialized_payload,
//                                     ))],
//                                 )),
//                             }
//                         },
//                         |reader_proxy, gap| {
//                             let mut destined_submessages_borrow = destined_submessages.borrow_mut();
//                             match destined_submessages_borrow.iter_mut().find(|(locator, _)| {
//                                 locator == &reader_proxy.unicast_locator_list[0]
//                             }) {
//                                 Some((_, submessages)) => submessages.push(
//                                     RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
//                                         gap.endianness_flag,
//                                         gap.reader_id,
//                                         gap.writer_id,
//                                         gap.gap_start,
//                                         gap.gap_list,
//                                     )),
//                                 ),
//                                 None => destined_submessages_borrow.push((
//                                     reader_proxy.unicast_locator_list[0],
//                                     vec![RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
//                                         gap.endianness_flag,
//                                         gap.reader_id,
//                                         gap.writer_id,
//                                         gap.gap_start,
//                                         gap.gap_list,
//                                     ))],
//                                 )),
//                             }
//                         },
//                     );
//                 }
//             },
//             RtpsWriterFlavor::Stateless(stateless_writer) => {
//                 stateless_writer.send_unsent_data(
//                     |reader_locator, data| {
//                         let mut destined_submessages_borrow = destined_submessages.borrow_mut();
//                         match destined_submessages_borrow
//                             .iter_mut()
//                             .find(|(locator, _)| locator == &reader_locator.locator)
//                         {
//                             Some((_, submessages)) => submessages.push(
//                                 RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
//                                     data.endianness_flag,
//                                     data.inline_qos_flag,
//                                     data.data_flag,
//                                     data.key_flag,
//                                     data.non_standard_payload_flag,
//                                     data.reader_id,
//                                     data.writer_id,
//                                     data.writer_sn,
//                                     data.inline_qos,
//                                     data.serialized_payload,
//                                 )),
//                             ),
//                             None => destined_submessages_borrow.push((
//                                 reader_locator.locator,
//                                 vec![RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
//                                     data.endianness_flag,
//                                     data.inline_qos_flag,
//                                     data.data_flag,
//                                     data.key_flag,
//                                     data.non_standard_payload_flag,
//                                     data.reader_id,
//                                     data.writer_id,
//                                     data.writer_sn,
//                                     data.inline_qos,
//                                     data.serialized_payload,
//                                 ))],
//                             )),
//                         }
//                     },
//                     |reader_locator, gap| {
//                         let mut destined_submessages_borrow = destined_submessages.borrow_mut();
//                         match destined_submessages_borrow
//                             .iter_mut()
//                             .find(|(locator, _)| locator == &reader_locator.locator)
//                         {
//                             Some((_, submessages)) => submessages.push(
//                                 RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
//                                     gap.endianness_flag,
//                                     gap.reader_id,
//                                     gap.writer_id,
//                                     gap.gap_start,
//                                     gap.gap_list,
//                                 )),
//                             ),
//                             None => destined_submessages_borrow.push((
//                                 reader_locator.locator,
//                                 vec![RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
//                                     gap.endianness_flag,
//                                     gap.reader_id,
//                                     gap.writer_id,
//                                     gap.gap_start,
//                                     gap.gap_list,
//                                 ))],
//                             )),
//                         }
//                     },
//                 );
//             }
//         }

//         destined_submessages.take()
//     }
// }

#[cfg(test)]
mod tests {

    use rust_rtps_pim::{
        behavior::writer::{
            reader_locator::RtpsReaderLocator, stateless_writer::RtpsStatelessWriterOperations,
        },
        structure::types::{ReliabilityKind, TopicKind, GUID_UNKNOWN},
    };

    use super::*;

    #[test]
    fn write_w_timestamp() {
        struct MockData(Vec<u8>);

        impl DdsSerialize for MockData {
            fn serialize<W: std::io::Write, R: crate::dds_type::Endianness>(
                &self,
                mut writer: W,
            ) -> DDSResult<()> {
                writer.write(&self.0).unwrap();
                Ok(())
            }
        }

        let guid = GUID_UNKNOWN;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let push_mode = true;
        let heartbeat_period = rust_rtps_pim::behavior::types::Duration::new(0, 200_000_000);
        let nack_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let nack_suppression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let data_max_size_serialized = None;
        let rtps_stateless_writer = RtpsWriterType::new(
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
        let a_reader_locator = RtpsReaderLocator {
            locator: Locator {
                kind: 1,
                port: 2,
                address: [3; 16],
            },
            expects_inline_qos: false,
        };

        let mut data_writer_impl =
            DataWriterImpl::new(DataWriterQos::default(), rtps_stateless_writer);
        data_writer_impl.reader_locator_add(a_reader_locator);

        let data_value = MockData(vec![0, 1, 0, 0, 7, 3]);
        data_writer_impl
            .write_w_timestamp(
                &data_value,
                None,
                rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
            )
            .unwrap();

        // let received_message = locator_message_receiver.try_recv().unwrap();
        // let endianness_flag = true;
        // let inline_qos_flag = true;
        // let data_flag = true;
        // let key_flag = false;
        // let non_standard_payload_flag = false;
        // let reader_id = EntityIdSubmessageElement {
        //     value: ENTITYID_UNKNOWN,
        // };
        // let writer_id = EntityIdSubmessageElement {
        //     value: ENTITYID_UNKNOWN,
        // };
        // let writer_sn = SequenceNumberSubmessageElement { value: 1 };
        // let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
        // let serialized_payload = SerializedDataSubmessageElement {
        //     value: &[0u8, 1, 0, 0, 7, 3][..],
        // };
        // let expected_message = vec![RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
        //     endianness_flag,
        //     inline_qos_flag,
        //     data_flag,
        //     key_flag,
        //     non_standard_payload_flag,
        //     reader_id,
        //     writer_id,
        //     writer_sn,
        //     inline_qos,
        //     serialized_payload,
        // ))];
        // assert_eq!(received_message, expected_message);
    }

    //     #[test]
    //     fn reader_locator_add() {
    //         let mut rtps_stateless_writer_impl: RtpsStatelessWriterImpl<MockHistoryCache> =
    //             RtpsStatelessWriterImpl::new(
    //                 GUID_UNKNOWN,
    //                 rust_rtps_pim::structure::types::TopicKind::WithKey,
    //                 rust_rtps_pim::structure::types::ReliabilityKind::BestEffort,
    //                 vec![],
    //                 vec![],
    //                 true,
    //                 DURATION_ZERO,
    //                 DURATION_ZERO,
    //                 DURATION_ZERO,
    //                 None,
    //             );

    //         let locator1 = Locator::new(1, 1, [1; 16]);
    //         let locator2 = Locator::new(2, 2, [2; 16]);
    //         let a_locator1 = RtpsReaderLocator::new(locator1, false);
    //         let a_locator2 = RtpsReaderLocator::new(locator2, false);
    //         rtps_stateless_writer_impl.reader_locator_add(a_locator1);
    //         rtps_stateless_writer_impl.reader_locator_add(a_locator2);

    //         assert_eq!(rtps_stateless_writer_impl.reader_locators.len(), 2);
    //     }

    //     #[test]
    //     fn reader_locator_remove() {
    //         let mut rtps_stateless_writer_impl: RtpsStatelessWriterImpl<MockHistoryCache> =
    //             RtpsStatelessWriterImpl::new(
    //                 GUID_UNKNOWN,
    //                 rust_rtps_pim::structure::types::TopicKind::WithKey,
    //                 rust_rtps_pim::structure::types::ReliabilityKind::BestEffort,
    //                 vec![],
    //                 vec![],
    //                 true,
    //                 DURATION_ZERO,
    //                 DURATION_ZERO,
    //                 DURATION_ZERO,
    //                 None,
    //             );

    //         let locator1 = Locator::new(1, 1, [1; 16]);
    //         let locator2 = Locator::new(2, 2, [2; 16]);
    //         let a_locator1 = RtpsReaderLocator::new(locator1, false);
    //         let a_locator2 = RtpsReaderLocator::new(locator2, false);
    //         rtps_stateless_writer_impl.reader_locator_add(a_locator1);
    //         rtps_stateless_writer_impl.reader_locator_add(a_locator2);

    //         rtps_stateless_writer_impl.reader_locator_remove(&locator2);

    //         assert_eq!(rtps_stateless_writer_impl.reader_locators.len(), 1);
    //     }
}
