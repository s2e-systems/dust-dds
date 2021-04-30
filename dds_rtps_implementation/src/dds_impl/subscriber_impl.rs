use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    domain::domain_participant::{DomainParticipantChild, TopicFactory},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::DDSResult,
    subscription::{data_reader::AnyDataReader, subscriber_listener::SubscriberListener},
};

use super::{domain_participant_impl::DomainParticipantImpl, topic_impl::TopicImpl};

pub struct SubscriberImpl<'a, PSM: rust_rtps_pim::PIM> {
    parent: &'a DomainParticipantImpl<'a, PSM>,
}

impl<'a, PSM: rust_rtps_pim::PIM, T: 'static> TopicFactory<'a, T> for SubscriberImpl<'a, PSM> {
    type TopicType = TopicImpl<'a, PSM, T>;

    fn create_topic(
        &'a self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<
            &'a (dyn rust_dds_api::topic::topic_listener::TopicListener<DataType = T> + 'a),
        >,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn delete_topic(&'a self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn find_topic(
        &self,
        _topic_name: &str,
        _timeout: rust_dds_api::dcps_psm::Duration,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn lookup_topicdescription(
        &self,
        _name: &str,
    ) -> Option<&'a (dyn rust_dds_api::topic::topic_description::TopicDescription<T> + 'a)> {
        todo!()
    }
}

// impl<'b, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types, T: 'static>
//     DataReaderFactory<T> for SubscriberImpl<'b, PSM>
// {
//     type TopicType = TopicImpl<'b, PSM, T>;
//     type DataReaderType = DataReaderImpl<'b, PSM, T>;

//     fn create_datareader<'a>(
//         &'a self,
//         a_topic: &'a Self::TopicType,
//         qos: Option<DataReaderQos>,
//         a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
//         mask: StatusMask,
//     ) -> Option<Self::DataReaderType> {
//         todo!()
//     }

//     fn delete_datareader(&self, a_datareader: &Self::DataReaderType) -> DDSResult<()> {
//         todo!()
//     }

//     fn lookup_datareader<'a>(&'a self, topic: &'a Self::TopicType) -> Option<Self::DataReaderType> {
//         todo!()
//     }
// }

impl<'a, 'b:'a, PSM: rust_rtps_pim::PIM> DomainParticipantChild<'a, 'b> for SubscriberImpl<'a, PSM> {
    type DomainParticipantType = DomainParticipantImpl<'b, PSM>;

    fn get_participant(&self) -> &Self::DomainParticipantType {
        todo!()
        // self.parent
    }
}

impl<'a, PSM: rust_rtps_pim::PIM> rust_dds_api::subscription::subscriber::Subscriber<'a>
    for SubscriberImpl<'a, PSM>
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

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos<'a>>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos<'a>> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos<'a>,
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

impl<'a, PSM: rust_rtps_pim::PIM> Entity for SubscriberImpl<'a, PSM> {
    type Qos = SubscriberQos<'a>;
    type Listener = &'a (dyn SubscriberListener + 'a);

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
