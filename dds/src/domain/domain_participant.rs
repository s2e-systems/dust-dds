use std::sync::Mutex;

use crate::infrastructure::status::StatusMask;
use crate::topic::{Topic, TopicListener, TopicDescription};
use crate::subscription::{Subscriber, SubscriberListener};
use crate::publication::{Publisher, PublisherListener};
use crate::infrastructure::entity::{Entity, StatusCondition};
use crate::domain::domain_participant_listener::DomainParticipantListener;
use crate::builtin_topics::{TopicBuiltinTopicData, ParticipantBuiltinTopicData};

use rust_dds_interface::types::{DomainId, ReturnCode, Duration, InstanceHandle, Time, ReturnCodes};
use rust_dds_interface::protocol::ProtocolParticipant;
use rust_dds_interface::qos::{DomainParticipantQos, TopicQos, PublisherQos, SubscriberQos};

/// The DomainParticipant object plays several roles:
/// - It acts as a container for all other Entity objects.
/// - It acts as factory for the Publisher, Subscriber, Topic, and MultiTopic Entity objects.
/// - It represents the participation of the application on a communication plane that isolates applications running on the
/// same set of physical computers from each other. A domain establishes a “virtual network” linking all applications that
/// share the same domainId and isolating them from applications running on different domains. In this way, several
/// independent distributed applications can coexist in the same physical network without interfering, or even being aware
/// of each other.
/// - It provides administration services in the domain, offering operations that allow the application to ‘ignore’ locally any
/// information about a given participant (ignore_participant), publication (ignore_publication), subscription
/// (ignore_subscription), or topic (ignore_topic).
///
/// The following sub clauses explain all the operations in detail.
/// The following operations may be called even if the DomainParticipant is not enabled. Other operations will have the value
/// NOT_ENABLED if called on a disabled DomainParticipant:
/// - Operations defined at the base-class level namely, set_qos, get_qos, set_listener, get_listener, and enable.
/// - Factory methods: create_topic, create_publisher, create_subscriber, delete_topic, delete_publisher,
/// delete_subscriber
/// - Operations that access the status: get_statuscondition
pub struct DomainParticipant{
    domain_id: DomainId,
    qos: DomainParticipantQos,
    a_listener: Box<dyn DomainParticipantListener>,
    mask: StatusMask,
    default_publisher_qos: Mutex<PublisherQos>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    default_topic_qos: Mutex<TopicQos>,
    protocol_participant: Mutex<Box<dyn ProtocolParticipant>>,
}


