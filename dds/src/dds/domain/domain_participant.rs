use std::time::Instant;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        dds::{
            any_topic_listener::AnyTopicListener,
            dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
            nodes::{
                DataReaderNode, DataWriterNode, DomainParticipantNode, SubscriberNode,
                SubscriberNodeKind, TopicNode, TopicNodeKind,
            },
            status_listener::ListenerTriggerKind,
        },
        rtps::types::GuidPrefix,
        rtps_udp_psm::udp_transport::UdpTransportRead,
        utils::condvar::DdsCondvar,
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::{
            InconsistentTopicStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
            SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    publication::{publisher::Publisher, publisher_listener::PublisherListener},
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
    topic_definition::{
        topic::Topic,
        topic_listener::TopicListener,
        type_support::{dds_serialize, DdsSerializedKey, DdsType},
    },
};

use super::{
    dcps_service::receive_builtin_message,
    domain_participant_factory::{
        DomainId, THE_DDS_DOMAIN_PARTICIPANT_FACTORY, THE_PARTICIPANT_FACTORY,
    },
    domain_participant_listener::DomainParticipantListener,
};

/// The [`DomainParticipant`] represents the participation of the application on a communication plane that isolates applications running on the
/// same set of physical computers from each other. A domain establishes a “virtual network” linking all applications that
/// share the same domain_id and isolating them from applications running on different domains. In this way, several
/// independent distributed applications can coexist in the same physical network without interfering, or even being aware
/// of each other.
///
/// The [`DomainParticipant`] object plays several roles:
/// - It acts as a container for all other Entity objects
/// - It acts as factory for the [`Publisher`], [`Subscriber`] and [`Topic`] Entity objects
/// - It provides administration services in the domain, offering operations that allow the application to ‘ignore’ locally any
/// information about a given participant ([`DomainParticipant::ignore_participant()`]), publication ([`DomainParticipant::ignore_publication()`]), subscription
/// ([`DomainParticipant::ignore_subscription()`]), or topic ([`DomainParticipant::ignore_topic()`]).
///
/// The following operations may be called even if the [`DomainParticipant`] is not enabled. Other operations will return a NotEnabled error if called on a disabled [`DomainParticipant`]:
/// - Operations defined at the base-class level namely, [`DomainParticipant::set_qos()`], [`DomainParticipant::get_qos()`], [`DomainParticipant::set_listener()`], and [`DomainParticipant::enable()`].
/// - Factory methods: [`DomainParticipant::create_topic()`], [`DomainParticipant::create_publisher()`], [`DomainParticipant::create_subscriber()`], [`DomainParticipant::delete_topic()`], [`DomainParticipant::delete_publisher()`],
/// [`DomainParticipant::delete_subscriber()`]
/// - Operations that access the status: [`DomainParticipant::get_statuscondition()`]
#[derive(PartialEq, Eq, Debug)]
pub struct DomainParticipant(DomainParticipantNode);

impl DomainParticipant {
    pub(crate) fn new(node: DomainParticipantNode) -> Self {
        Self(node)
    }

    pub(crate) fn node(&self) -> &DomainParticipantNode {
        &self.0
    }
}

impl Drop for DomainParticipant {
    fn drop(&mut self) {
        THE_PARTICIPANT_FACTORY.delete_participant(self).ok();
    }
}

