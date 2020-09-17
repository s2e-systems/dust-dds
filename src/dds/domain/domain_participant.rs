use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::dds::types::{StatusKind, ReturnCode, Duration, InstanceHandle, DomainId, Time, StatusMask};
use crate::dds::topic::topic::{Topic, TopicImpl};
use crate::dds::topic::topic_listener::TopicListener;
use crate::dds::topic::topic_description::TopicDescription;
use crate::dds::topic::qos::TopicQos;
use crate::dds::subscription::subscriber::{Subscriber, SubscriberImpl};
use crate::dds::subscription::subscriber::qos::SubscriberQos;
use crate::dds::subscription::subscriber_listener::SubscriberListener;
use crate::dds::publication::publisher::{Publisher, PublisherImpl};
use crate::dds::publication::publisher_listener::PublisherListener;
use crate::dds::publication::publisher::qos::PublisherQos;
use crate::dds::infrastructure::entity::{Entity, StatusCondition};
use crate::dds::domain::domain_participant_factory::DomainParticipantFactory;
use crate::dds::domain::domain_participant_listener::DomainParticipantListener;
use crate::dds::builtin_topics::{TopicBuiltinTopicData, ParticipantBuiltinTopicData};
use crate::dds::infrastructure::listener::NoListener;

use qos::DomainParticipantQos;