impl DomainParticipant {
    /// This operation creates a new DomainParticipant object. The DomainParticipant signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no DomainParticipant will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation get_default_participant_qos (2.2.2.2.2.6) and using the resulting
    /// QoS to create the DomainParticipant.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    pub fn new (
        _domain_id: DomainId,
        _qos_list: DomainParticipantQos,
        _a_listener: impl DomainParticipantListener,
        _mask: StatusMask,
        _enabled: bool,
    ) ->  Option<DomainParticipant> {
        todo!()
        // use rust_rtps::transport::udp::UdpTransport;
        // use rust_rtps::protocol::RtpsProtocol;

        // let interface = "Ethernet";
        // let userdata_transport = UdpTransport::default_userdata_transport(domain_id, interface).unwrap();
        // let metatraffic_transport = UdpTransport::default_metatraffic_transport(domain_id, interface).unwrap();
        // let domain_tag = "".to_string();
        // let lease_duration = rust_dds_interface::types::Duration{sec: 30, nanosec: 0};

        // let name = "rtps";
        // let protocol = match name {         
        //     "rtps" => RtpsProtocol::new(domain_id, userdata_transport, metatraffic_transport, domain_tag, lease_duration),
        //     _ => panic!("Protocol not valid"),
        // };
        // // protocol_participant.lock().unwrap().enable().unwrap();
   
        // let new_participant = DomainParticipant(Arc::new(DomainParticipantImpl::new(domain_id, qos_list, a_listener, mask, protocol)));
        
        // if enabled {
        //     new_participant.enable().ok()?;
        // }

        // Some(new_participant)
    }

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
        _a_listener: impl PublisherListener,
        _mask: StatusMask
    ) -> Option<Publisher> {
        let _publisher_qos = match qos {
            Some(qos) => qos,
            None => self.default_publisher_qos.lock().unwrap().clone(),
        };

        let protocol_publisher = self.protocol_participant.lock().unwrap().create_publisher();

        Some(Publisher::new(self, protocol_publisher))
    }

    /// This operation deletes an existing Publisher.
    /// A Publisher cannot be deleted if it has any attached DataWriter objects. If delete_publisher is called on a Publisher with
    /// existing DataWriter object, it will return PRECONDITION_NOT_MET.
    /// The delete_publisher operation must be called on the same DomainParticipant object used to create the Publisher. If
    /// delete_publisher is called on a different DomainParticipant, the operation will have no effect and it will return
    /// PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_publisher(
        &self,
        a_publisher: &mut Publisher
    ) -> ReturnCode<()> {
        if std::ptr::eq(self, a_publisher.get_participant()?) {
            a_publisher.delete()
        } else {
            Err(ReturnCodes::PreconditionNotMet("Subscriber can only be deleted by its parent participant"))
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
        _a_listener: impl SubscriberListener,
        _mask: StatusMask
    ) -> Option<Subscriber> {
        let _subscriber_qos = match qos {
            Some(qos) => qos,
            None => self.default_subscriber_qos.lock().unwrap().clone(),
        };

        let protocol_subscriber = self.protocol_participant.lock().unwrap().create_subscriber();

        Some(Subscriber::new(self, protocol_subscriber))
    }

    /// This operation deletes an existing Subscriber.
    /// A Subscriber cannot be deleted if it has any attached DataReader objects. If the delete_subscriber operation is called on a
    /// Subscriber with existing DataReader objects, it will return PRECONDITION_NOT_MET.
    /// The delete_subscriber operation must be called on the same DomainParticipant object used to create the Subscriber. If
    /// delete_subscriber is called on a different DomainParticipant, the operation will have no effect and it will return
    /// PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_subscriber(
        &self,
        a_subscriber: &mut Subscriber,
    ) -> ReturnCode<()> {
        if std::ptr::eq(self, a_subscriber.get_participant()?) {
            a_subscriber.delete()
        } else {
            Err(ReturnCodes::PreconditionNotMet("Subscriber can only be deleted by its parent participant"))
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
    pub fn create_topic(
        &self,
        topic_name: String,
        type_name: String,
        qos_list: TopicQos,
        a_listener: impl TopicListener,
        mask: StatusMask
    ) -> Option<Topic> {
        // DomainParticipantImpl::create_topic(&self.0, topic_name, type_name, qos_list, a_listener, mask)
        todo!()
    }

    /// This operation deletes a Topic.
    /// The deletion of a Topic is not allowed if there are any existing DataReader, DataWriter, ContentFilteredTopic, or MultiTopic
    /// objects that are using the Topic. If the delete_topic operation is called on a Topic with any of these existing objects attached to
    /// it, it will return PRECONDITION_NOT_MET.
    /// The delete_topic operation must be called on the same DomainParticipant object used to create the Topic. If delete_topic is
    /// called on a different DomainParticipant, the operation will have no effect and it will return PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_topic(
        &self,
        a_topic: &Topic,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::delete_topic(&self.0, a_topic)
        todo!()
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
    pub fn find_topic(
        &self,
        topic_name: String,
        timeout: Duration,
    ) -> Option<Topic> {
        // DomainParticipantImpl::find_topic(&self.0, topic_name, timeout)
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
    pub fn lookup_topicdescription(
        &self,
        name: String,
    ) -> Option<&dyn TopicDescription> {
        // DomainParticipantImpl::lookup_topicdescription(&self.0, name)
        todo!()
    }

    /// This operation allows access to the built-in Subscriber. Each DomainParticipant contains several built-in Topic objects as
    /// well as corresponding DataReader objects to access them. All these DataReader objects belong to a single built-in Subscriber.
    /// The built-in Topics are used to communicate information about other DomainParticipant, Topic, DataReader, and DataWriter
    /// objects. These built-in objects are described in 2.2.5, Built-in Topics.
    pub fn get_builtin_subscriber(&self) -> Subscriber {
        // DomainParticipantImpl::get_builtin_subscriber(&self.0)
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
    pub fn ignore_participant(
        &self,
        handle: InstanceHandle
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::ignore_participant(&self.0, handle)
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
    pub fn ignore_topic(
        &self,
        handle: InstanceHandle
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::ignore_topic(&self.0, handle)
        todo!()
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote publication; a publication is defined by
    /// the association of a topic name, and user data and partition set on the Publisher (see the “DCPSPublication” built-in Topic in
    /// 2.2.5). After this call, any data written related to that publication will be ignored.
    /// The DataWriter to ignore is identified by the handle argument. This handle is the one that appears in the SampleInfo retrieved
    /// when reading the data-samples from the built-in DataReader to the “DCPSPublication” topic.
    /// The ignore_publication operation is not required to be reversible. The Service offers no means to reverse it.
    /// Possible error codes returned in addition to the standard ones: OUT_OF_RESOURCES.
    pub fn ignore_publication(
        &self,
        handle: InstanceHandle
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::ignore_publication(&self.0, handle)
        todo!()
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote subscription; a subscription is defined by
    /// the association of a topic name, and user data and partition set on the Subscriber (see the “DCPSSubscription” built-in Topic
    /// in 2.2.5). After this call, any data received related to that subscription will be ignored.
    /// The DataReader to ignore is identified by the handle argument. This handle is the one that appears in the SampleInfo
    /// retrieved when reading the data-samples from the built-in DataReader to the “DCPSSubscription” topic.
    /// The ignore_subscription operation is not required to be reversible. The Service offers no means to reverse it.
    /// Possible error codes returned in addition to the standard ones: OUT_OF_RESOURCES.
    pub fn ignore_subscription(
        &self,
        handle: InstanceHandle
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::ignore_subscription(&self.0, handle)
        todo!()
    }

    /// This operation retrieves the domain_id used to create the DomainParticipant. The domain_id identifies the DDS domain to
    /// which the DomainParticipant belongs. As described in the introduction to 2.2.2.2.1 each DDS domain represents a separate
    /// data “communication plane” isolated from other domains
    pub fn get_domain_id(&self) -> DomainId {
        // DomainParticipantImpl::get_domain_id(&self.0)
        todo!()
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
        // DomainParticipantImpl::assert_liveliness(&self.0)
        todo!()
    }

    /// This operation sets a default value of the Publisher QoS policies which will be used for newly created Publisher entities in the
    /// case where the QoS policies are defaulted in the create_publisher operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value PUBLISHER_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values that would be used if the set_default_publisher_qos
    /// operation had never been called.
    pub fn set_default_publisher_qos(
        &self,
        qos: PublisherQos,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::set_default_publisher_qos(&self.0, qos)
        todo!()
    }

    /// This operation retrieves the default value of the Publisher QoS, that is, the QoS policies which will be used for newly created
    /// Publisher entities in the case where the QoS policies are defaulted in the create_publisher operation.
    /// The values retrieved get_default_publisher_qos will match the set of values specified on the last successful call to
    /// set_default_publisher_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_publisher_qos(
        &self,
        qos_list: &mut PublisherQos,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_default_publisher_qos(&self.0, qos_list)
        todo!()
    }

    /// This operation sets a default value of the Subscriber QoS policies that will be used for newly created Subscriber entities in the
    /// case where the QoS policies are defaulted in the create_subscriber operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value SUBSCRIBER_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values that would be used if the set_default_subscriber_qos
    /// operation had never been called.
    pub fn set_default_subscriber_qos(
        &self,
        qos: SubscriberQos,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::set_default_subscriber_qos(&self.0, qos)
        todo!()
    }

    /// This operation retrieves the default value of the Subscriber QoS, that is, the QoS policies which will be used for newly created
    /// Subscriber entities in the case where the QoS policies are defaulted in the create_subscriber operation.
    /// The values retrieved get_default_subscriber_qos will match the set of values specified on the last successful call to
    /// set_default_subscriber_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_subscriber_qos(
        &self,
        qos_list: &mut SubscriberQos,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_default_subscriber_qos(&self.0, qos_list)
        todo!()
    }

    /// This operation sets a default value of the Topic QoS policies which will be used for newly created Topic entities in the case
    /// where the QoS policies are defaulted in the create_topic operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value TOPIC_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be reset
    /// back to the initial values the factory would use, that is the values that would be used if the set_default_topic_qos operation
    /// had never been called.
    pub fn set_default_topic_qos(
        &self,
        qos: TopicQos,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::set_default_topic_qos(&self.0, qos)
        todo!()
    }

    /// This operation retrieves the default value of the Topic QoS, that is, the QoS policies that will be used for newly created Topic
    /// entities in the case where the QoS policies are defaulted in the create_topic operation.
    /// The values retrieved get_default_topic_qos will match the set of values specified on the last successful call to
    /// set_default_topic_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported QoS.
    pub fn get_default_topic_qos(
        &self,
        qos_list: &mut TopicQos,
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_default_topic_qos(&self.0, qos_list)
        todo!()
    }

    /// This operation retrieves the list of DomainParticipants that have been discovered in the domain and that the application has not
    /// indicated should be “ignored” by means of the DomainParticipant ignore_participant operation.
    /// The operation may fail if the infrastructure does not locally maintain the connectivity information. In this case the operation
    /// will return UNSUPPORTED.
    pub fn get_discovered_participants(
        &self,
        participant_handles: &mut [InstanceHandle]
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_discovered_participants(&self.0, participant_handles)
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
        participant_data: ParticipantBuiltinTopicData,
        participant_handle: InstanceHandle
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_discovered_participant_data(&self.0, participant_data, participant_handle)
        todo!()
    }

    /// This operation retrieves the list of Topics that have been discovered in the domain and that the application has not indicated
    /// should be “ignored” by means of the DomainParticipant ignore_topic operation.
    pub fn get_discovered_topics(
        &self,
        topic_handles: &mut [InstanceHandle]
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_discovered_topics(&self.0, topic_handles)
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
        topic_data: TopicBuiltinTopicData,
        topic_handle: InstanceHandle
    ) -> ReturnCode<()> {
        // DomainParticipantImpl::get_discovered_topic_data(&self.0, topic_data, topic_handle)
        todo!()
    }

    /// This operation checks whether or not the given a_handle represents an Entity that was created from the DomainParticipant.
    /// The containment applies recursively. That is, it applies both to entities (TopicDescription, Publisher, or Subscriber) created
    /// directly using the DomainParticipant as well as entities created using a contained Publisher, or Subscriber as the factory, and
    /// so forth.
    /// The instance handle for an Entity may be obtained from built-in topic data, from various statuses, or from the Entity operation
    /// get_instance_handle.
    pub fn contains_entity(
        &self,
        a_handle: InstanceHandle
    ) -> bool {
        // DomainParticipantImpl::contains_entity(&self.0, a_handle)
        todo!()
    }

    /// This operation returns the current value of the time that the service uses to time-stamp data-writes and to set the reception timestamp
    /// for the data-updates it receives.
    pub fn get_current_time() -> ReturnCode<Time> {
        // DomainParticipantImpl::get_current_time()
        todo!()
    }

}

impl Entity for DomainParticipant
{
    type Qos = DomainParticipantQos;
    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, qos_list: Self::Qos) -> ReturnCode<()> {
        // DomainParticipantImpl::set_qos(&self.0, qos_list)
        todo!()
    }

    fn get_qos(&self, qos_list: &mut Self::Qos) -> ReturnCode<()> {
        // DomainParticipantImpl::get_qos(&self.0, qos_list)
        todo!()
    }

    fn set_listener(&self, a_listener: Self::Listener, mask: StatusMask) -> ReturnCode<()> {
        // DomainParticipantImpl::set_listener(&self.0, a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> Self::Listener {
        // DomainParticipantImpl::get_listener(&self.0)
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        // DomainParticipantImpl::get_statuscondition(&self.0)
        todo!()
    }

    fn get_status_changes(&self, ) -> StatusMask {
        // DomainParticipantImpl::get_status_changes(&self.0)
        todo!()
    }

    fn enable(&self, ) -> ReturnCode<()> {
        // DomainParticipantImpl::enable(&self.0)
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        // DomainParticipantImpl::get_instance_handle(&self.0)
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::listener::NoListener;
    use rust_dds_interface::protocol::{ProtocolEntity, ProtocolPublisher, ProtocolSubscriber};

    struct MockProtocolParticipant;

    impl ProtocolEntity for MockProtocolParticipant {
        fn enable(&self) -> ReturnCode<()> {
            todo!()
        }

        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }
    }
    impl ProtocolParticipant for MockProtocolParticipant {
        fn create_publisher(&mut self) -> Box<dyn ProtocolPublisher> {
            Box::new(MockProtocolPublisher)
        }

        fn delete_publisher(&mut self, _publisher: &Box<dyn ProtocolPublisher>) {
            todo!()
        }

        fn create_subscriber(&mut self) -> Box<dyn ProtocolSubscriber> {
            Box::new(MockProtocolSubscriber)
        }

        fn delete_subscriber(&mut self, _subscriber: &Box<dyn ProtocolSubscriber>) {
            todo!()
        }

        fn get_builtin_subscriber(&self) -> Box<dyn rust_dds_interface::protocol::ProtocolSubscriber> {
            todo!()
        }
    }

    struct MockProtocolPublisher;
    impl ProtocolEntity for MockProtocolPublisher {
        fn enable(&self) -> ReturnCode<()> {
            todo!()
        }

        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }
    }

    impl ProtocolPublisher for MockProtocolPublisher {
        fn create_writer(&mut self, _topic_kind: rust_dds_interface::types::TopicKind, _data_writer_qos: &rust_dds_interface::qos::DataWriterQos) -> Box<dyn rust_dds_interface::protocol::ProtocolWriter> {
            todo!()
        }

        fn delete_writer(&mut self, _writer: &Box<dyn rust_dds_interface::protocol::ProtocolWriter>) {
            todo!()
        }
    }

    struct MockProtocolSubscriber;
    impl ProtocolEntity for MockProtocolSubscriber {
        fn enable(&self) -> ReturnCode<()> {
            todo!()
        }

        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }
    }

    impl ProtocolSubscriber for MockProtocolSubscriber {
        fn create_reader(&mut self, _topic_kind: rust_dds_interface::types::TopicKind, _data_reader_qos: &rust_dds_interface::qos::DataReaderQos) -> Box<dyn rust_dds_interface::protocol::ProtocolReader> {
            todo!()
        }
    }


    #[test]
    fn create_delete_publisher() {
        let dp = DomainParticipant {
            domain_id: 1,
            qos: DomainParticipantQos::default(),
            a_listener: Box::new(NoListener),
            mask: 0,
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            protocol_participant: Mutex::new(Box::new(MockProtocolParticipant)),
        };

        let mut publisher = dp.create_publisher(None, NoListener, 0).unwrap();

        dp.delete_publisher(&mut publisher).unwrap();

        // Verify that second publisher delete returns Already Deleted error
        match dp.delete_publisher(&mut publisher) {
            Err(ReturnCodes::AlreadyDeleted("Publisher already deleted")) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn create_delete_subscriber() {
        let dp = DomainParticipant {
            domain_id: 1,
            qos: DomainParticipantQos::default(),
            a_listener: Box::new(NoListener),
            mask: 0,
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            protocol_participant: Mutex::new(Box::new(MockProtocolParticipant)),
        };

        let mut subscriber = dp.create_subscriber(None, NoListener, 0).unwrap();

        dp.delete_subscriber(&mut subscriber).unwrap();

        // Verify that second publisher delete returns Already Deleted error
        match dp.delete_subscriber(&mut subscriber) {
            Err(ReturnCodes::AlreadyDeleted("Subscriber already deleted")) => assert!(true),
            _ => assert!(false),
        }
    }
}