use std::sync::{Arc, Mutex};

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::{
        domain_participant::{
            DomainParticipant, DomainParticipantChild, PublisherFactory, SubscriberFactory,
            TopicFactory,
        },
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::publisher_listener::PublisherListener,
    return_type::{DDSError, DDSResult},
    subscription::subscriber_listener::SubscriberListener,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps_pim::structure::{types::GUID, RTPSEntity};

use crate::rtps_impl::{
    rtps_participant_impl::RTPSParticipantImpl, rtps_writer_group_impl::RTPSWriterGroupImpl,
};

use super::{
    publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
};

const ENTITYKIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;
const ENTITYKIND_USER_DEFINED_READER_GROUP: u8 = 0x09;

pub struct DomainParticipantImpl<'a, PSM: rust_rtps_pim::PIM> {
    rtps_participant_impl: Mutex<RTPSParticipantImpl<PSM>>,
    default_publisher_qos: Mutex<PublisherQos<'a>>,
    default_subscriber_qos: Mutex<SubscriberQos<'a>>,
    default_topic_qos: Mutex<TopicQos<'a>>,
    publisher_counter: Mutex<u8>,
}

impl<'a, PSM: rust_rtps_pim::PIM> DomainParticipantImpl<'a, PSM> {
    pub fn new(domain_participant_impl: RTPSParticipantImpl<PSM>) -> Self {
        Self {
            rtps_participant_impl: Mutex::new(domain_participant_impl),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            publisher_counter: Mutex::new(0),
        }
    }
}

impl<'a, PSM: rust_rtps_pim::PIM + 'a> PublisherFactory<'a> for DomainParticipantImpl<'a, PSM> {
    type PublisherType = PublisherImpl<'a, PSM>;

    fn create_publisher(
        &'a self,
        qos: Option<PublisherQos<'a>>,
        _a_listener: Option<&'a (dyn PublisherListener + 'a)>,
        _mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let _publisher_qos = qos.unwrap_or(self.get_default_publisher_qos());
        let guid_prefix = self
            .rtps_participant_impl
            .lock()
            .unwrap()
            .guid()
            .prefix()
            .clone();
        let mut publisher_counter_lock = self.publisher_counter.lock().unwrap();
        *publisher_counter_lock += 1;
        let entity_id = [
            *publisher_counter_lock,
            0,
            0,
            ENTITYKIND_USER_DEFINED_WRITER_GROUP,
        ]
        .into();
        let guid = GUID::new(guid_prefix, entity_id);
        let group = Arc::new(Mutex::new(RTPSWriterGroupImpl::new(guid)));
        let publisher = PublisherImpl::new(self, Arc::downgrade(&group));
        self.rtps_participant_impl
            .lock()
            .unwrap()
            .rtps_writer_groups
            .push(group);
        Some(publisher)
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        if std::ptr::eq(a_publisher.get_participant(), self) {
            Ok(())
            // self.0
            //     .lock()
            //     .unwrap()
            //     .delete_publisher(&a_publisher.impl_ref)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }
}

impl<'a, 'b: 'a, PSM: rust_rtps_pim::PIM + 'a> SubscriberFactory<'a>
    for DomainParticipantImpl<'b, PSM>
{
    type SubscriberType = SubscriberImpl<'a, PSM>;

    fn create_subscriber(
        &'a self,
        _qos: Option<SubscriberQos>,
        _a_listener: Option<&'a (dyn SubscriberListener + 'a)>,
        _mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        todo!()
        //         // let impl_ref = self
        //         //     .0
        //         //     .lock()
        //         //     .unwrap()
        //         //     .create_subscriber(qos, a_listener, mask)
        //         //     .ok()?;

        //         // Some(Subscriber(Node {
        //         //     parent: self,
        //         //     impl_ref,
        //         // }))
    }

    fn delete_subscriber(&self, _a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
        todo!()
        //         // if std::ptr::eq(a_subscriber.parent, self) {
        //         //     self.0
        //         //         .lock()
        //         //         .unwrap()
        //         //         .delete_subscriber(&a_subscriber.impl_ref)
        //         // } else {
        //         //     Err(DDSError::PreconditionNotMet(
        //         //         "Subscriber can only be deleted from its parent participant",
        //         //     ))
        //         // }
    }

    fn get_builtin_subscriber(&'a self) -> Self::SubscriberType {
        todo!()
        //         //     self.builtin_entities
        //         //         .subscriber_list()
        //         //         .into_iter()
        //         //         .find(|x| {
        //         //             if let Some(subscriber) = x.get().ok() {
        //         //                 subscriber.group.entity.guid.entity_id().entity_kind()
        //         //                     == ENTITY_KIND_BUILT_IN_READER_GROUP
        //         //             } else {
        //         //                 false
        //         //             }
        //         //         })
        //         // }
    }
}