pub mod qos {
    use crate::dds::infrastructure::qos_policy::{
        UserDataQosPolicy, EntityFactoryQosPolicy
    };
    
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct DomainParticipantQos {
        pub user_data: UserDataQosPolicy,
        pub entity_factory: EntityFactoryQosPolicy,
    }
}

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
pub struct DomainParticipant(pub(crate) Arc<DomainParticipantImpl>);

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
        qos_list: PublisherQos,
        a_listener: impl PublisherListener,
        mask: StatusMask
    ) -> Option<Publisher> {
        DomainParticipantImpl::create_publisher(&self.0, qos_list, a_listener, mask)
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
        a_publisher: &Publisher
    ) -> ReturnCode {
        DomainParticipantImpl::delete_publisher(&self.0, a_publisher)
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
        qos_list: SubscriberQos,
        a_listener: impl SubscriberListener,
        mask: StatusMask
    ) -> Option<Subscriber> {
        DomainParticipantImpl::create_subscriber(&self.0, qos_list, a_listener, mask)
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
        a_subscriber: &Subscriber,
    ) -> ReturnCode {
        DomainParticipantImpl::delete_subscriber(&self.0, a_subscriber)
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
        DomainParticipantImpl::create_topic(&self.0, topic_name, type_name, qos_list, a_listener, mask)
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
    ) -> ReturnCode {
        DomainParticipantImpl::delete_topic(&self.0, a_topic)
    }

    /// The operation find_topic gives access to an existing (or ready to exist) enabled Topic, based on its name. The operation takes
    /// as arguments the name of the Topic and a timeout.
    /// If a Topic of the same name already exists, it gives access to it, otherwise it waits (blocks the caller) until another mechanism
    /// creates it (or the specified timeout occurs). This other mechanism can be another thread, a configuration tool, or some other
    /// middleware service. Note that the Topic is a local object10 that acts as a ‘proxy’ to designate the global concept of topic.
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
        DomainParticipantImpl::find_topic(&self.0, topic_name, timeout)
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
        DomainParticipantImpl::lookup_topicdescription(&self.0, name)
    }

    /// This operation allows access to the built-in Subscriber. Each DomainParticipant contains several built-in Topic objects as
    /// well as corresponding DataReader objects to access them. All these DataReader objects belong to a single built-in Subscriber.
    /// The built-in Topics are used to communicate information about other DomainParticipant, Topic, DataReader, and DataWriter
    /// objects. These built-in objects are described in 2.2.5, Built-in Topics.
    pub fn get_builtin_subscriber(&self) -> Subscriber {
        DomainParticipantImpl::get_builtin_subscriber(&self.0)
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
    ) -> ReturnCode{
        DomainParticipantImpl::ignore_participant(&self.0, handle)
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
    ) -> ReturnCode {
        DomainParticipantImpl::ignore_topic(&self.0, handle)
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
    ) -> ReturnCode {
        DomainParticipantImpl::ignore_publication(&self.0, handle)
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
    ) -> ReturnCode{
        DomainParticipantImpl::ignore_subscription(&self.0, handle)
    }

    /// This operation retrieves the domain_id used to create the DomainParticipant. The domain_id identifies the DDS domain to
    /// which the DomainParticipant belongs. As described in the introduction to 2.2.2.2.1 each DDS domain represents a separate
    /// data “communication plane” isolated from other domains
    pub fn get_domain_id(&self) -> DomainId {
        DomainParticipantImpl::get_domain_id(&self.0)
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
    pub fn delete_contained_entities(&self) -> ReturnCode {
        DomainParticipantImpl::delete_contained_entities(&self.0)
    }

    /// This operation manually asserts the liveliness of the DomainParticipant. This is used in combination with the LIVELINESS
    /// QoS policy (cf. 2.2.3, Supported QoS) to indicate to the Service that the entity remains active.
    /// This operation needs to only be used if the DomainParticipant contains DataWriter entities with the LIVELINESS set to
    /// MANUAL_BY_PARTICIPANT and it only affects the liveliness of those DataWriter entities. Otherwise, it has no effect.
    /// NOTE: Writing data via the write operation on a DataWriter asserts liveliness on the DataWriter itself and its
    /// DomainParticipant. Consequently the use of assert_liveliness is only needed if the application is not writing data regularly.
    /// Complete details are provided in 2.2.3.11, LIVELINESS
    pub fn assert_liveliness(&self) -> ReturnCode {
        DomainParticipantImpl::assert_liveliness(&self.0)
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
        qos_list: PublisherQos,
    ) -> ReturnCode {
        DomainParticipantImpl::set_default_publisher_qos(&self.0, qos_list)
    }

    /// This operation retrieves the default value of the Publisher QoS, that is, the QoS policies which will be used for newly created
    /// Publisher entities in the case where the QoS policies are defaulted in the create_publisher operation.
    /// The values retrieved get_default_publisher_qos will match the set of values specified on the last successful call to
    /// set_default_publisher_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_publisher_qos(
        &self,
        qos_list: &mut PublisherQos,
    ) -> ReturnCode {
        DomainParticipantImpl::get_default_publisher_qos(&self.0, qos_list)
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
        qos_list: SubscriberQos,
    ) -> ReturnCode {
        DomainParticipantImpl::set_default_subscriber_qos(&self.0, qos_list)
    }

    /// This operation retrieves the default value of the Subscriber QoS, that is, the QoS policies which will be used for newly created
    /// Subscriber entities in the case where the QoS policies are defaulted in the create_subscriber operation.
    /// The values retrieved get_default_subscriber_qos will match the set of values specified on the last successful call to
    /// set_default_subscriber_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_subscriber_qos(
        &self,
        qos_list: &mut SubscriberQos,
    ) -> ReturnCode {
        DomainParticipantImpl::get_default_subscriber_qos(&self.0, qos_list)
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
        qos_list: TopicQos,
    ) -> ReturnCode {
        DomainParticipantImpl::set_default_topic_qos(&self.0, qos_list)
    }

    /// This operation retrieves the default value of the Topic QoS, that is, the QoS policies that will be used for newly created Topic
    /// entities in the case where the QoS policies are defaulted in the create_topic operation.
    /// The values retrieved get_default_topic_qos will match the set of values specified on the last successful call to
    /// set_default_topic_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported QoS.
    pub fn get_default_topic_qos(
        &self,
        qos_list: &mut TopicQos,
    ) -> ReturnCode {
        DomainParticipantImpl::get_default_topic_qos(&self.0, qos_list)
    }

    /// This operation retrieves the list of DomainParticipants that have been discovered in the domain and that the application has not
    /// indicated should be “ignored” by means of the DomainParticipant ignore_participant operation.
    /// The operation may fail if the infrastructure does not locally maintain the connectivity information. In this case the operation
    /// will return UNSUPPORTED.
    pub fn get_discovered_participants(
        &self,
        participant_handles: &mut [InstanceHandle]
    ) -> ReturnCode {
        DomainParticipantImpl::get_discovered_participants(&self.0, participant_handles)
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
    ) -> ReturnCode {
        DomainParticipantImpl::get_discovered_participant_data(&self.0, participant_data, participant_handle)
    }

    /// This operation retrieves the list of Topics that have been discovered in the domain and that the application has not indicated
    /// should be “ignored” by means of the DomainParticipant ignore_topic operation.
    pub fn get_discovered_topics(
        &self,
        topic_handles: &mut [InstanceHandle]
    ) -> ReturnCode {
        DomainParticipantImpl::get_discovered_topics(&self.0, topic_handles)
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
    ) -> ReturnCode {
        DomainParticipantImpl::get_discovered_topic_data(&self.0, topic_data, topic_handle)
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
        DomainParticipantImpl::contains_entity(&self.0, a_handle)
    }

    /// This operation returns the current value of the time that the service uses to time-stamp data-writes and to set the reception timestamp
    /// for the data-updates it receives.
    pub fn get_current_time(
        &self,
        current_time: Time,
    ) -> ReturnCode {
        DomainParticipantImpl::get_current_time(&self.0, current_time)
    }

}