impl DomainParticipant {
    /// This operation creates a [`Publisher`] with the desired QoS policies and attaches to it the specified [`PublisherListener`].
    /// If the specified QoS policies are not consistent, the operation will fail and no [`Publisher`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the Publisher should be created with the default
    /// Publisher QoS set in the factory. The use of this value is equivalent to the application obtaining the default Publisher QoS by
    /// means of the operation [`DomainParticipant::get_default_publisher_qos()`] and using the resulting QoS to create the [`Publisher`].
    /// The created [`Publisher`] belongs to the [`DomainParticipant`] that is its factory.
    /// In case of failure, the operation will return an error and no [`Publisher`] will be created.
    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<Publisher> {
        let publisher = self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::create_publisher(dp, qos)
        })?;

        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.add_publisher_listener(
            publisher.guid(),
            a_listener,
            mask,
        );

        Ok(Publisher::new(publisher))
    }

    /// This operation deletes an existing [`Publisher`].
    /// A [`Publisher`] cannot be deleted if it has any attached [`DataWriter`](crate::publication::data_writer::DataWriter) objects. If [`DomainParticipant::delete_publisher()`]
    /// is called on a [`Publisher`] with existing [`DataWriter`](crate::publication::data_writer::DataWriter) objects, it will return a
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError) error.
    /// The [`DomainParticipant::delete_publisher()`] operation must be called on the same [`DomainParticipant`] object used to create the [`Publisher`].
    /// If [`DomainParticipant::delete_publisher()`] is called on a different [`DomainParticipant`], the operation will have no effect and it will return
    /// a PreconditionNotMet error.
    pub fn delete_publisher(&self, a_publisher: &Publisher) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::delete_publisher(
                dp,
                a_publisher.node().guid(),
            )
        })?;

        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.delete_publisher_listener(&a_publisher.node().guid());

        Ok(())
    }

    /// This operation creates a [`Subscriber`] with the desired QoS policies and attaches to it the specified [`SubscriberListener`].
    /// If the specified QoS policies are not consistent, the operation will fail and no [`Subscriber`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`Subscriber`] should be created with the
    /// default Subscriber QoS set in the factory. The use of this value is equivalent to the application obtaining the default
    /// Subscriber QoS by means of the operation [`Self::get_default_subscriber_qos()`] and using the resulting QoS to create the
    /// [`Subscriber`].
    /// The created [`Subscriber`] belongs to the [`DomainParticipant`] that is its factory.
    /// In case of failure, the operation will return an error and no [`Subscriber`] will be created.
    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<Subscriber> {
        let subscriber = self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::create_subscriber(dp, qos)
        })?;

        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.add_subscriber_listener(
            subscriber.guid(),
            a_listener,
            mask,
        );

        Ok(Subscriber::new(SubscriberNodeKind::UserDefined(subscriber)))
    }

    /// This operation deletes an existing [`Subscriber`].
    /// A [`Subscriber`] cannot be deleted if it has any attached [`DataReader`](crate::subscription::data_reader::DataReader) objects. If the [`DomainParticipant::delete_subscriber()`] operation is called on a
    /// [`Subscriber`] with existing [`DataReader`](crate::subscription::data_reader::DataReader) objects, it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// The [`DomainParticipant::delete_subscriber()`] operation must be called on the same [`DomainParticipant`] object used to create the Subscriber. If
    /// it is called on a different [`DomainParticipant`], the operation will have no effect and it will return
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    pub fn delete_subscriber(&self, a_subscriber: &Subscriber) -> DdsResult<()> {
        match a_subscriber.node() {
            SubscriberNodeKind::Builtin(_) => (),
            SubscriberNodeKind::UserDefined(s) => {
                self.call_participant_mut_method(|dp| {
                    crate::implementation::behavior::domain_participant::delete_subscriber(
                        dp,
                        s.guid(),
                    )
                })?;

                THE_DDS_DOMAIN_PARTICIPANT_FACTORY.delete_subscriber_listener(&s.guid());
            }
            SubscriberNodeKind::Listener(_) => (),
        }

        Ok(())
    }

    /// This operation creates a [`Topic`] with the desired QoS policies and attaches to it the specified [`TopicListener`].
    /// If the specified QoS policies are not consistent, the operation will fail and no [`Topic`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`Topic`] should be created with the default Topic QoS
    /// set in the factory. The use of this value is equivalent to the application obtaining the default Topic QoS by means of the
    /// operation [`DomainParticipant::get_default_topic_qos`] and using the resulting QoS to create the [`Topic`].
    /// The created [`Topic`] belongs to the [`DomainParticipant`] that is its factory.
    /// The [`Topic`] is bound to a type specified by the generic type parameter 'Foo'. Only types which implement
    /// [`DdsType`] and have a `'static` lifetime can be associated to a [`Topic`].
    /// In case of failure, the operation will return an error and no [`Topic`] will be created.
    pub fn create_topic<Foo>(
        &self,
        topic_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListener<Foo = Foo> + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<Topic<Foo>>
    where
        Foo: DdsType + 'static,
    {
        let topic = self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::create_topic(
                dp,
                topic_name,
                Foo::type_name(),
                qos,
            )
        })?;

        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.add_topic_listener(
            topic.guid(),
            a_listener.map::<Box<dyn AnyTopicListener + Send + Sync>, _>(|l| Box::new(l)),
            mask,
        );

        Ok(Topic::new(TopicNodeKind::UserDefined(topic)))
    }

    /// This operation deletes a [`Topic`].
    /// The deletion of a [`Topic`] is not allowed if there are any existing [`DataReader`](crate::subscription::data_reader::DataReader) or [`DataWriter`](crate::publication::data_writer::DataWriter)
    /// objects that are using the [`Topic`]. If the [`DomainParticipant::delete_topic()`] operation is called on a [`Topic`] with any of these existing objects attached to
    /// it, it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// The [`DomainParticipant::delete_topic()`] operation must be called on the same [`DomainParticipant`] object used to create the [`Topic`]. If [`DomainParticipant::delete_topic()`] is
    /// called on a different [`DomainParticipant`], the operation will have no effect and it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    pub fn delete_topic<Foo>(&self, a_topic: &Topic<Foo>) -> DdsResult<()> {
        match &a_topic.node() {
            TopicNodeKind::UserDefined(t) => {
                self.call_participant_mut_method(|dp| {
                    crate::implementation::behavior::domain_participant::delete_topic(dp, t.guid())
                })?;
                THE_DDS_DOMAIN_PARTICIPANT_FACTORY.delete_topic_listener(&t.guid());
            }
            TopicNodeKind::Listener(_) => todo!(),
        }

        Ok(())
    }

    /// This operation gives access to an existing (or ready to exist) enabled [`Topic`], based on its name. The operation takes
    /// as arguments the name of the [`Topic`], a timeout and the type as a generic type argument `Foo`.
    /// If a [`Topic`] of the same name and type already exists, it gives access to it, otherwise it waits (blocks the caller) until another mechanism
    /// creates it (or the specified timeout occurs). This other mechanism can be another thread, a configuration tool, or some other
    /// middleware service. Note that the [`Topic`] is a local object that acts as a ‘proxy’ to designate the global concept of topic.
    /// Middleware implementations could choose to propagate topics and make remotely created topics locally available.
    /// A [`Topic`] obtained by means of [`DomainParticipant::find_topic()`], must also be deleted by means of [`DomainParticipant::delete_topic()`] so that the local resources can be
    /// released. If a [`Topic`] is obtained multiple times by means of [`DomainParticipant::find_topic()`] or [`DomainParticipant::create_topic()`], it must also be deleted that same number
    /// of times using [`DomainParticipant::delete_topic()`].
    /// Regardless of whether the middleware chooses to propagate topics, the [`DomainParticipant::delete_topic()`] operation deletes only the local proxy.
    /// If the operation times-out, a [`DdsError::Timeout`](crate::infrastructure::error::DdsError) error is returned.
    pub fn find_topic<Foo>(&self, topic_name: &str, timeout: Duration) -> DdsResult<Topic<Foo>>
    where
        Foo: DdsType,
    {
        let start_time = Instant::now();

        while start_time.elapsed() < std::time::Duration::from(timeout) {
            if let Some(topic) = self.call_participant_mut_method(|dp| {
                Ok(
                    crate::implementation::behavior::domain_participant::find_topic(
                        dp,
                        topic_name,
                        Foo::type_name(),
                    ),
                )
            })? {
                return Ok(Topic::new(TopicNodeKind::UserDefined(topic)));
            }
        }

        Err(DdsError::Timeout)
    }

    /// This operation gives access to an existing locally-created [`Topic`], based on its name and type. The
    /// operation takes as argument the name of the [`Topic`] and the type as a generic type argument `Foo`.
    /// If a [`Topic`] of the same name already exists, it gives access to it, otherwise it returns a [`None`] value. The operation
    /// never blocks.
    /// The operation [`DomainParticipant::lookup_topicdescription()`] may be used to locate any locally-created [`Topic`].
    /// Unlike [`DomainParticipant::find_topic()`], the operation [`DomainParticipant::lookup_topicdescription()`] searches only among the locally created topics. Therefore, it should
    /// never create a new [`Topic`]. The [`Topic`] returned by [`DomainParticipant::lookup_topicdescription()`] does not require any extra
    /// deletion. It is still possible to delete the [`Topic`] returned by [`DomainParticipant::lookup_topicdescription()`], provided it has no readers or
    /// writers, but then it is really deleted and subsequent lookups will fail.
    /// If the operation fails to locate a [`Topic`], the operation succeeds and a [`None`] value is returned.
    pub fn lookup_topicdescription<Foo>(&self, topic_name: &str) -> DdsResult<Option<Topic<Foo>>>
    where
        Foo: DdsType,
    {
        self.call_participant_method(|dp| {
            Ok(
                crate::implementation::behavior::domain_participant::lookup_topicdescription(
                    dp,
                    topic_name,
                    Foo::type_name(),
                )?
                .map(|x| Topic::new(TopicNodeKind::UserDefined(x))),
            )
        })
    }

    /// This operation allows access to the built-in [`Subscriber`]. Each [`DomainParticipant`] contains several built-in [`Topic`] objects as
    /// well as corresponding [`DataReader`](crate::subscription::data_reader::DataReader) objects to access them. All these [`DataReader`](crate::subscription::data_reader::DataReader) objects belong to a single built-in [`Subscriber`].
    /// The built-in topics are used to communicate information about other [`DomainParticipant`], [`Topic`], [`DataReader`](crate::subscription::data_reader::DataReader), and [`DataWriter`](crate::publication::data_writer::DataWriter)
    /// objects.
    pub fn get_builtin_subscriber(&self) -> DdsResult<Subscriber> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_builtin_subscriber(dp)
                .map(|x| Subscriber::new(SubscriberNodeKind::Builtin(x)))
        })
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote domain participant. From that point
    /// onwards the Service will locally behave as if the remote participant did not exist. This means it will ignore any topic,
    /// publication, or subscription that originates on that domain participant.
    /// This operation can be used, in conjunction with the discovery of remote participants offered by means of the
    /// “DCPSParticipant” built-in [`Topic`], to provide, for example, access control.
    /// Application data can be associated with a [`DomainParticipant`] by means of the [`UserDataQosPolicy`](crate::infrastructure::qos_policy::UserDataQosPolicy).
    /// This application data is propagated as a field in the built-in topic and can be used by an application to implement its own access control policy.
    /// The domain participant to ignore is identified by the `handle` argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo)
    /// retrieved when reading the data-samples available for the built-in DataReader to the “DCPSParticipant” topic. The built-in
    /// [`DataReader`](crate::subscription::data_reader::DataReader) is read with the same read/take operations used for any DataReader.
    /// The [`DomainParticipant::ignore_participant()`] operation is not reversible.
    pub fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::ignore_participant(dp, handle)
        })
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote topic. This means it will locally ignore any
    /// publication or subscription to the Topic.
    /// This operation can be used to save local resources when the application knows that it will never publish or subscribe to data
    /// under certain topics.
    /// The Topic to ignore is identified by the handle argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo) retrieved when
    /// reading the data-samples from the built-in [`DataReader`](crate::subscription::data_reader::DataReader) to the “DCPSTopic” topic.
    /// The [`DomainParticipant::ignore_topic()`] operation is not reversible.
    pub fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::ignore_topic(dp, handle)
        })
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote publication; a publication is defined by
    /// the association of a topic name, and user data and partition set on the Publisher. After this call, any data written related to that publication will be ignored.
    /// The DataWriter to ignore is identified by the handle argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo) retrieved
    /// when reading the data-samples from the built-in [`DataReader`](crate::subscription::data_reader::DataReader) to the “DCPSPublication” topic.
    /// The [`DomainParticipant::ignore_publication()`] operation is not reversible.
    pub fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::ignore_publication(dp, handle)
        })
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote subscription; a subscription is defined by
    /// the association of a topic name, and user data and partition set on the Subscriber.
    /// After this call, any data received related to that subscription will be ignored.
    /// The DataReader to ignore is identified by the handle argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo)
    /// retrieved when reading the data-samples from the built-in [`DataReader`](crate::subscription::data_reader::DataReader) to the “DCPSSubscription” topic.
    /// The [`DomainParticipant::ignore_subscription()`] operation is not reversible.
    pub fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::ignore_subscription(dp, handle)
        })
    }

    /// This operation retrieves the [`DomainId`] used to create the DomainParticipant. The [`DomainId`] identifies the DDS domain to
    /// which the [`DomainParticipant`] belongs. Each DDS domain represents a separate data “communication plane” isolated from other domains.
    pub fn get_domain_id(&self) -> DdsResult<DomainId> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_domain_id(dp)
        })
    }

    /// This operation deletes all the entities that were created by means of the “create” operations on the DomainParticipant. That is,
    /// it deletes all contained [`Publisher`], [`Subscriber`] and [`Topic`] entities.
    /// Prior to deleting each contained entity, this operation will recursively call the corresponding `delete_contained_entities()`
    /// operation on each contained entity (if applicable). This pattern is applied recursively. In this manner the operation
    /// [`DomainParticipant::delete_contained_entities()`] will end up deleting all the entities recursively contained in the
    /// [`DomainParticipant`], that is also the [`DataWriter`](crate::publication::data_writer::DataWriter), [`DataReader`](crate::subscription::data_reader::DataReader).
    /// The operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError) if the any of the contained entities is in a state where it cannot be
    /// deleted.
    /// Once this operation returns successfully, the application may delete the [`DomainParticipant`] knowing that it has no
    /// contained entities.
    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::delete_contained_entities(dp)
        })
    }

    /// This operation manually asserts the liveliness of the [`DomainParticipant`]. This is used in combination
    /// with the [`LivelinessQosPolicy`](crate::infrastructure::qos_policy::LivelinessQosPolicy)
    /// to indicate to the Service that the entity remains active.
    /// This operation needs to only be used if the [`DomainParticipant`] contains  [`DataWriter`](crate::publication::data_writer::DataWriter) entities with the LIVELINESS set to
    /// MANUAL_BY_PARTICIPANT and it only affects the liveliness of those  [`DataWriter`](crate::publication::data_writer::DataWriter) entities. Otherwise, it has no effect.
    /// NOTE: Writing data via the write operation on a  [`DataWriter`](crate::publication::data_writer::DataWriter) asserts liveliness on the DataWriter itself and its
    /// [`DomainParticipant`]. Consequently the use of this operation is only needed if the application is not writing data regularly.
    pub fn assert_liveliness(&self) -> DdsResult<()> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::assert_liveliness(dp)
        })
    }

    /// This operation sets a default value of the Publisher QoS policies which will be used for newly created [`Publisher`] entities in the
    /// case where the QoS policies are defaulted in the [`DomainParticipant::create_publisher()`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistenPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values the default values of [`PublisherQos`].
    pub fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::set_default_publisher_qos(dp, qos)
        })
    }

    /// This operation retrieves the default value of the Publisher QoS, that is, the QoS policies which will be used for newly created
    /// [`Publisher`] entities in the case where the QoS policies are defaulted in the [`DomainParticipant::create_publisher()`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`DomainParticipant::set_default_publisher_qos()`], or else, if the call was never made, the default values of the [`PublisherQos`].
    pub fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_default_publisher_qos(dp)
        })
    }

    /// This operation sets a default value of the Subscriber QoS policies that will be used for newly created [`Subscriber`] entities in the
    /// case where the QoS policies are defaulted in the [`DomainParticipant::create_subscriber()`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistenPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the default values of [`SubscriberQos`].
    pub fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::set_default_subscriber_qos(dp, qos)
        })
    }

    /// This operation retrieves the default value of the Subscriber QoS, that is, the QoS policies which will be used for newly created
    /// [`Subscriber`] entities in the case where the QoS policies are defaulted in the [`DomainParticipant::create_subscriber()`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`DomainParticipant::set_default_subscriber_qos()`], or else, if the call was never made, the default values of [`SubscriberQos`].
    pub fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_default_subscriber_qos(dp)
        })
    }

    /// This operation sets a default value of the Topic QoS policies which will be used for newly created [`Topic`] entities in the case
    /// where the QoS policies are defaulted in the [`DomainParticipant::create_topic`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistenPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default QoS should be reset
    /// back to the initial values the factory would use, that is the default values of [`TopicQos`].
    pub fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::set_default_topic_qos(dp, qos)
        })
    }

    /// This operation retrieves the default value of the Topic QoS, that is, the QoS policies that will be used for newly created [`Topic`]
    /// entities in the case where the QoS policies are defaulted in the [`DomainParticipant::create_topic()`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`DomainParticipant::set_default_topic_qos()`], or else, if the call was never made, the default values of [`TopicQos`]
    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_default_topic_qos(dp)
        })
    }

    /// This operation retrieves the list of DomainParticipants that have been discovered in the domain and that the application has not
    /// indicated should be “ignored” by means of the [`DomainParticipant::ignore_participant()`] operation.
    pub fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_discovered_participants(dp)
        })
    }

    /// This operation retrieves information on a [`DomainParticipant`] that has been discovered on the network. The participant must
    /// be in the same domain as the participant on which this operation is invoked and must not have been “ignored” by means of the
    /// [`DomainParticipant::ignore_participant()`] operation.
    /// The participant_handle must correspond to such a DomainParticipant. Otherwise, the operation will fail and return
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// Use the operation [`DomainParticipant::get_discovered_participants()`] to find the DomainParticipants that are currently discovered.
    pub fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_discovered_participant_data(
                dp,
                participant_handle,
            )
        })
    }

    /// This operation retrieves the list of Topics that have been discovered in the domain and that the application has not indicated
    /// should be “ignored” by means of the [`DomainParticipant::ignore_topic()`] operation.
    pub fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_discovered_topics(dp)
        })
    }

    /// This operation retrieves information on a Topic that has been discovered on the network. The topic must have been created by
    /// a participant in the same domain as the participant on which this operation is invoked and must not have been “ignored” by
    /// means of the [`DomainParticipant::ignore_topic()`] operation.
    /// The `topic_handle` must correspond to such a topic. Otherwise, the operation will fail and return
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// Use the operation [`DomainParticipant::get_discovered_topics()`] to find the topics that are currently discovered.
    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_discovered_topic_data(
                dp,
                topic_handle,
            )
        })
    }

    /// This operation checks whether or not the given `a_handle` represents an Entity that was created from the [`DomainParticipant`].
    /// The containment applies recursively. That is, it applies both to entities ([`Topic`], [`Publisher`], or [`Subscriber`]) created
    /// directly using the [`DomainParticipant`] as well as entities created using a contained [`Publisher`], or [`Subscriber`] as the factory, and
    /// so forth.
    /// The instance handle for an Entity may be obtained from built-in topic data, from various statuses, or from the Entity operation
    /// `get_instance_handle`.
    pub fn contains_entity(&self, a_handle: InstanceHandle) -> DdsResult<bool> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::contains_entity(dp, a_handle)
        })
    }

    /// This operation returns the current value of the time that the service uses to time-stamp data-writes and to set the reception timestamp
    /// for the data-updates it receives.
    pub fn get_current_time(&self) -> DdsResult<Time> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_current_time(dp)
        })
    }
}

