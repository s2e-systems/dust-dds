use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, TopicQos},
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        publisher::{Publisher, PublisherDataWriterFactory},
    },
    return_type::{DDSError, DDSResult},
};

use crate::{
    dds_type::{DdsSerialize, DdsType},
    utils::shared_object::{
        rtps_shared_read_lock, rtps_shared_write_lock, rtps_weak_upgrade, RtpsWeak,
    },
};

use super::{
    data_writer_proxy::DataWriterProxy, publisher_impl::PublisherImpl, topic_proxy::TopicProxy,
};

pub struct PublisherProxy<'p> {
    participant: &'p dyn DomainParticipant,
    publisher_impl: RtpsWeak<PublisherImpl>,
}

impl<'p> PublisherProxy<'p> {
    pub fn new(
        participant: &'p dyn DomainParticipant,
        publisher_impl: RtpsWeak<PublisherImpl>,
    ) -> Self {
        Self {
            participant,
            publisher_impl,
        }
    }
}

impl AsRef<RtpsWeak<PublisherImpl>> for PublisherProxy<'_> {
    fn as_ref(&self) -> &RtpsWeak<PublisherImpl> {
        &self.publisher_impl
    }
}

impl<'dw, Foo> PublisherDataWriterFactory<'dw, Foo> for PublisherProxy<'_>
where
    Foo: DdsType + DdsSerialize + Send + 'static,
{
    type TopicType = TopicProxy<'dw, Foo>;
    type DataWriterType = DataWriterProxy<'dw, Foo>;

    fn datawriter_factory_create_datawriter(
        &'dw self,
        a_topic: &'dw Self::TopicType,
        qos: Option<DataWriterQos>,
        a_listener: Option<&'static dyn DataWriterListener<DataType = Foo>>,
        mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        todo!()
        // let data_writer_weak =
        //     rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl).ok()?)
        //         .datawriter_factory_create_datawriter(
        //             a_topic.topic_impl(),
        //             qos,
        //             a_listener,
        //             mask,
        //         )?;

        // let datawriter = DataWriterProxy::new(self, a_topic, data_writer_weak);

        // Some(datawriter)
    }

    fn datawriter_factory_delete_datawriter(
        &self,
        a_datawriter: &Self::DataWriterType,
    ) -> DDSResult<()> {
        let a_datawriter_shared = rtps_weak_upgrade(a_datawriter.as_ref())?;
        if std::ptr::eq(a_datawriter.get_publisher(), self) {
            rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?)
                .datawriter_factory_delete_datawriter(&a_datawriter_shared)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }

    fn datawriter_factory_lookup_datawriter(
        &'dw self,
        _topic: &'dw Self::TopicType,
    ) -> Option<Self::DataWriterType> {
        todo!()
    }
}

impl Publisher for PublisherProxy<'_> {
    fn suspend_publications(&self) -> DDSResult<()> {
        // self.rtps_writer_group_impl
        //     .upgrade()?
        //     .suspend_publications()
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        // self.rtps_writer_group_impl.upgrade()?.resume_publications()
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&mut self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
        // self.rtps_writer_group_impl
        //     .upgrade()?
        //     .set_default_datawriter_qos(qos)
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos {
        // self.default_datawriter_qos.lock().unwrap().clone()
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }
}

impl Entity for PublisherProxy<'_> {
    type Qos = <PublisherImpl as Entity>::Qos;
    type Listener = <PublisherImpl as Entity>::Listener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.publisher_impl)?).set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?)
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_instance_handle()
    }
}

// #[cfg(test)]
// mod tests {
//     use rust_dds_api::{
//         domain::domain_participant_listener::DomainParticipantListener,
//         infrastructure::{entity::Entity, qos::DomainParticipantQos},
//     };
//     use rust_rtps_pim::structure::types::GUID_UNKNOWN;

//     use crate::{
//         dds_impl::topic_impl::TopicImpl, rtps_impl::rtps_group_impl::RtpsGroupImpl,
//         utils::shared_object::RtpsShared,
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
//     fn create_datawriter() {
//         let participant = MockDomainParticipant;
//         let rtps_group = RtpsGroupImpl::new(GUID_UNKNOWN);
//         let data_writer_storage_list = vec![];
//         let publisher_storage = PublisherImpl::new(
//             PublisherQos::default(),
//             rtps_group,
//             data_writer_storage_list,
//         );
//         let publisher_storage_shared = RtpsShared::new(publisher_storage);
//         let publisher = PublisherProxy::new(&participant, publisher_storage_shared.downgrade());
//         let topic_storage = TopicImpl::new(TopicQos::default());
//         let topic_storage_shared = RtpsShared::new(topic_storage);
//         let topic =
//             TopicProxy::<MockKeyedType>::new(&participant, topic_storage_shared.downgrade());

//         let datawriter = publisher.create_datawriter_gat(&topic, None, None, 0);

//         assert!(datawriter.is_some());
//     }
// }
