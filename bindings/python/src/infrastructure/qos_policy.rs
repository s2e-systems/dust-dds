use pyo3::prelude::*;

use super::time::DurationKind;

#[pyclass]
#[derive(Clone)]
pub enum Length {
    Unlimited {},
    Limited { length: u32 },
}

impl From<Length> for dust_dds::infrastructure::qos_policy::Length {
    fn from(value: Length) -> Self {
        match value {
            Length::Unlimited {} => dust_dds::infrastructure::qos_policy::Length::Unlimited,
            Length::Limited { length } => {
                dust_dds::infrastructure::qos_policy::Length::Limited(length)
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::Length> for Length {
    fn from(value: dust_dds::infrastructure::qos_policy::Length) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::Length::Unlimited => Length::Unlimited {},
            dust_dds::infrastructure::qos_policy::Length::Limited(length) => {
                Length::Limited { length }
            }
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct UserDataQosPolicy(dust_dds::infrastructure::qos_policy::UserDataQosPolicy);

impl From<UserDataQosPolicy> for dust_dds::infrastructure::qos_policy::UserDataQosPolicy {
    fn from(value: UserDataQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::UserDataQosPolicy> for UserDataQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::UserDataQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl UserDataQosPolicy {
    #[new]
    pub fn new(value: Vec<u8>) -> Self {
        Self(dust_dds::infrastructure::qos_policy::UserDataQosPolicy { value })
    }

    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }

    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct EntityFactoryQosPolicy(dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy);

impl From<EntityFactoryQosPolicy> for dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy {
    fn from(value: EntityFactoryQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy> for EntityFactoryQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl EntityFactoryQosPolicy {
    #[new]
    pub fn new(autoenable_created_entities: bool) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy {
                autoenable_created_entities,
            },
        )
    }

    pub fn get_autoenable_created_entities(&self) -> bool {
        self.0.autoenable_created_entities
    }

    pub fn set_autoenable_created_entities(&mut self, value: bool) {
        self.0.autoenable_created_entities = value
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct TopicDataQosPolicy(dust_dds::infrastructure::qos_policy::TopicDataQosPolicy);

impl From<TopicDataQosPolicy> for dust_dds::infrastructure::qos_policy::TopicDataQosPolicy {
    fn from(value: TopicDataQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::TopicDataQosPolicy> for TopicDataQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::TopicDataQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl TopicDataQosPolicy {
    #[new]
    pub fn new(value: Vec<u8>) -> Self {
        Self(dust_dds::infrastructure::qos_policy::TopicDataQosPolicy { value })
    }

    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }

    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value
    }
}

#[derive(Clone, Copy)]
#[pyclass]
pub enum DurabilityQosPolicyKind {
    Volatile,
    TransientLocal,
}

impl From<DurabilityQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind
{
    fn from(value: DurabilityQosPolicyKind) -> Self {
        match value {
            DurabilityQosPolicyKind::Volatile => {
                dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Volatile
            }
            DurabilityQosPolicyKind::TransientLocal => {
                dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::TransientLocal
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind>
    for DurabilityQosPolicyKind
{
    fn from(value: dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Volatile => {
                DurabilityQosPolicyKind::Volatile
            }
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::TransientLocal => {
                DurabilityQosPolicyKind::TransientLocal
            }
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct DurabilityQosPolicy(dust_dds::infrastructure::qos_policy::DurabilityQosPolicy);

impl From<DurabilityQosPolicy> for dust_dds::infrastructure::qos_policy::DurabilityQosPolicy {
    fn from(value: DurabilityQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::DurabilityQosPolicy> for DurabilityQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::DurabilityQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DurabilityQosPolicy {
    #[new]
    pub fn new(kind: DurabilityQosPolicyKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::DurabilityQosPolicy { kind: kind.into() })
    }

    pub fn get_kind(&self) -> DurabilityQosPolicyKind {
        self.0.kind.into()
    }

    pub fn set_kind(&mut self, value: DurabilityQosPolicyKind) {
        self.0.kind = value.into()
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct DeadlineQosPolicy(dust_dds::infrastructure::qos_policy::DeadlineQosPolicy);

impl From<DeadlineQosPolicy> for dust_dds::infrastructure::qos_policy::DeadlineQosPolicy {
    fn from(value: DeadlineQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::DeadlineQosPolicy> for DeadlineQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::DeadlineQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DeadlineQosPolicy {
    #[new]
    pub fn new(period: DurationKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::DeadlineQosPolicy {
            period: period.into(),
        })
    }

    pub fn get_period(&self) -> DurationKind {
        self.0.period.into()
    }

    pub fn set_period(&mut self, value: DurationKind) {
        self.0.period = value.into()
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct LatencyBudgetQosPolicy(dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy);

impl From<LatencyBudgetQosPolicy> for dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy {
    fn from(value: LatencyBudgetQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy> for LatencyBudgetQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl LatencyBudgetQosPolicy {
    #[new]
    pub fn new(duration: DurationKind) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::LatencyBudgetQosPolicy {
                duration: duration.into(),
            },
        )
    }

    pub fn get_duration(&self) -> DurationKind {
        self.0.duration.into()
    }

    pub fn set_duration(&mut self, value: DurationKind) {
        self.0.duration = value.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub enum LivelinessQosPolicyKind {
    Automatic,
    ManualByParticipant,
    ManualByTopic,
}

impl From<LivelinessQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind
{
    fn from(value: LivelinessQosPolicyKind) -> Self {
        match value {
            LivelinessQosPolicyKind::Automatic => {
                dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::Automatic
            }
            LivelinessQosPolicyKind::ManualByParticipant => {
                dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::ManualByParticipant
            }
            LivelinessQosPolicyKind::ManualByTopic => {
                dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::ManualByTopic
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind>
    for LivelinessQosPolicyKind
{
    fn from(value: dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::Automatic => {
                LivelinessQosPolicyKind::Automatic
            }
            dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::ManualByParticipant => {
                LivelinessQosPolicyKind::ManualByParticipant
            }
            dust_dds::infrastructure::qos_policy::LivelinessQosPolicyKind::ManualByTopic => {
                LivelinessQosPolicyKind::ManualByTopic
            }
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct LivelinessQosPolicy(dust_dds::infrastructure::qos_policy::LivelinessQosPolicy);

impl From<LivelinessQosPolicy> for dust_dds::infrastructure::qos_policy::LivelinessQosPolicy {
    fn from(value: LivelinessQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::LivelinessQosPolicy> for LivelinessQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::LivelinessQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl LivelinessQosPolicy {
    #[new]
    pub fn new(kind: LivelinessQosPolicyKind, lease_duration: DurationKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::LivelinessQosPolicy {
            kind: kind.into(),
            lease_duration: lease_duration.into(),
        })
    }

    pub fn get_kind(&self) -> LivelinessQosPolicyKind {
        self.0.kind.into()
    }

    pub fn set_kind(&mut self, value: LivelinessQosPolicyKind) {
        self.0.kind = value.into()
    }

    pub fn get_lease_duration(&self) -> DurationKind {
        self.0.lease_duration.into()
    }

    pub fn set_lease_duration(&mut self, value: DurationKind) {
        self.0.lease_duration = value.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub enum ReliabilityQosPolicyKind {
    BestEffort,
    Reliable,
}

impl From<ReliabilityQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind
{
    fn from(value: ReliabilityQosPolicyKind) -> Self {
        match value {
            ReliabilityQosPolicyKind::BestEffort => {
                dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffort
            }
            ReliabilityQosPolicyKind::Reliable => {
                dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::Reliable
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind>
    for ReliabilityQosPolicyKind
{
    fn from(value: dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffort => {
                ReliabilityQosPolicyKind::BestEffort
            }
            dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::Reliable => {
                ReliabilityQosPolicyKind::Reliable
            }
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ReliabilityQosPolicy(dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy);

impl From<ReliabilityQosPolicy> for dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy {
    fn from(value: ReliabilityQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy> for ReliabilityQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl ReliabilityQosPolicy {
    #[new]
    pub fn new(kind: ReliabilityQosPolicyKind, max_blocking_time: DurationKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy {
            kind: kind.into(),
            max_blocking_time: max_blocking_time.into(),
        })
    }

    pub fn get_kind(&self) -> ReliabilityQosPolicyKind {
        self.0.kind.into()
    }

    pub fn set_kind(&mut self, value: ReliabilityQosPolicyKind) {
        self.0.kind = value.into()
    }

    pub fn get_max_blocking_time(&self) -> DurationKind {
        self.0.max_blocking_time.into()
    }

    pub fn set_max_blocking_time(&mut self, value: DurationKind) {
        self.0.max_blocking_time = value.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub enum DestinationOrderQosPolicyKind {
    ByReceptionTimestamp,
    BySourceTimestamp,
}

impl From<DestinationOrderQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind
{
    fn from(value: DestinationOrderQosPolicyKind) -> Self {
        match value {
            DestinationOrderQosPolicyKind::ByReceptionTimestamp => dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind::ByReceptionTimestamp,
            DestinationOrderQosPolicyKind::BySourceTimestamp => dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind::BySourceTimestamp,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind>
    for DestinationOrderQosPolicyKind
{
    fn from(value: dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind::ByReceptionTimestamp => DestinationOrderQosPolicyKind::ByReceptionTimestamp,
            dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicyKind::BySourceTimestamp => DestinationOrderQosPolicyKind::BySourceTimestamp,
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct DestinationOrderQosPolicy(
    dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy,
);

impl From<DestinationOrderQosPolicy>
    for dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy
{
    fn from(value: DestinationOrderQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy>
    for DestinationOrderQosPolicy
{
    fn from(value: dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DestinationOrderQosPolicy {
    #[new]
    pub fn new(kind: DestinationOrderQosPolicyKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::DestinationOrderQosPolicy { kind: kind.into() })
    }

    pub fn get_kind(&self) -> DestinationOrderQosPolicyKind {
        self.0.kind.into()
    }

    pub fn set_kind(&mut self, value: DestinationOrderQosPolicyKind) {
        self.0.kind = value.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub enum HistoryQosPolicyKind {
    KeepLast { depth: i32 },
    KeepAll {},
}

impl From<HistoryQosPolicyKind> for dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind {
    fn from(value: HistoryQosPolicyKind) -> Self {
        match value {
            HistoryQosPolicyKind::KeepLast { depth } => {
                dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(depth)
            }
            HistoryQosPolicyKind::KeepAll {} => {
                dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepAll
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind> for HistoryQosPolicyKind {
    fn from(value: dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(depth) => {
                HistoryQosPolicyKind::KeepLast { depth }
            }
            dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepAll {} => {
                HistoryQosPolicyKind::KeepAll {}
            }
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct HistoryQosPolicy(dust_dds::infrastructure::qos_policy::HistoryQosPolicy);

impl From<HistoryQosPolicy> for dust_dds::infrastructure::qos_policy::HistoryQosPolicy {
    fn from(value: HistoryQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::HistoryQosPolicy> for HistoryQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::HistoryQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl HistoryQosPolicy {
    #[new]
    pub fn new(kind: HistoryQosPolicyKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::HistoryQosPolicy { kind: kind.into() })
    }

    pub fn get_kind(&self) -> HistoryQosPolicyKind {
        self.0.kind.into()
    }

    pub fn set_kind(&mut self, value: HistoryQosPolicyKind) {
        self.0.kind = value.into()
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct ResourceLimitsQosPolicy(dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy);

impl From<ResourceLimitsQosPolicy>
    for dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy
{
    fn from(value: ResourceLimitsQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy>
    for ResourceLimitsQosPolicy
{
    fn from(value: dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl ResourceLimitsQosPolicy {
    #[new]
    pub fn new(
        max_samples: Length,
        max_instances: Length,
        max_samples_per_instance: Length,
    ) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::ResourceLimitsQosPolicy {
                max_samples: max_samples.into(),
                max_instances: max_instances.into(),
                max_samples_per_instance: max_samples_per_instance.into(),
            },
        )
    }

    pub fn get_max_samples(&self) -> Length {
        self.0.max_samples.into()
    }

    pub fn set_max_samples(&mut self, value: Length) {
        self.0.max_samples = value.into()
    }

    pub fn get_max_instances(&self) -> Length {
        self.0.max_instances.into()
    }

    pub fn set_max_instances(&mut self, value: Length) {
        self.0.max_instances = value.into()
    }

    pub fn get_max_samples_per_instance(&self) -> Length {
        self.0.max_samples_per_instance.into()
    }

    pub fn set_max_samples_per_instance(&mut self, value: Length) {
        self.0.max_samples_per_instance = value.into()
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct TransportPriorityQosPolicy(
    dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy,
);

impl From<TransportPriorityQosPolicy>
    for dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy
{
    fn from(value: TransportPriorityQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy>
    for TransportPriorityQosPolicy
{
    fn from(value: dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl TransportPriorityQosPolicy {
    #[new]
    pub fn new(value: i32) -> Self {
        Self(dust_dds::infrastructure::qos_policy::TransportPriorityQosPolicy { value })
    }

    pub fn get_value(&self) -> i32 {
        self.0.value
    }

    pub fn set_value(&mut self, value: i32) {
        self.0.value = value
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct LifespanQosPolicy(dust_dds::infrastructure::qos_policy::LifespanQosPolicy);

impl From<LifespanQosPolicy> for dust_dds::infrastructure::qos_policy::LifespanQosPolicy {
    fn from(value: LifespanQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::LifespanQosPolicy> for LifespanQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::LifespanQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl LifespanQosPolicy {
    #[new]
    pub fn new(duration: DurationKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::LifespanQosPolicy {
            duration: duration.into(),
        })
    }

    pub fn get_duration(&self) -> DurationKind {
        self.0.duration.into()
    }

    pub fn set_duration(&mut self, value: DurationKind) {
        self.0.duration = value.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub enum OwnershipQosPolicyKind {
    Shared,
}

impl From<OwnershipQosPolicyKind> for dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind {
    fn from(value: OwnershipQosPolicyKind) -> Self {
        match value {
            OwnershipQosPolicyKind::Shared => {
                dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind::Shared
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind> for OwnershipQosPolicyKind {
    fn from(value: dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::OwnershipQosPolicyKind::Shared => {
                OwnershipQosPolicyKind::Shared
            }
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct OwnershipQosPolicy(dust_dds::infrastructure::qos_policy::OwnershipQosPolicy);

impl From<OwnershipQosPolicy> for dust_dds::infrastructure::qos_policy::OwnershipQosPolicy {
    fn from(value: OwnershipQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::OwnershipQosPolicy> for OwnershipQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::OwnershipQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl OwnershipQosPolicy {
    #[new]
    pub fn new(kind: OwnershipQosPolicyKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::OwnershipQosPolicy { kind: kind.into() })
    }

    pub fn get_kind(&self) -> OwnershipQosPolicyKind {
        self.0.kind.into()
    }

    pub fn set_kind(&mut self, value: OwnershipQosPolicyKind) {
        self.0.kind = value.into()
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct GroupDataQosPolicy(dust_dds::infrastructure::qos_policy::GroupDataQosPolicy);

impl From<GroupDataQosPolicy> for dust_dds::infrastructure::qos_policy::GroupDataQosPolicy {
    fn from(value: GroupDataQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::GroupDataQosPolicy> for GroupDataQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::GroupDataQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl GroupDataQosPolicy {
    #[new]
    pub fn new(value: Vec<u8>) -> Self {
        Self(dust_dds::infrastructure::qos_policy::GroupDataQosPolicy { value })
    }

    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }

    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value;
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct PartitionQosPolicy(dust_dds::infrastructure::qos_policy::PartitionQosPolicy);

impl From<PartitionQosPolicy> for dust_dds::infrastructure::qos_policy::PartitionQosPolicy {
    fn from(value: PartitionQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::PartitionQosPolicy> for PartitionQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::PartitionQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl PartitionQosPolicy {
    #[new]
    pub fn new(name: Vec<String>) -> Self {
        Self(dust_dds::infrastructure::qos_policy::PartitionQosPolicy { name })
    }

    pub fn get_name(&self) -> Vec<String> {
        self.0.name.clone()
    }

    pub fn set_name(&mut self, value: Vec<String>) {
        self.0.name = value;
    }
}

#[pyclass]
#[derive(Clone)]
pub enum PresentationQosPolicyAccessScopeKind {
    Instance,
    Topic,
}

impl From<PresentationQosPolicyAccessScopeKind>
    for dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind
{
    fn from(value: PresentationQosPolicyAccessScopeKind) -> Self {
        match value {
            PresentationQosPolicyAccessScopeKind::Instance => {
                dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind::Instance
            }
            PresentationQosPolicyAccessScopeKind::Topic => {
                dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind::Topic
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind>
    for PresentationQosPolicyAccessScopeKind
{
    fn from(
        value: dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind,
    ) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind::Instance => PresentationQosPolicyAccessScopeKind::Instance,
            dust_dds::infrastructure::qos_policy::PresentationQosPolicyAccessScopeKind::Topic => PresentationQosPolicyAccessScopeKind::Topic,
        }
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct PresentationQosPolicy(dust_dds::infrastructure::qos_policy::PresentationQosPolicy);

impl From<PresentationQosPolicy> for dust_dds::infrastructure::qos_policy::PresentationQosPolicy {
    fn from(value: PresentationQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::PresentationQosPolicy> for PresentationQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::PresentationQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl PresentationQosPolicy {
    #[new]
    pub fn new(
        access_scope: PresentationQosPolicyAccessScopeKind,
        coherent_access: bool,
        ordered_access: bool,
    ) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::PresentationQosPolicy {
                access_scope: access_scope.into(),
                coherent_access,
                ordered_access,
            },
        )
    }

    pub fn get_access_scope(&self) -> PresentationQosPolicyAccessScopeKind {
        self.0.access_scope.into()
    }

    pub fn set_access_scope(&mut self, value: PresentationQosPolicyAccessScopeKind) {
        self.0.access_scope = value.into();
    }

    pub fn get_coherent_access(&self) -> bool {
        self.0.coherent_access
    }

    pub fn set_coherent_access(&mut self, value: bool) {
        self.0.coherent_access = value;
    }

    pub fn get_ordered_access(&self) -> bool {
        self.0.ordered_access
    }

    pub fn set_ordered_access(&mut self, value: bool) {
        self.0.ordered_access = value;
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct WriterDataLifecycleQosPolicy(
    dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy,
);

impl From<WriterDataLifecycleQosPolicy>
    for dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy
{
    fn from(value: WriterDataLifecycleQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy>
    for WriterDataLifecycleQosPolicy
{
    fn from(value: dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl WriterDataLifecycleQosPolicy {
    #[new]
    pub fn new(autodispose_unregistered_instances: bool) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::WriterDataLifecycleQosPolicy {
                autodispose_unregistered_instances,
            },
        )
    }

    pub fn get_autodispose_unregistered_instances(&self) -> bool {
        self.0.autodispose_unregistered_instances
    }

    pub fn set_autodispose_unregistered_instances(&mut self, value: bool) {
        self.0.autodispose_unregistered_instances = value;
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct TimeBasedFilterQosPolicy(dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy);

impl From<TimeBasedFilterQosPolicy>
    for dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy
{
    fn from(value: TimeBasedFilterQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy>
    for TimeBasedFilterQosPolicy
{
    fn from(value: dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl TimeBasedFilterQosPolicy {
    #[new]
    pub fn new(minimum_separation: DurationKind) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::TimeBasedFilterQosPolicy {
                minimum_separation: minimum_separation.into(),
            },
        )
    }

    pub fn get_minimum_separation(&self) -> DurationKind {
        self.0.minimum_separation.into()
    }

    pub fn set_minimum_separation(&mut self, value: DurationKind) {
        self.0.minimum_separation = value.into();
    }
}

#[pyclass]
#[derive(Clone, Default)]
pub struct ReaderDataLifecycleQosPolicy(
    dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy,
);

impl From<ReaderDataLifecycleQosPolicy>
    for dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy
{
    fn from(value: ReaderDataLifecycleQosPolicy) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy>
    for ReaderDataLifecycleQosPolicy
{
    fn from(value: dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy) -> Self {
        Self(value)
    }
}

#[pymethods]
impl ReaderDataLifecycleQosPolicy {
    #[new]
    pub fn new(
        autopurge_nowriter_samples_delay: DurationKind,
        autopurge_disposed_samples_delay: DurationKind,
    ) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::ReaderDataLifecycleQosPolicy {
                autopurge_nowriter_samples_delay: autopurge_nowriter_samples_delay.into(),
                autopurge_disposed_samples_delay: autopurge_disposed_samples_delay.into(),
            },
        )
    }

    pub fn get_autopurge_nowriter_samples_delay(&self) -> DurationKind {
        self.0.autopurge_nowriter_samples_delay.into()
    }

    pub fn set_autopurge_nowriter_samples_delay(&mut self, value: DurationKind) {
        self.0.autopurge_nowriter_samples_delay = value.into();
    }

    pub fn get_autopurge_disposed_samples_delay(&self) -> DurationKind {
        self.0.autopurge_disposed_samples_delay.into()
    }

    pub fn set_autopurge_disposed_samples_delay(&mut self, value: DurationKind) {
        self.0.autopurge_disposed_samples_delay = value.into();
    }
}

// default for Reliability is differnet for reader and writer, hence
// added here as constants
const DEFAULT_MAX_BLOCKING_TIME: dust_dds::infrastructure::time::Duration =
    dust_dds::infrastructure::time::Duration::new(0, 100_000_000);
pub const DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS: ReliabilityQosPolicy =
    ReliabilityQosPolicy(dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy {
        kind: dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffort,
        max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
            DEFAULT_MAX_BLOCKING_TIME,
        ),
    });
pub const DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER: ReliabilityQosPolicy =
    ReliabilityQosPolicy(dust_dds::infrastructure::qos_policy::ReliabilityQosPolicy {
        kind: dust_dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::Reliable,
        max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
            DEFAULT_MAX_BLOCKING_TIME,
        ),
    });
