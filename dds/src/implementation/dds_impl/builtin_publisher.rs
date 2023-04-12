use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup,
            messages::submessages::AckNackSubmessage,
            types::{EntityId, EntityKey, Guid, GuidPrefix, Locator, BUILT_IN_WRITER_GROUP},
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{error::DdsResult, qos::PublisherQos},
};

use super::{
    builtin_stateful_writer::BuiltinStatefulWriter,
    builtin_stateless_writer::BuiltinStatelessWriter,
    domain_participant_impl::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
    },
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    topic_impl::TopicImpl,
};

pub struct BuiltinPublisher {
    _qos: PublisherQos,
    _rtps_group: RtpsGroup,
    spdp_builtin_participant_writer: DdsShared<BuiltinStatelessWriter>,
    sedp_builtin_topics_writer: DdsShared<BuiltinStatefulWriter>,
    sedp_builtin_publications_writer: DdsShared<BuiltinStatefulWriter>,
    sedp_builtin_subscriptions_writer: DdsShared<BuiltinStatefulWriter>,
    enabled: DdsRwLock<bool>,
}

impl BuiltinPublisher {
    pub fn new(
        guid_prefix: GuidPrefix,
        sedp_topic_topics: DdsShared<TopicImpl>,
        sedp_topic_publications: DdsShared<TopicImpl>,
        sedp_topic_subscriptions: DdsShared<TopicImpl>,
        spdp_discovery_locator_list: &[Locator],
        sedp_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        let qos = PublisherQos::default();

        let entity_id = EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_WRITER_GROUP);
        let guid = Guid::new(guid_prefix, entity_id);
        let rtps_group = RtpsGroup::new(guid);

        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);
        let spdp_builtin_participant_writer = BuiltinStatelessWriter::new(
            spdp_builtin_participant_writer_guid,
            spdp_discovery_locator_list,
        );

        let sedp_builtin_topics_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer = BuiltinStatefulWriter::new(
            sedp_builtin_topics_writer_guid,
            sedp_topic_topics,
            sedp_condvar.clone(),
        );

        let sedp_builtin_publications_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = BuiltinStatefulWriter::new(
            sedp_builtin_publications_writer_guid,
            sedp_topic_publications,
            sedp_condvar.clone(),
        );

        let sedp_builtin_subscriptions_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer = BuiltinStatefulWriter::new(
            sedp_builtin_subscriptions_writer_guid,
            sedp_topic_subscriptions,
            sedp_condvar,
        );

        DdsShared::new(BuiltinPublisher {
            _qos: qos,
            _rtps_group: rtps_group,
            spdp_builtin_participant_writer,
            sedp_builtin_topics_writer,
            sedp_builtin_publications_writer,
            sedp_builtin_subscriptions_writer,
            enabled: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<BuiltinPublisher> {
    pub fn spdp_builtin_participant_writer(&self) -> &DdsShared<BuiltinStatelessWriter> {
        &self.spdp_builtin_participant_writer
    }

    pub fn sedp_builtin_topics_writer(&self) -> &DdsShared<BuiltinStatefulWriter> {
        &self.sedp_builtin_topics_writer
    }

    pub fn sedp_builtin_publications_writer(&self) -> &DdsShared<BuiltinStatefulWriter> {
        &self.sedp_builtin_publications_writer
    }

    pub fn sedp_builtin_subscriptions_writer(&self) -> &DdsShared<BuiltinStatefulWriter> {
        &self.sedp_builtin_subscriptions_writer
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        self.spdp_builtin_participant_writer.enable()?;
        self.sedp_builtin_publications_writer.enable()?;
        self.sedp_builtin_topics_writer.enable()?;
        self.sedp_builtin_subscriptions_writer.enable()?;

        Ok(())
    }
}

impl PublisherMessageReceiver for DdsShared<BuiltinPublisher> {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        self.sedp_builtin_publications_writer
            .on_acknack_submessage_received(acknack_submessage, message_receiver);
        self.sedp_builtin_subscriptions_writer
            .on_acknack_submessage_received(acknack_submessage, message_receiver);
        self.sedp_builtin_topics_writer
            .on_acknack_submessage_received(acknack_submessage, message_receiver);
    }

    fn on_nack_frag_submessage_received(
        &self,
        _nackfrag_submessage: &crate::implementation::rtps::messages::submessages::NackFragSubmessage,
        _message_receiver: &MessageReceiver,
    ) {
        // Only for user defined
        todo!()
    }
}
