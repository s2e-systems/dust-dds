pub const DURATION_INFINITE_SEC: i32 = 0x7fffffff;
pub const DURATION_INFINITE_NSEC: u32 = 0xffffffff;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Duration_t {
    pub sec: i32,
    pub nanosec: u32,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Time_t {
    pub sec: i32,
    pub nanosec: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OctetSeq {
    pub length: i32,
    pub buffer: *mut u8,
}

impl Default for OctetSeq {
    fn default() -> Self {
        Self {
            length: 0,
            buffer: std::ptr::null_mut(),
        }
    }
}

impl OctetSeq {
    pub unsafe fn to_vec(&self) -> Vec<u8> {
        if self.buffer.is_null() || self.length <= 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(self.buffer, self.length as usize).to_vec() }
        }
    }

    pub fn from_vec(v: &[u8]) -> Self {
        if v.is_empty() {
            Self::default()
        } else {
            let mut bytes = v.to_vec();
            bytes.shrink_to_fit();
            let length = bytes.len() as i32;
            let buffer = bytes.as_mut_ptr();
            std::mem::forget(bytes);
            Self { length, buffer }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringSeq {
    pub length: i32,
    pub buffer: *mut *mut std::os::raw::c_char,
}

impl Default for StringSeq {
    fn default() -> Self {
        Self {
            length: 0,
            buffer: std::ptr::null_mut(),
        }
    }
}

impl StringSeq {
    pub unsafe fn to_vec(&self) -> Vec<String> {
        if self.buffer.is_null() || self.length <= 0 {
            Vec::new()
        } else {
            let mut result = Vec::new();
            let slice = unsafe { std::slice::from_raw_parts(self.buffer, self.length as usize) };
            for &ptr in slice {
                if !ptr.is_null() {
                    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
                    result.push(c_str.to_string_lossy().into_owned());
                }
            }
            result
        }
    }

    pub fn from_vec(v: &[String]) -> Self {
        if v.is_empty() {
            Self::default()
        } else {
            let mut ptrs: Vec<*mut std::os::raw::c_char> = v
                .iter()
                .map(|s| {
                    let c_str = std::ffi::CString::new(s.as_str()).unwrap();
                    c_str.into_raw()
                })
                .collect();
            ptrs.shrink_to_fit();
            let length = ptrs.len() as i32;
            let buffer = ptrs.as_mut_ptr();
            std::mem::forget(ptrs);
            Self { length, buffer }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_octet_seq_free(seq: OctetSeq) {
    if !seq.buffer.is_null() && seq.length > 0 {
        unsafe {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(seq.buffer, seq.length as usize));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_string_seq_free(seq: StringSeq) {
    if !seq.buffer.is_null() && seq.length > 0 {
        unsafe {
            let ptrs = Vec::from_raw_parts(seq.buffer, seq.length as usize, seq.length as usize);
            for ptr in ptrs {
                if !ptr.is_null() {
                    let _ = std::ffi::CString::from_raw(ptr);
                }
            }
        }
    }
}

impl From<Duration_t> for dust_dds::infrastructure::time::DurationKind {
    fn from(value: Duration_t) -> Self {
        if value.sec == DURATION_INFINITE_SEC && value.nanosec == DURATION_INFINITE_NSEC {
            dust_dds::infrastructure::time::DurationKind::Infinite
        } else {
            dust_dds::infrastructure::time::DurationKind::Finite(
                dust_dds::infrastructure::time::Duration::new(value.sec, value.nanosec),
            )
        }
    }
}

impl From<dust_dds::infrastructure::time::DurationKind> for Duration_t {
    fn from(value: dust_dds::infrastructure::time::DurationKind) -> Self {
        match value {
            dust_dds::infrastructure::time::DurationKind::Infinite => Duration_t {
                sec: DURATION_INFINITE_SEC,
                nanosec: DURATION_INFINITE_NSEC,
            },
            dust_dds::infrastructure::time::DurationKind::Finite(d) => Duration_t {
                sec: d.sec(),
                nanosec: d.nanosec(),
            },
        }
    }
}

impl From<Duration_t> for dust_dds::infrastructure::time::Duration {
    fn from(value: Duration_t) -> Self {
        dust_dds::infrastructure::time::Duration::new(value.sec, value.nanosec)
    }
}

impl From<dust_dds::infrastructure::time::Duration> for Duration_t {
    fn from(value: dust_dds::infrastructure::time::Duration) -> Self {
        Duration_t {
            sec: value.sec(),
            nanosec: value.nanosec(),
        }
    }
}

impl From<Time_t> for dust_dds::infrastructure::time::Time {
    fn from(value: Time_t) -> Self {
        dust_dds::infrastructure::time::Time::new(value.sec, value.nanosec)
    }
}

impl From<dust_dds::infrastructure::time::Time> for Time_t {
    fn from(value: dust_dds::infrastructure::time::Time) -> Self {
        Time_t {
            sec: value.sec(),
            nanosec: value.nanosec(),
        }
    }
}

pub type QosPolicyId_t = i32;

pub const INVALID_QOS_POLICY_ID: QosPolicyId_t = 0;
pub const USERDATA_QOS_POLICY_ID: QosPolicyId_t = 1;
pub const DURABILITY_QOS_POLICY_ID: QosPolicyId_t = 2;
pub const PRESENTATION_QOS_POLICY_ID: QosPolicyId_t = 3;
pub const DEADLINE_QOS_POLICY_ID: QosPolicyId_t = 4;
pub const LATENCYBUDGET_QOS_POLICY_ID: QosPolicyId_t = 5;
pub const OWNERSHIP_QOS_POLICY_ID: QosPolicyId_t = 6;
pub const OWNERSHIPSTRENGTH_QOS_POLICY_ID: QosPolicyId_t = 7;
pub const LIVELINESS_QOS_POLICY_ID: QosPolicyId_t = 8;
pub const TIMEBASEDFILTER_QOS_POLICY_ID: QosPolicyId_t = 9;
pub const PARTITION_QOS_POLICY_ID: QosPolicyId_t = 10;
pub const RELIABILITY_QOS_POLICY_ID: QosPolicyId_t = 11;
pub const DESTINATIONORDER_QOS_POLICY_ID: QosPolicyId_t = 12;
pub const HISTORY_QOS_POLICY_ID: QosPolicyId_t = 13;
pub const RESOURCELIMITS_QOS_POLICY_ID: QosPolicyId_t = 14;
pub const ENTITYFACTORY_QOS_POLICY_ID: QosPolicyId_t = 15;
pub const WRITERDATALIFECYCLE_QOS_POLICY_ID: QosPolicyId_t = 16;
pub const READERDATALIFECYCLE_QOS_POLICY_ID: QosPolicyId_t = 17;
pub const TOPICDATA_QOS_POLICY_ID: QosPolicyId_t = 18;
pub const GROUPDATA_QOS_POLICY_ID: QosPolicyId_t = 19;
pub const TRANSPORTPRIORITY_QOS_POLICY_ID: QosPolicyId_t = 20;
pub const LIFESPAN_QOS_POLICY_ID: QosPolicyId_t = 21;
pub const DURABILITYSERVICE_QOS_POLICY_ID: QosPolicyId_t = 22;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UserDataQosPolicy {
    pub value: OctetSeq,
}

impl From<UserDataQosPolicy> for dust_dds::infrastructure::qos_policy::UserDataQosPolicy {
    fn from(val: UserDataQosPolicy) -> Self {
        Self {
            value: unsafe { val.value.to_vec() },
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::UserDataQosPolicy> for UserDataQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::UserDataQosPolicy) -> Self {
        Self {
            value: OctetSeq::from_vec(&val.value),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TopicDataQosPolicy {
    pub value: OctetSeq,
}

impl From<TopicDataQosPolicy> for dust_dds::infrastructure::qos_policy::TopicDataQosPolicy {
    fn from(val: TopicDataQosPolicy) -> Self {
        Self {
            value: unsafe { val.value.to_vec() },
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::TopicDataQosPolicy> for TopicDataQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::TopicDataQosPolicy) -> Self {
        Self {
            value: OctetSeq::from_vec(&val.value),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupDataQosPolicy {
    pub value: OctetSeq,
}

impl From<GroupDataQosPolicy> for dust_dds::infrastructure::qos_policy::GroupDataQosPolicy {
    fn from(val: GroupDataQosPolicy) -> Self {
        Self {
            value: unsafe { val.value.to_vec() },
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::GroupDataQosPolicy> for GroupDataQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::GroupDataQosPolicy) -> Self {
        Self {
            value: OctetSeq::from_vec(&val.value),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransportPriorityQosPolicy {
    pub value: i32,
}

impl From<TransportPriorityQosPolicy>
    for dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy
{
    fn from(val: TransportPriorityQosPolicy) -> Self {
        Self { value: val.value }
    }
}
impl From<dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy>
    for TransportPriorityQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy) -> Self {
        Self { value: val.value }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifespanQosPolicy {
    pub duration: Duration_t,
}

impl Default for LifespanQosPolicy {
    fn default() -> Self {
        Self {
            duration: Duration_t {
                sec: DURATION_INFINITE_SEC,
                nanosec: DURATION_INFINITE_NSEC,
            },
        }
    }
}

impl From<LifespanQosPolicy> for dust_dds::infrastructure::qos_policy::LifespanQosPolicy {
    fn from(val: LifespanQosPolicy) -> Self {
        Self {
            duration: val.duration.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::LifespanQosPolicy> for LifespanQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::LifespanQosPolicy) -> Self {
        Self {
            duration: val.duration.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum DurabilityQosPolicyKind {
    #[default]
    VOLATILE_DURABILITY_QOS,
    TRANSIENT_LOCAL_DURABILITY_QOS,
    TRANSIENT_DURABILITY_QOS,
    PERSISTENT_DURABILITY_QOS,
}


impl From<DurabilityQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind
{
    fn from(kind: DurabilityQosPolicyKind) -> Self {
        match kind {
            DurabilityQosPolicyKind::VOLATILE_DURABILITY_QOS => Self::Volatile,
            DurabilityQosPolicyKind::TRANSIENT_LOCAL_DURABILITY_QOS => Self::TransientLocal,
            DurabilityQosPolicyKind::TRANSIENT_DURABILITY_QOS => Self::Transient,
            DurabilityQosPolicyKind::PERSISTENT_DURABILITY_QOS => Self::Persistent,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind>
    for DurabilityQosPolicyKind
{
    fn from(kind: dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind) -> Self {
        match kind {
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Volatile => {
                Self::VOLATILE_DURABILITY_QOS
            }
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::TransientLocal => {
                Self::TRANSIENT_LOCAL_DURABILITY_QOS
            }
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Transient => {
                Self::TRANSIENT_DURABILITY_QOS
            }
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Persistent => {
                Self::PERSISTENT_DURABILITY_QOS
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DurabilityQosPolicy {
    pub kind: DurabilityQosPolicyKind,
}

impl From<DurabilityQosPolicy> for dust_dds::infrastructure::qos_policy::DurabilityQosPolicy {
    fn from(val: DurabilityQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::DurabilityQosPolicy> for DurabilityQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::DurabilityQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum PresentationQosPolicyAccessScopeKind {
    #[default]
    INSTANCE_PRESENTATION_QOS,
    TOPIC_PRESENTATION_QOS,
    GROUP_PRESENTATION_QOS,
}


impl From<PresentationQosPolicyAccessScopeKind>
    for dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind
{
    fn from(kind: PresentationQosPolicyAccessScopeKind) -> Self {
        match kind {
            PresentationQosPolicyAccessScopeKind::INSTANCE_PRESENTATION_QOS => Self::Instance,
            PresentationQosPolicyAccessScopeKind::TOPIC_PRESENTATION_QOS => Self::Topic,
            PresentationQosPolicyAccessScopeKind::GROUP_PRESENTATION_QOS => Self::Topic,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind>
    for PresentationQosPolicyAccessScopeKind
{
    fn from(
        kind: dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind,
    ) -> Self {
        match kind {
            dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind::Instance => Self::INSTANCE_PRESENTATION_QOS,
            dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind::Topic => Self::TOPIC_PRESENTATION_QOS,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PresentationQosPolicy {
    pub access_scope: PresentationQosPolicyAccessScopeKind,
    pub coherent_access: bool,
    pub ordered_access: bool,
}

impl From<PresentationQosPolicy> for dust_dds::infrastructure::qos_policy::PresentationQosPolicy {
    fn from(val: PresentationQosPolicy) -> Self {
        Self {
            access_scope: val.access_scope.into(),
            coherent_access: val.coherent_access,
            ordered_access: val.ordered_access,
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::PresentationQosPolicy> for PresentationQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::PresentationQosPolicy) -> Self {
        Self {
            access_scope: val.access_scope.into(),
            coherent_access: val.coherent_access,
            ordered_access: val.ordered_access,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeadlineQosPolicy {
    pub period: Duration_t,
}

impl Default for DeadlineQosPolicy {
    fn default() -> Self {
        Self {
            period: Duration_t {
                sec: DURATION_INFINITE_SEC,
                nanosec: DURATION_INFINITE_NSEC,
            },
        }
    }
}

impl From<DeadlineQosPolicy> for dust_dds::infrastructure::qos_policy::DeadlineQosPolicy {
    fn from(val: DeadlineQosPolicy) -> Self {
        Self {
            period: val.period.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::DeadlineQosPolicy> for DeadlineQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::DeadlineQosPolicy) -> Self {
        Self {
            period: val.period.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyBudgetQosPolicy {
    pub duration: Duration_t,
}

impl Default for LatencyBudgetQosPolicy {
    fn default() -> Self {
        Self {
            duration: Duration_t { sec: 0, nanosec: 0 },
        }
    }
}

impl From<LatencyBudgetQosPolicy> for dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy {
    fn from(val: LatencyBudgetQosPolicy) -> Self {
        Self {
            duration: val.duration.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy> for LatencyBudgetQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy) -> Self {
        Self {
            duration: val.duration.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum OwnershipQosPolicyKind {
    #[default]
    SHARED_OWNERSHIP_QOS,
    EXCLUSIVE_OWNERSHIP_QOS,
}


impl From<OwnershipQosPolicyKind> for dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind {
    fn from(kind: OwnershipQosPolicyKind) -> Self {
        match kind {
            OwnershipQosPolicyKind::SHARED_OWNERSHIP_QOS => Self::Shared,
            OwnershipQosPolicyKind::EXCLUSIVE_OWNERSHIP_QOS => Self::Exclusive,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind> for OwnershipQosPolicyKind {
    fn from(kind: dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind) -> Self {
        match kind {
            dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind::Shared => {
                Self::SHARED_OWNERSHIP_QOS
            }
            dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind::Exclusive => {
                Self::EXCLUSIVE_OWNERSHIP_QOS
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OwnershipQosPolicy {
    pub kind: OwnershipQosPolicyKind,
}

impl From<OwnershipQosPolicy> for dust_dds::infrastructure::qos_policy::OwnershipQosPolicy {
    fn from(val: OwnershipQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::OwnershipQosPolicy> for OwnershipQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::OwnershipQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OwnershipStrengthQosPolicy {
    pub value: i32,
}

impl From<OwnershipStrengthQosPolicy>
    for dust_dds::infrastructure::qos_policy::OwnershipStrengthQosPolicy
{
    fn from(val: OwnershipStrengthQosPolicy) -> Self {
        Self { value: val.value }
    }
}
impl From<dust_dds::infrastructure::qos_policy::OwnershipStrengthQosPolicy>
    for OwnershipStrengthQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::OwnershipStrengthQosPolicy) -> Self {
        Self { value: val.value }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum LivelinessQosPolicyKind {
    #[default]
    AUTOMATIC_LIVELINESS_QOS,
    MANUAL_BY_PARTICIPANT_LIVELINESS_QOS,
    MANUAL_BY_TOPIC_LIVELINESS_QOS,
}


impl From<LivelinessQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind
{
    fn from(kind: LivelinessQosPolicyKind) -> Self {
        match kind {
            LivelinessQosPolicyKind::AUTOMATIC_LIVELINESS_QOS => Self::Automatic,
            LivelinessQosPolicyKind::MANUAL_BY_PARTICIPANT_LIVELINESS_QOS => {
                Self::ManualByParticipant
            }
            LivelinessQosPolicyKind::MANUAL_BY_TOPIC_LIVELINESS_QOS => Self::ManualByTopic,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind>
    for LivelinessQosPolicyKind
{
    fn from(kind: dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind) -> Self {
        match kind {
            dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::Automatic => {
                Self::AUTOMATIC_LIVELINESS_QOS
            }
            dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::ManualByParticipant => {
                Self::MANUAL_BY_PARTICIPANT_LIVELINESS_QOS
            }
            dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::ManualByTopic => {
                Self::MANUAL_BY_TOPIC_LIVELINESS_QOS
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LivelinessQosPolicy {
    pub kind: LivelinessQosPolicyKind,
    pub lease_duration: Duration_t,
}

impl Default for LivelinessQosPolicy {
    fn default() -> Self {
        Self {
            kind: LivelinessQosPolicyKind::AUTOMATIC_LIVELINESS_QOS,
            lease_duration: Duration_t {
                sec: DURATION_INFINITE_SEC,
                nanosec: DURATION_INFINITE_NSEC,
            },
        }
    }
}

impl From<LivelinessQosPolicy> for dust_dds::infrastructure::qos_policy::LivelinessQosPolicy {
    fn from(val: LivelinessQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
            lease_duration: val.lease_duration.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::LivelinessQosPolicy> for LivelinessQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::LivelinessQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
            lease_duration: val.lease_duration.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeBasedFilterQosPolicy {
    pub minimum_separation: Duration_t,
}

impl From<TimeBasedFilterQosPolicy>
    for dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy
{
    fn from(val: TimeBasedFilterQosPolicy) -> Self {
        Self {
            minimum_separation: val.minimum_separation.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy>
    for TimeBasedFilterQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy) -> Self {
        Self {
            minimum_separation: val.minimum_separation.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartitionQosPolicy {
    pub name: StringSeq,
}

impl From<PartitionQosPolicy> for dust_dds::infrastructure::qos_policy::PartitionQosPolicy {
    fn from(val: PartitionQosPolicy) -> Self {
        Self {
            name: unsafe { val.name.to_vec() },
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::PartitionQosPolicy> for PartitionQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::PartitionQosPolicy) -> Self {
        Self {
            name: StringSeq::from_vec(&val.name),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum ReliabilityQosPolicyKind {
    #[default]
    BEST_EFFORT_RELIABILITY_QOS,
    RELIABLE_RELIABILITY_QOS,
}


impl From<ReliabilityQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind
{
    fn from(kind: ReliabilityQosPolicyKind) -> Self {
        match kind {
            ReliabilityQosPolicyKind::BEST_EFFORT_RELIABILITY_QOS => Self::BestEffort,
            ReliabilityQosPolicyKind::RELIABLE_RELIABILITY_QOS => Self::Reliable,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind>
    for ReliabilityQosPolicyKind
{
    fn from(kind: dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind) -> Self {
        match kind {
            dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffort => {
                Self::BEST_EFFORT_RELIABILITY_QOS
            }
            dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::Reliable => {
                Self::RELIABLE_RELIABILITY_QOS
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReliabilityQosPolicy {
    pub kind: ReliabilityQosPolicyKind,
    pub max_blocking_time: Duration_t,
}

impl Default for ReliabilityQosPolicy {
    fn default() -> Self {
        Self {
            kind: ReliabilityQosPolicyKind::BEST_EFFORT_RELIABILITY_QOS,
            max_blocking_time: Duration_t {
                sec: 0,
                nanosec: 100_000_000,
            },
        }
    }
}

impl From<ReliabilityQosPolicy> for dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy {
    fn from(val: ReliabilityQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
            max_blocking_time: val.max_blocking_time.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy> for ReliabilityQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
            max_blocking_time: val.max_blocking_time.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum DestinationOrderQosPolicyKind {
    #[default]
    BY_RECEPTION_TIMESTAMP_DESTINATIONORDER_QOS,
    BY_SOURCE_TIMESTAMP_DESTINATIONORDER_QOS,
}


impl From<DestinationOrderQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind
{
    fn from(kind: DestinationOrderQosPolicyKind) -> Self {
        match kind {
            DestinationOrderQosPolicyKind::BY_RECEPTION_TIMESTAMP_DESTINATIONORDER_QOS => {
                Self::ByReceptionTimestamp
            }
            DestinationOrderQosPolicyKind::BY_SOURCE_TIMESTAMP_DESTINATIONORDER_QOS => {
                Self::BySourceTimestamp
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind>
    for DestinationOrderQosPolicyKind
{
    fn from(kind: dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind) -> Self {
        match kind {
            dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind::ByReceptionTimestamp => Self::BY_RECEPTION_TIMESTAMP_DESTINATIONORDER_QOS,
            dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind::BySourceTimestamp => Self::BY_SOURCE_TIMESTAMP_DESTINATIONORDER_QOS,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DestinationOrderQosPolicy {
    pub kind: DestinationOrderQosPolicyKind,
}

impl From<DestinationOrderQosPolicy>
    for dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy
{
    fn from(val: DestinationOrderQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy>
    for DestinationOrderQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy) -> Self {
        Self {
            kind: val.kind.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum HistoryQosPolicyKind {
    #[default]
    KEEP_LAST_HISTORY_QOS,
    KEEP_ALL_HISTORY_QOS,
}


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryQosPolicy {
    pub kind: HistoryQosPolicyKind,
    pub depth: i32,
}

impl Default for HistoryQosPolicy {
    fn default() -> Self {
        Self {
            kind: HistoryQosPolicyKind::KEEP_LAST_HISTORY_QOS,
            depth: 1,
        }
    }
}

impl From<HistoryQosPolicy> for dust_dds::infrastructure::qos_policy::HistoryQosPolicy {
    fn from(val: HistoryQosPolicy) -> Self {
        let kind = match val.kind {
            HistoryQosPolicyKind::KEEP_LAST_HISTORY_QOS => {
                dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(
                    val.depth as u32,
                )
            }
            HistoryQosPolicyKind::KEEP_ALL_HISTORY_QOS => {
                dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepAll
            }
        };
        Self { kind }
    }
}
impl From<dust_dds::infrastructure::qos_policy::HistoryQosPolicy> for HistoryQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::HistoryQosPolicy) -> Self {
        match val.kind {
            dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(depth) => Self {
                kind: HistoryQosPolicyKind::KEEP_LAST_HISTORY_QOS,
                depth: depth as i32,
            },
            dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepAll => Self {
                kind: HistoryQosPolicyKind::KEEP_ALL_HISTORY_QOS,
                depth: 1,
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimitsQosPolicy {
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

impl Default for ResourceLimitsQosPolicy {
    fn default() -> Self {
        Self {
            max_samples: -1,
            max_instances: -1,
            max_samples_per_instance: -1,
        }
    }
}

impl From<ResourceLimitsQosPolicy>
    for dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy
{
    fn from(val: ResourceLimitsQosPolicy) -> Self {
        let map_len = |v: i32| {
            if v < 0 {
                dust_dds::infrastructure::qos_policy::Length::Unlimited
            } else {
                dust_dds::infrastructure::qos_policy::Length::Limited(v)
            }
        };
        Self {
            max_samples: map_len(val.max_samples),
            max_instances: map_len(val.max_instances),
            max_samples_per_instance: map_len(val.max_samples_per_instance),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy>
    for ResourceLimitsQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy) -> Self {
        let map_len = |l: dust_dds::infrastructure::qos_policy::Length| match l {
            dust_dds::infrastructure::qos_policy::Length::Unlimited => -1,
            dust_dds::infrastructure::qos_policy::Length::Limited(v) => v,
        };
        Self {
            max_samples: map_len(val.max_samples),
            max_instances: map_len(val.max_instances),
            max_samples_per_instance: map_len(val.max_samples_per_instance),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityFactoryQosPolicy {
    pub autoenable_created_entities: bool,
}

impl Default for EntityFactoryQosPolicy {
    fn default() -> Self {
        Self {
            autoenable_created_entities: true,
        }
    }
}

impl From<EntityFactoryQosPolicy> for dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy {
    fn from(val: EntityFactoryQosPolicy) -> Self {
        Self {
            autoenable_created_entities: val.autoenable_created_entities,
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy> for EntityFactoryQosPolicy {
    fn from(val: dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy) -> Self {
        Self {
            autoenable_created_entities: val.autoenable_created_entities,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriterDataLifecycleQosPolicy {
    pub autodispose_unregistered_instances: bool,
}

impl Default for WriterDataLifecycleQosPolicy {
    fn default() -> Self {
        Self {
            autodispose_unregistered_instances: true,
        }
    }
}

impl From<WriterDataLifecycleQosPolicy>
    for dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy
{
    fn from(val: WriterDataLifecycleQosPolicy) -> Self {
        Self {
            autodispose_unregistered_instances: val.autodispose_unregistered_instances,
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy>
    for WriterDataLifecycleQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy) -> Self {
        Self {
            autodispose_unregistered_instances: val.autodispose_unregistered_instances,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReaderDataLifecycleQosPolicy {
    pub autopurge_nowriter_samples_delay: Duration_t,
    pub autopurge_disposed_samples_delay: Duration_t,
}

impl Default for ReaderDataLifecycleQosPolicy {
    fn default() -> Self {
        Self {
            autopurge_nowriter_samples_delay: Duration_t {
                sec: DURATION_INFINITE_SEC,
                nanosec: DURATION_INFINITE_NSEC,
            },
            autopurge_disposed_samples_delay: Duration_t {
                sec: DURATION_INFINITE_SEC,
                nanosec: DURATION_INFINITE_NSEC,
            },
        }
    }
}

impl From<ReaderDataLifecycleQosPolicy>
    for dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy
{
    fn from(val: ReaderDataLifecycleQosPolicy) -> Self {
        Self {
            autopurge_nowriter_samples_delay: val.autopurge_nowriter_samples_delay.into(),
            autopurge_disposed_samples_delay: val.autopurge_disposed_samples_delay.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy>
    for ReaderDataLifecycleQosPolicy
{
    fn from(val: dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy) -> Self {
        Self {
            autopurge_nowriter_samples_delay: val.autopurge_nowriter_samples_delay.into(),
            autopurge_disposed_samples_delay: val.autopurge_disposed_samples_delay.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurabilityServiceQosPolicy {
    pub service_cleanup_delay: Duration_t,
    pub history_kind: HistoryQosPolicyKind,
    pub history_depth: i32,
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

impl Default for DurabilityServiceQosPolicy {
    fn default() -> Self {
        Self {
            service_cleanup_delay: Duration_t { sec: 0, nanosec: 0 },
            history_kind: HistoryQosPolicyKind::KEEP_LAST_HISTORY_QOS,
            history_depth: 1,
            max_samples: -1,
            max_instances: -1,
            max_samples_per_instance: -1,
        }
    }
}
