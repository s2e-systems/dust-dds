use std::sync::Mutex;

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::StatusCondition,
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener,
        publisher::DataWriterFactory, publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};
use rust_rtps_pim::structure::RTPSEntity;

use crate::{
    rtps_impl::rtps_writer_group_impl::RTPSWriterGroupImpl,
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{
    data_writer_impl::DataWriterImpl, topic_impl::TopicImpl, writer_factory::WriterFactory,
};

const ENTITYKIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
const ENTITYKIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
const ENTITYKIND_BUILTIN_WRITER_WITH_KEY: u8 = 0xc2;
const ENTITYKIND_BUILTIN_WRITER_NO_KEY: u8 = 0xc3;

pub struct PublisherImpl<'p> {
    participant: &'p dyn DomainParticipant,
    writer_factory: Mutex<WriterFactory>,
    default_datawriter_qos: Mutex<DataWriterQos>,
    rtps_writer_group_impl: RtpsWeak<RTPSWriterGroupImpl>,
}

impl<'p> PublisherImpl<'p> {
    pub fn new(
        participant: &'p dyn DomainParticipant,
        rtps_writer_group_impl: &RtpsShared<RTPSWriterGroupImpl>,
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

impl<'dw, 'p: 'dw, 't: 'dw, T: 'static> DataWriterFactory<'dw, 't, T> for PublisherImpl<'p> {
    type TopicType = TopicImpl<'t, T>;
    type DataWriterType = DataWriterImpl<'dw, T>;

    fn create_datawriter(
        &'dw self,
        a_topic: &'dw Self::TopicType,
        qos: Option<DataWriterQos>,
        a_listener: Option<&'static dyn DataWriterListener<DataPIM = T>>,
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

impl<'p> rust_dds_api::publication::publisher::Publisher for PublisherImpl<'p> {
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

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
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
        todo!()
    }
}

impl<'p> rust_dds_api::infrastructure::entity::Entity for PublisherImpl<'p>
{
    type Qos = PublisherQos;
    type Listener = &'static dyn PublisherListener;

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
        // Ok(crate::utils::instance_handle_from_guid(
        //     &self.rtps_writer_group_impl.upgrade()?.lock().guid(),
        // ))
    }
}