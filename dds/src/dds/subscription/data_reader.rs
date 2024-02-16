use crate::{
    implementation::{
        actors::{
            data_reader_actor::{self, DataReaderActor},
            data_writer_actor,
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor,
            subscriber_actor::{self, SubscriberActor},
            topic_actor::{self, TopicActor},
        },
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        rtps::messages::submessage_elements::Data,
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        error::DdsError,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    subscription::data_reader_listener::DataReaderListener,
    topic_definition::{
        topic::Topic,
        type_support::{DdsDeserialize, DdsKey, DdsSerialize},
    },
    {
        builtin_topics::PublicationBuiltinTopicData,
        infrastructure::{
            condition::StatusCondition,
            error::DdsResult,
            qos::DataReaderQos,
            status::{
                LivelinessChangedStatus, RequestedDeadlineMissedStatus,
                RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
                SubscriptionMatchedStatus,
            },
        },
    },
};

use std::marker::PhantomData;

use super::{
    sample_info::{
        InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE,
        ANY_VIEW_STATE,
    },
    subscriber::Subscriber,
};

/// A [`Sample`] contains the data and [`SampleInfo`] read by the [`DataReader`].
#[derive(Debug, PartialEq, Eq)]
pub struct Sample<Foo> {
    /// Data received by the [`DataReader`]. A sample might contain no valid data in which case this field is [`None`].
    data: Option<Data>,
    /// Information of the sample received by the [`DataReader`].
    sample_info: SampleInfo,
    phantom: PhantomData<Foo>,
}

impl<Foo> Sample<Foo> {
    pub(crate) fn new(data: Option<Data>, sample_info: SampleInfo) -> Self {
        Self {
            data,
            sample_info,
            phantom: PhantomData,
        }
    }
}

impl<'de, Foo> Sample<Foo>
where
    Foo: DdsDeserialize<'de>,
{
    pub fn data(&'de self) -> DdsResult<Foo> {
        match self.data.as_ref() {
            Some(data) => Ok(Foo::deserialize_data(data.as_ref())?),
            None => Err(DdsError::NoData),
        }
    }
}

impl<Foo> Sample<Foo> {
    pub fn sample_info(&self) -> SampleInfo {
        self.sample_info.clone()
    }
}

/// A [`DataReader`] allows the application (1) to declare the data it wishes to receive (i.e., make a subscription) and (2) to access the
/// data received by the attached [`Subscriber`].
///
/// A DataReader refers to exactly one [`Topic`] that identifies the data to be read. The subscription has a unique resulting type.
/// The data-reader may give access to several instances of the resulting type, which can be distinguished from each other by their key.
pub struct DataReader<Foo> {
    reader_address: ActorAddress<DataReaderActor>,
    subscriber_address: ActorAddress<SubscriberActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
    phantom: PhantomData<Foo>,
}

impl<Foo> DataReader<Foo> {
    pub(crate) fn new(
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            reader_address,
            subscriber_address,
            participant_address,
            runtime_handle,
            phantom: PhantomData,
        }
    }

    fn topic_address(&self) -> ActorAddress<TopicActor> {
        let user_defined_topic_list = self
            .participant_address
            .send_mail_and_await_reply_blocking(
                domain_participant_actor::get_user_defined_topic_list::new(),
            )
            .expect("should never fail");
        for topic in user_defined_topic_list {
            if topic.send_mail_and_await_reply_blocking(topic_actor::get_type_name::new())
                == self
                    .reader_address
                    .send_mail_and_await_reply_blocking(data_reader_actor::get_type_name::new())
                && topic.send_mail_and_await_reply_blocking(topic_actor::get_name::new())
                    == self
                        .reader_address
                        .send_mail_and_await_reply_blocking(data_reader_actor::get_topic_name::new())
            {
                return topic;
            }
        }
        panic!("Should always exist");
    }
}

