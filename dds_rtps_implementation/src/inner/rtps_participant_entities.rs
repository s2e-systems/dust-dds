use crate::utils::maybe_valid::MaybeValidList;
use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::qos::{PublisherQos, SubscriberQos, TopicQos},
    publication::publisher_listener::PublisherListener,
    return_type::{DDSError, DDSResult},
    subscription::subscriber_listener::SubscriberListener,
    topic::topic_listener::TopicListener,
};
use rust_rtps::{
    message_sender::RtpsMessageSender,
    transport::Transport,
    types::{ GuidPrefix},
};
use std::sync::Arc;

use super::{
    rtps_publisher_inner::{RtpsPublisherInner, RtpsPublisherInnerRef},
    rtps_subscriber_inner::{RtpsSubscriberInner, RtpsSubscriberInnerRef},
    rtps_topic_inner::{RtpsAnyTopicInner, RtpsAnyTopicInnerRef, RtpsTopicInner},
};

enum EntityType {
    BuiltIn,
    UserDefined,
}

pub struct RtpsParticipantEntities {
    entity_type: EntityType,
    publisher_list: MaybeValidList<Box<RtpsPublisherInner>>,
    subscriber_list: MaybeValidList<Box<RtpsSubscriberInner>>,
    topic_list: MaybeValidList<Arc<dyn RtpsAnyTopicInner>>,
    transport: Box<dyn Transport>,
}

impl RtpsParticipantEntities {
    pub fn new_builtin(transport: impl Transport) -> Self {
        Self::new(transport, EntityType::BuiltIn)
    }

    pub fn new_user_defined(transport: impl Transport) -> Self {
        Self::new(transport, EntityType::UserDefined)
    }

    fn new(transport: impl Transport, entity_type: EntityType) -> Self {
        Self {
            entity_type,
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
            topic_list: Default::default(),
            transport: Box::new(transport),
        }
    }

    pub fn publisher_list(&self) -> &MaybeValidList<Box<RtpsPublisherInner>> {
        &self.publisher_list
    }

    pub fn subscriber_list(&self) -> &MaybeValidList<Box<RtpsSubscriberInner>> {
        &self.subscriber_list
    }

    pub fn topic_list(&self) -> &MaybeValidList<Arc<dyn RtpsAnyTopicInner>> {
        &self.topic_list
    }

    pub fn transport(&self) -> &dyn Transport {
        self.transport.as_ref()
    }

