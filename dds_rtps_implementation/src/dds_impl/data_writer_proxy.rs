use crate::{dds_type::DDSType, utils::shared_object::RtpsWeak};
use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask,
    },
    infrastructure::{entity::StatusCondition, qos::DataWriterQos},
    publication::{data_writer_listener::DataWriterListener, publisher::Publisher},
    return_type::DDSResult,
    topic::topic::Topic,
};

use super::data_writer_impl::DataWriterImpl;

pub struct DataWriterProxy<'dw, T: 'static> {
    _publisher: &'dw dyn Publisher,
    _topic: &'dw dyn Topic<T>,
    data_writer_storage: RtpsWeak<DataWriterImpl>,
}

impl<'dw, T: 'static> DataWriterProxy<'dw, T> {
    pub fn new(
        publisher: &'dw dyn Publisher,
        topic: &'dw dyn Topic<T>,
        data_writer_storage: RtpsWeak<DataWriterImpl>,
    ) -> Self {
        Self {
            _publisher: publisher,
            _topic: topic,
            data_writer_storage,
        }
    }
}

impl<'dw, T: DDSType + 'static> rust_dds_api::publication::data_writer::DataWriter<T>
    for DataWriterProxy<'dw, T>
{
    fn register_instance(&self, _instance: T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
        // let timestamp = self.parent.0.parent.get_current_time()?;
        // self.register_instance_w_timestamp(instance, timestamp)
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: T,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        // let writer = self
        //     .rtps_writer_impl
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?;
        // let writer_guard = writer.lock().unwrap();
        // let _c = writer_guard.writer_cache();
        todo!()
    }

    fn unregister_instance(&self, _instance: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn unregister_instance_w_timestamp(
        &self,
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

    fn write(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn write_w_timestamp(
        &self,
        data: T,
        handle: Option<InstanceHandle>,
        timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
        // let writer_storage = self.data_writer_storage.upgrade()?;
        // let mut writer_storage_lock = writer_storage.lock();
        // writer_storage_lock.write_w_timestamp(data, handle, timestamp)
    }

    fn dispose(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
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

    fn get_topic(&self) -> &dyn Topic<T> {
        todo!()
    }

    fn get_publisher(&self) -> &dyn Publisher {
        todo!()
    }
}

impl<'dw, T: 'static> rust_dds_api::infrastructure::entity::Entity for DataWriterProxy<'dw, T> {
    type Qos = DataWriterQos;
    type Listener = &'static dyn DataWriterListener<DataPIM = T>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.data_writer_storage.upgrade()?.lock().set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // Ok(self.data_writer_storage.upgrade()?.lock().get_qos().clone())
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<'dw, T: 'static> rust_dds_api::publication::data_writer::AnyDataWriter
    for DataWriterProxy<'dw, T>
{
}

// #[cfg(test)]
// mod tests {
//     use std::marker::PhantomData;

//     use rust_dds_api::{
//         infrastructure::{
//             entity::Entity,
//             qos::{PublisherQos, TopicQos},
//         },
//         publication::{data_writer::DataWriter, publisher_listener::PublisherListener},
//         topic::{topic_description::TopicDescription, topic_listener::TopicListener},
//     };
//     use rust_rtps_pim::{
//         behavior::writer::{stateful_writer::RtpsStatefulWriterOperations, writer::RtpsWriter},
//         structure::{
//             types::{ReliabilityKind, TopicKind, GUID_UNKNOWN},
//             RtpsHistoryCache,
//         },
//     };

//     use crate::{dds_type::DDSType, utils::shared_object::RtpsShared};

//     use super::*;

//     #[derive(serde::Serialize, serde::Deserialize)]
//     struct MockData(u8, u8);

//     impl DDSType for MockData {
//         fn type_name() -> &'static str {
//             todo!()
//         }

//         fn has_key() -> bool {
//             todo!()
//         }
//     }

//     struct MockTopic<T>(PhantomData<T>);

//     impl<T: 'static> Topic<T> for MockTopic<T> {
//         fn get_inconsistent_topic_status(
//             &self,
//             _status: &mut rust_dds_api::dcps_psm::InconsistentTopicStatus,
//         ) -> DDSResult<()> {
//             todo!()
//         }
//     }

//     impl<T: 'static> TopicDescription<T> for MockTopic<T> {
//         fn get_participant(
//             &self,
//         ) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
//             todo!()
//         }

//         fn get_type_name(&self) -> DDSResult<&'static str> {
//             todo!()
//         }

//         fn get_name(&self) -> DDSResult<&str> {
//             todo!()
//         }
//     }

//     impl<T: 'static> Entity for MockTopic<T> {
//         type Qos = TopicQos;
//         type Listener = &'static dyn TopicListener<DataPIM = T>;

//         fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_qos(&self) -> DDSResult<Self::Qos> {
//             todo!()
//         }

//         fn set_listener(
//             &self,
//             _a_listener: Option<Self::Listener>,
//             _mask: StatusMask,
//         ) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
//             todo!()
//         }

//         fn get_statuscondition(&self) -> StatusCondition {
//             todo!()
//         }

//         fn get_status_changes(&self) -> StatusMask {
//             todo!()
//         }

//         fn enable(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
//             todo!()
//         }
//     }

//     struct MockPublisher;

//     impl Publisher for MockPublisher {
//         fn suspend_publications(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn resume_publications(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn begin_coherent_changes(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn end_coherent_changes(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_participant(
//             &self,
//         ) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
//             todo!()
//         }

//         fn delete_contained_entities(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_default_datawriter_qos(&self) -> DataWriterQos {
//             todo!()
//         }

//         fn copy_from_topic_qos(
//             &self,
//             _a_datawriter_qos: &mut DataWriterQos,
//             _a_topic_qos: &TopicQos,
//         ) -> DDSResult<()> {
//             todo!()
//         }
//     }

//     impl Entity for MockPublisher {
//         type Qos = PublisherQos;
//         type Listener = &'static dyn PublisherListener;

//         fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_qos(&self) -> DDSResult<Self::Qos> {
//             todo!()
//         }

//         fn set_listener(
//             &self,
//             _a_listener: Option<Self::Listener>,
//             _mask: StatusMask,
//         ) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
//             todo!()
//         }

//         fn get_statuscondition(&self) -> StatusCondition {
//             todo!()
//         }

//         fn get_status_changes(&self) -> StatusMask {
//             todo!()
//         }

//         fn enable(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
//             todo!()
//         }
//     }

//     #[test]
//     fn write_w_timestamp() {
//         let topic = MockTopic(PhantomData);
//         let publisher = MockPublisher;
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
//         let data_writer_storage = DataWriterImpl::new(DataWriterQos::default(), rtps_writer);
//         let data_writer_storage_shared = RtpsShared::new(data_writer_storage);
//         let data_writer =
//             DataWriterProxy::new(&publisher, &topic, data_writer_storage_shared.downgrade());

//         data_writer
//             .write_w_timestamp(
//                 MockData(7, 3),
//                 None,
//                 rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
//             )
//             .unwrap();

//         let data_writer_storage_lock = data_writer_storage_shared.lock();
//         let change = data_writer_storage_lock
//             .rtps_data_writer()
//             .writer_cache()
//             .get_change(&(1i64.into()))
//             .unwrap();

//         assert_eq!(change.data_value(), &[0, 1, 0, 0, 7, 3]);
//     }
// }