impl<Foo> Clone for DataReader<Foo> {
    fn clone(&self) -> Self {
        Self {
            reader_address: self.reader_address.clone(),
            subscriber_address: self.subscriber_address.clone(),
            participant_address: self.participant_address.clone(),
            runtime_handle: self.runtime_handle.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataReader<Foo> {
    /// This operation accesses a collection of [`Sample`] from the [`DataReader`]. The size of the returned collection will
    /// be limited to the specified `max_samples`. The properties of the data values collection and the setting of the
    /// [`PresentationQosPolicy`](crate::infrastructure::qos_policy::PresentationQosPolicy) may impose further limits
    /// on the size of the returned list:
    /// 1. If [`PresentationQosPolicy::access_scope`](crate::infrastructure::qos_policy::PresentationQosPolicy) is
    /// [`PresentationQosPolicyAccessScopeKind::Instance`](crate::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind),
    /// then the returned collection is a list where samples belonging to the same data-instance are consecutive.
    /// 2. If [`PresentationQosPolicy::access_scope`](crate::infrastructure::qos_policy::PresentationQosPolicy) is
    /// [`PresentationQosPolicyAccessScopeKind::Topic`](crate::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind) and
    /// [`PresentationQosPolicy::ordered_access`](crate::infrastructure::qos_policy::PresentationQosPolicy) is set to [`false`],
    /// then the returned collection is a list where samples belonging to the same data-instance are consecutive.
    /// 3. If [`PresentationQosPolicy::access_scope`](crate::infrastructure::qos_policy::PresentationQosPolicy) is
    /// [`PresentationQosPolicyAccessScopeKind::Topic`](crate::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind) and
    /// [`PresentationQosPolicy::ordered_access`](crate::infrastructure::qos_policy::PresentationQosPolicy) is set to [`true`],
    /// then the returned collection is a list where samples belonging to the same instance may or may not be consecutive.
    /// This is because to preserve order it may be necessary to mix samples from different instances.
    ///
    /// In any case, the relative order between the samples of one instance is consistent with the
    /// [`DestinationOrderQosPolicy`](crate::infrastructure::qos_policy::DestinationOrderQosPolicy):
    /// - If [`DestinationOrderQosPolicyKind::ByReceptionTimestamp`](crate::infrastructure::qos_policy::DestinationOrderQosPolicyKind),
    /// samples belonging to the same instances will appear in the relative order in which there were received
    /// (FIFO, earlier samples ahead of the later samples).
    /// - If  [`DestinationOrderQosPolicyKind::BySourceTimestamp`](crate::infrastructure::qos_policy::DestinationOrderQosPolicyKind),
    /// samples belonging to the same instances will appear in the relative order implied by the `source_timestamp`
    /// (FIFO, smaller values of `source_timestamp` ahead of the larger values).
    ///
    /// Each [`Sample`] contains the data and a [`SampleInfo`] which provides information, such as the
    /// [`SampleInfo::source_timestamp`], the [`SampleInfo::sample_state`], [`SampleInfo::view_state`], and
    /// [`SampleInfo::instance_state`], etc., about the corresponding sample.
    /// Some elements in the returned collection may not have valid data. If the [`SampleInfo::instance_state`] is
    /// [`InstanceStateKind::NotAliveDisposed`] or [`InstanceStateKind::NotAliveNoWriters`], then the last sample
    /// for that instance in the collection, that is, the one whose `[`SampleInfo::sample_rank`]==0` does not contain
    /// valid data. Samples that contain no data do not count towards the limits imposed by the
    /// [`ResourceLimitsQosPolicy`](crate::infrastructure::qos_policy::ResourceLimitsQosPolicy).
    /// The act of reading a sample sets its [`SampleInfo::sample_state`] to [`SampleStateKind::Read`]. If the sample
    /// belongs to the most recent generation of the instance, it will also set the [`SampleInfo::view_state`]
    /// of the instance to [`ViewStateKind::NotNew`]. It will not affect the  [`SampleInfo::instance_state`] of the instance.
    ///
    /// If the DataReader has no samples that meet the constraints, the return value will be
    /// [`DdsError::NoData`](crate::infrastructure::error::DdsError).
    #[tracing::instrument(skip(self))]
    pub fn read(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::read::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                None,
            ),
        )??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// This operation accesses a collection of [`Sample`] from the [`DataReader`]. This operation uses the same
    /// logic as the [`DataReader::read`]. The only difference with read is that the
    /// sampled returned by [`DataReader::take`] will no longer be accessible to successive calls to read or take.
    #[tracing::instrument(skip(self))]
    pub fn take(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::take::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                None,
            ),
        )??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// This operation reads the next, non-previously accessed [`Sample`] value from the [`DataReader`].
    /// The implied order among the samples stored in the [`DataReader`] is the same as for the [`DataReader::read`]
    /// operation. This operation is semantically equivalent to the read operation where the input Data sequence has
    /// `max_samples=1`, the `sample_states = &[SampleStateKind::NotRead]`, `view_states=ANY_VIEW_STATE`, and
    /// `instance_states=ANY_INSTANCE_STATE`.
    /// This operation provides a simplified API to ‘read’ samples avoiding the need for the application to manage
    /// sequences and specify states.
    #[tracing::instrument(skip(self))]
    pub fn read_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let mut samples = {
            self.reader_address.send_mail_and_await_reply_blocking(
                data_reader_actor::read::new(
                    1,
                    vec![SampleStateKind::NotRead],
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ),
            )??
        };
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(Sample::new(data, sample_info))
    }

