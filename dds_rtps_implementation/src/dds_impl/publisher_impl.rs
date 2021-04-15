use std::sync::{Arc, Mutex, Weak};

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    dds_type::DDSType,
    infrastructure::{
        entity::StatusCondition,
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    return_type::DDSResult,
};
use rust_rtps_pim::behavior::RTPSWriter;
use rust_rtps_udp_psm::RtpsUdpPsm;

use crate::rtps_impl::{
    rtps_group_impl::RTPSGroupImpl, rtps_history_cache_impl::RTPSHistoryCacheImpl,
    rtps_stateful_writer_impl::RTPSStatefulWriterImpl,
};

use super::{
    data_writer_impl::DataWriterImpl, domain_participant_impl::DomainParticipantImpl,
    topic_impl::TopicImpl,
};

pub struct PublisherImpl<'a> {
    parent: &'a DomainParticipantImpl,
    impl_ref: Weak<Mutex<RTPSGroupImpl>>,
}

impl<'a> PublisherImpl<'a> {
    pub fn new(parent: &'a DomainParticipantImpl, impl_ref: Weak<Mutex<RTPSGroupImpl>>) -> Self {
        Self { parent, impl_ref }
    }
}

impl<'a, T: DDSType> rust_dds_api::domain::domain_participant::TopicGAT<'a, T>
    for PublisherImpl<'a>
{
    type TopicType = TopicImpl<'a, T>;
}

impl<'a, T: DDSType> rust_dds_api::publication::publisher::DataWriterGAT<'a, T>
    for PublisherImpl<'a>
{
    type DataWriterType = DataWriterImpl<'a, T>;
}

impl<'a> rust_dds_api::domain::domain_participant::DomainParticipantChild<'a>
    for PublisherImpl<'a>
{
    type DomainParticipantType = DomainParticipantImpl;
}

impl<'a> rust_dds_api::publication::publisher::Publisher<'a> for PublisherImpl<'a> {
    fn create_datawriter<T: DDSType>(
        &'a self,
        _a_topic: &'a <Self as rust_dds_api::domain::domain_participant::TopicGAT<'a, T>>::TopicType,
        _qos: Option<DataWriterQos>,
        _a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        _mask: StatusMask,
    ) -> Option<<Self as rust_dds_api::publication::publisher::DataWriterGAT<'a, T>>::DataWriterType>
    {
        let rtps_writer = Arc::new(Mutex::new(RTPSStatefulWriterImpl {}));
        let rtps_writer_dyn: Arc<Mutex<dyn RTPSWriter<RtpsUdpPsm, RTPSHistoryCacheImpl>>> =
            rtps_writer.clone();
        let data_writer = DataWriterImpl::new(self, Arc::downgrade(&rtps_writer_dyn));
        self.impl_ref
            .upgrade()?
            .lock()
            .unwrap()
            .stateful_writer_list
            .push(rtps_writer);
        Some(data_writer)
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        _a_datawriter: &<Self as rust_dds_api::publication::publisher::DataWriterGAT<'a, T>>::DataWriterType,
    ) -> DDSResult<()> {
        todo!()
        // if std::ptr::eq(a_datawriter.parent.0, self) {
        //     self.impl_ref
        //         .upgrade()
        //         .ok_or(DDSError::AlreadyDeleted)?
        //         .lock()
        //         .unwrap()
        //         .delete_datawriter(&a_datawriter.impl_ref)
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Publisher can only be deleted from its parent participant",
        //     ))
        // }
    }

    fn lookup_datawriter<T: DDSType>(
        &self,
        _topic: &<Self as rust_dds_api::domain::domain_participant::TopicGAT<'a, T>>::TopicType,
    ) -> Option<<Self as rust_dds_api::publication::publisher::DataWriterGAT<'a, T>>::DataWriterType>
    {
        todo!()
    }

    fn suspend_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as rust_dds_api::domain::domain_participant::DomainParticipantChild<'a>>::DomainParticipantType{
        self.parent
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
        // self.publisher_ref.set_default_datawriter_qos(qos)
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
        // self.publisher_ref.get_default_datawriter_qos()
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a> rust_dds_api::infrastructure::entity::Entity for PublisherImpl<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener + 'a>;

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
        // self.publisher_ref.get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        domain::domain_participant::DomainParticipant, publication::publisher::Publisher,
    };

    use crate::rtps_impl::rtps_participant_impl::RTPSParticipantImpl;

    use super::*;

    struct MockData;

    impl DDSType for MockData {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            todo!()
        }

        fn key(&self) -> Vec<u8> {
            todo!()
        }

        fn serialize(&self) -> Vec<u8> {
            todo!()
        }

        fn deserialize(_data: Vec<u8>) -> Self {
            todo!()
        }
    }

    #[test]
    fn create_datawriter() {
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new());
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let a_topic = domain_participant
            .create_topic::<MockData>("Test", None, None, 0)
            .unwrap();

        let data_writer = publisher.create_datawriter(&a_topic, None, None, 0);

        assert!(data_writer.is_some());
    }
}
