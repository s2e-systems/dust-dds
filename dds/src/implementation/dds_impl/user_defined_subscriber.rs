use crate::{
    implementation::utils::{
        node::ChildNode,
        shared_object::{DdsShared, DdsWeak},
    },
    infrastructure::{
        condition::StatusCondition,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::{SampleLostStatus, StatusKind},
    },
    subscription::subscriber_listener::SubscriberListener,
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    domain_participant_impl::DomainParticipantImpl, topic_impl::TopicImpl,
    user_defined_data_reader::UserDefinedDataReader,
    user_defined_subscriber_impl::UserDefinedSubscriberImpl,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedSubscriber(ChildNode<UserDefinedSubscriberImpl, DomainParticipantImpl>);

impl UserDefinedSubscriber {
    pub fn new(node: ChildNode<UserDefinedSubscriberImpl, DomainParticipantImpl>) -> Self {
        Self(node)
    }

    pub fn create_datareader<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DdsShared<UserDefinedDataReader>>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let participant = self.0.get_parent().upgrade()?;
        let default_unicast_locator_list = participant.default_unicast_locator_list();
        let default_multicast_locator_list = participant.default_multicast_locator_list();

        self.0.get()?.create_datareader::<Foo>(
            a_topic,
            qos,
            a_listener,
            mask,
            default_unicast_locator_list,
            default_multicast_locator_list,
        )
    }

    pub fn delete_datareader(&self, a_datareader_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_datareader(a_datareader_handle)
    }

    pub fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<DdsShared<UserDefinedDataReader>>
    where
        Foo: DdsType,
    {
        self.0.get()?.lookup_datareader::<Foo>(topic_name)
    }

    pub fn notify_datareaders(&self) -> DdsResult<()> {
        self.0.get()?.notify_datareaders()
    }

    pub fn get_participant(&self) -> DdsWeak<DomainParticipantImpl> {
        self.0.get_parent()
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.0.get()?.delete_contained_entities()
    }

    pub fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.0.get()?.set_default_datareader_qos(qos)
    }

    pub fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self.0.get()?.get_default_datareader_qos())
    }

    pub fn set_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.0.get()?.set_qos(qos)
    }

    pub fn get_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        Ok(self.0.get()?.set_listener(a_listener, mask))
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.0.get()?.get_status_changes())
    }

    pub fn enable(&self) -> DdsResult<()> {
        // if !s.upgrade()?.get_participant().is_enabled() {
        //             return Err(DdsError::PreconditionNotMet(
        //                 "Parent participant is disabled".to_string(),
        //             ));
        //         }

        //         s.upgrade()?.enable()
        self.0.get()?.enable()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(self.0.get()?.get_statuscondition()))
    }
}
