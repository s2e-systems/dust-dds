use std::time::Instant;

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    implementation::{
        actors::{
            data_reader_actor, data_writer_actor,
            domain_participant_actor::{self, DomainParticipantActor, FooTypeSupport},
            publisher_actor, subscriber_actor, topic_actor,
        },
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        listeners::NoOpListener,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::{StatusKind, NO_STATUS},
        time::{Duration, Time},
    },
    publication::{publisher::Publisher, publisher_listener::PublisherListener},
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
    topic_definition::{
        topic::Topic,
        topic_listener::TopicListener,
        type_support::{DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml, DynamicTypeInterface},
    },
};

use super::{
    domain_participant_factory::{DomainId, DomainParticipantFactory},
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

pub struct DomainParticipant {
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl DomainParticipant {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            participant_address,
            runtime_handle,
        }
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
    #[tracing::instrument(skip(self, a_listener))]
    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: impl PublisherListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<Publisher> {
        let publisher_address = self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::create_publisher::new(
                qos,
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))?;

        let publisher = Publisher::new(
            publisher_address,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
            && self
                .participant_address
                .send_mail_and_await_reply_blocking(domain_participant_actor::get_qos::new())?
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enable()?;
        }

        Ok(publisher)
    }

    /// This operation deletes an existing [`Publisher`].
    /// A [`Publisher`] cannot be deleted if it has any attached [`DataWriter`](crate::publication::data_writer::DataWriter) objects. If [`DomainParticipant::delete_publisher()`]
    /// is called on a [`Publisher`] with existing [`DataWriter`](crate::publication::data_writer::DataWriter) objects, it will return a
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError) error.
    /// The [`DomainParticipant::delete_publisher()`] operation must be called on the same [`DomainParticipant`] object used to create the [`Publisher`].
    /// If [`DomainParticipant::delete_publisher()`] is called on a different [`DomainParticipant`], the operation will have no effect and it will return
    /// a PreconditionNotMet error.
    #[tracing::instrument(skip(self, a_publisher))]
    pub fn delete_publisher(&self, a_publisher: &Publisher) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_instance_handle::new(),
            )?
            != a_publisher.get_participant()?.get_instance_handle()?
        {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }

        self.participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::delete_user_defined_publisher::new(
                    a_publisher.get_instance_handle()?,
                ),
            )?
    }

    /// This operation creates a [`Subscriber`] with the desired QoS policies and attaches to it the specified [`SubscriberListener`].
    /// If the specified QoS policies are not consistent, the operation will fail and no [`Subscriber`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`Subscriber`] should be created with the
    /// default Subscriber QoS set in the factory. The use of this value is equivalent to the application obtaining the default
    /// Subscriber QoS by means of the operation [`Self::get_default_subscriber_qos()`] and using the resulting QoS to create the
    /// [`Subscriber`].
    /// The created [`Subscriber`] belongs to the [`DomainParticipant`] that is its factory.
    /// In case of failure, the operation will return an error and no [`Subscriber`] will be created.
    #[tracing::instrument(skip(self, a_listener))]
    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: impl SubscriberListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<Subscriber> {
        let subscriber_address = self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::create_subscriber::new(
                    qos,
                    Box::new(a_listener),
                    mask.to_vec(),
                    self.runtime_handle.clone(),
                ),
            )?;

        let subscriber = Subscriber::new(
            subscriber_address,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );

        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
            && self
                .participant_address
                .send_mail_and_await_reply_blocking(domain_participant_actor::get_qos::new())?
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enable()?;
        }

        Ok(subscriber)
    }

    /// This operation deletes an existing [`Subscriber`].
    /// A [`Subscriber`] cannot be deleted if it has any attached [`DataReader`](crate::subscription::data_reader::DataReader) objects. If the [`DomainParticipant::delete_subscriber()`] operation is called on a
    /// [`Subscriber`] with existing [`DataReader`](crate::subscription::data_reader::DataReader) objects, it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// The [`DomainParticipant::delete_subscriber()`] operation must be called on the same [`DomainParticipant`] object used to create the Subscriber. If
    /// it is called on a different [`DomainParticipant`], the operation will have no effect and it will return
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    #[tracing::instrument(skip(self, a_subscriber))]
    pub fn delete_subscriber(&self, a_subscriber: &Subscriber) -> DdsResult<()> {
        if self.get_instance_handle()? != a_subscriber.get_participant()?.get_instance_handle()? {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        self.participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::delete_user_defined_subscriber::new(
                    a_subscriber.get_instance_handle()?,
                ),
            )?
    }

    /// This operation creates a [`Topic`] with the desired QoS policies and attaches to it the specified [`TopicListener`].
    /// If the specified QoS policies are not consistent, the operation will fail and no [`Topic`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`Topic`] should be created with the default Topic QoS
    /// set in the factory. The use of this value is equivalent to the application obtaining the default Topic QoS by means of the
    /// operation [`DomainParticipant::get_default_topic_qos`] and using the resulting QoS to create the [`Topic`].
    /// The created [`Topic`] belongs to the [`DomainParticipant`] that is its factory.
    /// In case of failure, the operation will return an error and no [`Topic`] will be created.
    #[tracing::instrument(skip(self, a_listener))]
    pub fn create_topic<Foo>(
        &self,
        topic_name: &str,
        type_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: impl TopicListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<Topic>
    where
        Foo: DdsKey + DdsHasKey + DdsTypeXml,
    {
        let type_support = FooTypeSupport::new::<Foo>();

        self.create_dynamic_topic(topic_name, type_name, qos, a_listener, mask, type_support)
    }

    #[doc(hidden)]
    #[tracing::instrument(skip(self, a_listener, dynamic_type_representation))]
    pub fn create_dynamic_topic(
        &self,
        topic_name: &str,
        type_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: impl TopicListener + Send + 'static,
        mask: &[StatusKind],
        dynamic_type_representation: impl DynamicTypeInterface + Send + Sync + 'static,
    ) -> DdsResult<Topic> {
        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::register_type::new(
                type_name.to_string(),
                Box::new(dynamic_type_representation),
            ))?;

        let topic_address = self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::create_topic::new(
                topic_name.to_string(),
                type_name.to_string(),
                qos,
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))?;

        let topic = Topic::new(
            topic_address,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
            && self
                .participant_address
                .send_mail_and_await_reply_blocking(domain_participant_actor::get_qos::new())?
                .entity_factory
                .autoenable_created_entities
        {
            topic.enable()?;
        }

        Ok(topic)
    }

    /// This operation deletes a [`Topic`].
    /// The deletion of a [`Topic`] is not allowed if there are any existing [`DataReader`](crate::subscription::data_reader::DataReader) or [`DataWriter`](crate::publication::data_writer::DataWriter)
    /// objects that are using the [`Topic`]. If the [`DomainParticipant::delete_topic()`] operation is called on a [`Topic`] with any of these existing objects attached to
    /// it, it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// The [`DomainParticipant::delete_topic()`] operation must be called on the same [`DomainParticipant`] object used to create the [`Topic`]. If [`DomainParticipant::delete_topic()`] is
    /// called on a different [`DomainParticipant`], the operation will have no effect and it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    #[tracing::instrument(skip(self, a_topic))]
    pub fn delete_topic(&self, a_topic: &Topic) -> DdsResult<()> {
        if self.get_instance_handle()? != a_topic.get_participant()?.get_instance_handle()? {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        for publisher in self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_publisher_list::new(),
            )?
        {
            let data_writer_list = publisher
                .send_mail_and_await_reply_blocking(publisher_actor::data_writer_list::new())?;
            for data_writer in data_writer_list {
                if data_writer
                    .send_mail_and_await_reply_blocking(data_writer_actor::get_type_name::new())?
                    == a_topic.get_type_name()?
                    && data_writer.send_mail_and_await_reply_blocking(
                        data_writer_actor::get_topic_name::new(),
                    )? == a_topic.get_name()?
                {
                    return Err(DdsError::PreconditionNotMet(
                        "Topic still attached to some data writer".to_string(),
                    ));
                }
            }
        }

        for subscriber in self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_subscriber_list::new(),
            )?
        {
            let data_reader_list = subscriber
                .send_mail_and_await_reply_blocking(subscriber_actor::data_reader_list::new())?;
            for data_reader in data_reader_list {
                if data_reader
                    .send_mail_and_await_reply_blocking(data_reader_actor::get_type_name::new())?
                    == a_topic.get_type_name()?
                    && data_reader.send_mail_and_await_reply_blocking(
                        data_reader_actor::get_topic_name::new(),
                    )? == a_topic.get_name()?
                {
                    return Err(DdsError::PreconditionNotMet(
                        "Topic still attached to some data reader".to_string(),
                    ));
                }
            }
        }

        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::delete_topic::new(a_topic.get_instance_handle()?),
        )
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
    #[tracing::instrument(skip(self))]
    pub fn find_topic<Foo>(&self, topic_name: &str, timeout: Duration) -> DdsResult<Topic>
    where
        Foo: DdsKey + DdsHasKey + DdsTypeXml,
    {
        let start_time = Instant::now();

        while start_time.elapsed() < std::time::Duration::from(timeout) {
            for topic in self
                .participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_user_defined_topic_list::new(),
                )?
            {
                if topic.send_mail_and_await_reply_blocking(topic_actor::get_name::new())?
                    == topic_name
                {
                    return Ok(Topic::new(
                        topic,
                        self.participant_address.clone(),
                        self.runtime_handle.clone(),
                    ));
                }
            }

            for discovered_topic_handle in self
                .participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::discovered_topic_list::new(),
                )?
            {
                if let Ok(discovered_topic_data) = self
                    .participant_address
                    .send_mail_and_await_reply_blocking(
                        domain_participant_actor::discovered_topic_data::new(
                            discovered_topic_handle,
                        ),
                    )?
                {
                    if discovered_topic_data.name() == topic_name {
                        let qos = TopicQos {
                            topic_data: discovered_topic_data.topic_data().clone(),
                            durability: discovered_topic_data.durability().clone(),
                            deadline: discovered_topic_data.deadline().clone(),
                            latency_budget: discovered_topic_data.latency_budget().clone(),
                            liveliness: discovered_topic_data.liveliness().clone(),
                            reliability: discovered_topic_data.reliability().clone(),
                            destination_order: discovered_topic_data.destination_order().clone(),
                            history: discovered_topic_data.history().clone(),
                            resource_limits: discovered_topic_data.resource_limits().clone(),
                            transport_priority: discovered_topic_data.transport_priority().clone(),
                            lifespan: discovered_topic_data.lifespan().clone(),
                            ownership: discovered_topic_data.ownership().clone(),
                        };
                        let topic = self.create_topic::<Foo>(
                            topic_name,
                            discovered_topic_data.get_type_name(),
                            QosKind::Specific(qos),
                            NoOpListener::new(),
                            NO_STATUS,
                        )?;
                        return Ok(topic);
                    }
                }
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
    #[tracing::instrument(skip(self))]
    pub fn lookup_topicdescription(&self, _topic_name: &str) -> DdsResult<Option<Topic>> {
        todo!()
        // self.call_participant_method(|dp| {
        //     Ok(
        //         crate::implementation::behavior::domain_participant::lookup_topicdescription(
        //             dp,
        //             topic_name,
        //             Foo,
        //         )?
        //         .map(|x| Topic::new(TopicNodeKind::UserDefined(x))),
        //     )
        // })
    }

    /// This operation allows access to the built-in [`Subscriber`]. Each [`DomainParticipant`] contains several built-in [`Topic`] objects as
    /// well as corresponding [`DataReader`](crate::subscription::data_reader::DataReader) objects to access them. All these [`DataReader`](crate::subscription::data_reader::DataReader) objects belong to a single built-in [`Subscriber`].
    /// The built-in topics are used to communicate information about other [`DomainParticipant`], [`Topic`], [`DataReader`](crate::subscription::data_reader::DataReader), and [`DataWriter`](crate::publication::data_writer::DataWriter)
    /// objects.
    #[tracing::instrument(skip(self))]
    pub fn get_builtin_subscriber(&self) -> DdsResult<Subscriber> {
        Ok(Subscriber::new(
            self.participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_built_in_subscriber::new(),
                )?,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
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
    #[tracing::instrument(skip(self))]
    pub fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
        {
            self.participant_address.send_mail_and_await_reply_blocking(
                domain_participant_actor::ignore_participant::new(handle),
            )
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote topic. This means it will locally ignore any
    /// publication or subscription to the Topic.
    /// This operation can be used to save local resources when the application knows that it will never publish or subscribe to data
    /// under certain topics.
    /// The Topic to ignore is identified by the handle argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo) retrieved when
    /// reading the data-samples from the built-in [`DataReader`](crate::subscription::data_reader::DataReader) to the “DCPSTopic” topic.
    /// The [`DomainParticipant::ignore_topic()`] operation is not reversible.
    #[tracing::instrument(skip(self))]
    pub fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
        {
            self.participant_address.send_mail_and_await_reply_blocking(
                domain_participant_actor::ignore_topic::new(handle),
            )
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote publication; a publication is defined by
    /// the association of a topic name, and user data and partition set on the Publisher. After this call, any data written related to that publication will be ignored.
    /// The DataWriter to ignore is identified by the handle argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo) retrieved
    /// when reading the data-samples from the built-in [`DataReader`](crate::subscription::data_reader::DataReader) to the “DCPSPublication” topic.
    /// The [`DomainParticipant::ignore_publication()`] operation is not reversible.
    #[tracing::instrument(skip(self))]
    pub fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
        {
            self.participant_address.send_mail_and_await_reply_blocking(
                domain_participant_actor::ignore_publication::new(handle),
            )
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// This operation allows an application to instruct the Service to locally ignore a remote subscription; a subscription is defined by
    /// the association of a topic name, and user data and partition set on the Subscriber.
    /// After this call, any data received related to that subscription will be ignored.
    /// The DataReader to ignore is identified by the handle argument. This handle is the one that appears in the [`SampleInfo`](crate::subscription::sample_info::SampleInfo)
    /// retrieved when reading the data-samples from the built-in [`DataReader`](crate::subscription::data_reader::DataReader) to the “DCPSSubscription” topic.
    /// The [`DomainParticipant::ignore_subscription()`] operation is not reversible.
    #[tracing::instrument(skip(self))]
    pub fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
        {
            self.participant_address.send_mail_and_await_reply_blocking(
                domain_participant_actor::ignore_subscription::new(handle),
            )
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// This operation retrieves the [`DomainId`] used to create the DomainParticipant. The [`DomainId`] identifies the DDS domain to
    /// which the [`DomainParticipant`] belongs. Each DDS domain represents a separate data “communication plane” isolated from other domains.
    #[tracing::instrument(skip(self))]
    pub fn get_domain_id(&self) -> DdsResult<DomainId> {
        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::get_domain_id::new())
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
    #[tracing::instrument(skip(self))]
    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        for publisher in self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_publisher_list::new(),
            )?
        {
            for data_writer in publisher
                .send_mail_and_await_reply_blocking(publisher_actor::data_writer_list::new())?
            {
                publisher.send_mail_and_await_reply_blocking(
                    publisher_actor::datawriter_delete::new(
                        data_writer.send_mail_and_await_reply_blocking(
                            data_writer_actor::get_instance_handle::new(),
                        )?,
                    ),
                )?;
            }
            self.participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::delete_user_defined_publisher::new(
                        publisher.send_mail_and_await_reply_blocking(
                            publisher_actor::get_instance_handle::new(),
                        )?,
                    ),
                )??;
        }
        for subscriber in self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_subscriber_list::new(),
            )?
        {
            for data_reader in subscriber
                .send_mail_and_await_reply_blocking(subscriber_actor::data_reader_list::new())?
            {
                subscriber.send_mail_and_await_reply_blocking(
                    subscriber_actor::data_reader_delete::new(
                        data_reader.send_mail_and_await_reply_blocking(
                            data_reader_actor::get_instance_handle::new(),
                        )?,
                    ),
                )?;
            }
            self.participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::delete_user_defined_subscriber::new(
                        subscriber.send_mail_and_await_reply_blocking(
                            subscriber_actor::get_instance_handle::new(),
                        )?,
                    ),
                )??;
        }
        for topic in self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_topic_list::new(),
            )?
        {
            self.participant_address
                .send_mail_and_await_reply_blocking(domain_participant_actor::delete_topic::new(
                    topic.send_mail_and_await_reply_blocking(
                        topic_actor::get_instance_handle::new(),
                    )?,
                ))?;
        }
        Ok(())
    }

    /// This operation manually asserts the liveliness of the [`DomainParticipant`]. This is used in combination
    /// with the [`LivelinessQosPolicy`](crate::infrastructure::qos_policy::LivelinessQosPolicy)
    /// to indicate to the Service that the entity remains active.
    /// This operation needs to only be used if the [`DomainParticipant`] contains  [`DataWriter`](crate::publication::data_writer::DataWriter) entities with the LIVELINESS set to
    /// MANUAL_BY_PARTICIPANT and it only affects the liveliness of those  [`DataWriter`](crate::publication::data_writer::DataWriter) entities. Otherwise, it has no effect.
    /// NOTE: Writing data via the write operation on a  [`DataWriter`](crate::publication::data_writer::DataWriter) asserts liveliness on the DataWriter itself and its
    /// [`DomainParticipant`]. Consequently the use of this operation is only needed if the application is not writing data regularly.
    #[tracing::instrument(skip(self))]
    pub fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
        // self.call_participant_method(|dp| {
        //     crate::implementation::behavior::domain_participant::assert_liveliness(dp)
        // })
    }

    /// This operation sets a default value of the Publisher QoS policies which will be used for newly created [`Publisher`] entities in the
    /// case where the QoS policies are defaulted in the [`DomainParticipant::create_publisher()`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistenPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values the default values of [`PublisherQos`].
    #[tracing::instrument(skip(self))]
    pub fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::set_default_publisher_qos::new(qos),
        )
    }

    /// This operation retrieves the default value of the Publisher QoS, that is, the QoS policies which will be used for newly created
    /// [`Publisher`] entities in the case where the QoS policies are defaulted in the [`DomainParticipant::create_publisher()`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`DomainParticipant::set_default_publisher_qos()`], or else, if the call was never made, the default values of the [`PublisherQos`].
    #[tracing::instrument(skip(self))]
    pub fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::default_publisher_qos::new(),
        )
    }

    /// This operation sets a default value of the Subscriber QoS policies that will be used for newly created [`Subscriber`] entities in the
    /// case where the QoS policies are defaulted in the [`DomainParticipant::create_subscriber()`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistenPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the default values of [`SubscriberQos`].
    #[tracing::instrument(skip(self))]
    pub fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::set_default_subscriber_qos::new(qos),
        )
    }

    /// This operation retrieves the default value of the Subscriber QoS, that is, the QoS policies which will be used for newly created
    /// [`Subscriber`] entities in the case where the QoS policies are defaulted in the [`DomainParticipant::create_subscriber()`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`DomainParticipant::set_default_subscriber_qos()`], or else, if the call was never made, the default values of [`SubscriberQos`].
    #[tracing::instrument(skip(self))]
    pub fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::default_subscriber_qos::new(),
        )
    }

    /// This operation sets a default value of the Topic QoS policies which will be used for newly created [`Topic`] entities in the case
    /// where the QoS policies are defaulted in the [`DomainParticipant::create_topic`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistenPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default QoS should be reset
    /// back to the initial values the factory would use, that is the default values of [`TopicQos`].
    #[tracing::instrument(skip(self))]
    pub fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::set_default_topic_qos::new(qos),
        )
    }

    /// This operation retrieves the default value of the Topic QoS, that is, the QoS policies that will be used for newly created [`Topic`]
    /// entities in the case where the QoS policies are defaulted in the [`DomainParticipant::create_topic()`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`DomainParticipant::set_default_topic_qos()`], or else, if the call was never made, the default values of [`TopicQos`]
    #[tracing::instrument(skip(self))]
    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::default_topic_qos::new())
    }

    /// This operation retrieves the list of DomainParticipants that have been discovered in the domain and that the application has not
    /// indicated should be “ignored” by means of the [`DomainParticipant::ignore_participant()`] operation.
    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::get_discovered_participants::new(),
        )
    }

    /// This operation retrieves information on a [`DomainParticipant`] that has been discovered on the network. The participant must
    /// be in the same domain as the participant on which this operation is invoked and must not have been “ignored” by means of the
    /// [`DomainParticipant::ignore_participant()`] operation.
    /// The participant_handle must correspond to such a DomainParticipant. Otherwise, the operation will fail and return
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// Use the operation [`DomainParticipant::get_discovered_participants()`] to find the DomainParticipants that are currently discovered.
    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participant_data(
        &self,
        _participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        todo!()
    }

    /// This operation retrieves the list of Topics that have been discovered in the domain and that the application has not indicated
    /// should be “ignored” by means of the [`DomainParticipant::ignore_topic()`] operation.
    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::discovered_topic_list::new(),
        )
    }

    /// This operation retrieves information on a Topic that has been discovered on the network. The topic must have been created by
    /// a participant in the same domain as the participant on which this operation is invoked and must not have been “ignored” by
    /// means of the [`DomainParticipant::ignore_topic()`] operation.
    /// The `topic_handle` must correspond to such a topic. Otherwise, the operation will fail and return
    /// [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// Use the operation [`DomainParticipant::get_discovered_topics()`] to find the topics that are currently discovered.
    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::discovered_topic_data::new(topic_handle),
            )?
    }

    /// This operation checks whether or not the given `a_handle` represents an Entity that was created from the [`DomainParticipant`].
    /// The containment applies recursively. That is, it applies both to entities ([`Topic`], [`Publisher`], or [`Subscriber`]) created
    /// directly using the [`DomainParticipant`] as well as entities created using a contained [`Publisher`], or [`Subscriber`] as the factory, and
    /// so forth.
    /// The instance handle for an Entity may be obtained from built-in topic data, from various statuses, or from the Entity operation
    /// `get_instance_handle`.
    #[tracing::instrument(skip(self))]
    pub fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
        // self.call_participant_method(|dp| {
        //     crate::implementation::behavior::domain_participant::contains_entity(dp, a_handle)
        // })
    }

    /// This operation returns the current value of the time that the service uses to time-stamp data-writes and to set the reception timestamp
    /// for the data-updates it receives.
    #[tracing::instrument(skip(self))]
    pub fn get_current_time(&self) -> DdsResult<Time> {
        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::get_current_time::new())
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
    #[tracing::instrument(skip(self))]
    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                DomainParticipantFactory::get_instance().get_default_participant_qos()?
            }
            QosKind::Specific(q) => q,
        };

        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::set_qos::new(qos))
    }

    /// This operation allows access to the existing set of [`DomainParticipantQos`] policies.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::get_qos::new())
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    #[tracing::instrument(skip(self, a_listener))]
    pub fn set_listener(
        &self,
        a_listener: impl DomainParticipantListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address.send_mail_and_await_reply_blocking(
            domain_participant_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ),
        )
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
    #[tracing::instrument(skip(self))]
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
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
    #[tracing::instrument(skip(self))]
    pub fn enable(&self) -> DdsResult<()> {
        if !self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_enabled::new())?
        {
            self.participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_builtin_publisher::new(),
                )?
                .send_mail_and_await_reply_blocking(publisher_actor::enable::new())?;
            self.participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_built_in_subscriber::new(),
                )?
                .send_mail_and_await_reply_blocking(subscriber_actor::enable::new())?;

            for builtin_reader in self
                .participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_built_in_subscriber::new(),
                )?
                .send_mail_and_await_reply_blocking(subscriber_actor::data_reader_list::new())?
            {
                builtin_reader
                    .send_mail_and_await_reply_blocking(data_reader_actor::enable::new())?;
            }

            for builtin_writer in self
                .participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_builtin_publisher::new(),
                )?
                .send_mail_and_await_reply_blocking(publisher_actor::data_writer_list::new())?
            {
                builtin_writer
                    .send_mail_and_await_reply_blocking(data_writer_actor::enable::new())?;
            }

            self.participant_address
                .send_mail_and_await_reply_blocking(domain_participant_actor::enable::new())?;

            let domain_participant_address = self.participant_address.clone();

            // Spawn the task that regularly announces the domain participant
            self.runtime_handle.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                loop {
                    let r: DdsResult<()> = async {
                        let builtin_publisher = domain_participant_address
                            .send_mail_and_await_reply(domain_participant_actor::get_builtin_publisher::new())
                            .await?;
                        let data_writer_list = builtin_publisher
                            .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
                            .await?;
                        for data_writer in data_writer_list {
                            if data_writer
                                .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
                                .await
                                == Ok("SpdpDiscoveredParticipantData".to_string())
                            {
                                let spdp_discovered_participant_data = domain_participant_address
                                    .send_mail_and_await_reply(
                                        domain_participant_actor::as_spdp_discovered_participant_data::new(),
                                    )
                                    .await?;
                                let mut serialized_data = Vec::new();
                                spdp_discovered_participant_data.serialize_data(&mut serialized_data)?;

                                let timestamp = domain_participant_address
                                    .send_mail_and_await_reply(
                                        domain_participant_actor::get_current_time::new(),
                                    )
                                    .await?;

                                data_writer
                                    .send_mail_and_await_reply(
                                        data_writer_actor::write_w_timestamp::new(
                                            serialized_data,
                                            get_instance_handle_from_key(&spdp_discovered_participant_data.get_key()?)
                                                .unwrap(),
                                            None,
                                            timestamp,
                                        ),
                                    )
                                    .await??;


                                domain_participant_address.send_mail(domain_participant_actor::send_message::new()).await?;
                            }
                        }

                        Ok(())
                    }
                    .await;

                    if r.is_err() {
                        break;
                    }

                    interval.tick().await;
                }
            });
        }

        Ok(())
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    #[tracing::instrument(skip(self))]
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::get_instance_handle::new())
    }
}
