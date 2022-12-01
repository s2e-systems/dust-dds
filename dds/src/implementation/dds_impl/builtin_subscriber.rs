use std::sync::mpsc::SyncSender;

use crate::implementation::data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData;
use crate::implementation::data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData;
use crate::implementation::data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData;
use crate::implementation::data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData;
use crate::implementation::rtps::group::RtpsGroupImpl;
use crate::implementation::rtps::messages::submessages::{DataSubmessage, HeartbeatSubmessage};
use crate::implementation::rtps::transport::TransportWrite;
use crate::implementation::rtps::types::{EntityId, Guid, GuidPrefix, EntityKind};
use crate::infrastructure::error::DdsResult;
use crate::infrastructure::instance::InstanceHandle;
use crate::infrastructure::status::StatusKind;
use crate::subscription::subscriber_listener::SubscriberListener;
use crate::{
    infrastructure::{condition::StatusCondition, qos::SubscriberQos},
    topic_definition::type_support::DdsType,
};

use crate::implementation::utils::shared_object::{DdsRwLock, DdsShared};

use super::builtin_stateful_reader::BuiltinStatefulReader;
use super::builtin_stateless_reader::BuiltinStatelessReader;
use super::domain_participant_impl::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
};
use super::message_receiver::{MessageReceiver, SubscriberSubmessageReceiver};
use super::{topic_impl::TopicImpl, user_defined_data_reader::UserDefinedDataReader};

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
        notifications_sender: SyncSender<(Guid, StatusKind)>,
    ) -> DdsShared<Self> {
        let qos = SubscriberQos::default();

        let entity_id = EntityId::new([0, 0, 0], EntityKind::BuiltInReaderGroup);
        let guid = Guid::new(guid_prefix, entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);

        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);
        let spdp_builtin_participant_reader =
            BuiltinStatelessReader::new::<SpdpDiscoveredParticipantData>(
                spdp_builtin_participant_reader_guid,
                spdp_topic_participant,
                notifications_sender.clone(),
            );

        let sedp_builtin_topics_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = BuiltinStatefulReader::new::<DiscoveredTopicData>(
            sedp_builtin_topics_guid,
            sedp_topic_topics,
            notifications_sender.clone(),
        );

        let sedp_builtin_publications_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader = BuiltinStatefulReader::new::<DiscoveredWriterData>(
            sedp_builtin_publications_guid,
            sedp_topic_publications,
            notifications_sender.clone(),
        );

        let sedp_builtin_subscriptions_reader =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader = BuiltinStatefulReader::new::<DiscoveredReaderData>(
            sedp_builtin_subscriptions_reader,
            sedp_topic_subscriptions,
            notifications_sender,
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

    pub fn lookup_datareader<Foo>(
        &self,
        _topic: &DdsShared<TopicImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataReader>>
    where
        Foo: DdsType,
    {
        todo!()
    }
}

impl DdsShared<BuiltInSubscriber> {
    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.read_lock().clone()
    }

    pub fn get_listener(&self) -> DdsResult<Option<Box<dyn SubscriberListener>>> {
        todo!()
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
}

impl SubscriberSubmessageReceiver for DdsShared<BuiltInSubscriber> {
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        self.spdp_builtin_participant_reader
            .on_data_submessage_received(data_submessage, message_receiver);
        self.sedp_builtin_topics_reader
            .on_data_submessage_received(data_submessage, message_receiver);
        self.sedp_builtin_publications_reader
            .on_data_submessage_received(data_submessage, message_receiver);
        self.sedp_builtin_subscriptions_reader
            .on_data_submessage_received(data_submessage, message_receiver);
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

impl DdsShared<BuiltInSubscriber> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        self.sedp_builtin_topics_reader.send_message(transport);
        self.sedp_builtin_publications_reader
            .send_message(transport);
        self.sedp_builtin_subscriptions_reader
            .send_message(transport);
    }
}
