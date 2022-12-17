use super::{instance::InstanceHandle, qos_policy::QosPolicyId};

/// Enumeration of the different types of communication status
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatusKind {
    /// Another topic exists with the same name but different characteristics.
    InconsistentTopic,
    /// The deadline that the [`DataWriter`](crate::publication::data_writer::DataWriter) has committed through its
    ///  [`DeadlineQosPolicy`](crate::infrastructure::qos_policy::DeadlineQosPolicy) was not respected for a specific instance.
    OfferedDeadlineMissed,
    /// The deadline that the [`DataReader`](crate::subscription::data_reader::DataReader) was expecting through its
    /// [`DeadlineQosPolicy`](crate::infrastructure::qos_policy::DeadlineQosPolicy) was not respected for a specific instance.
    RequestedDeadlineMissed,
    /// A *QosPolicy* value was incompatible with what was requested.
    OfferedIncompatibleQos,
    /// A *QosPolicy* value was incompatible with what is offered.
    RequestedIncompatibleQos,
    /// A sample has been lost (never received).
    SampleLost,
    /// A (received) sample has been rejected.
    SampleRejected,
    /// New information is available.
    DataOnReaders,
    /// New information is available.
    DataAvailable,
    /// The liveliness that the [`DataWriter`](crate::publication::data_writer::DataWriter) has committed through its
    /// [`LivelinessQosPolicy`](crate::infrastructure::qos_policy::LivelinessQosPolicy) was not respected;
    /// thus [`DataReader`](crate::subscription::data_reader::DataReader) entities will consider the data writer as no longer *active*.
    LivelinessLost,
    /// The liveliness of one or more [`DataWriter`](crate::publication::data_writer::DataWriter) that were writing instances read
    /// through the [`DataReader`](crate::subscription::data_reader::DataReader) has changed.
    /// Some data writers have become *active* or *inactive*.
    LivelinessChanged,
    /// The  [`DataWriter`](crate::publication::data_writer::DataWriter) has found [`DataReader`](crate::subscription::data_reader::DataReader) that
    /// matches the [`Topic`](crate::topic_definition::topic::Topic)  and has compatible Qos, or has ceased to be matched with a
    /// [`DataReader`](crate::subscription::data_reader::DataReader) that was previously considered to be matched.
    PublicationMatched,
    /// The [`DataReader`](crate::subscription::data_reader::DataReader) has found a [`DataWriter`](crate::publication::data_writer::DataWriter)
    /// that matches the [`Topic`](crate::topic_definition::topic::Topic) and has compatible Qos, or has ceased to be matched with a
    /// [`DataWriter`](crate::publication::data_writer::DataWriter) that was previously considered to be matched.
    SubscriptionMatched,
}

/// Special constant representing an empty list of communication statuses
pub const NO_STATUS: &[StatusKind] = &[];

/// Structure holding the values related to the Inconsistent Topic communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InconsistentTopicStatus {
    /// Total cumulative count of the Topics discovered whose name matches
    /// the Topic to which this status is attached and whose type is inconsistent with the Topic.
    pub total_count: i32,
    /// The incremental number of inconsistent topics discovered since the
    /// last time the listener was called or the status was read.
    pub total_count_change: i32,
}

/// Structure holding the values related to the Sample Lost communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SampleLostStatus {
    /// Total cumulative count of all samples lost across of instances of data published under the Topic.
    pub total_count: i32,
    /// The incremental number of samples lost since the last time the listener was called or the status was read.
    pub total_count_change: i32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SampleRejectedStatusKind {
    NotRejected,
    RejectedByInstancesLimit,
    RejectedBySamplesLimit,
    RejectedBySamplesPerInstanceLimit,
}

impl Default for SampleRejectedStatusKind {
    fn default() -> Self {
        Self::NotRejected
    }
}

/// Structure holding the values related to the Sample Rejected communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SampleRejectedStatus {
    /// Total cumulative count of samples rejected by the DataReader.
    pub total_count: i32,
    /// The incremental number of samples rejected since the last time the listener was called or the status was read.
    pub total_count_change: i32,
    /// Reason for rejecting the last sample rejected. If no samples have been rejected, the reason is the special value NOT_REJECTED.
    pub last_reason: SampleRejectedStatusKind,
    /// Handle to the instance being updated by the last sample that was rejected.
    pub last_instance_handle: InstanceHandle,
}

/// Structure holding the values related to the Liveliness Lost communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct LivelinessLostStatus {
    /// Total cumulative number of times that a previously-alive DataWriter
    /// became not alive due to a failure to actively signal its liveliness within
    /// its offered liveliness period. This count does not change when an
    /// already not alive DataWriter simply remains not alive for another
    /// liveliness period.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
}

