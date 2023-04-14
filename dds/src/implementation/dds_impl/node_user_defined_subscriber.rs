use crate::{
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
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
    node_user_defined_data_reader::UserDefinedDataReaderNode, status_listener::StatusListener,
    user_defined_subscriber::UserDefinedSubscriber,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedSubscriberNode(
    ChildNode<UserDefinedSubscriber, RootNode<DomainParticipantImpl>>,
);

impl UserDefinedSubscriberNode {
    pub fn new(node: ChildNode<UserDefinedSubscriber, RootNode<DomainParticipantImpl>>) -> Self {
        Self(node)
    }

    pub fn create_datareader<Foo>(
        &self,
        type_name: &'static str,
        topic_name: String,
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
            type_name,
            topic_name,
            qos,
            a_listener,
            mask,
            default_unicast_locator_list,
            default_multicast_locator_list,
        )?;

        let node =
            UserDefinedDataReaderNode::new(ChildNode::new(reader.downgrade(), self.0.clone()));

        if self.0.get()?.is_enabled()
            && self
                .0
                .get()?
                .get_qos()
                .entity_factory
                .autoenable_created_entities
        {
            node.enable()?;
        }

        Ok(node)
    }

    pub fn delete_datareader(&self, a_datareader_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_datareader(a_datareader_handle)
    }

    pub fn lookup_datareader(
        &self,
        type_name: &str,
        topic_name: &str,
    ) -> DdsResult<Option<UserDefinedDataReaderNode>> {
        let reader = self
            .0
            .get()?
            .data_reader_list()
            .find_map(|data_reader| {
                if data_reader.get_topic_name() == topic_name
                    && data_reader.get_type_name() == type_name
                {
                    Some(data_reader)
                } else {
                    None
                }
            })
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))?;
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
        *self.0.get()?.get_status_listener_lock() = StatusListener::new(a_listener, mask);
        Ok(())
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
                for data_reader in self.0.get()?.data_reader_list() {
                    data_reader.enable()?;
                    let topic = self
                        .0
                        .parent()
                        .get()?
                        .topic_list()
                        .find(|t| {
                            t.get_name() == data_reader.get_topic_name()
                                && t.get_type_name() == data_reader.get_type_name()
                        })
                        .expect("Topic must exist");
                    data_reader.announce_reader(&topic.get_qos(), &self.0.get()?.get_qos());
                }
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(self.0.get()?.get_statuscondition()))
    }
}
