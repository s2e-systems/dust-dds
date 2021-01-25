use rust_dds_types::InstanceHandle;

use super::qos_policy::QosPolicyId;

pub type StatusMask = u32;

pub const INCONSISTENT_TOPIC_STATUS: StatusMask = 0x0001 << 0;
pub const OFFERED_DEADLINE_MISSED_STATUS: StatusMask = 0x0001 << 1;
pub const REQUESTED_DEADLINE_MISSED_STATUS: StatusMask = 0x0001 << 2;
pub const OFFERED_INCOMPATIBLE_QOS_STATUS: StatusMask = 0x0001 << 5;
pub const REQUESTED_INCOMPATIBLE_QOS_STATUS: StatusMask = 0x0001 << 6;
pub const SAMPLE_LOST_STATUS: StatusMask = 0x0001 << 7;
pub const SAMPLE_REJECTED_STATUS: StatusMask = 0x0001 << 8;
pub const DATA_ON_READERS_STATUS: StatusMask = 0x0001 << 9;
pub const DATA_AVAILABLE_STATUS: StatusMask = 0x0001 << 10;
pub const LIVELINESS_LOST_STATUS: StatusMask = 0x0001 << 11;
pub const LIVELINESS_CHANGED_STATUS: StatusMask = 0x0001 << 12;
pub const PUBLICATION_MATCHED_STATUS: StatusMask = 0x0001 << 13;
pub const SUBSCRIPTION_MATCHED_STATUS: StatusMask = 0x0001 << 14;

pub type SampleStateKind = u32;
pub type ViewStateKind = u32;
pub type InstanceStateKind = u32;

pub enum SampleRejectedStatusKind {
    NotRejected,
    RejectedByInstancesLimit,
    RejectedBySamplesLimit,
    RejectedBySamplesPerInstanceLimit,
}

pub struct QosPolicyCount {
    pub policy_id: QosPolicyId,
    pub count: i32,
}

pub struct SampleLostStatus {
    /// Total cumulative count of all samples lost across of instances of data published under the Topic.
    pub total_count: i32,
    /// The incremental number of samples lost since the last time the listener was called or the status was read.
    pub total_count_change: i32,
}

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
pub struct InconsistentTopicStatus {
    /// Total cumulative count of the Topics discovered whose name matches
    /// the Topic to which this status is attached and whose type is inconsistent with the Topic.
    pub total_count: i32,
    /// The incremental number of inconsistent topics discovered since the
    /// last time the listener was called or the status was read.
    pub total_count_change: i32,
}

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

pub struct RequestedIncompatibleQosStatus {
    /// Total cumulative number of times the concerned DataReader
    /// discovered a DataWriter for the same Topic with an offered QoS that
    /// was incompatible with that requested by the DataReader.
    pub total_count: i32,
    /// The change in total_count since the last time the listener was called or
    /// the status was read.
    pub total_count_change: i32,
    /// The QosPolicyId_t of one of the policies that was found to be
    /// incompatible the last time an incompatibility was detected.
    pub last_policy_id: QosPolicyId,
    /// A list containing for each policy the total number of times that the
    /// concerned DataReader discovered a DataWriter for the same Topic
    /// with an offered QoS that is incompatible with that requested by the
    /// DataReader.
    pub policies: Vec<QosPolicyCount>,
}

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
