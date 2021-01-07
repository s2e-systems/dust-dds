use crate::dds::{infrastructure::entity::{Entity, StatusCondition}, publication::data_writer::DataWriter};
use crate::dds::infrastructure::qos::{
    DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos,
};
use crate::dds::infrastructure::status::StatusMask;
use crate::dds::publication::data_writer::{RtpsDataWriter, AnyRtpsWriter};
use crate::dds::publication::publisher::{Publisher, RtpsPublisher};
use crate::dds::subscription::subscriber::{RtpsSubscriber, Subscriber};
use crate::dds::topic::topic::{AnyRtpsTopic, RtpsTopic, Topic};
use crate::rtps::behavior::endpoint_traits::{CacheChangeSender, DestinedMessages};
use crate::rtps::behavior::types::constants::DURATION_INFINITE;
use crate::rtps::message_sender::RtpsMessageSender;
use crate::rtps::structure::Participant;
use crate::rtps::transport::Transport;
use crate::rtps::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
use crate::rtps::types::{EntityId, EntityKind, GUID};
use crate::types::{DDSType, DomainId, Duration, InstanceHandle, ReturnCode, ReturnCodes, Time};
use crate::utils::maybe_valid::MaybeValidList;
use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dds::infrastructure::qos::DataWriterQos,
    dds::infrastructure::qos_policy::ReliabilityQosPolicyKind,
    discovery::types::SpdpDiscoveredParticipantData,
    rtps::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN},
};
use crate::{
    dds::{
        domain::domain_participant_listener::DomainParticipantListener,
        infrastructure::qos_policy::UserDataQosPolicy, publication::data_writer::WriterFlavor,
    },
    discovery::types::ParticipantProxy,
    rtps::{endpoint_types::BuiltInEndpointSet, types::Locator},
    types::{BuiltInTopicKey, TIME_INVALID},
    utils::maybe_valid::{MaybeValid, MaybeValidRef},
};
use std::sync::{atomic, Arc, Mutex};
use std::thread::JoinHandle;
use std::{cell::RefCell, sync::RwLock};

pub struct RtpsParticipant {
    participant: Participant,
    qos: Mutex<DomainParticipantQos>,
    default_publisher_qos: Mutex<PublisherQos>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    default_topic_qos: Mutex<TopicQos>,
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    publisher_list: MaybeValidList<Box<RtpsPublisher>>,
    builtin_publisher: RwLock<MaybeValid<Box<RtpsPublisher>>>,
    publisher_count: atomic::AtomicU8,
    subscriber_list: MaybeValidList<Box<RtpsSubscriber>>,
    subscriber_count: atomic::AtomicU8,
    topic_list: MaybeValidList<Arc<dyn AnyRtpsTopic>>,
    topic_count: atomic::AtomicU8,
    enabled: atomic::AtomicBool,
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        userdata_transport: impl Transport,
        metatraffic_transport: impl Transport,
        //     a_listener: impl DomainParticipantListener,
        //     mask: StatusMask,
    ) -> Self {
        // let domain_tag = "".to_string();
        // let lease_duration = Duration {
        //     sec: 30,
        //     nanosec: 0,
        // };
        let guid_prefix = [1; 12];
        let participant = Participant::new(guid_prefix, domain_id, PROTOCOL_VERSION_2_4, VENDOR_ID);
        // let sedp = SimpleEndpointDiscoveryProtocol::new(guid_prefix);
        let entity_id = EntityId::new([0, 0, 0x01], EntityKind::BuiltInWriterGroup);
        let publisher_qos = PublisherQos::default();
        let builtin_rtps_publisher =
            RtpsPublisher::new(GUID::new(guid_prefix, entity_id), publisher_qos, None, 0);

        RtpsParticipant {
            participant,
            qos: Mutex::new(qos),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            userdata_transport: Box::new(userdata_transport),
            metatraffic_transport: Box::new(metatraffic_transport),
            publisher_list: Default::default(),
            builtin_publisher: RwLock::new(MaybeValid::new(Box::new(builtin_rtps_publisher))),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_list: Default::default(),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_list: Default::default(),
            topic_count: atomic::AtomicU8::new(0),
            enabled: atomic::AtomicBool::new(false),
            // sedp,
        }
    }
}

pub struct DomainParticipant {
    pub(crate) inner: Arc<RtpsParticipant>,
    pub(crate) thread_list: RefCell<Vec<JoinHandle<()>>>,
}

