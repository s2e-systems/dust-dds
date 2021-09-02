use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::DDSResult,
    subscription::{
        data_reader::AnyDataReader, data_reader_listener::DataReaderListener,
        subscriber_listener::SubscriberListener,
    },
};

use crate::{dds_type::DDSType, utils::shared_object::RtpsWeak};

use super::{
    data_reader_impl::DataReaderImpl, data_reader_proxy::DataReaderProxy,
    subscriber_impl::SubscriberImpl, topic_impl::TopicImpl, topic_proxy::TopicProxy,
};

pub struct SubscriberProxy<'s> {
    participant: &'s dyn DomainParticipant,
    subscriber_storage: RtpsWeak<SubscriberImpl>,
}

impl<'s> SubscriberProxy<'s> {
    pub fn new(
        participant: &'s dyn DomainParticipant,
        subscriber_storage: RtpsWeak<SubscriberImpl>,
    ) -> Self {
        Self {
            participant,
            subscriber_storage,
        }
    }

    /// Get a reference to the subscriber impl's subscriber storage.
    pub(crate) fn subscriber_storage(&self) -> &RtpsWeak<SubscriberImpl> {
        &self.subscriber_storage
    }
}

impl<'dr, 's: 'dr, 't: 'dr, T: DDSType + 'static>
    rust_dds_api::subscription::subscriber::DataReaderGAT<'dr, 't, T> for SubscriberProxy<'s>
where
    T: for<'de> serde::Deserialize<'de>,
{
    type TopicType = TopicProxy<'t, T, TopicImpl>;
    type DataReaderType = DataReaderProxy<'dr, T, DataReaderImpl>;

    fn create_datareader_gat(
        &'dr self,
        a_topic: &'dr Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<&'static dyn DataReaderListener<DataPIM = T>>,
        mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        todo!()
        // let reader_storage_weak = self
        //     .subscriber_storage
        //     .upgrade()
        //     .ok()?
        //     .lock()
        //     .create_datareader((), qos, a_listener, mask)?;
        // let data_reader = DataReaderProxy::new(self, a_topic, reader_storage_weak);
        // Some(data_reader)
    }

    fn delete_datareader_gat(&self, _a_datareader: &Self::DataReaderType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datareader_gat<'a>(
        &'a self,
        _topic: &'a Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        todo!()
    }
}

impl<'s> rust_dds_api::subscription::subscriber::Subscriber for SubscriberProxy<'s> {
    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    /// This operation returns the DomainParticipant to which the Subscriber belongs.
    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }
}

impl<'s> Entity for SubscriberProxy<'s> {
    type Qos = SubscriberQos;
    type Listener = &'static dyn SubscriberListener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.subscriber_storage.upgrade()?.lock().set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // Ok(self.subscriber_storage.upgrade()?.lock().get_qos().clone())
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

// #[cfg(test)]
// mod tests {
//     use rust_dds_api::{
//         domain::domain_participant_listener::DomainParticipantListener,
//         infrastructure::qos::{DomainParticipantQos, PublisherQos},
//         subscription::subscriber::Subscriber,
//     };
//     use rust_rtps_pim::structure::types::GUID_UNKNOWN;

//     use crate::{
//         dds_impl::topic_impl::TopicImpl, dds_type::DDSType,
//         rtps_impl::rtps_group_impl::RtpsGroupImpl, utils::shared_object::RtpsShared,
//     };

//     use super::*;

//     #[derive(serde::Serialize, serde::Deserialize)]
//     struct MockKeyedType;

//     impl DDSType for MockKeyedType {
//         fn type_name() -> &'static str {
//             todo!()
//         }

//         fn has_key() -> bool {
//             true
//         }
//     }

//     struct MockDomainParticipant;

//     impl DomainParticipant for MockDomainParticipant {
//         fn lookup_topicdescription<'t, T>(
//             &'t self,
//             _name: &'t str,
//         ) -> Option<&'t dyn rust_dds_api::topic::topic_description::TopicDescription<T>>
//         where
//             Self: Sized,
//         {
//             todo!()
//         }

//         fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
//             todo!()
//         }

//         fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
//             todo!()
//         }

//         fn ignore_publication(&self, _handle: InstanceHandle) -> DDSResult<()> {
//             todo!()
//         }

//         fn ignore_subscription(&self, _handle: InstanceHandle) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_domain_id(&self) -> rust_dds_api::dcps_psm::DomainId {
//             todo!()
//         }

//         fn delete_contained_entities(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn assert_liveliness(&self) -> DDSResult<()> {
//             todo!()
//         }

//         fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_default_publisher_qos(&self) -> PublisherQos {
//             todo!()
//         }

//         fn set_default_subscriber_qos(
//             &self,
//             _qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
//         ) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_default_subscriber_qos(&self) -> rust_dds_api::infrastructure::qos::SubscriberQos {
//             todo!()
//         }

//         fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_default_topic_qos(&self) -> TopicQos {
//             todo!()
//         }

//         fn get_discovered_participants(
//             &self,
//             _participant_handles: &mut [InstanceHandle],
//         ) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_discovered_participant_data(
//             &self,
//             _participant_data: rust_dds_api::builtin_topics::ParticipantBuiltinTopicData,
//             _participant_handle: InstanceHandle,
//         ) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
//             todo!()
//         }

//         fn get_discovered_topic_data(
//             &self,
//             _topic_data: rust_dds_api::builtin_topics::TopicBuiltinTopicData,
//             _topic_handle: InstanceHandle,
//         ) -> DDSResult<()> {
//             todo!()
//         }

//         fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
//             todo!()
//         }

//         fn get_current_time(&self) -> DDSResult<rust_dds_api::dcps_psm::Time> {
//             todo!()
//         }
//     }

//     impl Entity for MockDomainParticipant {
//         type Qos = DomainParticipantQos;
//         type Listener = &'static dyn DomainParticipantListener;

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

//         fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
//             todo!()
//         }

//         fn get_status_changes(&self) -> DDSResult<StatusMask> {
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
//     fn create_datareader() {
//         let participant = MockDomainParticipant;
//         let rtps_group = RtpsGroupImpl::new(GUID_UNKNOWN);
//         let data_reader_storage_list = vec![];
//         let subscriber_storage = SubscriberImpl::new(
//             SubscriberQos::default(),
//             rtps_group,
//             data_reader_storage_list,
//         );
//         let subscriber_storage_shared = RtpsShared::new(subscriber_storage);
//         let subscriber = SubscriberProxy::new(&participant, subscriber_storage_shared.downgrade());
//         let topic_storage = TopicImpl::new(TopicQos::default());
//         let topic_storage_shared = RtpsShared::new(topic_storage);
//         let topic =
//             TopicProxy::<MockKeyedType>::new(&participant, topic_storage_shared.downgrade());

//         let datareader = subscriber.create_datareader(&topic, None, None, 0);

//         assert!(datareader.is_some());
//     }
// }
