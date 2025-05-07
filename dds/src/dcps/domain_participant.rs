use crate::{
    builtin_topics::TopicBuiltinTopicData,
    dcps::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        publisher::PublisherEntity,
        subscriber::SubscriberEntity,
        topic::TopicEntity,
    },
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
        status::StatusKind,
    },
};
use alloc::{string::String, vec::Vec};

use super::builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC};

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

pub struct DomainParticipantEntity<S, L> {
    domain_id: DomainId,
    domain_tag: String,
    instance_handle: InstanceHandle,
    qos: DomainParticipantQos,
    builtin_subscriber: SubscriberEntity<S, L>,
    builtin_publisher: PublisherEntity<S, L>,
    user_defined_subscriber_list: Vec<SubscriberEntity<S, L>>,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<PublisherEntity<S, L>>,
    default_publisher_qos: PublisherQos,
    topic_list: Vec<TopicEntity<S, L>>,
    default_topic_qos: TopicQos,
    discovered_participant_list: Vec<SpdpDiscoveredParticipantData>,
    discovered_topic_list: Vec<TopicBuiltinTopicData>,
    discovered_reader_list: Vec<DiscoveredReaderData>,
    discovered_writer_list: Vec<DiscoveredWriterData>,
    enabled: bool,
    ignored_participants: Vec<InstanceHandle>,
    ignored_publications: Vec<InstanceHandle>,
    ignored_subcriptions: Vec<InstanceHandle>,
    _ignored_topic_list: Vec<InstanceHandle>,
    listener_sender: L,
    listener_mask: Vec<StatusKind>,
    status_condition: S,
}

impl<S, L> DomainParticipantEntity<S, L> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener_sender: L,
        listener_mask: Vec<StatusKind>,
        status_condition: S,
        instance_handle: InstanceHandle,
        builtin_publisher: PublisherEntity<S, L>,
        builtin_subscriber: SubscriberEntity<S, L>,
        topic_list: Vec<TopicEntity<S, L>>,
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
            ignored_participants: Vec::new(),
            ignored_publications: Vec::new(),
            ignored_subcriptions: Vec::new(),
            _ignored_topic_list: Vec::new(),
            listener_sender,
            listener_mask,
            status_condition,
            domain_tag,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn status_condition(&self) -> &S {
        &self.status_condition
    }

    pub fn builtin_subscriber(&self) -> &SubscriberEntity<S, L> {
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

    pub fn builtin_subscriber_mut(&mut self) -> &mut SubscriberEntity<S, L> {
        &mut self.builtin_subscriber
    }

    pub fn builtin_publisher_mut(&mut self) -> &mut PublisherEntity<S, L> {
        &mut self.builtin_publisher
    }

    pub fn ignore_participant(&mut self, handle: InstanceHandle) {
        if !self.ignored_participants.contains(&handle) {
            self.ignored_participants.push(handle);
        }
    }

    pub fn ignore_subscription(&mut self, handle: InstanceHandle) {
        if !self.ignored_subcriptions.contains(&handle) {
            self.ignored_subcriptions.push(handle);
        }
    }

    pub fn ignore_publication(&mut self, handle: InstanceHandle) {
        if !self.ignored_publications.contains(&handle) {
            self.ignored_publications.push(handle);
        }
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

    pub fn get_subscriber(&self, handle: InstanceHandle) -> Option<&SubscriberEntity<S, L>> {
        self.user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_subscriber(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut SubscriberEntity<S, L>> {
        self.user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn insert_subscriber(&mut self, subscriber: SubscriberEntity<S, L>) {
        self.user_defined_subscriber_list.push(subscriber);
    }

    pub fn remove_subscriber(&mut self, handle: &InstanceHandle) -> Option<SubscriberEntity<S, L>> {
        let i = self
            .user_defined_subscriber_list
            .iter()
            .position(|x| &x.instance_handle() == handle)?;

        Some(self.user_defined_subscriber_list.remove(i))
    }

    pub fn subscriber_list(&mut self) -> impl Iterator<Item = &SubscriberEntity<S, L>> {
        self.user_defined_subscriber_list.iter()
    }

    pub fn drain_subscriber_list(&mut self) -> impl Iterator<Item = SubscriberEntity<S, L>> + '_ {
        self.user_defined_subscriber_list.drain(..)
    }

    pub fn get_publisher(&self, handle: InstanceHandle) -> Option<&PublisherEntity<S, L>> {
        self.user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_publisher(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut PublisherEntity<S, L>> {
        self.user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn insert_publisher(&mut self, publisher: PublisherEntity<S, L>) {
        self.user_defined_publisher_list.push(publisher);
    }

    pub fn remove_publisher(&mut self, handle: &InstanceHandle) -> Option<PublisherEntity<S, L>> {
        let i = self
            .user_defined_publisher_list
            .iter()
            .position(|x| &x.instance_handle() == handle)?;

        Some(self.user_defined_publisher_list.remove(i))
    }

    pub fn drain_publisher_list(&mut self) -> impl Iterator<Item = PublisherEntity<S, L>> + '_ {
        self.user_defined_publisher_list.drain(..)
    }

    pub fn publisher_list(&mut self) -> impl Iterator<Item = &PublisherEntity<S, L>> {
        self.user_defined_publisher_list.iter()
    }

    pub fn publisher_list_mut(&mut self) -> impl Iterator<Item = &mut PublisherEntity<S, L>> {
        self.user_defined_publisher_list.iter_mut()
    }

    pub fn get_topic(&self, topic_name: &str) -> Option<&TopicEntity<S, L>> {
        self.topic_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
    }

    pub fn get_mut_topic(&mut self, topic_name: &str) -> Option<&mut TopicEntity<S, L>> {
        self.topic_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
    }

    pub fn insert_topic(&mut self, topic: TopicEntity<S, L>) {
        match self
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name() == topic.topic_name())
        {
            Some(x) => *x = topic,
            None => self.topic_list.push(topic),
        }
    }

    pub fn remove_topic(&mut self, topic_name: &str) -> Option<TopicEntity<S, L>> {
        let index = self
            .topic_list
            .iter()
            .position(|x| x.topic_name() == topic_name)?;
        Some(self.topic_list.remove(index))
    }

    pub fn delete_all_topics(&mut self) {
        self.topic_list
            .retain(|x| BUILT_IN_TOPIC_NAME_LIST.contains(&x.topic_name()));
    }

    pub fn topic_list_mut(&mut self) -> impl Iterator<Item = &mut TopicEntity<S, L>> {
        self.topic_list.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        let no_user_defined_topics = self
            .topic_list
            .iter()
            .filter(|t| !BUILT_IN_TOPIC_NAME_LIST.contains(&t.topic_name()))
            .count()
            == 0;

        self.user_defined_publisher_list.is_empty()
            && self.user_defined_subscriber_list.is_empty()
            && no_user_defined_topics
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn listener(&self) -> &L {
        &self.listener_sender
    }

    pub fn set_listener(&mut self, listener_sender: L, status_kind: Vec<StatusKind>) {
        self.listener_sender = listener_sender;
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
