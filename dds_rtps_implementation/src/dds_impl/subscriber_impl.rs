use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    dds_type::DDSType,
    domain::domain_participant::{DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::DDSResult,
    subscription::{
        data_reader::AnyDataReader, data_reader_listener::DataReaderListener,
        subscriber::DataReaderGAT, subscriber_listener::SubscriberListener,
    },
};

use super::{
    data_reader_impl::DataReaderImpl, domain_participant_impl::DomainParticipantImpl,
    topic_impl::TopicImpl,
};

pub struct SubscriberImpl<'a, PSM: rust_rtps_pim::structure::Types> {
    parent: &'a DomainParticipantImpl<PSM>,
}

impl<'a, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types, T: DDSType>
    TopicGAT<'a, T> for SubscriberImpl<'a, PSM>
{
    type TopicType = TopicImpl<'a, PSM, T>;
}

impl<'a, PSM: rust_rtps_pim::structure::Types, T: DDSType> DataReaderGAT<'a, T>
    for SubscriberImpl<'a, PSM>
{
    type DataReaderType = DataReaderImpl<'a, PSM, T>;
}

impl<'a, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types>
    DomainParticipantChild<'a> for SubscriberImpl<'a, PSM>
{
    type DomainParticipantType = DomainParticipantImpl<PSM>;
}

impl<'a, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types>
    rust_dds_api::subscription::subscriber::Subscriber<'a> for SubscriberImpl<'a, PSM>
{
    fn create_datareader<T: DDSType>(
        &'a self,
        _a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        _qos: Option<DataReaderQos>,
        _a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        _mask: StatusMask,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        todo!()
        // let topic = a_topic.impl_ref.upgrade()?;
        // let data_reader_ref = self
        //     .impl_ref
        //     .upgrade()?
        //     .lock()
        //     .unwrap()
        //     .create_datareader(topic, qos, a_listener, mask)?;

        // Some(DataReader(Node {
        //     parent: (self, a_topic),
        //     impl_ref: data_reader_ref,
        // }))
    }

    fn delete_datareader<T: DDSType>(
        &'a self,
        _a_datareader: &<Self as DataReaderGAT<'a, T>>::DataReaderType,
    ) -> DDSResult<()> {
        todo!()
        // if std::ptr::eq(a_datareader.parent.0, self) {
        //     self.impl_ref
        //         .upgrade()
        //         .ok_or(DDSError::AlreadyDeleted)?
        //         .lock()
        //         .unwrap()
        //         .delete_datareader(&a_datareader.impl_ref)
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Publisher can only be deleted from its parent participant",
        //     ))
        // }
    }

    fn lookup_datareader<T: DDSType>(
        &self,
        _topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        todo!()
    }

    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        self.parent
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
}

impl<'a, PSM: rust_rtps_pim::structure::Types> Entity for SubscriberImpl<'a, PSM> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener + 'a>;

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
