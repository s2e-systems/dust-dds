use crate::{
    implementation::utils::{
        node::{ChildNode, RootNode},
        shared_object::{DdsRwLock, DdsShared},
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
    domain_participant_impl::DomainParticipantImpl, node_domain_participant::DomainParticipantNode,
    node_user_defined_data_writer::UserDefinedDataWriterNode,
    status_condition_impl::StatusConditionImpl, user_defined_publisher::UserDefinedPublisher,
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
        type_name: &'static str,
        topic_name: String,
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

        let data_writer = self.0.get()?.create_datawriter::<Foo>(
            type_name,
            topic_name,
            qos,
            a_listener,
            mask,
            default_unicast_locator_list,
            default_multicast_locator_list,
        )?;

        let data_writer_node =
            UserDefinedDataWriterNode::new(ChildNode::new(data_writer.downgrade(), self.0.clone()));

        if self.0.get()?.is_enabled()
            && self
                .0
                .get()?
                .get_qos()
                .entity_factory
                .autoenable_created_entities
        {
            data_writer_node.enable()?;
        }

        Ok(data_writer_node)
    }

    pub fn delete_datawriter(&self, data_writer_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_datawriter(data_writer_handle)
    }

    pub fn lookup_datawriter(
        &self,
        type_name: &'static str,
        topic_name: &str,
    ) -> DdsResult<UserDefinedDataWriterNode> {
        let writer = self.0.get()?.lookup_datawriter(type_name, topic_name)?;

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

    pub fn get_participant(&self) -> DdsResult<DomainParticipantNode> {
        Ok(DomainParticipantNode::new(self.0.parent().clone()))
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
        Ok(self.0.get()?.get_qos())
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

        if !self.0.get()?.is_enabled() {
            self.0.get()?.enable()?;

            if self
                .0
                .get()?
                .get_qos()
                .entity_factory
                .autoenable_created_entities
            {
                for data_writer in self.0.get()?.data_writer_list() {
                    data_writer.enable()?;
                    let topic = self
                        .0
                        .parent()
                        .get()?
                        .lookup_topicdescription(
                            data_writer.get_topic_name(),
                            data_writer.get_type_name(),
                        )
                        .expect("Topic must exist");
                    data_writer.announce_writer(&topic.get_qos(), &self.0.get()?.get_qos());
                }
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }
}