impl DomainParticipant {
    /// This operation creates a Publisher with the desired QoS policies and attaches to it the specified PublisherListener.
    /// If the specified QoS policies are not consistent, the operation will fail and no Publisher will be created.
    /// The special value PUBLISHER_QOS_DEFAULT can be used to indicate that the Publisher should be created with the default
    /// Publisher QoS set in the factory. The use of this value is equivalent to the application obtaining the default Publisher QoS by
    /// means of the operation get_default_publisher_qos (2.2.2.2.1.21) and using the resulting QoS to create the Publisher.
    /// The created Publisher belongs to the DomainParticipant that is its factory
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    pub fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        // _a_listener: impl PublisherListener,
        // _mask: StatusMask
    ) -> Option<Publisher> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.inner
                .publisher_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedWriterGroup);
        let new_publisher_guid = GUID::new(guid_prefix, entity_id);
        let new_publisher_qos = qos.unwrap_or(self.get_default_publisher_qos());
        let new_publisher = Box::new(RtpsPublisher::new(
            new_publisher_guid,
            new_publisher_qos,
            None,
            0,
        ));
        let rtps_publisher = self.inner.publisher_list.add(new_publisher)?;

        Some(Publisher {
            parent_participant: self,
            rtps_publisher,
        })
    }

    /// This operation deletes an existing Publisher.
    /// A Publisher cannot be deleted if it has any attached DataWriter objects. If delete_publisher is called on a Publisher with
    /// existing DataWriter object, it will return PRECONDITION_NOT_MET.
    /// The delete_publisher operation must be called on the same DomainParticipant object used to create the Publisher. If
    /// delete_publisher is called on a different DomainParticipant, the operation will have no effect and it will return
    /// PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_publisher(&self, a_publisher: &Publisher) -> ReturnCode<()> {
        let rtps_publisher_inner = a_publisher.rtps_publisher.value()?;
        if rtps_publisher_inner.writer_list.is_empty() {
            if self
                .inner
                .publisher_list
                .contains(&a_publisher.rtps_publisher)
            {
                a_publisher.rtps_publisher.delete();
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

    /// This operation creates a Subscriber with the desired QoS policies and attaches to it the specified SubscriberListener.
    /// If the specified QoS policies are not consistent, the operation will fail and no Subscriber will be created.
    /// The special value SUBSCRIBER_QOS_DEFAULT can be used to indicate that the Subscriber should be created with the
    /// default Subscriber QoS set in the factory. The use of this value is equivalent to the application obtaining the default
    /// Subscriber QoS by means of the operation get_default_subscriber_qos (2.2.2.2.1.21) and using the resulting QoS to create the
    /// Subscriber.
    /// The created Subscriber belongs to the DomainParticipant that is its factory.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    pub fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        // _a_listener: impl SubscriberListener,
        // _mask: StatusMask
    ) -> Option<Subscriber> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.inner
                .subscriber_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedReaderGroup);
        let new_subscriber_guid = GUID::new(guid_prefix, entity_id);
        let new_subscriber_qos = qos.unwrap_or(self.get_default_subscriber_qos());
        let new_subscriber = Box::new(RtpsSubscriber::new(
            new_subscriber_guid,
            new_subscriber_qos,
            None,
            0,
        ));
        let rtps_subscriber = self.inner.subscriber_list.add(new_subscriber)?;

        Some(Subscriber {
            parent_participant: self,
            rtps_subscriber,
        })
    }

    /// This operation deletes an existing Subscriber.
    /// A Subscriber cannot be deleted if it has any attached DataReader objects. If the delete_subscriber operation is called on a
    /// Subscriber with existing DataReader objects, it will return PRECONDITION_NOT_MET.
    /// The delete_subscriber operation must be called on the same DomainParticipant object used to create the Subscriber. If
    /// delete_subscriber is called on a different DomainParticipant, the operation will have no effect and it will return
    /// PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_subscriber(&self, a_subscriber: &Subscriber) -> ReturnCode<()> {
        let rtps_subscriber_inner = a_subscriber.rtps_subscriber.value()?;
        if rtps_subscriber_inner.reader_list.is_empty() {
            if self
                .inner
                .subscriber_list
                .contains(&a_subscriber.rtps_subscriber)
            {
                a_subscriber.rtps_subscriber.delete();
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

    /// This operation creates a Topic with the desired QoS policies and attaches to it the specified TopicListener.
    /// If the specified QoS policies are not consistent, the operation will fail and no Topic will be created.
    /// The special value TOPIC_QOS_DEFAULT can be used to indicate that the Topic should be created with the default Topic QoS
    /// set in the factory. The use of this value is equivalent to the application obtaining the default Topic QoS by means of the
    /// operation get_default_topic_qos (2.2.2.2.1.21) and using the resulting QoS to create the Topic.
    /// The created Topic belongs to the DomainParticipant that is its factory.
    /// The Topic is bound to a type described by the type_name argument. Prior to creating a Topic the type must have been
    /// registered with the Service. This is done using the register_type operation on a derived class of the TypeSupport interface as
    /// described in 2.2.2.3.6, TypeSupport Interface.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    pub fn create_topic<T: DDSType>(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        // _a_listener: impl TopicListener<T>,
        // _mask: StatusMask
    ) -> Option<Topic<T>> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.inner
                .topic_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedUnknown);
        let new_topic_guid = GUID::new(guid_prefix, entity_id);
        let new_topic_qos = qos.unwrap_or(self.get_default_topic_qos());
        let new_topic: Arc<RtpsTopic<T>> = Arc::new(RtpsTopic::new(
            new_topic_guid,
            topic_name.clone().into(),
            new_topic_qos,
            None,
            0,
        ));
        // discovery.insert_topic(&new_topic).ok()?;
        let rtps_topic = self.inner.topic_list.add(new_topic)?;

        Some(Topic {
            parent_participant: self,
            rtps_topic,
            marker: std::marker::PhantomData,
        })
    }

    /// This operation deletes a Topic.
    /// The deletion of a Topic is not allowed if there are any existing DataReader, DataWriter, ContentFilteredTopic, or MultiTopic
    /// objects that are using the Topic. If the delete_topic operation is called on a Topic with any of these existing objects attached to
    /// it, it will return PRECONDITION_NOT_MET.
    /// The delete_topic operation must be called on the same DomainParticipant object used to create the Topic. If delete_topic is
    /// called on a different DomainParticipant, the operation will have no effect and it will return PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_topic<T: DDSType>(&self, a_topic: &Topic<T>) -> ReturnCode<()> {
        let rtps_topic_inner = a_topic.rtps_topic.value()?;
        if self.inner.topic_list.contains(&a_topic.rtps_topic) {
            if Arc::strong_count(rtps_topic_inner) == 1 {
                // discovery.remove_topic(a_topic.value()?)?;
                a_topic.rtps_topic.delete();
                Ok(())
            } else {
                Err(ReturnCodes::PreconditionNotMet(
                    "Topic still attached to some data reader or data writer",
                ))
            }
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic not found in this participant",
            ))
        }
    }

    /// The operation find_topic gives access to an existing (or ready to exist) enabled Topic, based on its name. The operation takes
    /// as arguments the name of the Topic and a timeout.
    /// If a Topic of the same name already exists, it gives access to it, otherwise it waits (blocks the caller) until another mechanism
    /// creates it (or the specified timeout occurs). This other mechanism can be another thread, a configuration tool, or some other
    /// middleware service. Note that the Topic is a local object that acts as a ‘proxy’ to designate the global concept of topic.
    /// Middleware implementations could choose to propagate topics and make remotely created topics locally available.
    /// A Topic obtained by means of find_topic, must also be deleted by means of delete_topic so that the local resources can be
    /// released. If a Topic is obtained multiple times by means of find_topic or create_topic, it must also be deleted that same number
    /// of times using delete_topic.
    /// Regardless of whether the middleware chooses to propagate topics, the delete_topic operation deletes only the local proxy.
    /// If the operation times-out, a ‘nil’ value (as specified by the platform) is returned.
    pub fn find_topic<T: DDSType>(
        &self,
        _topic_name: &str,
        _timeout: Duration,
    ) -> Option<Topic<T>> {
        todo!()
    }

    /// The operation lookup_topicdescription gives access to an existing locally-created TopicDescription, based on its name. The
    /// operation takes as argument the name of the TopicDescription.
    /// If a TopicDescription of the same name already exists, it gives access to it, otherwise it returns a ‘nil’ value. The operation
    /// never blocks.
    /// The operation lookup_topicdescription may be used to locate any locally-created Topic, ContentFilteredTopic, and
    /// MultiTopic object.
    /// Unlike find_topic, the operation lookup_topicdescription searches only among the locally created topics. Therefore, it should
    /// never create a new TopicDescription. The TopicDescription returned by lookup_topicdescription does not require any extra
    /// deletion. It is still possible to delete the TopicDescription returned by lookup_topicdescription, provided it has no readers or
    /// writers, but then it is really deleted and subsequent lookups will fail.
    /// If the operation fails to locate a TopicDescription, a ‘nil’ value (as specified by the platform) is returned.
    pub fn lookup_topicdescription<T: DDSType>(&self, _name: &str) -> Option<Topic<T>> {
        todo!()
    }

    /// This operation allows access to the built-in Subscriber. Each DomainParticipant contains several built-in Topic objects as
    /// well as corresponding DataReader objects to access them. All these DataReader objects belong to a single built-in Subscriber.
    /// The built-in Topics are used to communicate information about other DomainParticipant, Topic, DataReader, and DataWriter
    /// objects. These built-in objects are described in 2.2.5, Built-in Topics.
    pub fn get_builtin_subscriber(&self) -> Subscriber {
        todo!()
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote domain participant. From that point
    /// onwards the Service will locally behave as if the remote participant did not exist. This means it will ignore any Topic,
    /// publication, or subscription that originates on that domain participant.
    /// This operation can be used, in conjunction with the discovery of remote participants offered by means of the
    /// “DCPSParticipant” built-in Topic, to provide, for example, access control. Application data can be associated with a
    /// DomainParticipant by means of the USER_DATA QoS policy. This application data is propagated as a field in the built-in
    /// topic and can be used by an application to implement its own access control policy. See 2.2.5, Built-in Topics for more details
    /// on the built-in topics.
    /// The domain participant to ignore is identified by the handle argument. This handle is the one that appears in the SampleInfo
    /// retrieved when reading the data-samples available for the built-in DataReader to the “DCPSParticipant” topic. The built-in
    /// DataReader is read with the same read/take operations used for any DataReader. These data-accessing operations are
    /// described in 2.2.2.5, Subscription Module.
    /// The ignore_participant operation is not required to be reversible. The Service offers no means to reverse it.
    /// Possible error codes returned in addition to the standard ones: OUT_OF_RESOURCES.
    pub fn ignore_participant(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    /// This operation allows an application to instruct the Service to locally ignore a Topic. This means it will locally ignore any
    /// publication or subscription to the Topic.
    /// This operation can be used to save local resources when the application knows that it will never publish or subscribe to data
    /// under certain topics.
    /// The Topic to ignore is identified by the handle argument. This handle is the one that appears in the SampleInfo retrieved when
    /// reading the data-samples from the built-in DataReader to the “DCPSTopic” topic.
    /// The ignore_topic operation is not required to be reversible. The Service offers no means to reverse it.
    /// Possible error codes returned in addition to the standard ones: OUT_OF_RESOURCES.
    pub fn ignore_topic(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote publication; a publication is defined by
    /// the association of a topic name, and user data and partition set on the Publisher (see the “DCPSPublication” built-in Topic in
    /// 2.2.5). After this call, any data written related to that publication will be ignored.
    /// The DataWriter to ignore is identified by the handle argument. This handle is the one that appears in the SampleInfo retrieved
    /// when reading the data-samples from the built-in DataReader to the “DCPSPublication” topic.
    /// The ignore_publication operation is not required to be reversible. The Service offers no means to reverse it.
    /// Possible error codes returned in addition to the standard ones: OUT_OF_RESOURCES.
    pub fn ignore_publication(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote subscription; a subscription is defined by
    /// the association of a topic name, and user data and partition set on the Subscriber (see the “DCPSSubscription” built-in Topic
    /// in 2.2.5). After this call, any data received related to that subscription will be ignored.
    /// The DataReader to ignore is identified by the handle argument. This handle is the one that appears in the SampleInfo
    /// retrieved when reading the data-samples from the built-in DataReader to the “DCPSSubscription” topic.
    /// The ignore_subscription operation is not required to be reversible. The Service offers no means to reverse it.
    /// Possible error codes returned in addition to the standard ones: OUT_OF_RESOURCES.
    pub fn ignore_subscription(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    /// This operation retrieves the domain_id used to create the DomainParticipant. The domain_id identifies the DDS domain to
    /// which the DomainParticipant belongs. As described in the introduction to 2.2.2.2.1 each DDS domain represents a separate
    /// data “communication plane” isolated from other domains
    pub fn get_domain_id(&self) -> DomainId {
        self.inner.participant.domain_id
    }

    /// This operation deletes all the entities that were created by means of the “create” operations on the DomainParticipant. That is,
    /// it deletes all contained Publisher, Subscriber, Topic, ContentFilteredTopic, and MultiTopic.
    /// Prior to deleting each contained entity, this operation will recursively call the corresponding delete_contained_entities
    /// operation on each contained entity (if applicable). This pattern is applied recursively. In this manner the operation
    /// delete_contained_entities on the DomainParticipant will end up deleting all the entities recursively contained in the
    /// DomainParticipant, that is also the DataWriter, DataReader, as well as the QueryCondition and ReadCondition objects
    /// belonging to the contained DataReaders.
    /// The operation will return PRECONDITION_NOT_MET if the any of the contained entities is in a state where it cannot be
    /// deleted.
    /// Once delete_contained_entities returns successfully, the application may delete the DomainParticipant knowing that it has no
    /// contained entities.
    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation manually asserts the liveliness of the DomainParticipant. This is used in combination with the LIVELINESS
    /// QoS policy (cf. 2.2.3, Supported QoS) to indicate to the Service that the entity remains active.
    /// This operation needs to only be used if the DomainParticipant contains DataWriter entities with the LIVELINESS set to
    /// MANUAL_BY_PARTICIPANT and it only affects the liveliness of those DataWriter entities. Otherwise, it has no effect.
    /// NOTE: Writing data via the write operation on a DataWriter asserts liveliness on the DataWriter itself and its
    /// DomainParticipant. Consequently the use of assert_liveliness is only needed if the application is not writing data regularly.
    /// Complete details are provided in 2.2.3.11, LIVELINESS
    pub fn assert_liveliness(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation sets a default value of the Publisher QoS policies which will be used for newly created Publisher entities in the
    /// case where the QoS policies are defaulted in the create_publisher operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value PUBLISHER_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values that would be used if the set_default_publisher_qos
    /// operation had never been called.
    pub fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.inner.default_publisher_qos.lock().unwrap() = qos;
        Ok(())
    }

    /// This operation retrieves the default value of the Publisher QoS, that is, the QoS policies which will be used for newly created
    /// Publisher entities in the case where the QoS policies are defaulted in the create_publisher operation.
    /// The values retrieved get_default_publisher_qos will match the set of values specified on the last successful call to
    /// set_default_publisher_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.inner.default_publisher_qos.lock().unwrap().clone()
    }

    /// This operation sets a default value of the Subscriber QoS policies that will be used for newly created Subscriber entities in the
    /// case where the QoS policies are defaulted in the create_subscriber operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value SUBSCRIBER_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values that would be used if the set_default_subscriber_qos
    /// operation had never been called.
    pub fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.inner.default_subscriber_qos.lock().unwrap() = qos;
        Ok(())
    }

    /// This operation retrieves the default value of the Subscriber QoS, that is, the QoS policies which will be used for newly created
    /// Subscriber entities in the case where the QoS policies are defaulted in the create_subscriber operation.
    /// The values retrieved get_default_subscriber_qos will match the set of values specified on the last successful call to
    /// set_default_subscriber_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.inner.default_subscriber_qos.lock().unwrap().clone()
    }

    /// This operation sets a default value of the Topic QoS policies which will be used for newly created Topic entities in the case
    /// where the QoS policies are defaulted in the create_topic operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value TOPIC_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be reset
    /// back to the initial values the factory would use, that is the values that would be used if the set_default_topic_qos operation
    /// had never been called.
    pub fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.inner.default_topic_qos.lock().unwrap() = qos;
        Ok(())
    }

    /// This operation retrieves the default value of the Topic QoS, that is, the QoS policies that will be used for newly created Topic
    /// entities in the case where the QoS policies are defaulted in the create_topic operation.
    /// The values retrieved get_default_topic_qos will match the set of values specified on the last successful call to
    /// set_default_topic_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported QoS.
    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.inner.default_topic_qos.lock().unwrap().clone()
    }

    /// This operation retrieves the list of DomainParticipants that have been discovered in the domain and that the application has not
    /// indicated should be “ignored” by means of the DomainParticipant ignore_participant operation.
    /// The operation may fail if the infrastructure does not locally maintain the connectivity information. In this case the operation
    /// will return UNSUPPORTED.
    pub fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }

    /// This operation retrieves information on a DomainParticipant that has been discovered on the network. The participant must
    /// be in the same domain as the participant on which this operation is invoked and must not have been “ignored” by means of the
    /// DomainParticipant ignore_participant operation.
    /// The participant_handle must correspond to such a DomainParticipant. Otherwise, the operation will fail and return
    /// PRECONDITION_NOT_MET.
    /// Use the operation get_discovered_participants to find the DomainParticipants that are currently discovered.
    /// The operation may also fail if the infrastructure does not hold the information necessary to fill in the participant_data. In this
    /// case the operation will return UNSUPPORTED.
    pub fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    /// This operation retrieves the list of Topics that have been discovered in the domain and that the application has not indicated
    /// should be “ignored” by means of the DomainParticipant ignore_topic operation.
    pub fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> ReturnCode<()> {
        todo!()
    }

    /// This operation retrieves information on a Topic that has been discovered on the network. The topic must have been created by
    /// a participant in the same domain as the participant on which this operation is invoked and must not have been “ignored” by
    /// means of the DomainParticipant ignore_topic operation.
    /// The topic_handle must correspond to such a topic. Otherwise, the operation will fail and return
    /// PRECONDITION_NOT_MET.
    /// Use the operation get_discovered_topics to find the topics that are currently discovered.
    /// The operation may also fail if the infrastructure does not hold the information necessary to fill in the topic_data. In this case
    /// the operation will return UNSUPPORTED.
    /// The operation may fail if the infrastructure does not locally maintain the connectivity information. In this case the operation
    /// will return UNSUPPORTED.
    pub fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    /// This operation checks whether or not the given a_handle represents an Entity that was created from the DomainParticipant.
    /// The containment applies recursively. That is, it applies both to entities (TopicDescription, Publisher, or Subscriber) created
    /// directly using the DomainParticipant as well as entities created using a contained Publisher, or Subscriber as the factory, and
    /// so forth.
    /// The instance handle for an Entity may be obtained from built-in topic data, from various statuses, or from the Entity operation
    /// get_instance_handle.
    pub fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
        todo!()
    }

    /// This operation returns the current value of the time that the service uses to time-stamp data-writes and to set the reception timestamp
    /// for the data-updates it receives.
    pub fn get_current_time(&self) -> ReturnCode<Time> {
        todo!()
    }
}

