use crate::{
    implementation::utils::{
        node::{ChildNode, RootNode},
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    domain_participant_impl::DomainParticipantImpl,
    node_user_defined_data_writer::UserDefinedDataWriterNode,
    status_condition_impl::StatusConditionImpl, topic_impl::TopicImpl,
    user_defined_publisher::UserDefinedPublisher,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedPublisherNode(
    ChildNode<UserDefinedPublisher, RootNode<DomainParticipantImpl>>,
);

// impl Drop for UserDefinedPublisherNode {
//     fn drop(&mut self) {
//         if self.publisher.weak_count() == 1 {
//             if let Ok(p) = self.get_participant() {
//                 p.delete_publisher(self).ok();
//             }
//         }
//     }
// }

impl UserDefinedPublisherNode {
    pub fn new(node: ChildNode<UserDefinedPublisher, RootNode<DomainParticipantImpl>>) -> Self {
        Self(node)
    }

    pub fn create_datawriter<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedDataWriterNode>
    where
        Foo: DdsType,
    {
        let participant = self.0.parent().get()?;
        let default_unicast_locator_list = participant.default_unicast_locator_list();
        let default_multicast_locator_list = participant.default_multicast_locator_list();

        let writer = self.0.get()?.create_datawriter::<Foo>(
            a_topic,
            qos,
            a_listener,
            mask,
            default_unicast_locator_list,
            default_multicast_locator_list,
        )?;

        Ok(UserDefinedDataWriterNode::new(ChildNode::new(
            writer.downgrade(),
            self.0.clone(),
        )))
    }

    pub fn delete_datawriter(&self, data_writer_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_datawriter(data_writer_handle)
    }

    pub fn lookup_datawriter<Foo>(
        &self,
        topic: &DdsShared<TopicImpl>,
    ) -> DdsResult<UserDefinedDataWriterNode>
    where
        Foo: DdsType,
    {
        let writer = self.0.get()?.lookup_datawriter::<Foo>(topic)?;

        Ok(UserDefinedDataWriterNode::new(ChildNode::new(
            writer.downgrade(),
            self.0.clone(),
        )))
    }

    pub fn suspend_publications(&self) -> DdsResult<()> {
        self.0.get()?.suspend_publications()
    }

    pub fn resume_publications(&self) -> DdsResult<()> {
        self.0.get()?.resume_publications()
    }

    pub fn begin_coherent_changes(&self) -> DdsResult<()> {
        self.0.get()?.begin_coherent_changes()
    }

    pub fn end_coherent_changes(&self) -> DdsResult<()> {
        self.0.get()?.end_coherent_changes()
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        self.0.get()?.wait_for_acknowledgments(max_wait)
    }

    pub fn get_participant(&self) -> DdsResult<DdsWeak<DomainParticipantImpl>> {
        Ok(self.0.parent().get()?.downgrade())
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.0.get()?.delete_contained_entities()
    }

    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.0.get()?.set_default_datawriter_qos(qos)
    }

    pub fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        Ok(self.0.get()?.get_default_datawriter_qos())
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.0.get()?.set_qos(qos)
    }

    pub fn get_qos(&self) -> DdsResult<PublisherQos> {
        Ok(self.0.get()?.get_qos().clone())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.0.get()?.set_listener(a_listener, mask);
        Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(self.0.get()?.get_statuscondition())
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.0.get()?.get_status_changes())
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.0.parent().get()?.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent participant is disabled".to_string(),
            ));
        }

        self.0.get()?.enable()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }
}