/// This implementation block contains the Entity operations for the [`DomainParticipant`].
impl DomainParticipant {
    /// This operation is used to set the QoS policies of the Entity and replacing the values of any policies previously set.
    /// Certain policies are “immutable;” they can only be set at Entity creation time, or before the entity is made enabled.
    /// If [`Self::set_qos()`] is invoked after the Entity is enabled and it attempts to change the value of an “immutable” policy, the operation will
    /// fail and returns [`DdsError::ImmutablePolicy`](crate::infrastructure::error::DdsError).
    /// Certain values of QoS policies can be incompatible with the settings of the other policies. This operation will also fail if it specifies
    /// a set of values that once combined with the existing values would result in an inconsistent set of policies. In this case,
    /// the return value is [`DdsError::InconsistentPolicy`](crate::infrastructure::error::DdsError).
    /// The existing set of policies are only changed if the [`Self::set_qos()`] operation succeeds. This is indicated by the [`Ok`] return value. In all
    /// other cases, none of the policies is modified.
    /// The parameter `qos` can be set to [`QosKind::Default`] to indicate that the QoS of the Entity should be changed to match the current default QoS set in the Entity’s factory.
    /// The operation [`Self::set_qos()`] cannot modify the immutable QoS so a successful return of the operation indicates that the mutable QoS for the Entity has been
    /// modified to match the current default for the Entity’s factory.
    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::set_qos(dp, qos)
        })
    }

    /// This operation allows access to the existing set of [`DomainParticipantQos`] policies.
    pub fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.call_participant_method(|dp| {
            crate::implementation::behavior::domain_participant::get_qos(dp)
        })
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
            &self.0.guid(),
            |domain_participant_listener| {
                Ok(domain_participant_listener
                    .ok_or(DdsError::AlreadyDeleted)?
                    .get_status_condition())
            },
        )
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
            &self.0.guid(),
            |domain_participant_listener| {
                Ok(domain_participant_listener
                    .ok_or(DdsError::AlreadyDeleted)?
                    .get_status_changes())
            },
        )
    }

    /// This operation enables the Entity. Entity objects can be created either enabled or disabled. This is controlled by the value of
    /// the [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) on the corresponding factory for the Entity.
    /// The default setting of [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) is such that, by default, it is not necessary to explicitly call enable on newly
    /// created entities.
    /// The [`Self::enable()`] operation is idempotent. Calling [`Self::enable()`] on an already enabled Entity returns [`Ok`] and has no effect.
    /// If an Entity has not yet been enabled, the following kinds of operations may be invoked on it:
    /// - Operations to set or get an Entity’s QoS policies (including default QoS policies) and listener
    /// - [`Self::get_statuscondition()`]
    /// - Factory and lookup operations
    /// - [`Self::get_status_changes()`] and other get status operations (although the status of a disabled entity never changes)
    /// Other operations may explicitly state that they may be called on disabled entities; those that do not will return the error
    /// NotEnabled.
    /// It is legal to delete an Entity that has not been enabled by calling the proper operation on its factory.
    /// Entities created from a factory that is disabled, are created disabled regardless of the setting of the
    /// [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy).
    /// Calling enable on an Entity whose factory is not enabled will fail and return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// If the `autoenable_created_entities` field of [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) is set to [`true`], the [`Self::enable()`] operation on the factory will
    /// automatically enable all entities created from the factory.
    /// The Listeners associated with an entity are not called until the entity is enabled. Conditions associated with an entity that is not
    /// enabled are “inactive”, that is, the operation [`StatusCondition::get_trigger_value()`] will always return `false`.
    pub fn enable(&self) -> DdsResult<()> {
        self.call_participant_mut_method(|dp| {
            crate::implementation::behavior::domain_participant::enable(dp)
        })
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.guid().into())
    }

    fn call_participant_method<F, O>(&self, f: F) -> DdsResult<O>
    where
        F: FnOnce(&DdsDomainParticipant) -> DdsResult<O>,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(&self.0.guid().prefix(), |dp| {
            f(dp.ok_or(DdsError::AlreadyDeleted)?)
        })
    }

    fn call_participant_mut_method<F, O>(&self, f: F) -> DdsResult<O>
    where
        F: FnOnce(&mut DdsDomainParticipant) -> DdsResult<O>,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&self.0.guid().prefix(), |dp| {
            f(dp.ok_or(DdsError::AlreadyDeleted)?)
        })
    }
}

