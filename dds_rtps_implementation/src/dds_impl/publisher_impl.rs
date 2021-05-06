use std::sync::{Mutex, Weak};

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    infrastructure::{
        entity::StatusCondition,
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{publisher::DataWriterFactory, publisher_listener::PublisherListener},
    return_type::DDSResult,
};

use crate::rtps_impl::rtps_writer_group_impl::RTPSWriterGroupImpl;

use super::{
    data_writer_impl::DataWriterImpl, domain_participant_impl::DomainParticipantImpl,
    topic_impl::TopicImpl,
};

const ENTITYKIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
const ENTITYKIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
const ENTITYKIND_BUILTIN_WRITER_WITH_KEY: u8 = 0xc2;
const ENTITYKIND_BUILTIN_WRITER_NO_KEY: u8 = 0xc3;

pub struct PublisherImpl<'publisher, 'participant: 'publisher, PSM: rust_rtps_pim::PIM> {
    parent: &'publisher DomainParticipantImpl<'participant, PSM>,
    impl_ref: Weak<Mutex<RTPSWriterGroupImpl<PSM>>>,
    default_datawriter_qos: Mutex<DataWriterQos<'publisher>>,
    datawriter_counter: Mutex<u8>,
}

impl<'publisher, 'participant: 'publisher, PSM: rust_rtps_pim::PIM>
    PublisherImpl<'publisher, 'participant, PSM>
{
    pub fn new(
        parent: &'publisher DomainParticipantImpl<'participant, PSM>,
        impl_ref: Weak<Mutex<RTPSWriterGroupImpl<PSM>>>,
    ) -> Self {
        Self {
            parent,
            impl_ref,
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            datawriter_counter: Mutex::new(0),
        }
    }
}

impl<
        'datawriter,
        'publisher: 'datawriter,
        'topic: 'datawriter,
        'participant: 'publisher + 'topic,
        T: 'topic,
        PSM: rust_rtps_pim::PIM,
    > DataWriterFactory<'datawriter, 'publisher, 'topic, 'participant, T>
    for PublisherImpl<'publisher, 'participant, PSM>
{
    type TopicType = TopicImpl<'topic, 'participant, T, PSM>;
    type DataWriterType = DataWriterImpl<'datawriter, 'publisher, 'topic, 'participant, T, PSM>;

    fn create_datawriter(
        &'datawriter self,
        a_topic: &'datawriter Self::TopicType,
        qos: Option<DataWriterQos<'datawriter>>,
        a_listener: Option<
            &'datawriter (dyn rust_dds_api::publication::data_writer_listener::DataWriterListener<
                DataType = T,
            > + 'datawriter),
        >,
        mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        todo!()
    }

    fn delete_datawriter(&self, a_datawriter: &Self::DataWriterType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datawriter(
        &'datawriter self,
        topic: &'datawriter Self::TopicType,
    ) -> Option<Self::DataWriterType> {
        todo!()
    }
}

impl<'publisher, 'participant: 'publisher, PSM: rust_rtps_pim::PIM>
    rust_dds_api::publication::publisher::Publisher<'publisher, 'participant>
    for PublisherImpl<'publisher, 'participant, PSM>
{
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

    fn get_participant(&self) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant<'participant> {
        self.parent
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos<'publisher>>) -> DDSResult<()> {
        let datawriter_qos = qos.unwrap_or_default();
        datawriter_qos.is_consistent()?;
        *self.default_datawriter_qos.lock().unwrap() = datawriter_qos;
        Ok(())
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos<'publisher> {
        self.default_datawriter_qos.lock().unwrap().clone()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos<'publisher>,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }


}

impl<'a, 'b: 'a, PSM: rust_rtps_pim::PIM> rust_dds_api::infrastructure::entity::Entity
    for PublisherImpl<'a, 'b, PSM>
{
    type Qos = PublisherQos<'a>;
    type Listener = &'a (dyn PublisherListener + 'a);

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
    // use rust_dds_api::{
    //     domain::domain_participant::DomainParticipant, publication::publisher::Publisher,
    //     return_type::DDSError,
    // };

    // use crate::rtps_impl::rtps_participant_impl::RTPSParticipantImpl;
    // use rust_rtps_udp_psm::RtpsUdpPsm;

    // use super::*;

    struct MockData;

    // impl DDSType for MockData {
    //     fn type_name() -> &'static str {
    //         todo!()
    //     }

    //     fn has_key() -> bool {
    //         todo!()
    //     }

    //     fn key(&self) -> Vec<u8> {
    //         todo!()
    //     }

    //     fn serialize(&self) -> Vec<u8> {
    //         todo!()
    //     }

    //     fn deserialize(_data: Vec<u8>) -> Self {
    //         todo!()
    //     }
    // }

    #[test]
    fn create_datawriter() {
        todo!()
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let a_topic = domain_participant
        //     .create_topic::<MockData>("Test", None, None, 0)
        //     .unwrap();

        // let data_writer = publisher.create_datawriter(&a_topic, None, None, 0);

        // assert!(data_writer.is_some());
    }

    #[test]
    fn create_delete_datawriter() {
        todo!()
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let a_topic = domain_participant
        //     .create_topic::<MockData>("Test", None, None, 0)
        //     .unwrap();

        // let a_datawriter = publisher
        //     .create_datawriter(&a_topic, None, None, 0)
        //     .unwrap();

        // let result = publisher.delete_datawriter(&a_datawriter);
        // assert!(result.is_ok());
    }

    #[test]
    fn set_default_datawriter_qos_some_value() {
        todo!()
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let mut qos = DataWriterQos::default();
        // qos.user_data.value = vec![1, 2, 3, 4];
        // publisher
        //     .set_default_datawriter_qos(Some(qos.clone()))
        //     .unwrap();
        // assert!(*publisher.default_datawriter_qos.lock().unwrap() == qos);
    }

    #[test]
    fn set_default_datawriter_qos_inconsistent() {
        todo!()
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let mut qos = DataWriterQos::default();
        // qos.resource_limits.max_samples_per_instance = 2;
        // qos.resource_limits.max_samples = 1;
        // let set_default_topic_qos_result = publisher.set_default_datawriter_qos(Some(qos.clone()));
        // assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
    }

    #[test]
    fn set_default_datawriter_qos_none() {
        todo!()
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let mut qos = DataWriterQos::default();
        // qos.user_data.value = vec![1, 2, 3, 4];
        // publisher
        //     .set_default_datawriter_qos(Some(qos.clone()))
        //     .unwrap();

        // publisher.set_default_datawriter_qos(None).unwrap();
        // assert!(*publisher.default_datawriter_qos.lock().unwrap() == DataWriterQos::default());
    }

    #[test]
    fn get_default_datawriter_qos() {
        todo!()
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let mut qos = DataWriterQos::default();
        // qos.user_data.value = vec![1, 2, 3, 4];
        // publisher
        //     .set_default_datawriter_qos(Some(qos.clone()))
        //     .unwrap();
        // assert!(publisher.get_default_datawriter_qos() == qos);
    }
}