    pub fn create_publisher(
        &self,
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: PublisherQos,
        _listener: Option<Box<dyn PublisherListener>>,
        _status_mask: StatusMask,
    ) -> Option<RtpsPublisherInnerRef> {
        let new_publisher = match self.entity_type {
            EntityType::BuiltIn => {
                RtpsPublisherInner::new_builtin(guid_prefix, entity_key, qos, None, 0)
            }
            EntityType::UserDefined => {
                RtpsPublisherInner::new_user_defined(guid_prefix, entity_key, qos, None, 0)
            }
        };
        self.publisher_list.add(Box::new(new_publisher))
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisherInnerRef) -> DDSResult<()> {
        let rtps_publisher = a_publisher.get()?;
        if rtps_publisher.writer_list.is_empty() {
            if self.publisher_list.contains(&a_publisher) {
                a_publisher.delete();
                Ok(())
            } else {
                Err(DDSError::PreconditionNotMet(
                    "Publisher not found in this participant",
                ))
            }
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher still contains data writers",
            ))
        }
    }

    pub fn create_subscriber(
        &self,
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: SubscriberQos,
        _a_listener: Option<Box<dyn SubscriberListener>>,
        _mask: StatusMask,
    ) -> Option<RtpsSubscriberInnerRef> {
        let new_subscriber = match self.entity_type {
            EntityType::BuiltIn => {
                RtpsSubscriberInner::new_builtin(guid_prefix, entity_key, qos, None, 0)
            }
            EntityType::UserDefined => {
                RtpsSubscriberInner::new_user_defined(guid_prefix, entity_key, qos, None, 0)
            }
        };
        self.subscriber_list.add(Box::new(new_subscriber))
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriberInnerRef) -> DDSResult<()> {
        let rtps_subscriber = a_subscriber.get()?;
        if rtps_subscriber.reader_list.is_empty() {
            if self.subscriber_list.contains(&a_subscriber) {
                a_subscriber.delete();
                Ok(())
            } else {
                Err(DDSError::PreconditionNotMet(
                    "Subscriber not found in this participant",
                ))
            }
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber still contains data readers",
            ))
        }
    }

    pub fn create_topic<'a, T: DDSType>(
        &'a self,
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic_name: &str,
        qos: TopicQos,
        a_listener: Option<Box<dyn TopicListener<T>>>,
        mask: StatusMask,
    ) -> Option<RtpsAnyTopicInnerRef<'a>> {
        qos.is_consistent().ok()?;

        let new_topic = Arc::new(RtpsTopicInner::new(
            guid_prefix,
            entity_key,
            topic_name.clone().into(),
            qos,
            a_listener,
            mask,
        ));
        self.topic_list.add(new_topic)
    }

    pub fn delete_topic(&self, a_topic: &RtpsAnyTopicInnerRef) -> DDSResult<()> {
        if self.topic_list.contains(&a_topic) {
            a_topic.delete()
        } else {
            Err(DDSError::PreconditionNotMet(
                "Topic not found in this participant",
            ))
        }
    }

    pub fn send_data(&self, guid_prefix: GuidPrefix) {
        for publisher in self.publisher_list.into_iter() {
            if let Some(publisher) = publisher.get().ok() {
                for writer in publisher.writer_list.into_iter() {
                    let destined_messages = writer.produce_messages();
                    let participant_guid_prefix = guid_prefix;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTransport;

    impl Transport for MockTransport {
        fn write(
            &self,
            _message: rust_rtps::messages::RtpsMessage,
            _destination_locator: &rust_rtps::types::Locator,
        ) {
            todo!()
        }

        fn read(
            &self,
        ) -> rust_rtps::transport::TransportResult<
            Option<(rust_rtps::messages::RtpsMessage, rust_rtps::types::Locator)>,
        > {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<rust_rtps::types::Locator> {
            todo!()
        }

        fn multicast_locator_list(&self) -> &Vec<rust_rtps::types::Locator> {
            todo!()
        }
    }

    struct TestTypeWithKey;

    impl DDSType for TestTypeWithKey {
        fn type_name() -> &'static str {
            "TestType"
        }

        fn has_key() -> bool {
            true
        }

        fn key(&self) -> Vec<u8> {
            todo!()
        }

        fn serialize(&self) -> Vec<u8> {
            todo!()
        }

        fn deserialize(_data: Vec<u8>) -> Self {
            todo!()
        }
    }

    #[test]
    fn create_built_in_entities() {
        let guid_prefix = [1; 12];
        let participant_entities = RtpsParticipantEntities::new_builtin(MockTransport);

        let publisher_qos = PublisherQos::default();
        let publisher_entity_key = [0, 1, 0];
        let publisher_listener = None;
        let publisher_status_mask = 0;
        let _publisher = participant_entities
            .create_publisher(
                guid_prefix,
                publisher_entity_key,
                publisher_qos,
                publisher_listener,
                publisher_status_mask,
            )
            .expect("Error creating publisher");

        let subscriber_qos = SubscriberQos::default();
        let subscriber_entity_key = [0, 1, 0];
        let subscriber_listener = None;
        let subscriber_status_mask = 0;
        let _subscriber = participant_entities
            .create_subscriber(
                guid_prefix,
                subscriber_entity_key,
                subscriber_qos,
                subscriber_listener,
                subscriber_status_mask,
            )
            .expect("Error creating subscriber");

        let topic_entity_key = [0, 1, 0];
        let topic_with_key_name = "TopicWithKey";
        let topic_with_key_qos = TopicQos::default();
        let topic_with_key_listener = None;
        let topic_with_key_status_mask = 0;
        let _topic_with_key = participant_entities
            .create_topic::<TestTypeWithKey>(
                guid_prefix,
                topic_entity_key,
                topic_with_key_name,
                topic_with_key_qos,
                topic_with_key_listener,
                topic_with_key_status_mask,
            )
            .expect("Error creating topic with key");
    }
}
