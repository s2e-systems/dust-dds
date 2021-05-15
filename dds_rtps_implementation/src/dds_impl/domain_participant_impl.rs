use std::sync::Mutex;

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::{publisher::Publisher, publisher_listener::PublisherListener},
    return_type::{DDSError, DDSResult},
    subscription::subscriber_listener::SubscriberListener,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps_pim::structure::RTPSEntity;

use crate::{
    rtps_impl::rtps_participant_impl::RTPSParticipantImpl, utils::shared_object::RtpsShared,
};

use super::{
    publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
    writer_group_factory::WriterGroupFactory,
};

pub struct DomainParticipantImpl<PSM: rust_rtps_pim::PIM> {
    writer_group_factory: Mutex<WriterGroupFactory<PSM>>,
    rtps_participant_impl: RtpsShared<RTPSParticipantImpl<PSM>>,
}

impl<PSM: rust_rtps_pim::PIM> DomainParticipantImpl<PSM> {
    pub fn new(guid_prefix: PSM::GuidPrefix) -> Self {
        Self {
            writer_group_factory: Mutex::new(WriterGroupFactory::new(guid_prefix)),
            rtps_participant_impl: RtpsShared::new(RTPSParticipantImpl::new(guid_prefix)),
        }
    }
}

impl<'p, PSM: rust_rtps_pim::PIM> rust_dds_api::domain::domain_participant::PublisherFactory<'p>
    for DomainParticipantImpl<PSM>
{
    type PublisherType = PublisherImpl<'p, PSM>;
    fn create_publisher(
        &'p self,
        qos: Option<PublisherQos>,
        a_listener: Option<&'static dyn PublisherListener>,
        mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let writer_group = self
            .writer_group_factory
            .lock()
            .unwrap()
            .create_writer_group(qos, a_listener, mask)
            .ok()?;
        let writer_group_shared = RtpsShared::new(writer_group);
        self.rtps_participant_impl
            .lock()
            .add_writer_group(writer_group_shared.clone());
        Some(PublisherImpl::new(self, &writer_group_shared))
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        if std::ptr::eq(a_publisher.get_participant(), self) {
            self.rtps_participant_impl
                .lock()
                .delete_writer_group(a_publisher.get_instance_handle()?)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }
}

impl<'s, PSM: rust_rtps_pim::PIM> rust_dds_api::domain::domain_participant::SubscriberFactory<'s>
    for DomainParticipantImpl<PSM>
{
    type SubscriberType = SubscriberImpl<'s, PSM>;

    fn create_subscriber(
        &'s self,
        _qos: Option<SubscriberQos>,
        _a_listener: Option<&'static dyn SubscriberListener>,
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

    fn get_builtin_subscriber(&'s self) -> Self::SubscriberType {
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

impl<'t, T: 'static, PSM: rust_rtps_pim::PIM>
    rust_dds_api::domain::domain_participant::TopicFactory<'t, T> for DomainParticipantImpl<PSM>
{
    type TopicType = TopicImpl<'t, T, PSM>;

    fn create_topic(
        &'t self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<&'static dyn TopicListener<DataType = T>>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn delete_topic(&self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn find_topic(&self, _topic_name: &str, _timeout: Duration) -> Option<Self::TopicType> {
        todo!()
    }
}

impl<PSM: rust_rtps_pim::PIM> rust_dds_api::domain::domain_participant::DomainParticipant
    for DomainParticipantImpl<PSM>
{
    fn lookup_topicdescription<'t, T>(
        &'t self,
        _name: &'t str,
    ) -> Option<&'t (dyn TopicDescription<T> + 't)> {
        todo!()
    }

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

    fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
        self.writer_group_factory
            .lock()
            .unwrap()
            .set_default_qos(qos);
        Ok(())
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        self.writer_group_factory.lock().unwrap().get_default_qos()
    }

    fn set_default_subscriber_qos(&self, _qos: Option<SubscriberQos>) -> DDSResult<()> {
        // *self.default_subscriber_qos.lock().unwrap() = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        // self.default_subscriber_qos.lock().unwrap().clone()
        todo!()
    }

    fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        let topic_qos = qos.unwrap_or_default();
        topic_qos.is_consistent()?;
        // *self.default_topic_qos.lock().unwrap() = topic_qos;
        Ok(())
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        // self.default_topic_qos.lock().unwrap().clone()
        todo!()
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

impl<PSM: rust_rtps_pim::PIM> Entity for DomainParticipantImpl<PSM> {
    type Qos = DomainParticipantQos;
    type Listener = &'static dyn DomainParticipantListener;

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
        let rtps_participant = self.rtps_participant_impl.clone();
        std::thread::spawn(move || {
            loop {
                if let Some(_rtps_participant) = rtps_participant.try_lock() {
                    // rtps_participant.send_data();
                    // rtps_participant.receive_data();
                    // rtps_participant.run_callbacks();
                }
            }
        });
        Ok(())
        // self.0.lock().unwrap().enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        Ok(crate::utils::instance_handle_from_guid(
            &self.rtps_participant_impl.lock().guid(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::domain::domain_participant::DomainParticipant;
    use rust_rtps_udp_psm::RtpsUdpPsm;

    use super::*;

    struct MockDDSType;

    #[test]
    fn set_default_publisher_qos_some_value() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new([1; 12]);
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();
        assert!(domain_participant_impl.get_default_publisher_qos() == qos);
    }

    #[test]
    fn set_default_publisher_qos_none() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new([1; 12]);
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl
            .set_default_publisher_qos(None)
            .unwrap();
        assert!(domain_participant_impl.get_default_publisher_qos() == PublisherQos::default());
    }

    // #[test]
    // fn set_default_subscriber_qos_some_value() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = SubscriberQos::default();
    //     qos.group_data.value = &[1, 2, 3, 4];
    //     domain_participant_impl
    //         .set_default_subscriber_qos(Some(qos.clone()))
    //         .unwrap();
    //     assert!(
    //         *domain_participant_impl
    //             .default_subscriber_qos
    //             .lock()
    //             .unwrap()
    //             == qos
    //     );
    // }

    // #[test]
    // fn set_default_subscriber_qos_none() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = SubscriberQos::default();
    //     qos.group_data.value = &[1, 2, 3, 4];
    //     domain_participant_impl
    //         .set_default_subscriber_qos(Some(qos.clone()))
    //         .unwrap();

    //     domain_participant_impl
    //         .set_default_subscriber_qos(None)
    //         .unwrap();
    //     assert!(
    //         *domain_participant_impl
    //             .default_subscriber_qos
    //             .lock()
    //             .unwrap()
    //             == SubscriberQos::default()
    //     );
    // }

    // #[test]
    // fn get_default_subscriber_qos() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = SubscriberQos::default();
    //     qos.group_data.value = &[1, 2, 3, 4];
    //     domain_participant_impl
    //         .set_default_subscriber_qos(Some(qos.clone()))
    //         .unwrap();
    //     assert!(domain_participant_impl.get_default_subscriber_qos() == qos);
    // }

    // #[test]
    // fn set_default_topic_qos_some_value() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = TopicQos::default();
    //     qos.topic_data.value = &[1, 2, 3, 4];
    //     domain_participant_impl
    //         .set_default_topic_qos(Some(qos.clone()))
    //         .unwrap();
    //     assert!(*domain_participant_impl.default_topic_qos.lock().unwrap() == qos);
    // }

    // #[test]
    // fn set_default_topic_qos_inconsistent() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = TopicQos::default();
    //     qos.resource_limits.max_samples_per_instance = 2;
    //     qos.resource_limits.max_samples = 1;
    //     let set_default_topic_qos_result =
    //         domain_participant_impl.set_default_topic_qos(Some(qos.clone()));
    //     assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
    // }

    // #[test]
    // fn set_default_topic_qos_none() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = TopicQos::default();
    //     qos.topic_data.value = &[1, 2, 3, 4];
    //     domain_participant_impl
    //         .set_default_topic_qos(Some(qos.clone()))
    //         .unwrap();

    //     domain_participant_impl.set_default_topic_qos(None).unwrap();
    //     assert!(*domain_participant_impl.default_topic_qos.lock().unwrap() == TopicQos::default());
    // }

    // #[test]
    // fn get_default_topic_qos() {
    //     let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
    //         DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
    //     let mut qos = TopicQos::default();
    //     qos.topic_data.value = &[1, 2, 3, 4];
    //     domain_participant_impl
    //         .set_default_topic_qos(Some(qos.clone()))
    //         .unwrap();
    //     assert!(domain_participant_impl.get_default_topic_qos() == qos);
    // }

    #[test]
    fn create_publisher() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new([1; 12]);
        let publisher = domain_participant_impl.create_publisher(None, None, 0);

        assert!(publisher.is_some())
    }

    #[test]
    fn create_topic() {
        let domain_participant_impl: DomainParticipantImpl<RtpsUdpPsm> =
            DomainParticipantImpl::new([1; 12]);
        let topic =
            domain_participant_impl.create_topic::<MockDDSType>("topic_name", None, None, 0);
        assert!(topic.is_some());
    }
}
