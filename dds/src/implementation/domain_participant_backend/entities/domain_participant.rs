use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    builtin_topics::TopicBuiltinTopicData,
    domain::domain_participant_factory::DomainId,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant_backend::services::domain_participant_service::BUILT_IN_TOPIC_NAME_LIST,
        listeners::domain_participant_listener::DomainParticipantListenerActor,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
    },
    runtime::actor::Actor,
};

use super::{publisher::PublisherEntity, subscriber::SubscriberEntity, topic::TopicEntity};

pub struct DomainParticipantEntity {
    domain_id: DomainId,
    domain_tag: String,
    instance_handle: InstanceHandle,
    qos: DomainParticipantQos,
    builtin_subscriber: SubscriberEntity,
    builtin_publisher: PublisherEntity,
    user_defined_subscriber_list: Vec<SubscriberEntity>,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<PublisherEntity>,
    default_publisher_qos: PublisherQos,
    topic_list: HashMap<String, TopicEntity>,
    default_topic_qos: TopicQos,
    discovered_participant_list: Vec<SpdpDiscoveredParticipantData>,
    discovered_topic_list: Vec<TopicBuiltinTopicData>,
    discovered_reader_list: Vec<DiscoveredReaderData>,
    discovered_writer_list: Vec<DiscoveredWriterData>,
    enabled: bool,
    ignored_participants: HashSet<InstanceHandle>,
    ignored_publications: HashSet<InstanceHandle>,
    ignored_subcriptions: HashSet<InstanceHandle>,
    _ignored_topic_list: HashSet<InstanceHandle>,
    listener: Option<Actor<DomainParticipantListenerActor>>,
    listener_mask: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
}

