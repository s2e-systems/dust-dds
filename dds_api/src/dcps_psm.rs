pub type DomainIdTypeNative = i32;
pub type HandleTypeNative = i32;
pub const HANDLE_NIL_NATIVE: HandleTypeNative = 0;
pub type BuiltInTopicKeyTypeNative = i32;

pub type DomainId = DomainIdTypeNative;
pub type InstanceHandle = HandleTypeNative;

#[derive(Debug, PartialEq)]
pub struct BuiltInTopicKey {
    pub value: [BuiltInTopicKeyTypeNative; 3],
}

pub type InstanceHandleSeq<'a> = &'a [InstanceHandle];
pub type ReturnCode = i32;
pub type QosPolicyId = i32;
pub type StringSeq<'a> = &'a [&'a str];

#[derive(PartialOrd, PartialEq, Debug, Clone)]
pub struct Duration {
    sec: i32,
    nanosec: u32,
}

impl Duration {
    pub fn new(sec: i32, nanosec: u32) -> Self {
        Self { sec, nanosec }
    }

    /// Get a reference to the duration's sec.
    pub fn sec(&self) -> &i32 {
        &self.sec
    }

    /// Get a reference to the duration's nanosec.
    pub fn nanosec(&self) -> &u32 {
        &self.nanosec
    }
}

pub struct Time {
    pub sec: i32,
    pub nanosec: u32,
}

// ----------------------------------------------------------------------
// Pre-defined values
// ----------------------------------------------------------------------
pub const HANDLE_NIL: InstanceHandle = HANDLE_NIL_NATIVE;
pub const LENGTH_UNLIMITED: i32 = -1;

pub const DURATION_INFINITE: Duration = Duration {
    sec: 0x7fffffff,
    nanosec: 0x7fffffff,
};
pub const DURATION_ZERO: Duration = Duration { sec: 0, nanosec: 0 };
pub const TIME_INVALID: Time = Time {
    sec: -1,
    nanosec: 0xffffffff,
};

// ----------------------------------------------------------------------
// Return codes
// ----------------------------------------------------------------------
pub const RETCODE_OK: ReturnCode = 0;
pub const RETCODE_ERROR: ReturnCode = 1;
pub const RETCODE_UNSUPPORTED: ReturnCode = 2;
pub const RETCODE_BAD_PARAMETER: ReturnCode = 3;
pub const RETCODE_PRECONDITION_NOT_MET: ReturnCode = 4;
pub const RETCODE_OUT_OF_RESOURCES: ReturnCode = 5;
pub const RETCODE_NOT_ENABLED: ReturnCode = 6;
pub const RETCODE_IMMUTABLE_POLICY: ReturnCode = 7;
pub const RETCODE_INCONSISTENT_POLICY: ReturnCode = 8;
pub const RETCODE_ALREADY_DELETED: ReturnCode = 9;
pub const RETCODE_TIMEOUT: ReturnCode = 10;
pub const RETCODE_NO_DATA: ReturnCode = 11;
pub const RETCODE_ILLEGAL_OPERATION: ReturnCode = 12;

// ----------------------------------------------------------------------
// Status to support listeners and conditions
// ----------------------------------------------------------------------
pub type StatusKind = u32;
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

pub struct InconsistentTopicStatus {
    /// Total cumulative count of the Topics discovered whose name matches
    /// the Topic to which this status is attached and whose type is inconsistent with the Topic.
    pub total_count: i32,
    /// The incremental number of inconsistent topics discovered since the
    /// last time the listener was called or the status was read.
    pub total_count_change: i32,
}

pub struct SampleLostStatus {
    /// Total cumulative count of all samples lost across of instances of data published under the Topic.
    pub total_count: i32,
    /// The incremental number of samples lost since the last time the listener was called or the status was read.
    pub total_count_change: i32,
}

pub enum SampleRejectedStatusKind {
    NotRejected,
    RejectedByInstancesLimit,
    RejectedBySamplesLimit,
    RejectedBySamplesPerInstanceLimit,
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

pub struct QosPolicyCount {
    pub policy_id: QosPolicyId,
    pub count: i32,
}

pub type QosPolicyCountSeq<'a> = &'a [QosPolicyCount];

pub struct OfferedIncompatibleQosStatus<'a> {
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
    pub policies: &'a [QosPolicyCount],
}

pub struct RequestedIncompatibleQosStatus<'a> {
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
    pub policies: &'a [QosPolicyCount],
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SampleStateKind {
    Read,
    NotRead,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewStateKind {
    New,
    NotNew,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InstanceStateKind {
    Alive,
    NotAliveDisposed,
    NotAliveNoWriters,
}
