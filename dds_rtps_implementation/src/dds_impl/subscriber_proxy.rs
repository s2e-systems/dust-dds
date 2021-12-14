use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, TopicQos},
    },
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::{AnyDataReader, DataReader},
        data_reader_listener::DataReaderListener,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
    },
    topic::topic_description::TopicDescription,
};

use crate::utils::shared_object::{
    rtps_shared_read_lock, rtps_shared_write_lock, rtps_weak_upgrade, RtpsWeak,
};

use super::{
    data_reader_proxy::DataReaderProxy, subscriber_impl::SubscriberImpl, topic_proxy::TopicProxy,
};

pub struct SubscriberProxy<'s> {
    participant: &'s dyn DomainParticipant,
    subscriber_impl: RtpsWeak<SubscriberImpl>,
}

impl<'s> SubscriberProxy<'s> {
    pub fn new(
        participant: &'s dyn DomainParticipant,
        subscriber_impl: RtpsWeak<SubscriberImpl>,
    ) -> Self {
        Self {
            participant,
            subscriber_impl,
        }
    }
}

impl AsRef<RtpsWeak<SubscriberImpl>> for SubscriberProxy<'_> {
    fn as_ref(&self) -> &RtpsWeak<SubscriberImpl> {
        &self.subscriber_impl
    }
}

impl<'dr, 's, Foo> SubscriberDataReaderFactory<'dr, Foo> for SubscriberProxy<'s>
where
    Foo: 'dr,
{
    type TopicType = TopicProxy<'dr, Foo>;
    type DataReaderType = DataReaderProxy<'dr, Foo>;

    fn datareader_factory_create_datareader(
        &'dr self,
        a_topic: &'dr Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<&'static dyn DataReaderListener<DataType = Foo>>,
        mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        todo!()
        // let reader_storage_weak =
        //     rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl).ok()?)
        //         .datareader_factory_create_datareader(
        //             a_topic.topic_impl(),
        //             qos,
        //             a_listener,
        //             mask,
        //         )?;
        // let data_reader = DataReaderProxy::new(self, a_topic, reader_storage_weak);
        // Some(data_reader)
    }

    fn datareader_factory_delete_datareader(
        &self,
        a_datareader: &Self::DataReaderType,
    ) -> DDSResult<()> {
        if std::ptr::eq(a_datareader.get_subscriber(), self) {
            rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?)
                .datareader_factory_delete_datareader(a_datareader.as_ref())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }

    fn datareader_factory_lookup_datareader<'a>(
        &'a self,
        _topic: &'a Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        todo!()
    }
}

impl Subscriber for SubscriberProxy<'_> {
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

impl Entity for SubscriberProxy<'_> {
    type Qos = <SubscriberImpl as Entity>::Qos;
    type Listener = <SubscriberImpl as Entity>::Listener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?)
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_instance_handle()
    }
}