impl Entity for DomainParticipant {
    type Qos = DomainParticipantQos;
    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, _qos: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        Ok(self.inner.qos.lock().unwrap().clone())
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        let participant_inner = self.inner.clone();
        let rw_builtin_publisher = participant_inner.builtin_publisher.read().unwrap();
        let builtin_publisher = Publisher {
            parent_participant: self,
            rtps_publisher: MaybeValidRef(rw_builtin_publisher),
        };
        let guid_prefix = participant_inner.participant.entity.guid.prefix();
        let builtin_topic_guid = GUID::new(guid_prefix, ENTITYID_UNKNOWN);
        let builtin_topic_name = "BuildinTopic".to_string();
        let builtin_topic_qos = TopicQos::default();
        let builtin_topic = Arc::new(RtpsTopic::<SpdpDiscoveredParticipantData>::new(
            builtin_topic_guid,
            builtin_topic_name,
            builtin_topic_qos,
            None,
            0,
        ));
        let builtin_writer_guid =
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let mut builtin_writer_qos = DataWriterQos::default();
        builtin_writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;

        let builtin_writer = RtpsDataWriter::<SpdpDiscoveredParticipantData>::new_stateless(
            builtin_writer_guid,
            builtin_topic.clone(),
            builtin_writer_qos,
            None,
            0,
        );
        {
            let mut writer = builtin_writer.writer.lock().unwrap();
            if let Some(writer) = writer.try_get_stateless() {
                writer.reader_locator_add(Locator::new_udpv4(7400, [239,255,0,0]));
                // let change = writer.new_change(crate::types::ChangeKind::Alive, Some(vec![0,0,0,1,1,2,3,4]), None, [0;16]);
                // writer.writer_cache.add_change(change);
            }
        }
        let rtps_topic = self.inner.topic_list.add(builtin_topic.clone()).unwrap();