    /// This operation takes the next, non-previously accessed [`Sample`] value from the [`DataReader`].
    /// The implied order among the samples stored in the [`DataReader`] is the same as for the [`DataReader::read`]
    /// operation. This operation is semantically equivalent to the read operation where the input Data sequence has
    /// `max_samples=1`, the `sample_states = &[SampleStateKind::NotRead]`, `view_states=ANY_VIEW_STATE`, and
    /// `instance_states=ANY_INSTANCE_STATE`.
    /// This operation provides a simplified API to ‘take’ samples avoiding the need for the application to manage
    /// sequences and specify states.
    #[tracing::instrument(skip(self))]
    pub fn take_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let mut samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::take::new(
                1,
                vec![SampleStateKind::NotRead],
                ANY_VIEW_STATE.to_vec(),
                ANY_INSTANCE_STATE.to_vec(),
                None,
            ),
        )??;
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(Sample::new(data, sample_info))
    }

    /// This operation accesses a collection of [`Sample`] from the [`DataReader`]. The
    /// behavior is identical to [`DataReader::read`] except that all samples returned
    /// belong to the single specified instance whose handle is `a_handle`.
    /// Upon successful return, the collection will contain samples all belonging to the
    /// same instance. The corresponding [`SampleInfo`] verifies [`SampleInfo::instance_handle`] == a_handle.
    /// This operation return [`DdsError::BadParameter`](crate::infrastructure::error::DdsError)
    /// if the [`InstanceHandle`] `a_handle` does not correspond to an existing
    /// data object known to the [`DataReader`].
    #[tracing::instrument(skip(self))]
    pub fn read_instance(
        &self,
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::read::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                Some(a_handle),
            ),
        )??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// This operation accesses a collection of [`Sample`] from the [`DataReader`]. The
    /// behavior is identical to [`DataReader::take`] except that all samples returned
    /// belong to the single specified instance whose handle is `a_handle`.
    /// Upon successful return, the collection will contain samples all belonging to the
    /// same instance. The corresponding [`SampleInfo`] verifies [`SampleInfo::instance_handle`] == a_handle.
    /// This operation return [`DdsError::BadParameter`](crate::infrastructure::error::DdsError)
    /// if the [`InstanceHandle`] `a_handle` does not correspond to an existing
    /// data object known to the [`DataReader`].
    #[tracing::instrument(skip(self))]
    pub fn take_instance(
        &self,
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::take::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                Some(a_handle),
            ),
        )??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// This operation accesses a collection of [`Sample`] from the [`DataReader`] where all the samples belong to a single instance.
    /// The behavior is similar to [`DataReader::read_instance`] except that the actual instance is not directly specified.
    /// Rather the samples will all belong to the ‘next’ instance with instance_handle ‘greater’ than the specified
    /// `previous_handle` that has available samples.
    /// This operation implies the existence of a total order *greater-than* relationship between the instance handles.
    /// The specifics of this relationship are not all important and are implementation specific. The important thing is that,
    /// according to the middleware, all instances are ordered relative to each other. This ordering is between the instance handles
    /// and it does not depend on the state of the instance (e.g., whether it has data or not) and must be defined even for
    /// instance handles that do not correspond to instances currently managed by the [`DataReader`].
    /// The behavior of this operation is as if [`DataReader::read_instance`] was invoked passing the smallest `instance_handle`
    /// among all the ones that (a) are greater than `previous_handle` and (b) have available samples (i.e., samples that meet the
    /// constraints imposed by the specified states). If [`None`] is used as the `previous_handle` argument the operation will
    /// return the samples for the instance which has the smallest instance_handle among allthe instances that contain available samples.
    /// The operation [`DataReader::read_next_instance`] is intended to be used in an application-driven iteration where the application starts by
    /// passing `previous_handle==None`, examines the samples returned, and then uses the [`SampleInfo::instance_handle`] returned in
    /// as the value of the `previous_handle` argument to the next call to [`DataReader::read_next_instance`]. The iteration continues
    /// until the operation returns the value [`DdsError::NoData`](crate::infrastructure::error::DdsError).
    /// Note that it is possible to call this operation with a `previous_handle` that does not correspond to an
    /// instance currently managed by the [`DataReader`]. One practical situation where this may occur is when an application is iterating
    /// though all the instances, takes all the samples of a [`InstanceStateKind::NotAliveNoWriters`] instance (at which point the
    /// instance information may be removed, and thus the handle becomes invalid) and tries to read the next instance.
    /// The behavior of this operation generally follows the same rules as the [`DataReader::read`] operation regarding the pre-conditions
    /// and post-conditions and returned values.
    #[tracing::instrument(skip(self))]
    pub fn read_next_instance(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::read_next_instance::new(
                max_samples,
                previous_handle,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
            ),
        )??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// This operation accesses a collection of [`Sample`] values from the [`DataReader`] and removes them from the [`DataReader`].
    /// This operation has the same behavior as [`DataReader::read_next_instance`] except that the samples are ‘taken’ from the [`DataReader`] such
    /// that they are no longer accessible via subsequent ‘read’ or ‘take’ operations.
    #[tracing::instrument(skip(self))]
    pub fn take_next_instance(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let samples = self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::take_next_instance::new(
                max_samples,
                previous_handle,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
            ),
        )??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// This operation can be used to retrieve the instance key that corresponds to an `handle`.
    /// The operation will only fill the fields that form the key inside the `key_holder` instance.
    /// This operation may return [`DdsError::BadParameter`](crate::infrastructure::error::DdsError)
    /// if the [`InstanceHandle`] `handle` does not correspond to an existing data object known to the [`DataReader`].
    #[tracing::instrument(skip(self, _key_holder))]
    pub fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    /// This operation takes as a parameter an instance and returns an [`InstanceHandle`] handle
    /// that can be used in subsequent operations that accept an instance handle as an argument.
    /// The instance parameter is only used for the purpose of examining the fields that define the
    /// key. This operation does not register the instance in question. If the instance has not
    /// been previously registered, or if for any other reason the Service is unable to provide
    /// an instance handle, the operation will succeed and return [`None`].
    #[tracing::instrument(skip(self, _instance))]
    pub fn lookup_instance(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }
}