impl Entity for DomainParticipant
{
    type Qos = DomainParticipantQos;
    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, qos_list: Self::Qos) -> ReturnCode {
        DomainParticipantImpl::set_qos(&self.0, qos_list)
    }

    fn get_qos(&self, qos_list: &mut Self::Qos) -> ReturnCode {
        DomainParticipantImpl::get_qos(&self.0, qos_list)
    }

    fn set_listener(&self, a_listener: Self::Listener, mask: &[StatusKind]) -> ReturnCode {
        DomainParticipantImpl::set_listener(&self.0, a_listener, mask)
    }

    fn get_listener(&self, ) -> Self::Listener {
        DomainParticipantImpl::get_listener(&self.0)
    }

    fn get_statuscondition(&self, ) -> crate::dds::infrastructure::entity::StatusCondition {
        DomainParticipantImpl::get_statuscondition(&self.0)
    }

    fn get_status_changes(&self, ) -> StatusKind {
        DomainParticipantImpl::get_status_changes(&self.0)
    }

    fn enable(&self, ) -> ReturnCode {
        DomainParticipantImpl::enable(&self.0)
    }

    fn get_instance_handle(&self, ) -> InstanceHandle {
        DomainParticipantImpl::get_instance_handle(&self.0)
    }
}

impl Drop for DomainParticipant {
    fn drop(&mut self) {
        if Arc::strong_count(&self.0) == 1 {
            DomainParticipantFactory::get_instance().remove_participant_reference(&Arc::downgrade(&self.0));
        }
    }
}

pub struct DomainParticipantImpl{
    domain_id: DomainId,
    qos: DomainParticipantQos,
    a_listener: Box<dyn DomainParticipantListener>,
    mask: StatusMask,
    publisher_list: Mutex<Vec<Arc<PublisherImpl>>>,
    publisher_default_qos: Mutex<PublisherQos>,
    subscriber_list: Mutex<Vec<Arc<SubscriberImpl>>>,
    subscriber_default_qos: Mutex<SubscriberQos>,
    topic_list: Mutex<Vec<Arc<TopicImpl>>>,
    topic_default_qos: Mutex<TopicQos>
}

impl DomainParticipantImpl{
    pub(crate) fn create_publisher(
        this: &Arc<DomainParticipantImpl>,
        _qos_list: PublisherQos,
        _a_listener: impl PublisherListener,
        _mask: StatusMask
    ) -> Option<Publisher> {
        let publisher_impl = Arc::new(PublisherImpl::new(Arc::downgrade(this)));
        let publisher = Publisher(Arc::downgrade(&publisher_impl));

        this.publisher_list.lock().ok()?.push(publisher_impl);

        Some(publisher)
    }

    pub(crate) fn delete_publisher(
        this: &Arc<DomainParticipantImpl>,
        a_publisher: &Publisher
    ) -> ReturnCode {
        // TODO: Shouldn't be deleted if it still contains entities but can't yet be done because the publisher is not implemented
        let mut publisher_list = this.publisher_list.lock().unwrap();
        let index = publisher_list.iter().position(|x| std::ptr::eq(x.as_ref(), a_publisher.0.upgrade().unwrap().as_ref())).unwrap();
        publisher_list.swap_remove(index);
        ReturnCode::Ok
    }