        let topic: Topic<SpdpDiscoveredParticipantData> = Topic {
            parent_participant: self,
            rtps_topic,
            marker: std::marker::PhantomData,
        };
        let builtin_writer_box: Box<dyn AnyRtpsWriter> = Box::new(builtin_writer);
        let rtps_datawriter= builtin_publisher.rtps_publisher.get().unwrap().writer_list.add(builtin_writer_box).unwrap();
        let data_writer = DataWriter {
            parent_publisher: &builtin_publisher,
            topic: &topic,
            rtps_datawriter: rtps_datawriter,
        };

        let key = BuiltInTopicKey([1, 2, 3]);
        let user_data = UserDataQosPolicy { value: vec![] };
        let dds_participant_data = ParticipantBuiltinTopicData { key, user_data };
        let participant_proxy = ParticipantProxy {
            domain_id: participant_inner.participant.domain_id,
            domain_tag: "".to_string(),
            protocol_version: participant_inner.participant.protocol_version,
            guid_prefix: guid_prefix,
            vendor_id: participant_inner.participant.vendor_id,
            expects_inline_qos: true,
            available_built_in_endpoints: BuiltInEndpointSet { value: 9 },
            // built_in_endpoint_qos:
            metatraffic_unicast_locator_list: participant_inner
                .metatraffic_transport
                .unicast_locator_list()
                .clone(),
            metatraffic_multicast_locator_list: participant_inner
                .metatraffic_transport
                .multicast_locator_list()
                .clone(),
            default_unicast_locator_list: vec![],
            default_multicast_locator_list: vec![],
            manual_liveliness_count: 8,
        };
        let lease_duration = DURATION_INFINITE;

