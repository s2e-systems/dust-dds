use crate::{
    domain::domain_participant::DomainParticipant,
    implementation::{
        dds::{
            dds_data_writer,
            dds_data_writer_listener::DdsDataWriterListener,
            dds_domain_participant,
            nodes::{DataWriterNode, DataWriterNodeKind, PublisherNode},
        },
        rtps::messages::overall_structure::RtpsMessageHeader,
        utils::actor::spawn_actor,
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
    topic_definition::type_support::{DdsGetKey, DdsHasKey},
};

use super::{data_writer_listener::DataWriterListener, publisher_listener::PublisherListener};

/// The [`Publisher`] acts on the behalf of one or several [`DataWriter`] objects that belong to it. When it is informed of a change to the
/// data associated with one of its [`DataWriter`] objects, it decides when it is appropriate to actually send the data-update message.
/// In making this decision, it considers any extra information that goes with the data (timestamp, writer, etc.) as well as the QoS
/// of the [`Publisher`] and the [`DataWriter`].
pub struct Publisher(PublisherNode);

impl Publisher {
    pub(crate) fn new(publisher: PublisherNode) -> Self {
        Self(publisher)
    }

    pub(crate) fn node(&self) -> &PublisherNode {
        &self.0
    }
}

// impl Drop for Publisher {
//     fn drop(&mut self) {
//         todo!()
//         // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&self.0.guid().prefix(), |dp| {
//         //     if let Some(dp) = dp {
//         //         crate::implementation::behavior::domain_participant::delete_publisher(
//         //             dp,
//         //             self.0.guid(),
//         //         )
//         //         .ok();
//         //     }
//         // })
//     }
// }

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
        a_topic: &Topic,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListener<Foo> + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DataWriter<Foo>>
    where
        Foo: DdsHasKey + DdsGetKey + serde::Serialize + Send + 'static,
    {
        let default_unicast_locator_list = self
            .0
            .parent_participant()
            .get_default_unicast_locator_list()?;
        let default_multicast_locator_list = self
            .0
            .parent_participant()
            .get_default_multicast_locator_list()?;
        let data_max_size_serialized = self.0.parent_participant().data_max_size_serialized()?;

        let listener = a_listener.map(|l| spawn_actor(DdsDataWriterListener::new(Box::new(l))));
        let data_writer_address = self.0.address().create_datawriter(
            a_topic.get_type_name()?,
            a_topic.get_name()?,
            Foo::HAS_KEY,
            data_max_size_serialized,
            qos,
            listener,
            mask.to_vec(),
            default_unicast_locator_list,
            default_multicast_locator_list,
        )??;

        let data_writer = DataWriter::new(DataWriterNodeKind::UserDefined(DataWriterNode::new(
            data_writer_address,
            self.0.address().clone(),
            self.0.parent_participant().clone(),
        )));

        if self.0.address().is_enabled()?
            && self
                .0
                .address()
                .get_qos()?
                .entity_factory
                .autoenable_created_entities
        {
            data_writer.enable()?
        }

        Ok(data_writer)
    }

    /// This operation deletes a [`DataWriter`] that belongs to the [`Publisher`]. This operation must be called on the
    /// same [`Publisher`] object used to create the [`DataWriter`]. If [`Publisher::delete_datawriter`] is called on a
    /// different [`Publisher`], the operation will have no effect and it will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// The deletion of the [`DataWriter`] will automatically unregister all instances. Depending on the settings of the
    /// [`WriterDataLifecycleQosPolicy`](crate::infrastructure::qos_policy::WriterDataLifecycleQosPolicy), the deletion of the
    /// [`DataWriter`].
    pub fn delete_datawriter<Foo>(&self, a_datawriter: &DataWriter<Foo>) -> DdsResult<()> {
        match a_datawriter.node() {
            DataWriterNodeKind::Listener(_) => Err(DdsError::IllegalOperation),
            DataWriterNodeKind::UserDefined(dw) => {
                let writer_handle = dw.address().get_instance_handle()?;
                if self.0.address().guid()? != dw.parent_publisher().guid()? {
                    return Err(DdsError::PreconditionNotMet(
                        "Data writer can only be deleted from its parent publisher".to_string(),
                    ));
                }

                let writer_is_enabled = dw.address().is_enabled()?;
                self.0.address().datawriter_delete(writer_handle)?;

                // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
                if writer_is_enabled {
                    let instance_serialized_key =
                        cdr::serialize::<_, _, cdr::CdrLe>(&writer_handle, cdr::Infinite)
                            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
                            .expect("Failed to serialize data");

                    let timestamp = dw.parent_participant().get_current_time()?;

                    if let Some(sedp_writer_announcer) = dw
                        .parent_participant()
                        .send_and_reply_blocking(dds_domain_participant::GetBuiltinPublisher)?
                        .data_writer_list()?
                        .iter()
                        .find(|x| x.get_type_name().unwrap() == "DiscoveredWriterData")
                    {
                        sedp_writer_announcer.dispose_w_timestamp(
                            instance_serialized_key,
                            writer_handle,
                            timestamp,
                        )??;

                        sedp_writer_announcer.send_only_blocking(
                            dds_data_writer::SendMessage::new(
                                RtpsMessageHeader::new(
                                    dw.parent_participant().get_protocol_version()?,
                                    dw.parent_participant().get_vendor_id()?,
                                    dw.parent_participant().get_guid()?.prefix(),
                                ),
                                dw.parent_participant().get_udp_transport_write()?,
                                dw.parent_participant().get_current_time()?,
                            ),
                        )?;
                    }
                    // let timestamp = domain_participant.get_current_time();

                    // domain_participant
                    //     .get_builtin_publisher_mut()
                    //     .stateful_data_writer_list()
                    //     .iter()
                    //     .find(|x| x.get_type_name().unwrap() == DiscoveredWriterData)
                    //     .unwrap()
                    //     .dispose_w_timestamp(instance_serialized_key, writer_handle, timestamp)
                    //     .expect("Should not fail to write built-in message");
                }

                Ok(())
            }
        }

        // Ok(())
    }

    /// This operation retrieves a previously created [`DataWriter`] belonging to the [`Publisher`] that is attached to a [`Topic`] with a matching
    /// `topic_name`. If no such [`DataWriter`] exists, the operation will succeed but return [`None`].
    /// If multiple [`DataWriter`] attached to the [`Publisher`] satisfy this condition, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_datawriter<Foo>(&self, _topic_name: &str) -> DdsResult<Option<DataWriter<Foo>>>
    where
        Foo: DdsHasKey,
    {
        todo!()
        // self.call_participant_mut_method(|dp| {
        //     Ok(
        //         crate::implementation::behavior::user_defined_publisher::lookup_datawriter(
        //             dp,
        //             self.0.guid(),
        //             Foo,
        //             topic_name,
        //         )?
        //         .map(|x| DataWriter::new(DataWriterNodeKind::UserDefined(x))),
        //     )
        // })
    }

    /// This operation indicates to the Service that the application is about to make multiple modifications using [`DataWriter`] objects
    /// belonging to the [`Publisher`]. It is a hint to the Service so it can optimize its performance by e.g., holding the
    /// dissemination of the modifications and then batching them. It is not required that the Service use this hint in any way.
    /// The use of this operation must be matched by a corresponding call to [`Publisher::resume_publications`] indicating that the set of
    /// modifications has completed. If the [`Publisher`] is deleted before [`Publisher::resume_publications`] is called, any suspended updates yet to
    /// be published will be discarded.
    pub fn suspend_publications(&self) -> DdsResult<()> {
        todo!()
    }

    /// This operation indicates to the Service that the application has completed the multiple changes initiated by the previous
    /// [`Publisher::suspend_publications`] call. This is a hint to the Service that can be used by a Service implementation to
    /// e.g., batch all the modifications made since the [`Publisher::suspend_publications`].
    /// The call to [`Publisher::resume_publications`] must match a previous call to [`Publisher::suspend_publications`] otherwise
    /// the operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    pub fn resume_publications(&self) -> DdsResult<()> {
        todo!()
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
        todo!()
    }

    /// This operation terminates the *coherent set* initiated by the matching call to [`Publisher::begin_coherent_changes`]. If there is no matching
    /// call to [`Publisher::begin_coherent_changes`], the operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    pub fn end_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    /// This operation blocks the calling thread until either all data written by the reliable [`DataWriter`] entities is acknowledged by all
    /// matched reliable [`DataReader`](crate::subscription::data_reader::DataReader) entities, or else the duration specified by
    /// the `max_wait` parameter elapses, whichever happens first. A return value of [`Ok`] indicates that all the samples written
    /// have been acknowledged by all reliable matched data readers; a return value of [`DdsError::Timeout`](crate::infrastructure::error::DdsError)
    /// indicates that `max_wait` elapsed before all the data was acknowledged.
    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    /// This operation returns the [`DomainParticipant`] to which the [`Publisher`] belongs.
    pub fn get_participant(&self) -> DdsResult<DomainParticipant> {
        Ok(DomainParticipant::new(self.0.parent_participant().clone()))
    }

    /// This operation deletes all the entities that were created by means of the [`Publisher::create_datawriter`] operations.
    /// That is, it deletes all contained [`DataWriter`] objects.
    /// The operation will return [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError) if the any of the
    /// contained entities is in a state where it cannot be deleted.
    /// Once this operation returns successfully, the application may delete the [`Publisher`] knowing that it has no
    /// contained [`DataWriter`] objects
    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
        // crate::implementation::behavior::user_defined_publisher::delete_contained_entities()
    }

    /// This operation sets the default value of the [`DataWriterQos`] which will be used for newly created [`DataWriter`] entities in
    /// the case where the qos policies are defaulted in the [`Publisher::create_datawriter`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return [`DdsError::InconsistentPolicy`](crate::infrastructure::error::DdsError).
    /// The special value [`QosKind::Default`] may be passed to this operation to indicate that the default qos should be
    /// reset back to the initial values the factory would use, that is the default value of [`DataWriterQos`].
    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.0.address().get_default_datawriter_qos()?,
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.0.address().set_default_datawriter_qos(qos)
    }

    /// This operation retrieves the default factory value of the [`DataWriterQos`], that is, the qos policies which will be used for newly created
    /// [`DataWriter`] entities in the case where the qos policies are defaulted in the [`Publisher::create_datawriter`] operation.
    /// The values retrieved by this operation will match the set of values specified on the last successful call to
    /// [`Publisher::set_default_datawriter_qos`], or else, if the call was never made, the default values of [`DataWriterQos`].
    pub fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        self.0.address().get_default_datawriter_qos()
    }

    /// This operation copies the policies in the `a_topic_qos` to the corresponding policies in the `a_datawriter_qos`.
    /// This is a convenience operation most useful in combination with the operations [`Publisher::get_default_datawriter_qos`] and
    /// [`Topic::get_qos`]. This operation can be used to merge the [`DataWriterQos`] default qos policies with the
    /// corresponding ones on the [`Topic`]. The resulting qos can then be used to create a new [`DataWriter`], or set its qos.
    /// This operation does not check the resulting `a_datawriter_qos` for consistency. This is because the merged `a_datawriter_qos`
    /// may not be the final one, as the application can still modify some policies prior to applying the policies to the [`DataWriter`].
    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
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
    pub fn set_qos(&self, _qos: QosKind<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    /// This operation allows access to the existing set of [`PublisherQos`] policies.
    pub fn get_qos(&self) -> DdsResult<PublisherQos> {
        self.0.address().get_qos()
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
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
    pub fn enable(&self) -> DdsResult<()> {
        self.0.address().enable()
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.0.address().get_instance_handle()
    }
}
