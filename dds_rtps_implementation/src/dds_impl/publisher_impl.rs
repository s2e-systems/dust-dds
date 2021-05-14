use std::sync::Mutex;

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    infrastructure::{
        entity::StatusCondition,
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterFactory, PublisherParent},
        publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};
use rust_rtps_pim::structure::RTPSEntity;

use crate::{
    rtps_impl::rtps_writer_group_impl::RTPSWriterGroupImpl,
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{
    data_writer_impl::DataWriterImpl, domain_participant_impl::DomainParticipantImpl,
    topic_impl::TopicImpl, writer_factory::WriterFactory,
};

const ENTITYKIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
const ENTITYKIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
const ENTITYKIND_BUILTIN_WRITER_WITH_KEY: u8 = 0xc2;
const ENTITYKIND_BUILTIN_WRITER_NO_KEY: u8 = 0xc3;

pub struct PublisherImpl<'p, 'dp: 'p, PSM: rust_rtps_pim::PIM> {
    participant: &'p DomainParticipantImpl<'dp, PSM>,
    writer_factory: Mutex<WriterFactory<PSM>>,
    default_datawriter_qos: Mutex<DataWriterQos<'dp>>,
    rtps_writer_group_impl: RtpsWeak<RTPSWriterGroupImpl<'dp, PSM>>,
}

impl<'p, 'dp: 'p, PSM: rust_rtps_pim::PIM> PublisherImpl<'p, 'dp, PSM> {
    pub fn new(
        participant: &'p DomainParticipantImpl<'dp, PSM>,
        rtps_writer_group_impl: &RtpsShared<RTPSWriterGroupImpl<'dp, PSM>>,
    ) -> Self {
        let writer_factory = WriterFactory::new(*rtps_writer_group_impl.lock().guid().prefix());
        Self {
            participant,
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            writer_factory: Mutex::new(writer_factory),
            rtps_writer_group_impl: rtps_writer_group_impl.downgrade(),
        }
    }
}

impl<'p, 'dp: 'p, PSM: rust_rtps_pim::PIM> PublisherParent<'dp> for PublisherImpl<'p, 'dp, PSM> {
    type DomainParticipantType = DomainParticipantImpl<'dp, PSM>;

    fn get_participant(&self) -> &Self::DomainParticipantType {
        &self.participant
    }
}

impl<'dw, 'p: 'dw, 't: 'dw, 'dp: 'p + 't, T: 'dp, PSM: rust_rtps_pim::PIM>
    DataWriterFactory<'dw, 'p, 't, 'dp, T> for PublisherImpl<'p, 'dp, PSM>
{
    type TopicType = TopicImpl<'t, 'dp, T, PSM>;
    type DataWriterType = DataWriterImpl<'dw, 'p, 't, 'dp, T, PSM>;

    fn create_datawriter(
        &'dw self,
        a_topic: &'dw Self::TopicType,
        qos: Option<DataWriterQos<'dp>>,
        a_listener: Option<&'dp (dyn DataWriterListener<DataType = T> + 'dp)>,
        mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        let qos = qos.unwrap_or(self.default_datawriter_qos.lock().unwrap().clone());
        let rtps_writer = self
            .writer_factory
            .lock()
            .unwrap()
            .create_datawriter(qos, a_listener, mask);
        let rtps_writer_shared = RtpsShared::new(rtps_writer);
        let rtps_writer_weak = rtps_writer_shared.downgrade();
        self.rtps_writer_group_impl
            .upgrade()
            .ok()?
            .lock()
            .add_writer(rtps_writer_shared);

        Some(DataWriterImpl::new(self, a_topic, rtps_writer_weak))
    }

    fn delete_datawriter(&self, a_datawriter: &Self::DataWriterType) -> DDSResult<()> {
        if std::ptr::eq(a_datawriter.get_publisher(), self) {
            todo!()
            // self.rtps_writer_group_impl
            // .upgrade()?
            // .delete_datawriter(a_datawriter.get_instance_handle()?)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher",
            ))
        }
    }

    fn lookup_datawriter(&'dw self, _topic: &'dw Self::TopicType) -> Option<Self::DataWriterType> {
        todo!()
    }
}

impl<'p, 'dp: 'p, PSM: rust_rtps_pim::PIM> rust_dds_api::publication::publisher::Publisher<'p, 'dp>
    for PublisherImpl<'p, 'dp, PSM>
{
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

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos<'dp>>) -> DDSResult<()> {
        // self.rtps_writer_group_impl
        //     .upgrade()?
        //     .set_default_datawriter_qos(qos)
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos<'dp> {
        // self.default_datawriter_qos.lock().unwrap().clone()
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos<'dp>,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'p, 'dp: 'p, PSM: rust_rtps_pim::PIM> rust_dds_api::infrastructure::entity::Entity
    for PublisherImpl<'p, 'dp, PSM>
{
    type Qos = PublisherQos<'dp>;
    type Listener = &'dp (dyn PublisherListener + 'dp);

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
        Ok(crate::utils::instance_handle_from_guid(
            &self.rtps_writer_group_impl.upgrade()?.lock().guid(),
        ))
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