impl DomainParticipantEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener: Option<Actor<DomainParticipantListenerActor>>,
        listener_mask: Vec<StatusKind>,
        status_condition: Actor<StatusConditionActor>,
        instance_handle: InstanceHandle,
        builtin_publisher: PublisherEntity,
        builtin_subscriber: SubscriberEntity,
        topic_list: HashMap<String, TopicEntity>,
        domain_tag: String,
    ) -> Self {
        Self {
            domain_id,
            instance_handle,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: Vec::new(),
            default_publisher_qos: PublisherQos::default(),
            topic_list,
            default_topic_qos: TopicQos::default(),
            discovered_participant_list: Vec::new(),
            discovered_topic_list: Vec::new(),
            discovered_reader_list: Vec::new(),
            discovered_writer_list: Vec::new(),
            enabled: false,
            ignored_participants: HashSet::new(),
            ignored_publications: HashSet::new(),
            ignored_subcriptions: HashSet::new(),
            _ignored_topic_list: HashSet::new(),
            listener,
            listener_mask,
            status_condition,
            domain_tag,
        }
    }

    pub fn get_current_time(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
    }

    pub fn builtin_subscriber(&self) -> &SubscriberEntity {
        &self.builtin_subscriber
    }

    pub fn add_discovered_topic(&mut self, topic_builtin_topic_data: TopicBuiltinTopicData) {
        match self
            .discovered_topic_list
            .iter_mut()
            .find(|t| t.key() == topic_builtin_topic_data.key())
        {
            Some(x) => *x = topic_builtin_topic_data,
            None => self.discovered_topic_list.push(topic_builtin_topic_data),
        }
    }

    pub fn remove_discovered_writer(&mut self, discovered_writer_handle: &InstanceHandle) {
        self.discovered_writer_list
            .retain(|x| &x.dds_publication_data.key().value != discovered_writer_handle.as_ref());
    }

    pub fn qos(&self) -> &DomainParticipantQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DomainParticipantQos) {
        self.qos = qos;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn builtin_subscriber_mut(&mut self) -> &mut SubscriberEntity {
        &mut self.builtin_subscriber
    }

    pub fn builtin_publisher_mut(&mut self) -> &mut PublisherEntity {
        &mut self.builtin_publisher
    }

    pub fn ignore_participant(&mut self, handle: InstanceHandle) {
        self.ignored_participants.insert(handle);
    }

    pub fn ignore_subscription(&mut self, handle: InstanceHandle) {
        self.ignored_subcriptions.insert(handle);
    }

    pub fn ignore_publication(&mut self, handle: InstanceHandle) {
        self.ignored_publications.insert(handle);
    }

    pub fn get_default_topic_qos(&self) -> &TopicQos {
        &self.default_topic_qos
    }

    pub fn set_default_topic_qos(&mut self, qos: TopicQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.default_topic_qos = qos;
        Ok(())
    }

    pub fn get_discovered_participants(&self) -> Vec<InstanceHandle> {
        self.discovered_participant_list
            .iter()
            .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
            .collect()
    }

    pub fn get_discovered_participant_data(
        &self,
        participant_handle: &InstanceHandle,
    ) -> Option<&SpdpDiscoveredParticipantData> {
        self.discovered_participant_list
            .iter()
            .find(|p| &p.dds_participant_data.key().value == participant_handle.as_ref())
    }

    pub fn get_discovered_topics(&self) -> Vec<InstanceHandle> {
        self.discovered_topic_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: &InstanceHandle,
    ) -> Option<&TopicBuiltinTopicData> {
        self.discovered_topic_list
            .iter()
            .find(|x| &x.key().value == topic_handle.as_ref())
    }

    pub fn find_topic(&self, topic_name: &str) -> Option<&TopicBuiltinTopicData> {
        self.discovered_topic_list
            .iter()
            .find(|&discovered_topic_data| discovered_topic_data.name() == topic_name)
    }

    pub fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        match self.discovered_participant_list.iter_mut().find(|p| {
            p.dds_participant_data.key() == discovered_participant_data.dds_participant_data.key()
        }) {
            Some(x) => *x = discovered_participant_data,
            None => self
                .discovered_participant_list
                .push(discovered_participant_data),
        }
    }

    pub fn remove_discovered_participant(
        &mut self,
        discovered_participant_handle: &InstanceHandle,
    ) {
        self.discovered_participant_list.retain(|p| {
            &p.dds_participant_data.key().value != discovered_participant_handle.as_ref()
        });
    }

    pub fn add_discovered_reader(&mut self, discovered_reader_data: DiscoveredReaderData) {
        match self.discovered_reader_list.iter_mut().find(|x| {
            x.dds_subscription_data.key() == discovered_reader_data.dds_subscription_data.key()
        }) {
            Some(x) => *x = discovered_reader_data,
            None => self.discovered_reader_list.push(discovered_reader_data),
        }
    }

    pub fn remove_discovered_reader(&mut self, discovered_reader_handle: &InstanceHandle) {
        self.discovered_reader_list
            .retain(|x| &x.dds_subscription_data.key().value != discovered_reader_handle.as_ref());
    }

    pub fn discovered_reader_data_list(&self) -> impl Iterator<Item = &DiscoveredReaderData> {
        self.discovered_reader_list.iter()
    }

    pub fn add_discovered_writer(&mut self, discovered_writer_data: DiscoveredWriterData) {
        match self.discovered_writer_list.iter_mut().find(|x| {
            x.dds_publication_data.key() == discovered_writer_data.dds_publication_data.key()
        }) {
            Some(x) => *x = discovered_writer_data,
            None => self.discovered_writer_list.push(discovered_writer_data),
        }
    }

    pub fn publication_builtin_topic_data_list(
        &self,
    ) -> impl Iterator<Item = &DiscoveredWriterData> {
        self.discovered_writer_list.iter()
    }

    pub fn default_subscriber_qos(&self) -> &SubscriberQos {
        &self.default_subscriber_qos
    }

    pub fn default_publisher_qos(&self) -> &PublisherQos {
        &self.default_publisher_qos
    }

    pub fn set_default_subscriber_qos(&mut self, default_subscriber_qos: SubscriberQos) {
        self.default_subscriber_qos = default_subscriber_qos;
    }

    pub fn set_default_publisher_qos(&mut self, default_publisher_qos: PublisherQos) {
        self.default_publisher_qos = default_publisher_qos;
    }

    pub fn get_subscriber(&self, handle: InstanceHandle) -> Option<&SubscriberEntity> {
        self.user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_subscriber(&mut self, handle: InstanceHandle) -> Option<&mut SubscriberEntity> {
        self.user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn insert_subscriber(&mut self, subscriber: SubscriberEntity) {
        self.user_defined_subscriber_list.push(subscriber);
    }

    pub fn remove_subscriber(&mut self, handle: &InstanceHandle) -> Option<SubscriberEntity> {
        let i = self
            .user_defined_subscriber_list
            .iter()
            .position(|x| &x.instance_handle() == handle)?;

        Some(self.user_defined_subscriber_list.remove(i))
    }

    pub fn subscriber_list(&mut self) -> impl Iterator<Item = &SubscriberEntity> {
        self.user_defined_subscriber_list.iter()
    }

    pub fn drain_subscriber_list(&mut self) -> impl Iterator<Item = SubscriberEntity> + '_ {
        self.user_defined_subscriber_list.drain(..)
    }

    pub fn get_publisher(&self, handle: InstanceHandle) -> Option<&PublisherEntity> {
        self.user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_publisher(&mut self, handle: InstanceHandle) -> Option<&mut PublisherEntity> {
        self.user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn insert_publisher(&mut self, publisher: PublisherEntity) {
        self.user_defined_publisher_list.push(publisher);
    }

    pub fn remove_publisher(&mut self, handle: &InstanceHandle) -> Option<PublisherEntity> {
        let i = self
            .user_defined_publisher_list
            .iter()
            .position(|x| &x.instance_handle() == handle)?;

        Some(self.user_defined_publisher_list.remove(i))
    }

    pub fn drain_publisher_list(&mut self) -> impl Iterator<Item = PublisherEntity> + '_ {
        self.user_defined_publisher_list.drain(..)
    }

    pub fn publisher_list(&mut self) -> impl Iterator<Item = &PublisherEntity> {
        self.user_defined_publisher_list.iter()
    }

    pub fn publisher_list_mut(&mut self) -> impl Iterator<Item = &mut PublisherEntity> {
        self.user_defined_publisher_list.iter_mut()
    }

    pub fn get_topic(&self, topic_name: &str) -> Option<&TopicEntity> {
        self.topic_list.get(topic_name)
    }

    pub fn get_mut_topic(&mut self, topic_name: &str) -> Option<&mut TopicEntity> {
        self.topic_list.get_mut(topic_name)
    }

    pub fn insert_topic(&mut self, topic: TopicEntity) {
        self.topic_list.insert(topic.topic_name().to_owned(), topic);
    }

    pub fn remove_topic(&mut self, topic_name: &str) -> Option<TopicEntity> {
        self.topic_list.remove(topic_name)
    }

    pub fn delete_all_topics(&mut self) {
        self.topic_list
            .retain(|_, x| BUILT_IN_TOPIC_NAME_LIST.contains(&x.topic_name()));
    }

    pub fn topic_list(&mut self) -> impl Iterator<Item = &TopicEntity> {
        self.topic_list.values()
    }

    pub fn is_empty(&self) -> bool {
        let no_user_defined_topics = self
            .topic_list
            .keys()
            .filter(|t| !BUILT_IN_TOPIC_NAME_LIST.contains(&t.as_ref()))
            .count()
            == 0;

        self.user_defined_publisher_list.is_empty()
            && self.user_defined_subscriber_list.is_empty()
            && no_user_defined_topics
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn listener(&self) -> Option<&Actor<DomainParticipantListenerActor>> {
        self.listener.as_ref()
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Actor<DomainParticipantListenerActor>>,
        status_kind: Vec<StatusKind>,
    ) {
        self.listener = listener;
        self.listener_mask = status_kind;
    }

    pub fn domain_id(&self) -> i32 {
        self.domain_id
    }

    pub fn domain_tag(&self) -> &str {
        &self.domain_tag
    }

    pub fn discovered_participant_list(
        &self,
    ) -> impl Iterator<Item = &SpdpDiscoveredParticipantData> {
        self.discovered_participant_list.iter()
    }
}
