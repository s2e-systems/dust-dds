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

    #[getter]
    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }

    #[setter]
    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value.into()
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

    #[getter]
    pub fn get_autoenable_created_entities(&self) -> bool {
        self.0.autoenable_created_entities
    }

    #[setter]
    pub fn set_autoenable_created_entities(&mut self, value: bool) {
        self.0.autoenable_created_entities = value.into()
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

    #[getter]
    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }

    #[setter]
    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value.into()
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

    #[getter]
    pub fn get_kind(&self) -> DurabilityQosPolicyKind {
        self.0.kind.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_period(&self) -> DurationKind {
        self.0.period.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_duration(&self) -> DurationKind {
        self.0.duration.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_kind(&self) -> LivelinessQosPolicyKind {
        self.0.kind.into()
    }

    #[setter]
    pub fn set_kind(&mut self, value: LivelinessQosPolicyKind) {
        self.0.kind = value.into()
    }

    #[getter]
    pub fn get_lease_duration(&self) -> DurationKind {
        self.0.lease_duration.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_kind(&self) -> ReliabilityQosPolicyKind {
        self.0.kind.into()
    }

    #[setter]
    pub fn set_kind(&mut self, value: ReliabilityQosPolicyKind) {
        self.0.kind = value.into()
    }

    #[getter]
    pub fn get_max_blocking_time(&self) -> DurationKind {
        self.0.max_blocking_time.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_kind(&self) -> DestinationOrderQosPolicyKind {
        self.0.kind.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_kind(&self) -> HistoryQosPolicyKind {
        self.0.kind.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_max_samples(&self) -> Length {
        self.0.max_samples.into()
    }

    #[setter]
    pub fn set_max_samples(&mut self, value: Length) {
        self.0.max_samples = value.into()
    }

    #[getter]
    pub fn get_max_instances(&self) -> Length {
        self.0.max_instances.into()
    }

    #[setter]
    pub fn set_max_instances(&mut self, value: Length) {
        self.0.max_instances = value.into()
    }

    #[getter]
    pub fn get_max_samples_per_instance(&self) -> Length {
        self.0.max_samples_per_instance.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_value(&self) -> i32 {
        self.0.value.into()
    }

    #[setter]
    pub fn set_value(&mut self, value: i32) {
        self.0.value = value.into()
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

    #[getter]
    pub fn get_duration(&self) -> DurationKind {
        self.0.duration.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_kind(&self) -> OwnershipQosPolicyKind {
        self.0.kind.into()
    }

    #[setter]
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

    #[getter]
    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }

    #[setter]
    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value;
    }
}
