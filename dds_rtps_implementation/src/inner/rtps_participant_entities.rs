use crate::{
    rtps_topic::{AnyRtpsTopic, RtpsAnyTopicRef, RtpsTopicInner},
    utils::maybe_valid::{MaybeValidList, MaybeValidRef},
};
use rust_dds_api::{
    infrastructure::{
        qos::{PublisherQos, SubscriberQos, TopicQos},
        status::StatusMask,
    },
    publication::publisher_listener::PublisherListener,
    subscription::subscriber_listener::SubscriberListener,
    topic::topic_listener::TopicListener,
};
use rust_dds_types::{DDSType, ReturnCode, ReturnCodes};
use rust_rtps::{
    message_sender::RtpsMessageSender,
    transport::Transport,
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_READER_GROUP, ENTITY_KIND_USER_DEFINED_READER_GROUP,
            ENTITY_KIND_USER_DEFINED_UNKNOWN,
        },
        EntityId, GuidPrefix, GUID,
    },
};
use std::sync::{atomic, Arc};

use super::{
    rtps_publisher_inner::{RtpsPublisherInner, RtpsPublisherRef},
    rtps_subscriber_inner::{RtpsSubscriberInner, RtpsSubscriberRef},
};

enum EntityType {
    BuiltIn,
    UserDefined,
}

pub struct RtpsParticipantEntities {
    guid_prefix: GuidPrefix,
    transport: Box<dyn Transport>,
    entity_type: EntityType,
    publisher_list: MaybeValidList<Box<RtpsPublisherInner>>,
    publisher_count: atomic::AtomicU8,
    subscriber_list: MaybeValidList<Box<RtpsSubscriberInner>>,
    subscriber_count: atomic::AtomicU8,
    topic_list: MaybeValidList<Arc<dyn AnyRtpsTopic>>,
    topic_count: atomic::AtomicU8,
}

impl RtpsParticipantEntities {
    pub fn new_builtin(guid_prefix: GuidPrefix, transport: impl Transport) -> Self {
        Self::new(guid_prefix, transport, EntityType::BuiltIn)
    }

    pub fn new_user_defined(guid_prefix: GuidPrefix, transport: impl Transport) -> Self {
        Self::new(guid_prefix, transport, EntityType::UserDefined)
    }

    fn new(guid_prefix: GuidPrefix, transport: impl Transport, entity_type: EntityType) -> Self {
        Self {
            guid_prefix,
            transport: Box::new(transport),
            entity_type,
            publisher_list: Default::default(),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_list: Default::default(),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_list: Default::default(),
            topic_count: atomic::AtomicU8::new(0),
        }
    }

    pub fn publisher_list(&self) -> &MaybeValidList<Box<RtpsPublisherInner>> {
        &self.publisher_list
    }

    pub fn subscriber_list(&self) -> &MaybeValidList<Box<RtpsSubscriberInner>> {
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
        qos: PublisherQos,
        _listener: Option<Box<dyn PublisherListener>>,
        _status_mask: StatusMask,
    ) -> Option<RtpsPublisherRef> {
        let entity_key = [
            0,
            self.publisher_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let new_publisher = match self.entity_type {
            EntityType::BuiltIn => {
                RtpsPublisherInner::new_builtin(self.guid_prefix, entity_key, qos, None, 0)
            }
            EntityType::UserDefined => {
                RtpsPublisherInner::new_user_defined(self.guid_prefix, entity_key, qos, None, 0)
            }
        };
        self.publisher_list.add(Box::new(new_publisher))
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
        qos: SubscriberQos,
        _a_listener: Option<Box<dyn SubscriberListener>>,
        _mask: StatusMask,
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
        let new_subscriber = Box::new(RtpsSubscriberInner::new(new_subscriber_guid, qos, None, 0));

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

    pub fn create_topic<'a, T: DDSType>(
        &'a self,
        topic_name: &str,
        qos: TopicQos,
        a_listener: Option<Box<dyn TopicListener<T>>>,
        mask: StatusMask,
    ) -> Option<RtpsAnyTopicRef<'a>> {
        qos.is_consistent().ok()?;
        let entity_key = [
            0,
            self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let new_topic_guid = GUID::new(self.guid_prefix, entity_id);
        let new_topic = Arc::new(RtpsTopicInner::new(
            new_topic_guid,
            topic_name.clone().into(),
            qos,
            a_listener,
            mask,
        ));
        self.topic_list.add(new_topic)
    }

    pub fn delete_topic(&self, a_topic: &RtpsAnyTopicRef) -> ReturnCode<()> {
        if self.topic_list.contains(&a_topic) {
            a_topic.delete()
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic not found in this participant",
            ))
        }
    }

    // pub fn send_data(&self) {
    //     for publisher in self.publisher_list.into_iter() {
    //         // if let Some(publisher) = publisher.get().ok() {
    //         //     for writer in publisher.writer_list.into_iter() {
    //         //         // println!(
    //         //         //     "last_change_sequence_number = {:?}",
    //         //         //     writer_flavor.last_change_sequence_number
    //         //         // );
    //         //         let destined_messages = writer.produce_messages();
    //         //         let participant_guid_prefix = self.guid_prefix;
    //         //         RtpsMessageSender::send_cache_change_messages(
    //         //             participant_guid_prefix,
    //         //             self.transport.as_ref(),
    //         //             destined_messages,
    //         //         );
    //         //     }
    //         // }
    //     }
    // }
}
