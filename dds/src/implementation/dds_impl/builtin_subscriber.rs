use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        rtps::{
            group::RtpsGroupImpl,
            messages::submessages::{DataSubmessage, HeartbeatSubmessage},
            transport::TransportWrite,
            types::{EntityId, EntityKey, Guid, GuidPrefix, BUILT_IN_READER_GROUP},
        },
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::SubscriberQos,
        status::StatusKind,
    },
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_stateful_reader::{
        BuiltInStatefulReaderDataSubmessageReceivedResult, BuiltinStatefulReader,
    },
    builtin_stateless_reader::{
        BuiltInStatelessReaderDataSubmessageReceivedResult, BuiltinStatelessReader,
    },
    domain_participant_impl::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
    },
    message_receiver::{MessageReceiver, SubscriberSubmessageReceiver},
    topic_impl::TopicImpl,
};

pub enum BuiltinDataReaderKind<'a> {
    Stateless(&'a DdsShared<BuiltinStatelessReader>),
    Stateful(&'a DdsShared<BuiltinStatefulReader>),
}

pub struct BuiltInSubscriber {
    qos: DdsRwLock<SubscriberQos>,
    rtps_group: RtpsGroupImpl,
    spdp_builtin_participant_reader: DdsShared<BuiltinStatelessReader>,
    sedp_builtin_topics_reader: DdsShared<BuiltinStatefulReader>,
    sedp_builtin_publications_reader: DdsShared<BuiltinStatefulReader>,
    sedp_builtin_subscriptions_reader: DdsShared<BuiltinStatefulReader>,
    enabled: DdsRwLock<bool>,
}

impl BuiltInSubscriber {
    pub fn new(
        guid_prefix: GuidPrefix,
        spdp_topic_participant: DdsShared<TopicImpl>,
        sedp_topic_topics: DdsShared<TopicImpl>,
        sedp_topic_publications: DdsShared<TopicImpl>,
        sedp_topic_subscriptions: DdsShared<TopicImpl>,
    ) -> DdsShared<Self> {
        let qos = SubscriberQos::default();

        let entity_id = EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_READER_GROUP);
        let guid = Guid::new(guid_prefix, entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);

        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);
        let spdp_builtin_participant_reader =
            BuiltinStatelessReader::new::<SpdpDiscoveredParticipantData>(
                spdp_builtin_participant_reader_guid,
                spdp_topic_participant,
            );

        let sedp_builtin_topics_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = BuiltinStatefulReader::new::<DiscoveredTopicData>(
            sedp_builtin_topics_guid,
            sedp_topic_topics,
        );

        let sedp_builtin_publications_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader = BuiltinStatefulReader::new::<DiscoveredWriterData>(
            sedp_builtin_publications_guid,
            sedp_topic_publications,
        );

        let sedp_builtin_subscriptions_reader =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader = BuiltinStatefulReader::new::<DiscoveredReaderData>(
            sedp_builtin_subscriptions_reader,
            sedp_topic_subscriptions,
        );

        DdsShared::new(BuiltInSubscriber {
            qos: DdsRwLock::new(qos),
            rtps_group,
            spdp_builtin_participant_reader,
            enabled: DdsRwLock::new(false),
            sedp_builtin_topics_reader,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_reader,
        })
    }
}

impl DdsShared<BuiltInSubscriber> {
    pub fn spdp_builtin_participant_reader(&self) -> &DdsShared<BuiltinStatelessReader> {
        &self.spdp_builtin_participant_reader
    }

    pub fn sedp_builtin_topics_reader(&self) -> &DdsShared<BuiltinStatefulReader> {
        &self.sedp_builtin_topics_reader
    }

    pub fn sedp_builtin_publications_reader(&self) -> &DdsShared<BuiltinStatefulReader> {
        &self.sedp_builtin_publications_reader
    }

    pub fn sedp_builtin_subscriptions_reader(&self) -> &DdsShared<BuiltinStatefulReader> {
        &self.sedp_builtin_subscriptions_reader
    }

    pub fn lookup_datareader<Foo>(&self, topic_name: &str) -> DdsResult<BuiltinDataReaderKind>
    where
        Foo: DdsType,
    {
        match topic_name {
            "DCPSParticipant" if Foo::type_name() == ParticipantBuiltinTopicData::type_name() => {
                Ok(BuiltinDataReaderKind::Stateless(
                    &self.spdp_builtin_participant_reader,
                ))
            }
            "DCPSTopic" if Foo::type_name() == TopicBuiltinTopicData::type_name() => Ok(
                BuiltinDataReaderKind::Stateful(&self.sedp_builtin_topics_reader),
            ),
            "DCPSPublication" if Foo::type_name() == PublicationBuiltinTopicData::type_name() => {
                Ok(BuiltinDataReaderKind::Stateful(
                    &self.sedp_builtin_publications_reader,
                ))
            }
            "DCPSSubscription" if Foo::type_name() == SubscriptionBuiltinTopicData::type_name() => {
                Ok(BuiltinDataReaderKind::Stateful(
                    &self.sedp_builtin_subscriptions_reader,
                ))
            }

            _ => Err(DdsError::BadParameter),
        }
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.read_lock().clone()
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        self.spdp_builtin_participant_reader.enable()?;
        self.sedp_builtin_topics_reader.enable()?;
        self.sedp_builtin_subscriptions_reader.enable()?;
        self.sedp_builtin_publications_reader.enable()?;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        self.sedp_builtin_topics_reader.send_message(transport);
        self.sedp_builtin_publications_reader
            .send_message(transport);
        self.sedp_builtin_subscriptions_reader
            .send_message(transport);
    }
}

impl SubscriberSubmessageReceiver for DdsShared<BuiltInSubscriber> {
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        if self
            .spdp_builtin_participant_reader
            .on_data_submessage_received(data_submessage, message_receiver)
            == BuiltInStatelessReaderDataSubmessageReceivedResult::NewDataAvailable
        {
            self.spdp_builtin_participant_reader.on_data_available();
            return;
        }

        if self
            .sedp_builtin_topics_reader
            .on_data_submessage_received(data_submessage, message_receiver)
            == BuiltInStatefulReaderDataSubmessageReceivedResult::NewDataAvailable
        {
            self.sedp_builtin_publications_reader.on_data_available();
            return;
        }

        if self
            .sedp_builtin_publications_reader
            .on_data_submessage_received(data_submessage, message_receiver)
            == BuiltInStatefulReaderDataSubmessageReceivedResult::NewDataAvailable
        {
            self.sedp_builtin_publications_reader.on_data_available();
            return;
        }

        if self
            .sedp_builtin_subscriptions_reader
            .on_data_submessage_received(data_submessage, message_receiver)
            == BuiltInStatefulReaderDataSubmessageReceivedResult::NewDataAvailable
        {
            self.sedp_builtin_subscriptions_reader.on_data_available();
        }
    }

    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.sedp_builtin_topics_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        self.sedp_builtin_publications_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        self.sedp_builtin_subscriptions_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
    }
}