/////////////////////////////////////////////////////////////////////////

pub async fn task_send_entity_announce(
    participant_guid_prefix: GuidPrefix,
    mut announce_receiver: Receiver<AnnounceKind>,
) {
    loop {
        if let Some(announce_kind) = announce_receiver.recv().await {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&participant_guid_prefix, |dp| {
                if let Some(dp) = dp {
                    announce_entity(dp, announce_kind);
                }
            })
        }
    }
}

fn announce_entity(domain_participant: &mut DdsDomainParticipant, announce_kind: AnnounceKind) {
    match announce_kind {
        AnnounceKind::CreatedDataReader(discovered_reader_data) => {
            announce_created_data_reader(domain_participant, discovered_reader_data)
        }
        AnnounceKind::CreatedDataWriter(discovered_writer_data) => {
            announce_created_data_writer(domain_participant, discovered_writer_data)
        }
        AnnounceKind::CratedTopic(discovered_topic_data) => {
            announce_created_topic(domain_participant, discovered_topic_data)
        }
        AnnounceKind::DeletedDataReader(deleted_reader_handle) => {
            announce_deleted_reader(domain_participant, deleted_reader_handle)
        }
        AnnounceKind::DeletedDataWriter(deleted_writer_handle) => {
            announce_deleted_writer(domain_participant, deleted_writer_handle)
        }
        AnnounceKind::DeletedParticipant => (),
    }
}