    pub(crate) fn create_subscriber(
        this: &Arc<DomainParticipantImpl>,
        _qos_list: SubscriberQos,
        _a_listener: impl SubscriberListener,
        _mask: StatusMask
    ) -> Option<Subscriber> {
        let subscriber_impl = Arc::new(SubscriberImpl::new(Arc::downgrade(this)));
        let subscriber = Subscriber(Arc::downgrade(&subscriber_impl));

        this.subscriber_list.lock().ok()?.push(subscriber_impl);

        Some(subscriber)
    }

    pub(crate) fn delete_subscriber(
        this: &Arc<DomainParticipantImpl>,
        a_subscriber: &Subscriber,
    ) -> ReturnCode {
        // TODO: Shouldn't be deleted if it still contains entities but can't yet be done because the subscriber is not implemented
        let mut subscriber_list = this.subscriber_list.lock().unwrap();
        let index = subscriber_list.iter().position(|x| std::ptr::eq(x.as_ref(), a_subscriber.0.upgrade().unwrap().as_ref())).unwrap();
        subscriber_list.swap_remove(index);
        ReturnCode::Ok
    }

    pub(crate) fn create_topic(
        this: &Arc<DomainParticipantImpl>,
        topic_name: String,
        type_name: String,
        _qos_list: TopicQos,
        _a_listener: impl TopicListener,
        _mask: StatusMask
    ) -> Option<Topic> {
        let topic_impl = Arc::new(TopicImpl::new(Arc::downgrade(this), topic_name, type_name));
        let topic = Topic(Arc::downgrade(&topic_impl));

        this.topic_list.lock().ok()?.push(topic_impl);

        Some(topic)
    }

    pub(crate) fn delete_topic(
        this: &Arc<DomainParticipantImpl>,
        a_topic: &Topic,
    ) -> ReturnCode {
        // TODO: Shouldn't be deleted if there are any existing DataReader, DataWriter, ContentFilteredTopic, or MultiTopic
        // objects that are using the Topic. It can't yet be done because the functionality is not implemented
        let mut topic_list = this.topic_list.lock().unwrap();
        let index = topic_list.iter().position(|x| std::ptr::eq(x.as_ref(), a_topic.0.upgrade().unwrap().as_ref())).unwrap();
        topic_list.swap_remove(index);
        ReturnCode::Ok
    }

    pub(crate) fn find_topic(
        _this: &Arc<DomainParticipantImpl>,
        _topic_name: String,
        _timeout: Duration,
    ) -> Option<Topic> {
        todo!()
    }

    pub(crate) fn lookup_topicdescription(
        _this: &Arc<DomainParticipantImpl>,
        _name: String,
    ) -> Option<&dyn TopicDescription> {
        todo!()
    }

    pub(crate) fn get_builtin_subscriber(_this: &Arc<DomainParticipantImpl>,) -> Subscriber {
        todo!()
    }

