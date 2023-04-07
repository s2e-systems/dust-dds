use crate::{
    implementation::utils::{
        node::{ChildNode, RootNode},
        shared_object::DdsShared,
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
    domain_participant_impl::DomainParticipantImpl, node_domain_participant::DomainParticipantNode,
    node_user_defined_data_reader::UserDefinedDataReaderNode, topic_impl::TopicImpl,
    user_defined_subscriber::UserDefinedSubscriber,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedSubscriberNode(
    ChildNode<UserDefinedSubscriber, RootNode<DomainParticipantImpl>>,
);

// impl Drop for UserDefinedSubscriberNode {
//     fn drop(&mut self) {
//         match &self.subscriber {
//             SubscriberKind::BuiltIn(_) => (), // Built-in subscribers don't get deleted
//             SubscriberKind::UserDefined(subscriber) => {
//                 if subscriber.weak_count() == 1 {
//                     if let Ok(p) = self.get_participant() {
//                         p.delete_subscriber(self).ok();
//                     }
//                 }
//             }
//         }
//     }
// }

impl UserDefinedSubscriberNode {
    pub fn new(node: ChildNode<UserDefinedSubscriber, RootNode<DomainParticipantImpl>>) -> Self {
        Self(node)
    }

    pub fn create_datareader<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedDataReaderNode>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let participant = self.0.parent().get()?;
        let default_unicast_locator_list = participant.default_unicast_locator_list();
        let default_multicast_locator_list = participant.default_multicast_locator_list();

        let reader = self.0.get()?.create_datareader::<Foo>(
            a_topic,
            qos,
            a_listener,
            mask,
            default_unicast_locator_list,
            default_multicast_locator_list,
        )?;

        Ok(UserDefinedDataReaderNode::new(ChildNode::new(
            reader.downgrade(),
            self.0.clone(),
        )))
    }

    pub fn delete_datareader(&self, a_datareader_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_datareader(a_datareader_handle)
    }

    pub fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<UserDefinedDataReaderNode>>
    where
        Foo: DdsType,
    {
        let reader = self.0.get()?.lookup_datareader::<Foo>(topic_name)?;
        Ok(Some(UserDefinedDataReaderNode::new(ChildNode::new(
            reader.downgrade(),
            self.0.clone(),
        ))))
    }

    pub fn notify_datareaders(&self) -> DdsResult<()> {
        self.0.get()?.notify_datareaders()
    }

    pub fn get_participant(&self) -> DdsResult<DomainParticipantNode> {
        Ok(DomainParticipantNode::new(self.0.parent().clone()))
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        self.0.get()?.get_sample_lost_status()
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
        self.0.get()?.set_listener(a_listener, mask);
        Ok(())
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
