use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

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
    behavior::{writer::writer::{RtpsWriter, RtpsWriterOperations}, stateless_writer_behavior::StatelessWriterBehavior},
    messages::submessages::RtpsSubmessageType,
    structure::{
        types::{ChangeKind, Locator},
        RtpsHistoryCache,
    },
};

use crate::{
    dds_type::{BigEndian, DdsSerialize},
    rtps_impl::{
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
        rtps_writer_history_cache_impl::WriterHistoryCache,
    },
    utils::{message_sender::RtpsSubmessageSender, transport::RtpsSubmessageWrite},
};

pub enum RtpsWriterFlavor {
    Stateful(RtpsStatefulWriterImpl),
    Stateless(RtpsStatelessWriterImpl),
}

impl Deref for RtpsWriterFlavor {
    type Target = RtpsWriter<Vec<Locator>, WriterHistoryCache>;

    fn deref(&self) -> &Self::Target {
        match self {
            RtpsWriterFlavor::Stateful(stateful_writer) => &*stateful_writer,
            RtpsWriterFlavor::Stateless(stateless_writer) => &*stateless_writer,
        }
    }
}

impl DerefMut for RtpsWriterFlavor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            RtpsWriterFlavor::Stateful(stateful_writer) => &mut *stateful_writer,
            RtpsWriterFlavor::Stateless(stateless_writer) => &mut *stateless_writer,
        }
    }
}

pub struct DataWriterImpl {
    _qos: DataWriterQos,
    rtps_writer_impl: RtpsWriterFlavor,
}

impl DataWriterImpl {
    pub fn new(qos: DataWriterQos, rtps_writer_impl: RtpsWriterFlavor) -> Self {
        Self {
            _qos: qos,
            rtps_writer_impl,
        }
    }
}

impl<T> DataWriter<T> for DataWriterImpl
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

    fn write(&mut self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn write_w_timestamp(
        &mut self,
        data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        let mut bytes = Vec::new();
        data.serialize::<_, BigEndian>(&mut bytes)?;
        let change = self
            .rtps_writer_impl
            .new_change(ChangeKind::Alive, bytes, &[], 0);
        let writer_cache = &mut self.rtps_writer_impl.writer_cache;
        let time = rust_rtps_pim::messages::types::Time(0);
        writer_cache.set_source_timestamp(Some(time));
        writer_cache.add_change(change);
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

impl Entity for DataWriterImpl {
    type Qos = DataWriterQos;
    type Listener = &'static dyn DataWriterListener<DataPIM = ()>;

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

impl RtpsSubmessageSender for DataWriterImpl {
    fn create_submessages(&mut self) -> Vec<(Locator, Vec<RtpsSubmessageWrite<'_>>)> {
        let destined_submessages: Vec<(Locator, Vec<RtpsSubmessageWrite>)> = Vec::new();
        let destined_submessages = RefCell::new(destined_submessages);
        match &mut self.rtps_writer_impl {
            RtpsWriterFlavor::Stateful(_stateful_writer) => todo!(),
            RtpsWriterFlavor::Stateless(stateless_writer) => {
                stateless_writer.0.send_unsent_data(
                    |reader_locator, data| {
                        let mut destined_submessages_borrow = destined_submessages.borrow_mut();
                        match destined_submessages_borrow
                            .iter_mut()
                            .find(|(locator, _)| locator == &reader_locator.locator)
                        {
                            Some((_, submessages)) => {
                                submessages.push(RtpsSubmessageType::Data(data))
                            }
                            None => destined_submessages_borrow.push((
                                reader_locator.locator,
                                vec![RtpsSubmessageType::Data(data)],
                            )),
                        }
                    },
                    |reader_locator, gap| {
                        let mut destined_submessages_borrow = destined_submessages.borrow_mut();
                        match destined_submessages_borrow
                            .iter_mut()
                            .find(|(locator, _)| locator == &reader_locator.locator)
                        {
                            Some((_, submessages)) => {
                                submessages.push(RtpsSubmessageType::Gap(gap))
                            }
                            None => destined_submessages_borrow
                                .push((reader_locator.locator, vec![RtpsSubmessageType::Gap(gap)])),
                        }
                    },
                );
            }
        }

        destined_submessages.take()
    }
}

// #[cfg(test)]
// mod tests {
//     use rust_rtps_pim::{
//         behavior::writer::stateful_writer::RtpsStatefulWriterOperations,
//         structure::types::{ReliabilityKind, TopicKind, GUID_UNKNOWN},
//     };

//     use super::*;

//     #[test]
//     fn write_w_timestamp() {
//         struct MockData<'a>(&'a [u8]);

//         impl DdsSerialize for MockData<'_> {
//             fn serialize<W: std::io::Write, R: crate::dds_type::Endianness>(
//                 &self,
//                 mut writer: W,
//             ) -> DDSResult<()> {
//                 writer.write(self.0).unwrap();
//                 Ok(())
//             }
//         }

//         let guid = GUID_UNKNOWN;
//         let topic_kind = TopicKind::WithKey;
//         let reliability_level = ReliabilityKind::BestEffort;
//         let unicast_locator_list = &[];
//         let multicast_locator_list = &[];
//         let push_mode = true;
//         let heartbeat_period = rust_rtps_pim::behavior::types::Duration::new(0, 200_000_000);
//         let nack_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
//         let nack_suppression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
//         let data_max_size_serialized = None;
//         let rtps_writer = RtpsStatefulWriterOperations::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             unicast_locator_list,
//             multicast_locator_list,
//             push_mode,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//             data_max_size_serialized,
//         );
//         let mut data_writer_impl = DataWriterImpl::new(DataWriterQos::default(), rtps_writer);

//         let data_value = [0, 1, 0, 0, 7, 3];
//         data_writer_impl
//             .write_w_timestamp(
//                 MockData(&data_value),
//                 None,
//                 rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
//             )
//             .unwrap();

//         let change = data_writer_impl
//             .rtps_writer_impl
//             .writer_cache()
//             .get_change(&(1i64.into()))
//             .unwrap();

//         assert_eq!(change.data_value, &data_value);
//     }
// }
