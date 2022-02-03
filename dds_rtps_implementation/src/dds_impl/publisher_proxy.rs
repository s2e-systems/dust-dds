use std::sync::atomic::AtomicU8;

use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        publisher::{Publisher, PublisherDataWriterFactory},
        publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};
use rust_rtps_pim::{
    behavior::writer::writer::{RtpsWriterAttributes, RtpsWriterOperations},
    structure::history_cache::RtpsHistoryCacheOperations,
};

use crate::{
    dds_type::{DdsSerialize, DdsType},
    rtps_impl::rtps_group_impl::RtpsGroupImpl,
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{RtpsShared, RtpsWeak},
    },
};

use super::{
    data_writer_proxy::DataWriterAttributes,
    data_writer_proxy::DataWriterProxy,
    domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
    topic_proxy::TopicProxy,
};

pub struct PublisherAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub _qos: PublisherQos,
    pub rtps_group: RtpsGroupImpl,
    pub data_writer_list: Vec<RtpsShared<DataWriterAttributes<Rtps>>>,
    pub user_defined_data_writer_counter: AtomicU8,
    pub default_datawriter_qos: DataWriterQos,
    pub sedp_builtin_publications_announcer: Option<RtpsShared<DataWriterAttributes<Rtps>>>,
    pub parent_participant: RtpsWeak<DomainParticipantAttributes<Rtps>>,
}

impl<Rtps> PublisherAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        sedp_builtin_publications_announcer: Option<RtpsShared<DataWriterAttributes<Rtps>>>,
        parent_participant: RtpsWeak<DomainParticipantAttributes<Rtps>>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            data_writer_list: Vec::new(),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            sedp_builtin_publications_announcer,
            parent_participant,
        }
    }
}

#[derive(Clone)]
pub struct PublisherProxy<Rtps>(pub(crate) RtpsWeak<PublisherAttributes<Rtps>>)
where
    Rtps: RtpsStructure;

impl<Rtps> PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(publisher_impl: RtpsWeak<PublisherAttributes<Rtps>>) -> Self {
        Self(publisher_impl)
    }
}