    pub(crate) fn ignore_participant(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub(crate) fn ignore_topic(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub(crate) fn ignore_publication(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub(crate) fn ignore_subscription(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub(crate) fn get_domain_id(this: &Arc<DomainParticipantImpl>) -> DomainId {
        this.domain_id
    }

    pub(crate) fn delete_contained_entities(_this: &Arc<DomainParticipantImpl>) -> ReturnCode {
        todo!()   
    }

    pub(crate) fn assert_liveliness(_this: &Arc<DomainParticipantImpl>) -> ReturnCode {
        todo!()   
    }

    pub(crate) fn set_default_publisher_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: PublisherQos,
    ) -> ReturnCode {
        *this.publisher_default_qos.lock().unwrap() = qos;
        ReturnCode::Ok
    }

    pub(crate) fn get_default_publisher_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: &mut PublisherQos,
    ) -> ReturnCode {
        qos.clone_from(&this.publisher_default_qos.lock().unwrap());
        ReturnCode::Ok
    }

    pub(crate) fn set_default_subscriber_qos(
        _this: &Arc<DomainParticipantImpl>,
        _qos_list: SubscriberQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_default_subscriber_qos(
        _this: &Arc<DomainParticipantImpl>,
        _qos_list: &mut SubscriberQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn set_default_topic_qos(
        _this: &Arc<DomainParticipantImpl>,
        _qos_list: TopicQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_default_topic_qos(
        _this: &Arc<DomainParticipantImpl>,
        _qos_list: &mut TopicQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_discovered_participants(
        _this: &Arc<DomainParticipantImpl>,
        _participant_handles: &mut [InstanceHandle]
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_discovered_participant_data(
        _this: &Arc<DomainParticipantImpl>,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_discovered_topics(
        _this: &Arc<DomainParticipantImpl>,
        _topic_handles: &mut [InstanceHandle]
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_discovered_topic_data(
        _this: &Arc<DomainParticipantImpl>,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn contains_entity(
        _this: &Arc<DomainParticipantImpl>,
        _a_handle: InstanceHandle
    ) -> bool {
        todo!()
    }

    pub(crate) fn get_current_time(
        _this: &Arc<DomainParticipantImpl>,
        _current_time: Time,
    ) -> ReturnCode {
        todo!()
    }

    //////////////// Entity trait methods
    pub(crate) fn set_qos(_this: &Arc<DomainParticipantImpl>, _qos_list: DomainParticipantQos) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_qos(_this: &Arc<DomainParticipantImpl>, _qos_list: &mut DomainParticipantQos) -> ReturnCode {
        todo!()
    }

    pub(crate) fn set_listener(_this: &Arc<DomainParticipantImpl>, _a_listener: Box<dyn DomainParticipantListener>, _mask: &[StatusKind]) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_listener(_this: &Arc<DomainParticipantImpl>, ) -> Box<dyn DomainParticipantListener> {
        todo!()
    }

    pub(crate) fn get_statuscondition(_this: &Arc<DomainParticipantImpl>) -> StatusCondition {
        todo!()
    }

    pub(crate) fn get_status_changes(_this: &Arc<DomainParticipantImpl>) -> StatusKind {
        todo!()
    }

    pub(crate) fn enable(_this: &Arc<DomainParticipantImpl>) -> ReturnCode {
        //TODO: This is to prevent the ParticipantFactory test from panicking
        ReturnCode::Ok
    }

    pub(crate) fn get_instance_handle(_this: &Arc<DomainParticipantImpl>) -> InstanceHandle {
        todo!()
    }

    //////////////// From here on are the functions that do not belong to the standard API
    pub(crate) fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        a_listener: impl DomainParticipantListener,
        mask: StatusMask,
    ) -> Self {
        
        if !Any::is::<NoListener>(&a_listener) {
            println!("TODO: Use the real listener")
        }

        Self {
            domain_id,
            qos,
            a_listener: Box::new(a_listener),
            mask,
            publisher_list: Mutex::new(Vec::new()),
            publisher_default_qos: Mutex::new(PublisherQos::default()),
            subscriber_list: Mutex::new(Vec::new()),
            subscriber_default_qos: Mutex::new(SubscriberQos::default()),
            topic_list: Mutex::new(Vec::new()),
            topic_default_qos: Mutex::new(TopicQos::default()),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::infrastructure::listener::NoListener;

    #[test]
    fn create_publisher() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0));

        {
            assert_eq!(domain_participant_impl.publisher_list.lock().unwrap().len(), 0);
            let _publisher = DomainParticipantImpl::create_publisher(&domain_participant_impl,PublisherQos::default(), NoListener, 0);
            assert_eq!(domain_participant_impl.publisher_list.lock().unwrap().len(), 1);
        }

        assert_eq!(domain_participant_impl.publisher_list.lock().unwrap().len(), 0);
    }

    #[test]
    fn create_subscriber() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0));

        {
            assert_eq!(domain_participant_impl.subscriber_list.lock().unwrap().len(), 0);
            let _subscriber = DomainParticipantImpl::create_subscriber(&domain_participant_impl,SubscriberQos::default(), NoListener, 0);
            assert_eq!(domain_participant_impl.subscriber_list.lock().unwrap().len(), 1);
        }

        assert_eq!(domain_participant_impl.subscriber_list.lock().unwrap().len(), 0);
    }

    #[test]
    fn create_topic() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0));

        {
            assert_eq!(domain_participant_impl.topic_list.lock().unwrap().len(), 0);
            let _topic = DomainParticipantImpl::create_topic(&domain_participant_impl,"name".to_string(), "type".to_string(), TopicQos::default(), NoListener, 0);
            assert_eq!(domain_participant_impl.topic_list.lock().unwrap().len(), 1);
        }

        assert_eq!(domain_participant_impl.topic_list.lock().unwrap().len(), 0);
    }
}