        let data = SpdpDiscoveredParticipantData {
            dds_participant_data,
            participant_proxy,
            lease_duration,
        };
        data_writer.write_w_timestamp(data, None, TIME_INVALID).ok();




        
        if self.inner.enabled.load(atomic::Ordering::Acquire) == false {
            self.inner.enabled.store(true, atomic::Ordering::Release);

            let mut thread_list = self.thread_list.borrow_mut();
            let participant_inner = self.inner.clone();
            thread_list.push(std::thread::spawn(move || {
                while participant_inner.enabled.load(atomic::Ordering::Acquire) {
                    let participant = &participant_inner.participant;
                    let participant_guid_prefix = participant.entity.guid.prefix();
                    //let transport = &participant_inner.userdata_transport;
                    let transport = &participant_inner.metatraffic_transport;

                    //for publisher in participant_inner.publisher_list.iter() {
                    let publisher_rw = participant_inner.builtin_publisher.read().unwrap();
                    let publisher = publisher_rw.get().unwrap();
                    //if let Some(publisher) = publisher.read().unwrap().get() {
                    for writer in publisher.writer_list.iter() {
                        if let Some(writer) = writer.read().unwrap().get() {
                            let mut stateful_writer = writer.writer().lock().unwrap();
                            println!(
                                "last_change_sequence_number = {:?}",
                                stateful_writer.last_change_sequence_number
                            );
                            let destined_messages = stateful_writer.produce_messages();
                            RtpsMessageSender::send_cache_change_messages(
                                participant_guid_prefix,
                                transport.as_ref(),
                                destined_messages,
                            );
                        }
                    }
                    //}
                    //}
                    std::thread::sleep(std::time::Duration::from_secs(1))
                }
            }));
        }

