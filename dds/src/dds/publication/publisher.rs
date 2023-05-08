use crate::{
    domain::{
        domain_participant::DomainParticipant,
        domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
    },
    implementation::dds_impl::{
        any_data_writer_listener::AnyDataWriterListener,
        dds_domain_participant::DdsDomainParticipant, node_kind::DataWriterNodeKind,
        node_user_defined_publisher::UserDefinedPublisherNode,
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    publication::data_writer::DataWriter,
    topic_definition::topic::Topic,
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::{data_writer_listener::DataWriterListener, publisher_listener::PublisherListener};

/// The [`Publisher`] acts on the behalf of one or several [`DataWriter`] objects that belong to it. When it is informed of a change to the
/// data associated with one of its [`DataWriter`] objects, it decides when it is appropriate to actually send the data-update message.
/// In making this decision, it considers any extra information that goes with the data (timestamp, writer, etc.) as well as the QoS
/// of the [`Publisher`] and the [`DataWriter`].
#[derive(Eq, PartialEq, Debug)]
pub struct Publisher(pub(crate) UserDefinedPublisherNode);

impl Publisher {
    pub(crate) fn new(publisher: UserDefinedPublisherNode) -> Self {
        Self(publisher)
    }
}

impl Publisher {
    /// This operation creates a [`DataWriter`]. The returned [`DataWriter`] will be attached and belongs to the [`Publisher`].
    /// The [`DataWriter`] returned by this operation has an associated [`Topic`] and a type `Foo`.
    /// The [`Topic`] passed to this operation must have been created from the same [`DomainParticipant`] that was used to create this
    /// [`Publisher`]. If the [`Topic`] was created from a different [`DomainParticipant`], the operation will fail and
    /// return a [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError). In case of failure, the operation
    /// will return an error and no writer will be created.
    ///
    /// The special value [`QosKind::Default`] can be used to indicate that the [`DataWriter`] should be created with the
    /// default qos set in the factory. The use of this value is equivalent to the application obtaining the default
    /// [`DataWriterQos`] by means of the operation [`Publisher::get_default_datawriter_qos`] and using the resulting qos
    /// to create the [`DataWriter`]. A common application pattern to construct the [`DataWriterQos`] to ensure consistency with the
    /// associated [`TopicQos`] is to:
    /// 1. Retrieve the QoS policies on the associated [`Topic`] by means of the [`Topic::get_qos`] operation.
    /// 2. Retrieve the default [`DataWriterQos`] qos by means of the [`Publisher::get_default_datawriter_qos`] operation.
    /// 3. Combine those two qos policies using the [`Publisher::copy_from_topic_qos`] and selectively modify policies as desired and
    /// use the resulting [`DataWriterQos`] to construct the [`DataWriter`].

    pub fn create_datawriter<Foo>(
        &self,
        a_topic: &Topic<Foo>,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DataWriter<Foo>>
    where
        Foo: DdsType + DdsSerialize + 'static,
    {
        let type_name = a_topic.get_type_name()?;
        let topic_name = a_topic.get_name()?;

        self.call_participant_mut_method(|dp| {
            #[allow(clippy::redundant_closure)]
            self.0
                .create_datawriter::<Foo>(
                    dp,
                    type_name,
                    topic_name,
                    qos,
                    a_listener
                        .map::<Box<dyn AnyDataWriterListener + Send + Sync>, _>(|x| Box::new(x)),
                    mask,
                )
                .map(|x| DataWriter::new(DataWriterNodeKind::UserDefined(x)))
        })
    }

    /// This operation deletes a [`DataWriter`] that belongs to the [`Publisher`]. This operation must be called on the
    /// same [`Publisher`] object used to create the [`DataWriter`]. If [`Publisher::delete_datawriter`] is called on a
    /// different [`Publisher`], the operation will have no effect and it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// The deletion of the [`DataWriter`] will automatically unregister all instances. Depending on the settings of the
    /// [`WriterDataLifecycleQosPolicy`](crate::infrastructure::qos_policy::WriterDataLifecycleQosPolicy), the deletion of the
    /// [`DataWriter`].
    pub fn delete_datawriter<Foo>(&self, a_datawriter: &DataWriter<Foo>) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize + 'static,
    {
        self.call_participant_mut_method(|dp| {
            self.0
                .delete_datawriter(dp, a_datawriter.get_instance_handle()?)
        })
    }

    /// This operation retrieves a previously created [`DataWriter`] belonging to the [`Publisher`] that is attached to a [`Topic`] with a matching
    /// `topic_name`. If no such [`DataWriter`] exists, the operation will succeed but return [`None`].
    /// If multiple [`DataWriter`] attached to the [`Publisher`] satisfy this condition, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_datawriter<Foo>(&self, topic: &Topic<Foo>) -> DdsResult<Option<DataWriter<Foo>>>
    where
        Foo: DdsType + DdsSerialize,
    {
        self.0
            .lookup_datawriter(topic.get_type_name()?, &topic.get_name()?)
            .map(|x| Some(DataWriter::new(DataWriterNodeKind::UserDefined(x))))
    }

    /// This operation indicates to the Service that the application is about to make multiple modifications using [`DataWriter`] objects
    /// belonging to the [`Publisher`]. It is a hint to the Service so it can optimize its performance by e.g., holding the
    /// dissemination of the modifications and then batching them. It is not required that the Service use this hint in any way.
    /// The use of this operation must be matched by a corresponding call to [`Publisher::resume_publications`] indicating that the set of
    /// modifications has completed. If the [`Publisher`] is deleted before [`Publisher::resume_publications`] is called, any suspended updates yet to
    /// be published will be discarded.
    pub fn suspend_publications(&self) -> DdsResult<()> {
        self.0.suspend_publications()
    }

    /// This operation indicates to the Service that the application has completed the multiple changes initiated by the previous
    /// [`Publisher::suspend_publications`] call. This is a hint to the Service that can be used by a Service implementation to
    /// e.g., batch all the modifications made since the [`Publisher::suspend_publications`].
    /// The call to [`Publisher::resume_publications`] must match a previous call to [`Publisher::suspend_publications`] otherwise
    /// the operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    pub fn resume_publications(&self) -> DdsResult<()> {
        self.0.resume_publications()
    }

    /// This operation requests that the application will begin a *coherent set* of modifications using [`DataWriter`] objects attached to
    /// the [`Publisher`]. The *coherent set* will be completed by a matching call to [`Publisher::end_coherent_changes`].
    /// A *coherent set* is a set of modifications that must be propagated in such a way that they are interpreted at the receivers’ side
    /// as a consistent set of modifications; that is, the receiver will only be able to access the data after all the modifications in the set
    /// are available at the receiver end. This does not imply that the middleware has to encapsulate all the modifications in a single message;
    /// it only implies that the receiving applications will behave as if this was the case.
    /// A connectivity change may occur in the middle of a set of coherent changes; for example, the set of partitions used by the
    /// [`Publisher`] or one of its subscribers may change, a late-joining [`DataReader`](crate::subscription::data_reader::DataReader)
    /// may appear on the network, or a communication failure may occur. In the event that such a change prevents an entity from
    /// receiving the entire set of coherent changes, that entity must behave as if it had received none of the set.
    /// These calls can be nested. In that case, the coherent set terminates only with the last call to [`Publisher::end_coherent_changes`].
    /// The support for *coherent changes* enables a publishing application to change the value of several data-instances that could
    /// belong to the same or different topics and have those changes be seen *atomically* by the readers. This is useful in cases where
    /// the values are inter-related (for example, if there are two data-instances representing the ‘altitude’ and ‘velocity vector’ of the
    /// same aircraft and both are changed, it may be useful to communicate those values in a way the reader can see both together;
    /// otherwise, it may e.g., erroneously interpret that the aircraft is on a collision course).
    pub fn begin_coherent_changes(&self) -> DdsResult<()> {
        self.0.begin_coherent_changes()
    }

    /// This operation terminates the *coherent set* initiated by the matching call to [`Publisher::begin_coherent_changes`]. If there is no matching
    /// call to [`Publisher::begin_coherent_changes`], the operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    pub fn end_coherent_changes(&self) -> DdsResult<()> {
        self.0.end_coherent_changes()
    }

    /// This operation blocks the calling thread until either all data written by the reliable [`DataWriter`] entities is acknowledged by all
    /// matched reliable [`DataReader`](crate::subscription::data_reader::DataReader) entities, or else the duration specified by
    /// the `max_wait` parameter elapses, whichever happens first. A return value of [`Ok`] indicates that all the samples written
    /// have been acknowledged by all reliable matched data readers; a return value of [`DdsError::Timeout`](crate::infrastructure::error::DdsError)
    /// indicates that `max_wait` elapsed before all the data was acknowledged.
    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        self.0.wait_for_acknowledgments(max_wait)
    }

    /// This operation returns the [`DomainParticipant`] to which the [`Publisher`] belongs.
    pub fn get_participant(&self) -> DdsResult<DomainParticipant> {
        Ok(DomainParticipant::new(self.0.get_participant()?))
    }

    /// This operation deletes all the entities that were created by means of the [`Publisher::create_datawriter`] operations.
    /// That is, it deletes all contained [`DataWriter`] objects.
    /// The operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError) if the any of the
    /// contained entities is in a state where it cannot be deleted.
    /// Once this operation returns successfully, the application may delete the [`Publisher`] knowing that it has no
    /// contained [`DataWriter`] objects
    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.0.delete_contained_entities()
    }

    /// This operation sets the default value of the [`DataWriterQos`] which will be used for newly created [`DataWriter`] entities in
    /// the case where the qos policies are defaulted in the [`Publisher::create_datawriter`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistentPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default qos should be
    /// reset back to the initial values the factory would use, that is the default value of [`DataWriterQos`].
    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.call_participant_method(|dp| self.0.set_default_datawriter_qos(dp, qos))
    }

    /// This operation retrieves the default factory value of the [`DataWriterQos`], that is, the qos policies which will be used for newly created
    /// [`DataWriter`] entities in the case where the qos policies are defaulted in the [`Publisher::create_datawriter`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`Publisher::set_default_datawriter_qos`], or else, if the call was never made, the default values of [`DataWriterQos`].
    pub fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        self.call_participant_method(|dp| self.0.get_default_datawriter_qos(dp))
    }

    /// This operation copies the policies in the `a_topic_qos` to the corresponding policies in the `a_datawriter_qos`.
    /// This is a convenience operation most useful in combination with the operations [`Publisher::get_default_datawriter_qos`] and
    /// [`Topic::get_qos`]. This operation can be used to merge the [`DataWriterQos`] default qos policies with the
    /// corresponding ones on the [`Topic`]. The resulting qos can then be used to create a new [`DataWriter`], or set its qos.
    /// This operation does not check the resulting `a_datawriter_qos` for consistency. This is because the merged `a_datawriter_qos`
    /// may not be the final one, as the application can still modify some policies prior to applying the policies to the [`DataWriter`].
    pub fn copy_from_topic_qos(
        &self,
        a_datawriter_qos: &mut DataWriterQos,
        a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        self.0.copy_from_topic_qos(a_datawriter_qos, a_topic_qos)
    }
}

/// This implementation block contains the Entity operations for the [`Publisher`].
impl Publisher {
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
    pub fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.0.set_qos(qos)
    }

    /// This operation allows access to the existing set of [`PublisherQos`] policies.
    pub fn get_qos(&self) -> DdsResult<PublisherQos> {
        self.call_participant_method(|dp| self.0.get_qos(dp))
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.0.set_listener(a_listener, mask)
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(self.0.get_statuscondition()?))
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        self.0.get_status_changes()
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
        self.0.enable()
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.0.get_instance_handle()
    }

    fn call_participant_method<F, O>(&self, f: F) -> DdsResult<O>
    where
        F: FnOnce(&DdsDomainParticipant) -> DdsResult<O>,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(&self.0.parent().prefix(), |dp| {
            f(dp.ok_or(DdsError::AlreadyDeleted)?)
        })
    }

    fn call_participant_mut_method<F, O>(&self, f: F) -> DdsResult<O>
    where
        F: FnOnce(&mut DdsDomainParticipant) -> DdsResult<O>,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&self.0.parent().prefix(), |dp| {
            f(dp.ok_or(DdsError::AlreadyDeleted)?)
        })
    }
}
