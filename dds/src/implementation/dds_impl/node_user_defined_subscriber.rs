use crate::{
    implementation::{
        rtps::{
            endpoint::RtpsEndpoint,
            reader::RtpsReader,
            stateful_reader::RtpsStatefulReader,
            types::{
                EntityId, EntityKey, Guid, TopicKind, USER_DEFINED_READER_NO_KEY,
                USER_DEFINED_READER_WITH_KEY,
            },
        },
        utils::node::ChildNode,
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::{SampleLostStatus, StatusKind},
        time::DURATION_ZERO,
    },
    subscription::subscriber_listener::SubscriberListener,
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener, dds_data_reader::DdsDataReader,
    dds_domain_participant::AnnounceKind,
    dds_domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
    dds_subscriber::DdsSubscriber, node_domain_participant::DomainParticipantNode,
    node_user_defined_data_reader::UserDefinedDataReaderNode, status_listener::StatusListener,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedSubscriberNode(ChildNode<DdsSubscriber, Guid>);

impl UserDefinedSubscriberNode {
    pub fn new(node: ChildNode<DdsSubscriber, Guid>) -> Self {
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
        let (default_unicast_locator_list, default_multicast_locator_list) =
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY
                .domain_participant_list()
                .get_participant(self.0.parent(), |dp| {
                    (
                        dp.unwrap().default_unicast_locator_list(),
                        dp.unwrap().default_multicast_locator_list(),
                    )
                });

        let qos = match qos {
            QosKind::Default => self.0.get()?.get_default_datareader_qos(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let entity_kind = match Foo::has_key() {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(self.0.get()?.guid().entity_id().entity_key())[0],
            self.0.get()?.get_unique_reader_id(),
            0,
        ]);

        let entity_id = EntityId::new(entity_key, entity_kind);

        let guid = Guid::new(self.0.get()?.guid().prefix(), entity_id);

        let topic_kind = match Foo::has_key() {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<Foo>(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                default_unicast_locator_list,
                default_multicast_locator_list,
            ),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        ));

        let data_reader = DdsDataReader::new(rtps_reader, type_name, topic_name, a_listener, mask);

        self.0.get()?.stateful_data_reader_add(data_reader.clone());

        let node =
            UserDefinedDataReaderNode::new(ChildNode::new(data_reader.downgrade(), self.0.clone()));

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
        let data_reader = self
            .0
            .get()?
            .stateful_data_reader_list()
            .into_iter()
            .find(|x| x.get_instance_handle() == a_datareader_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data reader can only be deleted from its parent subscriber".to_string(),
                )
            })?
            .clone();

        self.0
            .get()?
            .stateful_data_reader_delete(a_datareader_handle);

        if data_reader.is_enabled() {
            self.0
                .parent()
                .parent()
                .get()?
                .announce_sender()
                .send(AnnounceKind::DeletedDataReader(
                    data_reader.get_instance_handle(),
                ))
                .ok();
        }

        Ok(())
    }

    pub fn lookup_datareader(
        &self,
        type_name: &str,
        topic_name: &str,
    ) -> DdsResult<Option<UserDefinedDataReaderNode>> {
        let reader = self
            .0
            .get()?
            .stateful_data_reader_list()
            .into_iter()
            .find(|data_reader| {
                data_reader.get_topic_name() == topic_name
                    && data_reader.get_type_name() == type_name
            })
            .cloned()
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))?;
        Ok(Some(UserDefinedDataReaderNode::new(ChildNode::new(
            reader.downgrade(),
            self.0.clone(),
        ))))
    }

    pub fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn get_participant(&self) -> DdsResult<DomainParticipantNode> {
        Ok(DomainParticipantNode::new(self.0.parent().clone()))
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        for data_reader in self.0.get()?.stateful_data_reader_drain().into_iter() {
            if data_reader.is_enabled() {
                self.0
                    .parent()
                    .parent()
                    .get()?
                    .announce_sender()
                    .send(AnnounceKind::DeletedDataReader(
                        data_reader.get_instance_handle(),
                    ))
                    .ok();
            }
        }
        Ok(())
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
                for data_reader in &self.0.get()?.stateful_data_reader_list() {
                    data_reader.enable()?;
                    let topic = self
                        .0
                        .parent()
                        .get()?
                        .topic_list()
                        .into_iter()
                        .find(|t| {
                            t.get_name() == data_reader.get_topic_name()
                                && t.get_type_name() == data_reader.get_type_name()
                        })
                        .cloned()
                        .expect("Topic must exist");
                    self.0
                        .parent()
                        .parent()
                        .get()?
                        .announce_sender()
                        .send(AnnounceKind::CreatedDataReader(
                            data_reader.as_discovered_reader_data(
                                &topic.get_qos(),
                                &self.0.get()?.get_qos(),
                            ),
                        ))
                        .ok();
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