        Ok(())
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.inner.participant.entity.guid.into())
    }
}

impl Drop for DomainParticipant {
    fn drop(&mut self) {
        self.inner.enabled.store(false, atomic::Ordering::Release);
        for thread in self.thread_list.borrow_mut().drain(..) {
            thread.join().ok();
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::rtps::messages::RtpsMessage;
//     use crate::rtps::transport::TransportResult;
//     use crate::rtps::types::Locator;
//     use crate::types::{Duration, TopicKind};

//     struct MockTransport;
//     impl Transport for MockTransport {
//         fn write(&self, _message: RtpsMessage, _destination_locator: &Locator) {
//             todo!()
//         }

//         fn read(&self) -> TransportResult<Option<(RtpsMessage, Locator)>> {
//             todo!()
//         }

//         fn unicast_locator_list(&self) -> &Vec<Locator> {
//             todo!()
//         }

//         fn multicast_locator_list(&self) -> &Vec<Locator> {
//             todo!()
//         }
//     }

//     #[test]
//     fn create_publisher() {
//         let participant = RtpsParticipant::new(
//             0,
//             DomainParticipantQos::default(),
//             MockTransport,
//             MockTransport,
//         )
//         .unwrap();

//         let publisher1_default_qos = participant.create_publisher(None).unwrap();
//         let publisher1_instance_handle = publisher1_default_qos.get_instance_handle().unwrap();

//         let mut qos = PublisherQos::default();
//         qos.partition.name = "Test".to_string();
//         let publisher2_custom_qos = participant.create_publisher(Some(qos.clone())).unwrap();
//         let publisher2_instance_handle = publisher2_custom_qos.get_instance_handle().unwrap();

//         // Test correct qos and instance handle
//         assert_eq!(
//             publisher1_default_qos.get_qos().unwrap(),
//             PublisherQos::default()
//         );
//         assert_eq!(
//             publisher1_instance_handle[0..12],
//             participant.inner.participant.entity.guid.prefix()
//         );
//         assert_eq!(publisher1_instance_handle[12..15], [0, 0, 0]);
//         assert_eq!(publisher1_instance_handle[15], 0x08);

//         assert_eq!(publisher2_custom_qos.get_qos().unwrap(), qos);
//         assert_eq!(
//             publisher2_instance_handle[0..12],
//             participant.inner.participant.entity.guid.prefix()
//         );
//         assert_eq!(publisher2_instance_handle[12..15], [0, 1, 0]);
//         assert_eq!(publisher2_instance_handle[15], 0x08);
//     }

//     #[test]
//     fn create_subscriber() {
//         let participant = RtpsParticipant::new(
//             0,
//             DomainParticipantQos::default(),
//             MockTransport,
//             MockTransport,
//         )
//         .unwrap();

//         let subscriber1_default_qos = participant.create_subscriber(None).unwrap();
//         let subscriber1_instance_handle = subscriber1_default_qos.get_instance_handle().unwrap();

//         let mut qos = SubscriberQos::default();
//         qos.partition.name = "Test".to_string();
//         let subscriber2_custom_qos = participant.create_subscriber(Some(qos.clone())).unwrap();
//         let subscriber2_instance_handle = subscriber2_custom_qos.get_instance_handle().unwrap();

//         // Test correct qos and instance handle
//         assert_eq!(
//             subscriber1_default_qos.get_qos().unwrap(),
//             SubscriberQos::default()
//         );
//         assert_eq!(
//             subscriber1_instance_handle[0..12],
//             participant.inner.participant.entity.guid.prefix()
//         );
//         assert_eq!(subscriber1_instance_handle[12..15], [0, 0, 0]);
//         assert_eq!(subscriber1_instance_handle[15], 0x09);

//         assert_eq!(subscriber2_custom_qos.get_qos().unwrap(), qos);
//         assert_eq!(
//             subscriber2_instance_handle[0..12],
//             participant.inner.participant.entity.guid.prefix()
//         );
//         assert_eq!(subscriber2_instance_handle[12..15], [0, 1, 0]);
//         assert_eq!(subscriber2_instance_handle[15], 0x09);
//     }

//     #[test]
//     fn create_topic() {
//         let participant = RtpsParticipant::new(
//             0,
//             DomainParticipantQos::default(),
//             MockTransport,
//             MockTransport,
//         )
//         .unwrap();

//         let topic1_default_qos = participant
//             .create_topic(
//                 "abc".to_string(),
//                 "TestType",
//                 TopicKind::WithKey,
//                 None,
//                 participant.get_endpoint_discovery(),
//             )
//             .unwrap();
//         let topic1_instance_handle = topic1_default_qos.get_instance_handle().unwrap();

//         let mut qos = TopicQos::default();
//         qos.deadline.period = Duration {
//             sec: 1,
//             nanosec: 10,
//         };
//         let topic2_custom_qos = participant
//             .create_topic(
//                 "def".to_string(),
//                 "TestType2",
//                 TopicKind::WithKey,
//                 Some(qos.clone()),
//                 participant.get_endpoint_discovery(),
//             )
//             .unwrap();
//         let topic2_instance_handle = topic2_custom_qos.get_instance_handle().unwrap();

//         // Test correct qos and instance handle
//         assert_eq!(topic1_default_qos.get_qos().unwrap(), TopicQos::default());
//         assert_eq!(
//             topic1_instance_handle[0..12],
//             participant.inner.participant.entity.guid.prefix()
//         );
//         assert_eq!(topic1_instance_handle[12..15], [0, 0, 0]);
//         assert_eq!(topic1_instance_handle[15], 0x00);

//         assert_eq!(topic2_custom_qos.get_qos().unwrap(), qos);
//         assert_eq!(
//             topic2_instance_handle[0..12],
//             participant.inner.participant.entity.guid.prefix()
//         );
//         assert_eq!(topic2_instance_handle[12..15], [0, 1, 0]);
//         assert_eq!(topic2_instance_handle[15], 0x00);
//     }

//     #[test]
//     fn enable_threads() {
//         let participant = RtpsParticipant::new(
//             0,
//             DomainParticipantQos::default(),
//             MockTransport,
//             MockTransport,
//         )
//         .unwrap();
//         participant.enable().unwrap();
//     }
// }