pub async fn task_user_defined_receive(
    participant_guid_prefix: GuidPrefix,
    mut default_unicast_transport: UdpTransportRead,
    listener_sender: Sender<ListenerTriggerKind>,
) {
    loop {
        if let Some((locator, message)) = default_unicast_transport.read().await {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
                &participant_guid_prefix,
                |dp| {
                    if let Some(dp) = dp {
                        dp.receive_user_defined_data(locator, message, &listener_sender)
                            .ok();
                    }
                },
            );
        }
    }
}

pub async fn task_metatraffic_unicast_receive(
    participant_guid_prefix: GuidPrefix,
    mut metatraffic_unicast_transport: UdpTransportRead,
    sedp_condvar: DdsCondvar,
    listener_sender: Sender<ListenerTriggerKind>,
) {
    loop {
        if let Some((locator, message)) = metatraffic_unicast_transport.read().await {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&participant_guid_prefix, |dp| {
                if let Some(dp) = dp {
                    receive_builtin_message(dp, message, locator, &sedp_condvar, &listener_sender)
                }
            })
        }
    }
}

pub async fn task_metatraffic_multicast_receive(
    participant_guid_prefix: GuidPrefix,
    mut metatraffic_multicast_transport: UdpTransportRead,
    sedp_condvar: DdsCondvar,
    listener_sender: Sender<ListenerTriggerKind>,
) {
    loop {
        if let Some((locator, message)) = metatraffic_multicast_transport.read().await {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&participant_guid_prefix, |dp| {
                if let Some(dp) = dp {
                    super::dcps_service::receive_builtin_message(
                        dp,
                        message,
                        locator,
                        &sedp_condvar,
                        &listener_sender,
                    )
                }
            })
        }
    }
}