impl<Foo, Rtps> PublisherDataWriterFactory<Foo> for PublisherProxy<Rtps>
where
    Foo: DdsType + DdsSerialize + Send + Sync + 'static,
    Rtps: RtpsStructure,
    Rtps::StatelessWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    Rtps::StatefulWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    <Rtps::StatelessWriter as RtpsWriterAttributes>::WriterHistoryCacheType:
        RtpsHistoryCacheOperations<
            CacheChangeType = <Rtps::StatelessWriter as RtpsWriterOperations>::CacheChangeType,
        >,
    <Rtps::StatefulWriter as RtpsWriterAttributes>::WriterHistoryCacheType:
        RtpsHistoryCacheOperations<
            CacheChangeType = <Rtps::StatefulWriter as RtpsWriterOperations>::CacheChangeType,
        >,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataWriterType = DataWriterProxy<Foo, Rtps>;

    fn datawriter_factory_create_datawriter(
        &self,
        _a_topic: &Self::TopicType,
        _qos: Option<DataWriterQos>,
        _a_listener: Option<&'static dyn DataWriterListener>,
        _mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        // let publisher_shared = rtps_weak_upgrade(&self.publisher_impl).ok()?;
        // let topic_shared = rtps_weak_upgrade(a_topic.as_ref()).ok()?;
        // let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
        // let user_defined_data_writer_counter = self
        //     .user_defined_data_writer_counter
        //     .fetch_add(1, atomic::Ordering::SeqCst);
        // let (entity_kind, topic_kind) = match Foo::has_key() {
        //     true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
        //     false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        // };
        // let entity_id = EntityId::new(
        //     [
        //         self.rtps_group.guid().entity_id().entity_key()[0],
        //         user_defined_data_writer_counter,
        //         0,
        //     ],
        //     entity_kind,
        // );
        // let guid = Guid::new(*self.rtps_group.guid().prefix(), entity_id);
        // let reliability_level = match qos.reliability.kind {
        //     ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        //     ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        // };
        // let unicast_locator_list = &[];
        // let multicast_locator_list = &[];
        // let push_mode = true;
        // let heartbeat_period = rust_rtps_pim::behavior::types::Duration::new(0, 200_000_000);
        // let nack_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        // let nack_suppression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        // let data_max_size_serialized = None;
        // let rtps_writer_impl = RtpsWriter::Stateful(RtpsStatefulWriterImpl::new(
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
        // ));

        // if let Some(sedp_builtin_publications_announcer) = &self.sedp_builtin_publications_announcer
        // {
        //     let topic_shared = rtps_shared_read_lock(a_topic);
        //     let mut sedp_builtin_publications_announcer_lock =
        //         rtps_shared_write_lock(sedp_builtin_publications_announcer);
        //     let sedp_discovered_writer_data = SedpDiscoveredWriterData {
        //         writer_proxy: RtpsWriterProxy {
        //             remote_writer_guid: guid,
        //             unicast_locator_list: vec![],
        //             multicast_locator_list: vec![],
        //             data_max_size_serialized: None,
        //             remote_group_entity_id: EntityId::new([0; 3], 0),
        //         },
        //         publication_builtin_topic_data: PublicationBuiltinTopicData {
        //             key: BuiltInTopicKey { value: guid.into() },
        //             participant_key: BuiltInTopicKey { value: [1; 16] },
        //             topic_name: topic_shared.topic_name.clone(),
        //             type_name: Foo::type_name().to_string(),
        //             durability: DurabilityQosPolicy::default(),
        //             durability_service: DurabilityServiceQosPolicy::default(),
        //             deadline: DeadlineQosPolicy::default(),
        //             latency_budget: LatencyBudgetQosPolicy::default(),
        //             liveliness: LivelinessQosPolicy::default(),
        //             reliability: ReliabilityQosPolicy {
        //                 kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
        //                 max_blocking_time: Duration::new(3, 0),
        //             },
        //             lifespan: LifespanQosPolicy::default(),
        //             user_data: UserDataQosPolicy::default(),
        //             ownership: OwnershipQosPolicy::default(),
        //             ownership_strength: OwnershipStrengthQosPolicy::default(),
        //             destination_order: DestinationOrderQosPolicy::default(),
        //             presentation: PresentationQosPolicy::default(),
        //             partition: PartitionQosPolicy::default(),
        //             topic_data: TopicDataQosPolicy::default(),
        //             group_data: GroupDataQosPolicy::default(),
        //         },
        //     };
        //     sedp_builtin_publications_announcer_lock
        //         .write_w_timestamp(
        //             &sedp_discovered_writer_data,
        //             None,
        //             Time { sec: 0, nanosec: 0 },
        //         )
        //         .unwrap();
        // }
        // let data_writer_impl = DataWriterImpl::new(qos, rtps_writer_impl);
        // let data_writer_impl_shared = rtps_shared_new(data_writer_impl);
        // self.data_writer_list
        //     .lock()
        //     .unwrap()
        //     .push(data_writer_impl_shared.clone());
        // // Some(data_writer_impl_shared)
        // let data_writer_weak = rtps_shared_downgrade(&data_writer_shared);
        // let datawriter = DataWriterProxy::new(self.clone(), a_topic.clone(), data_writer_weak);
        // Some(datawriter)
        todo!()
    }

    fn datawriter_factory_delete_datawriter(
        &self,
        a_datawriter: &Self::DataWriterType,
    ) -> DDSResult<()> {
        let _a_datawriter_shared = a_datawriter.as_ref().upgrade()?;
        if std::ptr::eq(&a_datawriter.get_publisher()?, self) {
            todo!()
            // PublisherDataWriterFactory::<Foo>::datawriter_factory_delete_datawriter(
            //     &*rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?),
            //     &a_datawriter_shared,
            // )
        } else {
            Err(DDSError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }

    fn datawriter_factory_lookup_datawriter(
        &self,
        _topic: &Self::TopicType,
    ) -> Option<Self::DataWriterType> {
        // let data_writer_impl_list_lock = self.data_writer_list.lock().unwrap();
        // let found_data_writer = data_writer_impl_list_lock.iter().cloned().find(|_x| true);

        // if let Some(found_data_writer) = found_data_writer {
        // return Some(found_data_writer);
        // };
        // None
        todo!()
    }
}

impl<Rtps> Publisher for PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
    Rtps::StatelessWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    Rtps::StatefulWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    <Rtps::StatelessWriter as RtpsWriterAttributes>::WriterHistoryCacheType:
        RtpsHistoryCacheOperations<
            CacheChangeType = <Rtps::StatelessWriter as RtpsWriterOperations>::CacheChangeType,
        >,
    <Rtps::StatefulWriter as RtpsWriterAttributes>::WriterHistoryCacheType:
        RtpsHistoryCacheOperations<
            CacheChangeType = <Rtps::StatefulWriter as RtpsWriterOperations>::CacheChangeType,
        >,
{
    type DomainParticipant = DomainParticipantProxy<Rtps>;

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

    fn get_participant(&self) -> DDSResult<Self::DomainParticipant> {
        let publisher_attributes = self.0.upgrade()?;
        let publisher_attributes_lock = publisher_attributes.read_lock();
        Ok(DomainParticipantProxy::new(
            publisher_attributes_lock.parent_participant.clone(),
        ))
    }
}

impl<Rtps> Entity for PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = PublisherQos;
    type Listener = &'static dyn PublisherListener;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.publisher_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?)
        //     .set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_instance_handle()
        todo!()
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