/// Structure holding the values related to the Liveliness Changed communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct LivelinessChangedStatus {
    /// The total number of currently active DataWriters that write the Topic
    /// read by the DataReader. This count increases when a newly matched
    /// DataWriter asserts its liveliness for the first time or when a DataWriter
    /// previously considered to be not alive reasserts its liveliness. The count
    /// decreases when a DataWriter considered alive fails to assert its
    /// liveliness and becomes not alive, whether because it was deleted
    /// normally or for some other reason.
    pub alive_count: i32,
    /// The total count of currently DataWriters that write the Topic read by
    /// the DataReader that are no longer asserting their liveliness. This count
    /// increases when a DataWriter considered alive fails to assert its
    /// liveliness and becomes not alive for some reason other than the normal
    /// deletion of that DataWriter. It decreases when a previously not alive
    /// DataWriter either reasserts its liveliness or is deleted normally.
    pub not_alive_count: i32,
    /// The change in the alive_count since the last time the listener was
    /// called or the status was read.
    pub alive_count_change: i32,
    /// The change in the not_alive_count since the last time the listener was
    /// called or the status was read.
    pub not_alive_count_change: i32,
    /// Handle to the last DataWriter whose change in liveliness caused this
    /// status to change.
    pub last_publication_handle: InstanceHandle,
}

/// Structure holding the values related to the Offered Deadline Missed communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct OfferedDeadlineMissedStatus {
    /// Total cumulative number of offered deadline periods elapsed during
    /// which a DataWriter failed to provide data. Missed deadlines
    /// accumulate; that is, each deadline period the total_count will be
    /// incremented by one.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
    /// Handle to the last instance in the DataWriter for which an offered
    /// deadline was missed.
    pub last_instance_handle: InstanceHandle,
}

/// Structure holding the values related to the Requested Deadline Missed communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RequestedDeadlineMissedStatus {
    /// Total cumulative number of missed deadlines detected for any instance
    /// read by the DataReader. Missed deadlines accumulate; that is, each
    /// deadline period the total_count will be incremented by one for each
    /// instance for which data was not received.
    pub total_count: i32,
    /// The incremental number of deadlines detected since the last time the
    /// listener was called or the status was read.
    pub total_count_change: i32,
    /// Handle to the last instance in the DataReader for which a deadline was detected
    pub last_instance_handle: InstanceHandle,
}

/// Structure associating the QosPolicyId and the number of time it appeared in the related communication status.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QosPolicyCount {
    pub policy_id: QosPolicyId,
    pub count: i32,
}

/// Structure holding the values related to the Offered Incompatible Qos communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct OfferedIncompatibleQosStatus {
    /// Total cumulative number of times the concerned DataWriter
    /// discovered a DataReader for the same Topic with a requested QoS that
    /// is incompatible with that offered by the DataWriter.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
    /// The PolicyId_t of one of the policies that was found to be
    /// incompatible the last time an incompatibility was detected.
    pub last_policy_id: QosPolicyId,
    /// A list containing for each policy the total number of times that the
    /// concerned DataWriter discovered a DataReader for the same Topic
    /// with a requested QoS that is incompatible with that offered by the
    /// DataWriter.
    pub policies: Vec<QosPolicyCount>,
}

/// Structure holding the values related to the Requested Incompatible Qos communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RequestedIncompatibleQosStatus {
    /// Total cumulative number of times the concerned DataReader
    /// discovered a DataWriter for the same Topic with an offered QoS that
    /// was incompatible with that requested by the DataReader.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
    /// The QosPolicyId of one of the policies that was found to be
    /// incompatible the last time an incompatibility was detected.
    pub last_policy_id: QosPolicyId,
    /// A list containing for each policy the total number of times that the
    /// concerned DataReader discovered a DataWriter for the same Topic
    /// with an offered QoS that is incompatible with that requested by the
    /// DataReader.
    pub policies: Vec<QosPolicyCount>,
}

/// Structure holding the values related to the Publication Matched communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PublicationMatchedStatus {
    /// Total cumulative count the concerned DataWriter discovered a
    /// “match” with a DataReader. That is, it found a DataReader for the
    /// same Topic with a requested QoS that is compatible with that offered
    /// by the DataWriter.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
    /// Handle to the last DataReader that matched the DataWriter causing the
    /// status to change.
    pub last_subscription_handle: InstanceHandle,
    /// The number of DataReaders currently matched to the concerned
    /// DataWriter.
    pub current_count: i32,
    // The change in current_count since the last time the listener was called
    // or the status was read.
    pub current_count_change: i32,
}

/// Structure holding the values related to the Subscription Matched communication status.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SubscriptionMatchedStatus {
    /// Total cumulative count the concerned DataReader discovered a
    /// “match” with a DataWriter. That is, it found a DataWriter for the same
    /// Topic with a requested QoS that is compatible with that offered by the
    /// DataReader.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
    /// Handle to the last DataWriter that matched the DataReader causing the
    /// status to change.
    pub last_publication_handle: InstanceHandle,
    /// The number of DataWriters currently matched to the concerned
    /// DataReader.
    pub current_count: i32,
    /// The change in current_count since the last time the listener was called
    /// or the status was read.
    pub current_count_change: i32,
}