pub async fn task_update_communication_status(
    participant_guid_prefix: GuidPrefix,
    listener_sender: Sender<ListenerTriggerKind>,
) {
    loop {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&participant_guid_prefix, |dp| {
            if let Some(dp) = dp {
                dp.update_communication_status(&listener_sender).ok();
            }
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}

pub async fn task_listener_receiver(mut listener_receiver: Receiver<ListenerTriggerKind>) {
    loop {
        if let Some(l) = listener_receiver.recv().await {
            match l {
                ListenerTriggerKind::RequestedDeadlineMissed(dr) => {
                    on_requested_deadline_missed_communication_change(dr)
                }
                ListenerTriggerKind::OnDataAvailable(dr) => {
                    on_data_available_communication_change(dr)
                }
                ListenerTriggerKind::SubscriptionMatched(dr) => {
                    on_subscription_matched_communication_change(dr)
                }
                ListenerTriggerKind::RequestedIncompatibleQos(dr) => {
                    on_requested_incompatible_qos_communication_change(dr)
                }
                ListenerTriggerKind::OnSampleRejected(dr) => {
                    on_sample_rejected_communication_change(dr)
                }
                ListenerTriggerKind::OnSampleLost(dr) => on_sample_lost_communication_change(dr),
                ListenerTriggerKind::OfferedIncompatibleQos(dw) => {
                    on_offered_incompatible_qos_communication_change(dw)
                }
                ListenerTriggerKind::PublicationMatched(dw) => {
                    on_publication_matched_communication_change(dw)
                }
                ListenerTriggerKind::InconsistentTopic(t) => {
                    on_inconsistent_topic_communication_change(t)
                }
            }
        }
    }
}

fn on_requested_deadline_missed_communication_change(data_reader_node: DataReaderNode) {
    fn get_requested_deadline_missed_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<RequestedDeadlineMissedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .get_participant_mut(&data_reader_node.parent_participant().prefix(), |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_requested_deadline_missed_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            })
    }

    let status_kind = StatusKind::RequestedDeadlineMissed;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_requested_deadline_missed_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_requested_deadline_missed(data_reader_node, status);
                }
                true
            }
            _ => false,
        },
    );

    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_requested_deadline_missed_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_requested_deadline_missed(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );

        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_requested_deadline_missed_status(&data_reader_node)
                        {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_requested_deadline_missed(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_data_available_communication_change(data_reader_node: DataReaderNode) {
    let data_on_reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
        &data_reader_node.parent_subscriber(),
        |subscriber_listener| match subscriber_listener {
            Some(l) if l.is_enabled(&StatusKind::DataOnReaders) => {
                l.listener_mut()
                    .as_mut()
                    .expect("Listener should be some")
                    .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                        SubscriberNode::new(
                            data_reader_node.parent_subscriber(),
                            data_reader_node.parent_participant(),
                        ),
                    )));
                true
            }
            _ => false,
        },
    );
    if !data_on_reader_listener {
        let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
            &data_reader_node.guid(),
            |data_reader_listener| match data_reader_listener {
                Some(l) if l.is_enabled(&StatusKind::DataAvailable) => {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_data_available(data_reader_node);
                    true
                }
                _ => false,
            },
        );
        if !reader_listener {
            let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
                &data_reader_node.parent_subscriber(),
                |subscriber_listener| match subscriber_listener {
                    Some(l) if l.is_enabled(&StatusKind::DataAvailable) => {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_data_available(&data_reader_node);

                        true
                    }
                    _ => false,
                },
            );
            if !subscriber_listener {
                THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                    &data_reader_node.parent_participant(),
                    |participant_listener| match participant_listener {
                        Some(l) if l.is_enabled(&StatusKind::DataAvailable) => l
                            .listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_data_available(&data_reader_node),
                        _ => (),
                    },
                );
            }
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
        &data_reader_node.parent_subscriber(),
        |subscriber_listener| {
            if let Some(l) = subscriber_listener {
                l.add_communication_state(StatusKind::DataOnReaders);
            }
        },
    );

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(StatusKind::DataAvailable);
            }
        },
    )
}