impl<Foo> DataReader<Foo> {
    /// This operation allows access to the [`LivelinessChangedStatus`].
    #[tracing::instrument(skip(self))]
    pub fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        todo!()
    }

    /// This operation allows access to the [`RequestedDeadlineMissedStatus`].
    #[tracing::instrument(skip(self))]
    pub fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        todo!()
    }

    /// This operation allows access to the [`RequestedIncompatibleQosStatus`].
    #[tracing::instrument(skip(self))]
    pub fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        todo!()
    }

    /// This operation allows access to the [`SampleLostStatus`].
    #[tracing::instrument(skip(self))]
    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    /// This operation allows access to the [`SampleRejectedStatus`].
    #[tracing::instrument(skip(self))]
    pub fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        todo!()
    }

    /// This operation allows access to the [`SubscriptionMatchedStatus`].
    #[tracing::instrument(skip(self))]
    pub fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::get_subscription_matched_status::new(),
        )
    }

    /// This operation returns the [`Topic`] associated with the [`DataReader`]. This is the same [`Topic`]
    /// that was used to create the [`DataReader`].
    #[tracing::instrument(skip(self))]
    pub fn get_topicdescription(&self) -> DdsResult<Topic> {
        Ok(Topic::new(
            self.topic_address(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    /// This operation returns the [`Subscriber`] to which the [`DataReader`] belongs.
    #[tracing::instrument(skip(self))]
    pub fn get_subscriber(&self) -> DdsResult<Subscriber> {
        Ok(Subscriber::new(
            self.subscriber_address.clone(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    /// This operation blocks the calling thread until either all “historical” data is received, or else the
    /// duration specified by the `max_wait` parameter elapses, whichever happens first.
    /// A return value of [`Ok`] indicates that all the “historical” data was received;
    /// a return value of [`DdsError`](crate::infrastructure::error::DdsError) indicates that `max_wait`
    /// elapsed before all the data was received.
    /// This operation is intended only for [`DataReader`] entities that have a non-VOLATILE
    /// [`DurabilityQosPolicy`](crate::infrastructure::qos_policy::DurabilityQosPolicy).
    /// As soon as an application enables a non-VOLATILE [`DataReader`] it will start receiving both
    /// “historical” data, i.e., the data that was written prior to the time the [`DataReader`] joined the
    /// domain, as well as any new data written by the [`DataWriter`](crate::publication::data_writer::DataWriter) entities.
    /// There are situations where the application logic may require the application to wait until all “historical”
    /// data is received.
    #[tracing::instrument(skip(self))]
    pub fn wait_for_historical_data(&self, max_wait: Duration) -> DdsResult<()> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < std::time::Duration::from(max_wait) {
            if self.reader_address.send_mail_and_await_reply_blocking(
                data_reader_actor::is_historical_data_received::new(),
            )?? {
                return Ok(());
            }
        }

        Err(DdsError::Timeout)
    }

    /// This operation retrieves information on a publication that is currently “associated” with the [`DataReader`];
    /// that is, a publication with a matching [`Topic`] and compatible qos that the application  has not indicated should be ignored by means of the
    /// [`DomainParticipant::ignore_publication`](crate::domain::domain_participant::DomainParticipant) operation.
    /// The `publication_handle` must correspond to a publication currently associated with the [`DataReader`] otherwise the operation
    /// will fail and return [`DdsError::BadParameter`](crate::infrastructure::error::DdsError).
    /// The operation [`DataReader::get_matched_publications`] can be used to find the publications that are
    /// currently matched with the [`DataReader`].
    #[tracing::instrument(skip(self))]
    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::get_matched_publication_data::new(publication_handle),
        )?
    }

    /// This operation retrieves the list of publications currently “associated” with the [`DataReader`]; that is, publications that have a
    /// matching [`Topic`] and compatible qos that the application has not indicated should be ignored by means of the
    /// [`DomainParticipant::ignore_publication`](crate::domain::domain_participant::DomainParticipant) operation.
    /// The handles returned are the ones that are used by the DDS implementation to locally identify
    /// the corresponding matched [`DataWriter`](crate::publication::data_writer::DataWriter) entities. These handles match the ones that appear in the
    /// [`SampleInfo::instance_handle`](crate::subscription::sample_info::SampleInfo) when reading the “DCPSPublications” builtin topic.
    #[tracing::instrument(skip(self))]
    pub fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::get_matched_publications::new())
    }
}

impl<Foo> DataReader<Foo> {
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
    pub fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => self.subscriber_address.send_mail_and_await_reply_blocking(
                subscriber_actor::get_default_datareader_qos::new(),
            )?,
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::set_qos::new(q))??;

        if self
            .reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::is_enabled::new())?
        {
            let type_name = self
                .reader_address
                .send_mail_and_await_reply_blocking(data_reader_actor::get_type_name::new())?;
            let type_support = self
                .participant_address
                .send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_type_support::new(type_name.clone()),
                )?
                .ok_or_else(|| {
                    DdsError::PreconditionNotMet(format!(
                        "Type with name {} not registered with parent domain participant",
                        type_name
                    ))
                })?;
            announce_data_reader(
                &self.participant_address,
                self.reader_address.send_mail_and_await_reply_blocking(
                    data_reader_actor::as_discovered_reader_data::new(
                        TopicQos::default(),
                        self.subscriber_address
                            .send_mail_and_await_reply_blocking(subscriber_actor::get_qos::new())?,
                        self.participant_address
                            .send_mail_and_await_reply_blocking(
                                domain_participant_actor::get_default_unicast_locator_list::new(),
                            )?,
                        self.participant_address
                            .send_mail_and_await_reply_blocking(
                                domain_participant_actor::get_default_multicast_locator_list::new(),
                            )?,
                        type_support.xml_type(),
                    ),
                )?,
            )?;
        }

        Ok(())
    }

    /// This operation allows access to the existing set of [`DataReaderQos`] policies.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        self.reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::get_qos::new())
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::get_statuscondition::new())
            .map(StatusCondition::new)
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
    /// Entities created from a factory that is disabled, are created disabled regardless of the setting of the ENTITY_FACTORY Qos
    /// policy.
    /// Calling enable on an Entity whose factory is not enabled will fail and return PRECONDITION_NOT_MET.
    /// If the `autoenable_created_entities` field of [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) is set to [`true`], the [`Self::enable()`] operation on the factory will
    /// automatically enable all entities created from the factory.
    /// The Listeners associated with an entity are not called until the entity is enabled. Conditions associated with an entity that is not
    /// enabled are “inactive,” that is, the operation [`StatusCondition::get_trigger_value()`] will always return `false`.
    #[tracing::instrument(skip(self))]
    pub fn enable(&self) -> DdsResult<()> {
        if !self
            .reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::is_enabled::new())?
        {
            self.reader_address
                .send_mail_and_await_reply_blocking(data_reader_actor::enable::new())?;
        }

        let type_name = self
            .reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::get_type_name::new())?;
        let type_support = self
            .participant_address
            .send_mail_and_await_reply_blocking(domain_participant_actor::get_type_support::new(
                type_name.clone(),
            ))?
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;

        announce_data_reader(
            &self.participant_address,
            self.reader_address.send_mail_and_await_reply_blocking(
                data_reader_actor::as_discovered_reader_data::new(
                    TopicQos::default(),
                    self.subscriber_address
                        .send_mail_and_await_reply_blocking(subscriber_actor::get_qos::new())?,
                    self.participant_address
                        .send_mail_and_await_reply_blocking(
                            domain_participant_actor::get_default_unicast_locator_list::new(),
                        )?,
                    self.participant_address
                        .send_mail_and_await_reply_blocking(
                            domain_participant_actor::get_default_multicast_locator_list::new(),
                        )?,
                    type_support.xml_type(),
                ),
            )?,
        )?;

        Ok(())
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    #[tracing::instrument(skip(self))]
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.reader_address
            .send_mail_and_await_reply_blocking(data_reader_actor::get_instance_handle::new())
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
        a_listener: impl DataReaderListener<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.reader_address.send_mail_and_await_reply_blocking(
            data_reader_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ),
        )
    }
}
pub trait AnyDataReader {}

