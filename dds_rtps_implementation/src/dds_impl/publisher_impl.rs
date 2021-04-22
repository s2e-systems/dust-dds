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
    return_type::{DDSError, DDSResult},
};
use rust_rtps_pim::{
    behavior::{RTPSWriter, stateless_writer::RTPSStatelessWriter},
    structure::{
        types::{Locator, GUID},
        RTPSEntity,
    },
};
use rust_rtps_udp_psm::RtpsUdpPsm;

use crate::rtps_impl::{
    rtps_stateful_writer_impl::RTPSStatefulWriterImpl,
    rtps_stateless_writer_impl::RTPSStatelessWriterImpl,
    rtps_writer_group_impl::RTPSWriterGroupImpl,
};

use super::{
    data_writer_impl::DataWriterImpl, domain_participant_impl::DomainParticipantImpl,
    topic_impl::TopicImpl,
};

const ENTITYKIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
const ENTITYKIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
const ENTITYKIND_BUILTIN_WRITER_WITH_KEY: u8 = 0xc2;
const ENTITYKIND_BUILTIN_WRITER_NO_KEY: u8 = 0xc3;

pub struct PublisherImpl<'a> {
    parent: &'a DomainParticipantImpl,
    impl_ref: Weak<Mutex<RTPSWriterGroupImpl<RtpsUdpPsm>>>,
    default_datawriter_qos: Mutex<DataWriterQos>,
    datawriter_counter: Mutex<u8>,
}

impl<'a> PublisherImpl<'a> {
    pub fn new(
        parent: &'a DomainParticipantImpl,
        impl_ref: Weak<Mutex<RTPSWriterGroupImpl<RtpsUdpPsm>>>,
    ) -> Self {
        Self {
            parent,
            impl_ref,
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            datawriter_counter: Mutex::new(0),
        }
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
        qos: Option<DataWriterQos>,
        _a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        _mask: StatusMask,
    ) -> Option<<Self as rust_dds_api::publication::publisher::DataWriterGAT<'a, T>>::DataWriterType>
    {
        let rtps_writer_group = self.impl_ref.upgrade()?;
        let mut rtps_writer_group_lock = rtps_writer_group.lock().unwrap();
        let mut datawriter_counter_lock = self.datawriter_counter.lock().unwrap();
        *datawriter_counter_lock += 1;

        let use_stateless_writer = true;

        let prefix = rtps_writer_group_lock.guid().prefix().clone();
        let parent_entityid: [u8; 4] = rtps_writer_group_lock.guid().entity_id().clone().into();

        let entity_id = [
            parent_entityid[0],
            *datawriter_counter_lock,
            0,
            ENTITYKIND_USER_DEFINED_WRITER_WITH_KEY,
        ]
        .into();
        let guid = GUID::new(prefix, entity_id);

        let datawriter_qos = qos.unwrap_or_default();
        let rtps_writer_dyn: Arc<Mutex<dyn RTPSWriter<RtpsUdpPsm>>> =
            if use_stateless_writer {
                let mut stateless_writer = RTPSStatelessWriterImpl::new(datawriter_qos, guid);
                stateless_writer.reader_locator_add(Locator::new(
                    <RtpsUdpPsm as rust_rtps_pim::structure::Types>::LOCATOR_KIND_UDPv4,
                    7400,
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
                ));
                let rtps_writer = Arc::new(Mutex::new(stateless_writer));
                rtps_writer_group_lock
                    .stateless_writer_list
                    .push(rtps_writer.clone());
                rtps_writer
            } else {
                let rtps_writer = Arc::new(Mutex::new(RTPSStatefulWriterImpl {}));
                rtps_writer_group_lock
                    .stateful_writer_list
                    .push(rtps_writer.clone());
                rtps_writer
            };

        let data_writer = DataWriterImpl::new(self, Arc::downgrade(&rtps_writer_dyn));
        Some(data_writer)
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        a_datawriter: &<Self as rust_dds_api::publication::publisher::DataWriterGAT<'a, T>>::DataWriterType,
    ) -> DDSResult<()> {
        if std::ptr::eq(a_datawriter.parent, self) {
            let rtps_group = self.impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?;
            let rtps_writer = a_datawriter
                .rtps_writer
                .upgrade()
                .ok_or(DDSError::AlreadyDeleted)?;
            let mut rtps_group_lock = rtps_group.lock().unwrap();
            let rtps_writer_guid = rtps_writer.lock().unwrap().guid();
            if let Some(stateless_writer_index) = rtps_group_lock
                .stateless_writer_list
                .iter()
                .map(|x| x.lock().unwrap())
                .position(|x| x.guid() == rtps_writer_guid)
            {
                rtps_group_lock
                    .stateless_writer_list
                    .remove(stateless_writer_index);
            }
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
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

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let datawriter_qos = qos.unwrap_or_default();
        datawriter_qos.is_consistent()?;
        *self.default_datawriter_qos.lock().unwrap() = datawriter_qos;
        Ok(())
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.lock().unwrap().clone()
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
        return_type::DDSError,
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
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let a_topic = domain_participant
            .create_topic::<MockData>("Test", None, None, 0)
            .unwrap();

        let data_writer = publisher.create_datawriter(&a_topic, None, None, 0);

        assert!(data_writer.is_some());
    }

    #[test]
    fn create_delete_datawriter() {
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let a_topic = domain_participant
            .create_topic::<MockData>("Test", None, None, 0)
            .unwrap();

        let a_datawriter = publisher
            .create_datawriter(&a_topic, None, None, 0)
            .unwrap();

        let result = publisher.delete_datawriter(&a_datawriter);
        assert!(result.is_ok());
    }

    #[test]
    fn set_default_datawriter_qos_some_value() {
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let mut qos = DataWriterQos::default();
        qos.user_data.value = vec![1, 2, 3, 4];
        publisher
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();
        assert!(*publisher.default_datawriter_qos.lock().unwrap() == qos);
    }

    #[test]
    fn set_default_datawriter_qos_inconsistent() {
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let mut qos = DataWriterQos::default();
        qos.resource_limits.max_samples_per_instance = 2;
        qos.resource_limits.max_samples = 1;
        let set_default_topic_qos_result = publisher.set_default_datawriter_qos(Some(qos.clone()));
        assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
    }

    #[test]
    fn set_default_datawriter_qos_none() {
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let mut qos = DataWriterQos::default();
        qos.user_data.value = vec![1, 2, 3, 4];
        publisher
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();

        publisher.set_default_datawriter_qos(None).unwrap();
        assert!(*publisher.default_datawriter_qos.lock().unwrap() == DataWriterQos::default());
    }

    #[test]
    fn get_default_datawriter_qos() {
        let domain_participant = DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let mut qos = DataWriterQos::default();
        qos.user_data.value = vec![1, 2, 3, 4];
        publisher
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();
        assert!(publisher.get_default_datawriter_qos() == qos);
    }
}
