use std::sync::{atomic, Arc};

use crate::{
    dds::infrastructure::qos::{PublisherQos, SubscriberQos, TopicQos},
    rtps::{
        behavior::endpoint_traits::CacheChangeSender,
        message_sender::RtpsMessageSender,
        transport::Transport,
        types::{
            constants::{
                ENTITY_KIND_BUILT_IN_READER_GROUP, ENTITY_KIND_BUILT_IN_WRITER_GROUP,
                ENTITY_KIND_USER_DEFINED_READER_GROUP, ENTITY_KIND_USER_DEFINED_UNKNOWN,
                ENTITY_KIND_USER_DEFINED_WRITER_GROUP,
            },
            EntityId, GuidPrefix, GUID,
        },
    },
    types::{DDSType, ReturnCode, ReturnCodes},
    utils::maybe_valid::MaybeValidList,
};

use super::{
    rtps_publisher::{RtpsPublisher, RtpsPublisherRef},
    rtps_subscriber::{RtpsSubscriber, RtpsSubscriberRef},
    rtps_topic::{AnyRtpsTopic, RtpsAnyTopicRef, RtpsTopic},
};

enum EntityType {
    BuiltIn,
    UserDefined,
}

pub struct RtpsParticipantEntities {
    guid_prefix: GuidPrefix,
    transport: Box<dyn Transport>,
    entity_type: EntityType,
    publisher_list: MaybeValidList<Box<RtpsPublisher>>,
    publisher_count: atomic::AtomicU8,
    subscriber_list: MaybeValidList<Box<RtpsSubscriber>>,
    subscriber_count: atomic::AtomicU8,
    topic_list: MaybeValidList<Arc<dyn AnyRtpsTopic>>,
    topic_count: atomic::AtomicU8,
}

impl RtpsParticipantEntities {
    pub fn new_builtin(guid_prefix: GuidPrefix, transport: impl Transport) -> Self {
        Self {
            guid_prefix,
            transport: Box::new(transport),
            entity_type: EntityType::BuiltIn,
            publisher_list: Default::default(),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_list: Default::default(),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_list: Default::default(),
            topic_count: atomic::AtomicU8::new(0),
        }
    }

    pub fn new_user_defined(guid_prefix: GuidPrefix, transport: impl Transport) -> Self {
        Self {
            guid_prefix,
            transport: Box::new(transport),
            entity_type: EntityType::UserDefined,
            publisher_list: Default::default(),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_list: Default::default(),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_list: Default::default(),
            topic_count: atomic::AtomicU8::new(0),
        }
    }

    pub fn publisher_list(&self) -> &MaybeValidList<Box<RtpsPublisher>> {
        &self.publisher_list
    }

    pub fn subscriber_list(&self) -> &MaybeValidList<Box<RtpsSubscriber>> {
        &self.subscriber_list
    }

    pub fn topic_list(&self) -> &MaybeValidList<Arc<dyn AnyRtpsTopic>> {
        &self.topic_list
    }

    pub fn transport(&self) -> &dyn Transport {
        self.transport.as_ref()
    }

    pub fn create_publisher(
        &self,
        qos: PublisherQos
    ) -> Option<RtpsPublisherRef> {
        let entity_key = [
            0,
            self.publisher_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match self.entity_type {
            EntityType::BuiltIn => ENTITY_KIND_BUILT_IN_WRITER_GROUP,
            EntityType::UserDefined => ENTITY_KIND_USER_DEFINED_WRITER_GROUP,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_publisher_guid = GUID::new(self.guid_prefix, entity_id);
        let new_publisher = Box::new(RtpsPublisher::new(new_publisher_guid, qos, None, 0));
        self.publisher_list.add(new_publisher)
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisherRef) -> ReturnCode<()> {
        let rtps_publisher = a_publisher.get()?;
        if rtps_publisher.writer_list.is_empty() {
            if self.publisher_list.contains(&a_publisher) {
                a_publisher.delete();
                Ok(())
            } else {
                Err(ReturnCodes::PreconditionNotMet(
                    "Publisher not found in this participant",
                ))
            }
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Publisher still contains data writers",
            ))
        }
    }

    pub fn create_subscriber(
        &self,
        qos: SubscriberQos, // _a_listener: impl SubscriberListener,
                                 // _mask: StatusMask
    ) -> Option<RtpsSubscriberRef> {
        let entity_key = [
            0,
            self.subscriber_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match self.entity_type {
            EntityType::BuiltIn => ENTITY_KIND_BUILT_IN_READER_GROUP,
            EntityType::UserDefined => ENTITY_KIND_USER_DEFINED_READER_GROUP,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_subscriber_guid = GUID::new(self.guid_prefix, entity_id);
        let new_subscriber = Box::new(RtpsSubscriber::new(new_subscriber_guid, qos, None, 0));

        self.subscriber_list.add(new_subscriber)
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriberRef) -> ReturnCode<()> {
        let rtps_subscriber = a_subscriber.get()?;
        if rtps_subscriber.reader_list.is_empty() {
            if self.subscriber_list.contains(&a_subscriber) {
                a_subscriber.delete();
                Ok(())
            } else {
                Err(ReturnCodes::PreconditionNotMet(
                    "Subscriber not found in this participant",
                ))
            }
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Subscriber still contains data readers",
            ))
        }
    }

    pub fn create_topic<T: DDSType>(
        &self,
        topic_name: &str,
        qos: TopicQos,
        // _a_listener: impl TopicListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyTopicRef> {
        qos.is_consistent().ok()?;
        let entity_key = [
            0,
            self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let new_topic_guid = GUID::new(self.guid_prefix, entity_id);
        let new_topic: Arc<RtpsTopic<T>> = Arc::new(RtpsTopic::new(
            new_topic_guid,
            topic_name.clone().into(),
            qos,
            None,
            0,
        ));
        self.topic_list.add(new_topic)
    }

    pub fn delete_topic<T: DDSType>(&self, a_topic: &RtpsAnyTopicRef) -> ReturnCode<()> {
        if self.topic_list.contains(&a_topic) {
            a_topic.delete()
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic not found in this participant",
            ))
        }
    }

    pub fn send_data(&self) {
        for publisher in self.publisher_list.into_iter() {
            if let Some(publisher) = publisher.get().ok() {
                for writer in publisher.writer_list.into_iter() {
                    if let Some(writer) = writer.get().ok() {
                        let mut writer_flavor = writer.writer();
                        println!(
                            "last_change_sequence_number = {:?}",
                            writer_flavor.last_change_sequence_number
                        );
                        let destined_messages = writer_flavor.produce_messages();
                        let participant_guid_prefix = writer_flavor.endpoint.entity.guid.prefix();
                        RtpsMessageSender::send_cache_change_messages(
                            participant_guid_prefix,
                            self.transport.as_ref(),
                            destined_messages,
                        );
                    }
                }
            }
        }
    }
}