fn announce_data_reader(
    domain_participant: &ActorAddress<DomainParticipantActor>,
    discovered_reader_data: DiscoveredReaderData,
) -> DdsResult<()> {
    let mut serialized_data = Vec::new();
    discovered_reader_data.serialize_data(&mut serialized_data)?;
    let timestamp = domain_participant
        .send_mail_and_await_reply_blocking(domain_participant_actor::get_current_time::new())?;

    let builtin_publisher = domain_participant.send_mail_and_await_reply_blocking(
        domain_participant_actor::get_builtin_publisher::new(),
    )?;
    let data_writer_list = builtin_publisher
        .send_mail_and_await_reply_blocking(publisher_actor::data_writer_list::new())?;
    for dw in data_writer_list {
        if dw.send_mail_and_await_reply_blocking(data_writer_actor::get_type_name::new())
            == Ok("DiscoveredReaderData".to_string())
        {
            let instance_handle = get_instance_handle_from_key(&discovered_reader_data.get_key()?)?;
            dw.send_mail_and_await_reply_blocking(data_writer_actor::write_w_timestamp::new(
                serialized_data,
                instance_handle,
                None,
                timestamp,
            ))??;

            domain_participant.send_mail_blocking(domain_participant_actor::send_message::new())?;
            break;
        }
    }

    Ok(())
}

impl<Foo> AnyDataReader for DataReader<Foo> {}
