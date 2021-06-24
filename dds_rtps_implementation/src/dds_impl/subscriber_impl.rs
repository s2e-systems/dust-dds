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
use rust_rtps_pim::structure::types::GUIDPIM;

use crate::{
    rtps_impl::rtps_reader_group_impl::RTPSReaderGroupImpl, utils::shared_object::RtpsWeak,
};

use super::{data_reader_impl::DataReaderImpl, topic_impl::TopicImpl};

pub struct SubscriberImpl<'s, PSM>
where
    PSM: GUIDPIM<PSM>,
{
    participant: &'s dyn DomainParticipant,
    rtps_reader_group_impl: RtpsWeak<RTPSReaderGroupImpl<PSM>>,
}

impl<'dr, 's: 'dr, 't: 'dr, T: 'static, PSM>
    rust_dds_api::subscription::subscriber::DataReaderFactory<'dr, 't, T>
    for SubscriberImpl<'s, PSM>
where
    PSM: GUIDPIM<PSM>,
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

impl<'s, PSM> rust_dds_api::subscription::subscriber::Subscriber for SubscriberImpl<'s, PSM>
where
    PSM: GUIDPIM<PSM>,
{
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

impl<'s, PSM> Entity for SubscriberImpl<'s, PSM>
where
    PSM: GUIDPIM<PSM>,
{
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
