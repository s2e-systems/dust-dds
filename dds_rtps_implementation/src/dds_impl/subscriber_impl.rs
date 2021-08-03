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

use crate::{
    rtps_impl::rtps_group_impl::RtpsGroupImpl,
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{
    data_reader_impl::{DataReaderImpl, DataReaderStorage},
    topic_impl::TopicImpl,
};

pub struct SubscriberStorage {
    qos: SubscriberQos,
    rtps_group: RtpsGroupImpl,
    data_reader_storage_list: Vec<RtpsShared<DataReaderStorage>>,
}

impl SubscriberStorage {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        data_reader_storage_list: Vec<RtpsShared<DataReaderStorage>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_storage_list,
        }
    }

    /// Get a reference to the subscriber storage's readers.
    pub fn readers(&self) -> &[RtpsShared<DataReaderStorage>] {
        self.data_reader_storage_list.as_slice()
    }
}

pub struct SubscriberImpl<'s> {
    participant: &'s dyn DomainParticipant,
    subscriber_storage: RtpsWeak<SubscriberStorage>,
}

impl<'s> SubscriberImpl<'s> {
    pub fn new(
        participant: &'s dyn DomainParticipant,
        subscriber_storage: RtpsWeak<SubscriberStorage>,
    ) -> Self {
        Self {
            participant,
            subscriber_storage,
        }
    }

    /// Get a reference to the subscriber impl's subscriber storage.
    pub(crate) fn subscriber_storage(&self) -> &RtpsWeak<SubscriberStorage> {
        &self.subscriber_storage
    }
}

impl<'dr, 's: 'dr, 't: 'dr, T: 'static>
    rust_dds_api::subscription::subscriber::DataReaderFactory<'dr, 't, T> for SubscriberImpl<'s>
where
    T: for<'de> serde::Deserialize<'de>,
{
    type TopicType = TopicImpl<'t, T>;
    type DataReaderType = DataReaderImpl<'dr, T>;

    fn create_datareader(
        &'dr self,
        _a_topic: &'dr Self::TopicType,
        _qos: Option<DataReaderQos>,
        _a_listener: Option<&'static dyn DataReaderListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        todo!()
    }

    fn delete_datareader(&self, _a_datareader: &Self::DataReaderType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datareader<'a>(
        &'a self,
        _topic: &'a Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        todo!()
    }
}

impl<'s> rust_dds_api::subscription::subscriber::Subscriber for SubscriberImpl<'s> {
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

impl<'s> Entity for SubscriberImpl<'s> {
    type Qos = SubscriberQos;
    type Listener = &'static dyn SubscriberListener;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .set_qos(qos))
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .get_qos()
        //     .clone())
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
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        domain::domain_participant_listener::DomainParticipantListener,
        infrastructure::qos::{DomainParticipantQos, PublisherQos},
        subscription::subscriber::Subscriber,
    };
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    use crate::{dds_impl::topic_impl::TopicStorage, dds_type::DDSType};

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct MockKeyedType;

    impl DDSType for MockKeyedType {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            true
        }
    }

    struct MockDomainParticipant;

    impl DomainParticipant for MockDomainParticipant {
        fn lookup_topicdescription<'t, T>(
            &'t self,
            _name: &'t str,
        ) -> Option<&'t dyn rust_dds_api::topic::topic_description::TopicDescription<T>>
        where
            Self: Sized,
        {
            todo!()
        }

        fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_publication(&self, handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_subscription(&self, handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn get_domain_id(&self) -> rust_dds_api::dcps_psm::DomainId {
            todo!()
        }

        fn delete_contained_entities(&self) -> DDSResult<()> {
            todo!()
        }

        fn assert_liveliness(&self) -> DDSResult<()> {
            todo!()
        }

        fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
            todo!()
        }

        fn get_default_publisher_qos(&self) -> PublisherQos {
            todo!()
        }

        fn set_default_subscriber_qos(
            &self,
            qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_default_subscriber_qos(&self) -> rust_dds_api::infrastructure::qos::SubscriberQos {
            todo!()
        }

        fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
            todo!()
        }

        fn get_default_topic_qos(&self) -> TopicQos {
            todo!()
        }

        fn get_discovered_participants(
            &self,
            participant_handles: &mut [InstanceHandle],
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_participant_data(
            &self,
            participant_data: rust_dds_api::builtin_topics::ParticipantBuiltinTopicData,
            participant_handle: InstanceHandle,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_topics(&self, topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_topic_data(
            &self,
            topic_data: rust_dds_api::builtin_topics::TopicBuiltinTopicData,
            topic_handle: InstanceHandle,
        ) -> DDSResult<()> {
            todo!()
        }

        fn contains_entity(&self, a_handle: InstanceHandle) -> bool {
            todo!()
        }

        fn get_current_time(&self) -> DDSResult<rust_dds_api::dcps_psm::Time> {
            todo!()
        }
    }

    impl Entity for MockDomainParticipant {
        type Qos = DomainParticipantQos;
        type Listener = &'static dyn DomainParticipantListener;

        fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
            todo!()
        }

        fn get_qos(&self) -> DDSResult<Self::Qos> {
            todo!()
        }

        fn set_listener(
            &self,
            a_listener: Option<Self::Listener>,
            mask: StatusMask,
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
            todo!()
        }

        fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
            todo!()
        }
    }

    #[test]
    fn create_datareader() {
        let participant = MockDomainParticipant;
        let rtps_group = RtpsGroupImpl::new(GUID_UNKNOWN);
        let data_reader_storage_list = vec![];
        let subscriber_storage = SubscriberStorage::new(
            SubscriberQos::default(),
            rtps_group,
            data_reader_storage_list,
        );
        let subscriber_storage_shared = RtpsShared::new(subscriber_storage);
        let subscriber = SubscriberImpl::new(&participant, subscriber_storage_shared.downgrade());
        let topic_storage = TopicStorage::new(TopicQos::default());
        let topic_storage_shared = RtpsShared::new(topic_storage);
        let topic = TopicImpl::<MockKeyedType>::new(&participant, topic_storage_shared.downgrade());

        let datawriter = subscriber.create_datareader(&topic, None, None, 0);

        assert!(datawriter.is_some());
    }
}