fn on_subscription_matched_communication_change(data_reader_node: DataReaderNode) {
    fn get_subscription_matched_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_reader_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_subscription_matched_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            },
        )
    }

    let status_kind = StatusKind::SubscriptionMatched;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_subscription_matched_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_subscription_matched(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_subscription_matched_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_subscription_matched(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_subscription_matched_status(&data_reader_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_subscription_matched(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_requested_incompatible_qos_communication_change(data_reader_node: DataReaderNode) {
    fn get_requested_incompatible_qos_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_reader_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_requested_incompatible_qos_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            },
        )
    }

    let status_kind = StatusKind::RequestedIncompatibleQos;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_requested_incompatible_qos_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_requested_incompatible_qos(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_requested_incompatible_qos_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_requested_incompatible_qos(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_requested_incompatible_qos_status(&data_reader_node)
                        {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_requested_incompatible_qos(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_sample_rejected_communication_change(data_reader_node: DataReaderNode) {
    fn get_sample_rejected_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<SampleRejectedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .get_participant_mut(&data_reader_node.parent_participant().prefix(), |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_sample_rejected_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            })
    }

    let status_kind = StatusKind::SampleRejected;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_sample_rejected_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_sample_rejected(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_sample_rejected_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_sample_rejected(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_sample_rejected_status(&data_reader_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_sample_rejected(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_sample_lost_communication_change(data_reader_node: DataReaderNode) {
    fn get_sample_lost_status(data_reader_node: &DataReaderNode) -> DdsResult<SampleLostStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_reader_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_sample_lost_status(
                    dp.ok_or(DdsError::AlreadyDeleted)?,
                    data_reader_node.guid(),
                    data_reader_node.parent_subscriber(),
                )
            },
        )
    }

    let status_kind = StatusKind::SampleLost;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_sample_lost_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_sample_lost(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_sample_lost_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_sample_lost(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_sample_lost_status(&data_reader_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_sample_lost(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_offered_incompatible_qos_communication_change(data_writer_node: DataWriterNode) {
    fn get_offered_incompatible_qos_status(
        data_writer_node: &DataWriterNode,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_writer_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_writer::get_offered_incompatible_qos_status(
                    dp.ok_or(DdsError::AlreadyDeleted)?,
                    data_writer_node.guid(),
                    data_writer_node.parent_publisher(),
                )
            },
        )
    }

    let status_kind = StatusKind::OfferedIncompatibleQos;
    let writer_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| match data_writer_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_offered_incompatible_qos_status(&data_writer_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_offered_incompatible_qos(data_writer_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !writer_listener {
        let publisher_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_publisher_listener(
            &data_writer_node.parent_publisher(),
            |publisher_listener| match publisher_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_offered_incompatible_qos_status(&data_writer_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_offered_incompatible_qos(&data_writer_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !publisher_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_writer_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_offered_incompatible_qos_status(&data_writer_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_offered_incompatible_qos(&data_writer_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| {
            if let Some(l) = data_writer_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_publication_matched_communication_change(data_writer_node: DataWriterNode) {
    fn get_publication_matched_status(
        data_writer_node: &DataWriterNode,
    ) -> DdsResult<PublicationMatchedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_writer_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_writer::get_publication_matched_status(dp.ok_or(DdsError::AlreadyDeleted)?,data_writer_node.guid(),
                data_writer_node.parent_publisher(),)
            },
        )
    }

    let status_kind = StatusKind::PublicationMatched;
    let writer_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| match data_writer_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_publication_matched_status(&data_writer_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_publication_matched(data_writer_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !writer_listener {
        let publisher_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_publisher_listener(
            &data_writer_node.parent_publisher(),
            |publisher_listener| match publisher_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_publication_matched_status(&data_writer_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_publication_matched(&data_writer_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !publisher_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_writer_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_publication_matched_status(&data_writer_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_publication_matched(&data_writer_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| {
            if let Some(l) = data_writer_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_inconsistent_topic_communication_change(topic_node: TopicNode) {
    fn get_inconsistent_topic_status(topic_node: &TopicNode) -> DdsResult<InconsistentTopicStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &topic_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_topic::get_inconsistent_topic_status(
                    dp.ok_or(DdsError::AlreadyDeleted)?,
                    topic_node.guid(),
                )
            },
        )
    }

    let status_kind = StatusKind::InconsistentTopic;
    let topic_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_topic_listener(
        &topic_node.guid(),
        |topic_listener| match topic_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_inconsistent_topic_status(&topic_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_inconsistent_topic(topic_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !topic_listener {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
            &topic_node.parent_participant(),
            |participant_listener| match participant_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_inconsistent_topic_status(&topic_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_inconsistent_topic(&topic_node, status)
                    }
                }
                _ => (),
            },
        );
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_topic_listener(&topic_node.guid(), |topic_listener| {
        if let Some(l) = topic_listener {
            l.add_communication_state(status_kind);
        }
    })
}

fn announce_created_data_reader(
    domain_participant: &mut DdsDomainParticipant,
    discovered_reader_data: DiscoveredReaderData,
) {
    let reader_proxy = ReaderProxy::new(
        discovered_reader_data.reader_proxy().remote_reader_guid(),
        discovered_reader_data
            .reader_proxy()
            .remote_group_entity_id(),
        domain_participant.default_unicast_locator_list().to_vec(),
        domain_participant.default_multicast_locator_list().to_vec(),
        discovered_reader_data.reader_proxy().expects_inline_qos(),
    );
    let reader_data = &DiscoveredReaderData::new(
        reader_proxy,
        discovered_reader_data
            .subscription_builtin_topic_data()
            .clone(),
    );

    let serialized_data = dds_serialize(reader_data).expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();
    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredReaderData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            reader_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_created_data_writer(
    domain_participant: &mut DdsDomainParticipant,
    discovered_writer_data: DiscoveredWriterData,
) {
    let writer_data = &DiscoveredWriterData::new(
        discovered_writer_data.dds_publication_data().clone(),
        WriterProxy::new(
            discovered_writer_data.writer_proxy().remote_writer_guid(),
            discovered_writer_data
                .writer_proxy()
                .remote_group_entity_id(),
            domain_participant.default_unicast_locator_list().to_vec(),
            domain_participant.default_multicast_locator_list().to_vec(),
            discovered_writer_data
                .writer_proxy()
                .data_max_size_serialized(),
        ),
    );

    let serialized_data = dds_serialize(writer_data).expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredWriterData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            writer_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_created_topic(
    domain_participant: &mut DdsDomainParticipant,
    discovered_topic: DiscoveredTopicData,
) {
    let serialized_data = dds_serialize(&discovered_topic).expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredTopicData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            discovered_topic.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_deleted_reader(
    domain_participant: &mut DdsDomainParticipant,
    reader_handle: InstanceHandle,
) {
    let serialized_key = DdsSerializedKey::from(reader_handle.as_ref());
    let instance_serialized_key =
        cdr::serialize::<_, _, cdr::CdrLe>(&serialized_key, cdr::Infinite)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
            .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredReaderData::type_name())
        .unwrap()
        .dispose_w_timestamp(instance_serialized_key, reader_handle, timestamp)
        .expect("Should not fail to write built-in message");
}

fn announce_deleted_writer(
    domain_participant: &mut DdsDomainParticipant,
    writer_handle: InstanceHandle,
) {
    let serialized_key = DdsSerializedKey::from(writer_handle.as_ref());
    let instance_serialized_key =
        cdr::serialize::<_, _, cdr::CdrLe>(&serialized_key, cdr::Infinite)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
            .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredWriterData::type_name())
        .unwrap()
        .dispose_w_timestamp(instance_serialized_key, writer_handle, timestamp)
        .expect("Should not fail to write built-in message");
}