impl<'a, 'b: 'a, PSM: rust_rtps_pim::PIM + 'a, T: 'a> TopicFactory<'a, T>
    for DomainParticipantImpl<'b, PSM>
{
    type TopicType = TopicImpl<'a, PSM, T>;

    fn create_topic(
        &'a self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<&'a (dyn TopicListener<DataType = T> + 'a)>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn delete_topic(&'a self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn find_topic(&self, _topic_name: &str, _timeout: Duration) -> Option<Self::TopicType> {
        todo!()
    }

    fn lookup_topicdescription(&self, _name: &str) -> Option<&'a (dyn TopicDescription<T> + 'a)> {
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::PIM + 'a>
    rust_dds_api::domain::domain_participant::DomainParticipant<'a>
    for DomainParticipantImpl<'a, PSM>
{
    fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> DomainId {
        // self.domain_id
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&self, qos: Option<PublisherQos<'a>>) -> DDSResult<()> {
        *self.default_publisher_qos.lock().unwrap() = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_publisher_qos(&self) -> PublisherQos<'a> {
        self.default_publisher_qos.lock().unwrap().clone()
    }

    fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos<'a>>) -> DDSResult<()> {
        *self.default_subscriber_qos.lock().unwrap() = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos<'a> {
        self.default_subscriber_qos.lock().unwrap().clone()
    }

    fn set_default_topic_qos(&self, qos: Option<TopicQos<'a>>) -> DDSResult<()> {
        let topic_qos = qos.unwrap_or_default();
        topic_qos.is_consistent()?;
        *self.default_topic_qos.lock().unwrap() = topic_qos;
        Ok(())
    }

    fn get_default_topic_qos(&self) -> TopicQos<'a> {
        self.default_topic_qos.lock().unwrap().clone()
    }

    fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
        todo!()
    }

    fn get_current_time(&self) -> DDSResult<Time> {
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::PIM> Entity for DomainParticipantImpl<'a, PSM> {
    type Qos = DomainParticipantQos<'a>;
    type Listener = &'a (dyn DomainParticipantListener + 'a);

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.0.lock().unwrap().set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // Ok(self.0.lock().unwrap().get_qos())
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

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // self.0.lock().unwrap().enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::domain::domain_participant::DomainParticipant;
    use rust_dds_api::return_type::DDSError;
    use rust_rtps_udp_psm::RtpsUdpPsm;

    use super::*;

    struct MockDDSType;

    #[test]
    fn set_default_publisher_qos_some_value() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();
        assert!(
            *domain_participant_impl
                .default_publisher_qos
                .lock()
                .unwrap()
                == qos
        );
    }

    #[test]
    fn set_default_publisher_qos_none() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl
            .set_default_publisher_qos(None)
            .unwrap();
        assert!(
            *domain_participant_impl
                .default_publisher_qos
                .lock()
                .unwrap()
                == PublisherQos::default()
        );
    }

    #[test]
    fn get_default_publisher_qos() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();
        assert!(domain_participant_impl.get_default_publisher_qos() == qos);
    }

    #[test]
    fn set_default_subscriber_qos_some_value() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = SubscriberQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();
        assert!(
            *domain_participant_impl
                .default_subscriber_qos
                .lock()
                .unwrap()
                == qos
        );
    }

    #[test]
    fn set_default_subscriber_qos_none() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = SubscriberQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl
            .set_default_subscriber_qos(None)
            .unwrap();
        assert!(
            *domain_participant_impl
                .default_subscriber_qos
                .lock()
                .unwrap()
                == SubscriberQos::default()
        );
    }

    #[test]
    fn get_default_subscriber_qos() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = SubscriberQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();
        assert!(domain_participant_impl.get_default_subscriber_qos() == qos);
    }

    #[test]
    fn set_default_topic_qos_some_value() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = TopicQos::default();
        qos.topic_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();
        assert!(*domain_participant_impl.default_topic_qos.lock().unwrap() == qos);
    }

    #[test]
    fn set_default_topic_qos_inconsistent() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = TopicQos::default();
        qos.resource_limits.max_samples_per_instance = 2;
        qos.resource_limits.max_samples = 1;
        let set_default_topic_qos_result =
            domain_participant_impl.set_default_topic_qos(Some(qos.clone()));
        assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
    }

    #[test]
    fn set_default_topic_qos_none() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = TopicQos::default();
        qos.topic_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl.set_default_topic_qos(None).unwrap();
        assert!(*domain_participant_impl.default_topic_qos.lock().unwrap() == TopicQos::default());
    }

    #[test]
    fn get_default_topic_qos() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let mut qos = TopicQos::default();
        qos.topic_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();
        assert!(domain_participant_impl.get_default_topic_qos() == qos);
    }

    #[test]
    fn create_publisher() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant_impl.create_publisher(None, None, 0);

        assert!(publisher.is_some())
    }

    #[test]
    fn create_topic() {
        // let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let topic =
        //     domain_participant_impl.create_topic::<MockDDSType>("topic_name", None, None, 0);
        // assert!(topic.is_some());
    }
}
