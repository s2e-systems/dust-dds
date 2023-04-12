use std::marker::PhantomData;

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::dds_impl::{
        any_data_writer_listener::AnyDataWriterListener,
        node_kind::{DataWriterNodeKind, TopicNodeKind},
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    publication::{data_writer_listener::DataWriterListener, publisher::Publisher},
    topic_definition::{
        topic::Topic,
        type_support::{DdsSerialize, DdsType, LittleEndian},
    },
};

/// The [`DataWriter`] allows the application to set the value of the
/// data to be published under a given [`Topic`].
pub struct DataWriter<Foo>(DataWriterNodeKind, PhantomData<Foo>);

impl<Foo> DataWriter<Foo> {
    pub(crate) fn new(data_writer: DataWriterNodeKind) -> Self {
        Self(data_writer, PhantomData)
    }
}

impl<Foo> DataWriter<Foo>
where
    Foo: DdsType + DdsSerialize,
{
    /// This operation informs the Service that the application will be modifying a particular instance.
    /// It gives an opportunity to the Service to pre-configure itself to improve performance. It takes
    /// as a parameter an `instance` (to get the key value) and returns an [`InstanceHandle`] that can be
    /// used in successive [`DataWriter::write`] or [`DataWriter::dispose`] operations.
    /// This operation should be invoked prior to calling any operation that modifies the instance, such as
    /// [`DataWriter::write`], [`DataWriter::write_w_timestamp`], [`DataWriter::dispose`] and [`DataWriter::dispose_w_timestamp`].
    /// The operation may return [`None`] if the Service does not want to allocate any handle for that instance.
    /// This operation may block and return [`DdsError::Timeout`](crate::infrastructure::error::DdsError) or
    /// [`DdsError::OutOfResources`](crate::infrastructure::error::DdsError) under the same circumstances
    /// described for [`DataWriter::write`].
    /// This operation is idempotent. If it is called for an already registered instance, it just returns the already
    /// allocated [`InstanceHandle`]. This may be used to lookup and retrieve the handle allocated to a given instance.
    /// The explicit use of this operation is optional as the application may call directly [`DataWriter::write`]
    /// and specify no [`InstanceHandle`] to indicate that the *key* should be examined to identify the instance.
    pub fn register_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let timestamp = self
            .get_publisher()?
            .get_participant()?
            .get_current_time()?;
        self.register_instance_w_timestamp(instance, timestamp)
    }

    /// This operation performs the same function and return the same values as [`DataWriter::register_instance`] and can be used instead of
    /// [`DataWriter::register_instance`] in the cases where the application desires to specify the value for the `source_timestamp`.
    /// The `source_timestamp` potentially affects the relative order in which readers observe events from multiple writers.
    /// For details see [`DestinationOrderQosPolicy`](crate::infrastructure::qos_policy::DestinationOrderQosPolicy).
    pub fn register_instance_w_timestamp(
        &self,
        instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => {
                w.register_instance_w_timestamp(instance.get_serialized_key(), timestamp)
            }
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation reverses the action of [`DataWriter::register_instance`]. It should only be called on an
    /// instance that is currently registered. This operation should be called just once per instance,
    /// regardless of how many times [`DataWriter::register_instance`] was called for that instance.
    /// This operation informs the Service that the [`DataWriter`] is not intending to modify any more of that
    /// data instance. This operation also indicates that the Service can locally remove all information regarding
    /// that instance. The application should not attempt to use the handle previously allocated to that instance
    /// after calling [`DataWriter::unregister_instance`].
    /// If [`None`] is used as the `handle` argument it indicates that the identity of the instance should
    /// be automatically deduced from the instance (by means of the key).
    /// If handle is any value other than [`None`], then it must correspond to the value returned by register_instance when the
    /// instance (identified by its key) was registered. Otherwise the behavior is as follows:
    /// - If the handle corresponds to an existing instance but does not correspond to the same instance referred by the 'instance'
    /// parameter, the operation fails and returns [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// - If the handle does not correspond to an existing instance the operation fails and returns
    /// [`DdsError::BadParameter`](crate::infrastructure::error::DdsError).
    /// If after that, the application wants to modify (write or dispose) the instance, it has to register it again,
    /// or else pass [`None`] as the `handle` value of those operations.
    /// This operation does not indicate that the instance is deleted (that is the purpose of dispose). This operation
    /// just indicates that the [`DataWriter`] no longer has ‘anything to say’ about the instance.
    /// [`DataReader`](crate::subscription::data_reader::DataReader) entities that are reading the instance will eventually
    /// receive a sample with an [`InstanceStateKind::NotAliveNoWriter`](crate::subscription::sample_info::InstanceStateKind)
    /// if no other [`DataWriter`] entities are writing the instance.
    /// This operation can affect the ownership of the data instance as described
    /// in [`OwnershipQosPolicy`](crate::infrastructure::qos_policy::OwnershipQosPolicy).
    /// If the [`DataWriter`] was the exclusive owner of the instance, then calling [`DataWriter::unregister_instance`]
    /// will relinquish that ownership.
    /// This operation may block and return [`DdsError::Timeout`](crate::infrastructure::error::DdsError) under the
    /// same circumstances described for [`DataWriter::write`].
    pub fn unregister_instance(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()?
            .get_participant()?
            .get_current_time()?;
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    /// This operation performs the same function and returns the same values as [`DataWriter::unregister_instance`] and can
    /// be used instead of [`DataWriter::unregister_instance`] in the cases where the application desires to specify the
    /// value for the `source_timestamp`.
    /// The `source_timestamp` potentially affects the relative order in which readers observe events from multiple writers.
    /// For details see [`DestinationOrderQosPolicy`](crate::infrastructure::qos_policy::DestinationOrderQosPolicy).
    pub fn unregister_instance_w_timestamp(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if Foo::has_key() {
            let instance_handle = match handle {
                Some(h) => {
                    if let Some(stored_handle) = self.lookup_instance(instance)? {
                        if stored_handle == h {
                            Ok(h)
                        } else {
                            Err(DdsError::PreconditionNotMet(
                                "Handle does not match instance".to_string(),
                            ))
                        }
                    } else {
                        Err(DdsError::BadParameter)
                    }
                }
                None => {
                    if let Some(stored_handle) = self.lookup_instance(instance)? {
                        Ok(stored_handle)
                    } else {
                        Err(DdsError::PreconditionNotMet(
                            "Instance not registered with this DataWriter".to_string(),
                        ))
                    }
                }
            }?;

            let mut serialized_key = Vec::new();
            instance
                .get_serialized_key()
                .serialize::<_, LittleEndian>(&mut serialized_key)?;

            match &self.0 {
                DataWriterNodeKind::UserDefined(w) => {
                    w.unregister_instance_w_timestamp(serialized_key, instance_handle, timestamp)
                }
                DataWriterNodeKind::Listener(_) => todo!(),
            }
        } else {
            Err(DdsError::IllegalOperation)
        }
    }

    /// This operation can be used to retrieve the instance key that corresponds to an `handle`. The operation will only fill the
    /// fields that form the key inside the `key_holder` instance.
    /// This operation returns [`DdsError::BadParameter`](crate::infrastructure::error::DdsError) if the `handle` does not
    /// correspond to an existing data object known to the [`DataWriter`].
    pub fn get_key_value(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_key_value(key_holder, handle),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation takes as a parameter an instance and returns an [`InstanceHandle`] that can be used in subsequent operations
    /// that accept an [`InstanceHandle`] as an argument. The `instance` parameter is only used for the purpose of examining the
    /// fields that define the key.
    /// This operation does not register the instance in question. If the instance has not been previously registered, or if for any other
    /// reason the Service is unable to provide an [`InstanceHandle`], the operation will return [`None`].
    pub fn lookup_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.lookup_instance(instance.get_serialized_key()),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation modifies the value of a data instance. When this operation is used, the Service will automatically supply the
    /// value of the source timestamp that is made available to [`DataReader`](crate::subscription::data_reader::DataReader)
    /// objects by means of the [`SampleInfo::source_timestamp`](crate::subscription::sample_info::SampleInfo).
    /// As a side effect, this operation asserts liveliness on the [`DataWriter`] itself, the [`Publisher`] and the
    /// [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
    /// If [`None`] is used as the `handle` argument this indicates that the identity of the instance should be automatically deduced
    /// from the `data` (by means of the key).
    /// If `handle` is any value other than [`None`], then it must correspond to the value returned by [`DataWriter::register_instance`]
    /// when the instance (identified by its key) was registered. Otherwise the behavior is as follows:
    /// - If the `handle` corresponds to an existing instance but does not correspond to the same instance referred by the ‘data’
    /// parameter, the operation fails and returns [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError).
    /// - If the `handle` does not correspond to an existing instance the operation fails and returns [`DdsError::BadParameter`](crate::infrastructure::error::DdsError).
    ///
    /// If the [`ReliabilityQosPolicy`](crate::infrastructure::qos_policy::ReliabilityQosPolicyKind) is set to [`ReliabilityQosPolicyKind::Reliable`](crate::infrastructure::qos_policy::ReliabilityQosPolicyKind) this operation may block if the modification would cause data to be lost
    /// or else cause one of the limits specified in the [`ResourceLimitsQosPolicy`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy) to be exceeded. Under these circumstances, the
    /// [`ReliabilityQosPolicy::max_blocking_time`](crate::infrastructure::qos_policy::ReliabilityQosPolicy) configures the maximum time the [`DataWriter::write`] operation may block waiting for space to become
    /// available. If [`ReliabilityQosPolicy::max_blocking_time`](crate::infrastructure::qos_policy::ReliabilityQosPolicy) elapses before the [`DataWriter`] is able to store the modification without exceeding the limits,
    /// the write operation will fail and return [`DdsError::Timeout`](crate::infrastructure::error::DdsError).
    /// Specifically, the [`DataWriter::write`] operation may block in the following situations (note that the list may not be exhaustive),
    /// even if configured with [`HistoryQosPolicyKind::KeepLast`](crate::infrastructure::qos_policy::HistoryQosPolicyKind):
    /// - If ([`ResourceLimitsQosPolicy::max_samples`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy) <
    /// [`ResourceLimitsQosPolicy::max_instances`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy) * [`HistoryQosPolicy::depth`](crate::infrastructure::qos_policy::HistoryQosPolicy)), then in the
    /// situation where the [`ResourceLimitsQosPolicy::max_samples`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy) resource limit is exhausted
    /// the Service is allowed to discard samples of some other instance as long as at least one sample remains for such an instance.
    /// If it is still not possible to make space available to store the modification, the writer is allowed to block.
    /// - If ([`ResourceLimitsQosPolicy::max_samples`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy) < [`ResourceLimitsQosPolicy::max_instances`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy)),
    /// then the [`DataWriter`] may block regardless of the [`HistoryQosPolicy::depth`](crate::infrastructure::qos_policy::HistoryQosPolicy).
    ///
    /// Instead of blocking, the write operation is allowed to return immediately with the error code [`DdsError::OutOfResources`](crate::infrastructure::error::DdsError)
    /// provided that the reason for blocking would be that the [`ResourceLimitsQosPolicy`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy)
    /// is exceeded and the service determines that even waiting the [`ReliabilityQosPolicy::max_waiting_time`](crate::infrastructure::qos_policy::ReliabilityQosPolicy) has no
    /// chance of freeing the necessary resources. For example, if the only way to gain the necessary resources would be for the user to unregister an instance.
    pub fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()?
            .get_participant()?
            .get_current_time()?;
        self.write_w_timestamp(data, handle, timestamp)
    }

    /// This operation performs the same function and returns the same values as [`DataWriter::write`] and can
    /// be used instead of [`DataWriter::write`] in the cases where the application desires to specify the
    /// value for the `source_timestamp`.
    /// The `source_timestamp` potentially affects the relative order in which readers observe events from multiple writers.
    /// For details see [`DestinationOrderQosPolicy`](crate::infrastructure::qos_policy::DestinationOrderQosPolicy).
    pub fn write_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)?;

        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.write_w_timestamp(
                serialized_data,
                data.get_serialized_key(),
                handle,
                timestamp,
            ),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation requests the middleware to delete the data (the actual deletion is postponed until there is no more use for that
    /// data in the whole system). In general, applications are made aware of the deletion by means of operations on the
    /// [`DataReader`](crate::subscription::data_reader::DataReader) objects that already knew the instance.
    /// This operation does not modify the value of the instance. The `handle` parameter is passed just for the purposes of identifying
    /// the instance.
    /// When this operation is used, the Service will automatically supply the value of the source timestamp that is made available
    /// to [`DataReader`](crate::subscription::data_reader::DataReader) objects by means of the
    /// [`SampleInfo::source_timestamp`](crate::subscription::sample_info::SampleInfo).
    /// The constraints on the values of the handle parameter and the corresponding error behavior are the same specified for the
    /// [`DataWriter::unregister_instance`] operation.
    /// This operation may block and return [`DdsError::Timeout`](crate::infrastructure::error::DdsError) or
    /// [`DdsError::OutOfResources`](crate::infrastructure::error::DdsError) under the same circumstances described for [`DataWriter::write`].
    pub fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()?
            .get_participant()?
            .get_current_time()?;
        self.dispose_w_timestamp(data, handle, timestamp)
    }

    /// This operation performs the same function and returns the same values as [`DataWriter::dispose`] and can
    /// be used instead of [`DataWriter::dispose`] in the cases where the application desires to specify the
    /// value for the `source_timestamp`.
    /// The `source_timestamp` potentially affects the relative order in which readers observe events from multiple writers.
    /// For details see [`DestinationOrderQosPolicy`](crate::infrastructure::qos_policy::DestinationOrderQosPolicy).
    pub fn dispose_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let instance_handle = match handle {
            Some(h) => {
                if let Some(stored_handle) = self.lookup_instance(data)? {
                    if stored_handle == h {
                        Ok(h)
                    } else {
                        Err(DdsError::PreconditionNotMet(
                            "Handle does not match instance".to_string(),
                        ))
                    }
                } else {
                    Err(DdsError::BadParameter)
                }
            }
            None => {
                if let Some(stored_handle) = self.lookup_instance(data)? {
                    Ok(stored_handle)
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Instance not registered with this DataWriter".to_string(),
                    ))
                }
            }
        }?;

        let mut serialized_key = Vec::new();
        data.get_serialized_key()
            .serialize::<_, LittleEndian>(&mut serialized_key)?;

        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => {
                w.dispose_w_timestamp(serialized_key, instance_handle, timestamp)
            }
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation blocks the calling thread until either all data written by the [`DataWriter`] is acknowledged by all
    /// matched [`DataReader`](crate::subscription::data_reader::DataReader) entities that have
    /// [`ReliabilityQosPolicyKind::Reliable`](crate::infrastructure::qos_policy::ReliabilityQosPolicyKind), or else the duration
    /// specified by the `max_wait` parameter elapses, whichever happens first. A return value of [`Ok`] indicates that all the samples
    /// written have been acknowledged by all reliable matched data readers; a return value of [`DdsError::Timeout`](crate::infrastructure::error::DdsError)
    /// indicates that `max_wait` elapsed before all the data was acknowledged.
    /// This operation is intended to be used only if the DataWriter has [`ReliabilityQosPolicyKind::Reliable`](crate::infrastructure::qos_policy::ReliabilityQosPolicyKind).
    /// Otherwise the operation will return immediately with [`Ok`].
    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.wait_for_acknowledgments(max_wait),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation allows access to the [`LivelinessLostStatus`].
    pub fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_liveliness_lost_status(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation allows access to the [`OfferedDeadlineMissedStatus`].
    pub fn get_offered_deadline_missed_status(&self) -> DdsResult<OfferedDeadlineMissedStatus> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_offered_deadline_missed_status(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation allows access to the [`OfferedIncompatibleQosStatus`].
    pub fn get_offered_incompatible_qos_status(&self) -> DdsResult<OfferedIncompatibleQosStatus> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_offered_incompatible_qos_status(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation allows access to the [`PublicationMatchedStatus`].
    pub fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_publication_matched_status(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation returns the [`Topic`] associated with the [`DataWriter`]. This is the same [`Topic`] that was used to create the [`DataWriter`].
    pub fn get_topic(&self) -> DdsResult<Topic<Foo>> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => {
                Ok(Topic::new(TopicNodeKind::UserDefined(w.get_topic()?)))
            }
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation returns the [`Publisher`] to which the [`DataWriter`] object belongs.
    pub fn get_publisher(&self) -> DdsResult<Publisher> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => Ok(Publisher::new(w.get_publisher())),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation manually asserts the liveliness of the [`DataWriter`]. This is used in combination with the
    /// [`LivelinessQosPolicy`](crate::infrastructure::qos_policy::LivelinessQosPolicy) to indicate to the Service that the entity remains active.
    /// This operation need only be used if the [`LivelinessQosPolicy`](crate::infrastructure::qos_policy::LivelinessQosPolicy) setting is either
    /// [`LivelinessQosPolicyKind::ManualByParticipant`](crate::infrastructure::qos_policy::LivelinessQosPolicyKind) or
    /// [`LivelinessQosPolicyKind::ManualByTopic`](crate::infrastructure::qos_policy::LivelinessQosPolicyKind). Otherwise, it has no effect.
    /// NOTE: Writing data via the [`DataWriter::write`] operation asserts liveliness on the [`DataWriter`] itself and its
    /// [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant). Consequently the use of this operation is only needed
    /// if the application is not writing data regularly.
    pub fn assert_liveliness(&self) -> DdsResult<()> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.assert_liveliness(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation retrieves information on a subscription that is currently “associated” with the [`DataWriter`]; that is, a subscription
    /// with a matching [`Topic`] and compatible QoS that the application has not indicated should be ignored by means of the
    /// [`DomainParticipant::ignore_subscription`](crate::domain::domain_participant::DomainParticipant) operation.
    /// The `subscription_handle` must correspond to a subscription currently associated with the [`DataWriter`], otherwise the operation
    /// will fail and return [`DdsError::BadParameter`](crate::infrastructure::error::DdsError). The operation [`DataWriter::get_matched_subscriptions`]
    /// can be used to find the subscriptions that are currently matched with the [`DataWriter`].
    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => {
                w.get_matched_subscription_data(subscription_handle)
            }
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation retrieves the list of subscriptions currently “associated” with the [`DataWriter`]]; that is, subscriptions that have a
    /// matching [`Topic`] and compatible QoS that the application has not indicated should be “ignored” by means of the
    ///  [`DomainParticipant::ignore_subscription`](crate::domain::domain_participant::DomainParticipant) operation.
    /// The handles returned are the ones that are used by the DDS implementation to locally identify the corresponding matched
    /// [`DataReader`](crate::subscription::data_reader::DataReader) entities. These handles match the ones that appear in the
    /// [`SampleInfo::instance_handle`](crate::subscription::sample_info::SampleInfo) field when reading the “DCPSSubscriptions” builtin topic.
    pub fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_matched_subscriptions(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }
}

/// This implementation block contains the Entity operations for the [`DataWriter`].
impl<Foo> DataWriter<Foo>
where
    Foo: DdsType + DdsSerialize + 'static,
{
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
    pub fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.set_qos(qos),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation allows access to the existing set of [`DataWriterQos`] policies.
    pub fn get_qos(&self) -> DdsResult<DataWriterQos> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_qos(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) =>
            {
                #[allow(clippy::redundant_closure)]
                w.set_listener(
                    a_listener
                        .map::<Box<dyn AnyDataWriterListener + Send + Sync>, _>(|l| Box::new(l)),
                    mask,
                )
            }
            DataWriterNodeKind::Listener(_) => Err(DdsError::IllegalOperation),
        }
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => {
                Ok(StatusCondition::new(w.get_statuscondition()?))
            }
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_status_changes(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
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
    /// Entities created from a factory that is disabled, are created disabled regardless of the setting of the ENTITY_FACTORY Qos
    /// policy.
    /// Calling enable on an Entity whose factory is not enabled will fail and return PRECONDITION_NOT_MET.
    /// If the `autoenable_created_entities` field of [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) is set to [`true`], the [`Self::enable()`] operation on the factory will
    /// automatically enable all entities created from the factory.
    /// The Listeners associated with an entity are not called until the entity is enabled. Conditions associated with an entity that is not
    /// enabled are “inactive,” that is, the operation [`StatusCondition::get_trigger_value()`] will always return `false`.
    pub fn enable(&self) -> DdsResult<()> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.enable(),
            DataWriterNodeKind::Listener(_) => Err(DdsError::IllegalOperation),
        }
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        match &self.0 {
            DataWriterNodeKind::UserDefined(w) => w.get_instance_handle(),
            DataWriterNodeKind::Listener(_) => todo!(),
        }
    }
}

pub trait AnyDataWriter {}
