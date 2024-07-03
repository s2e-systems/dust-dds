use pyo3::prelude::*;

use super::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, EntityFactoryQosPolicy,
    GroupDataQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
    LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
    ReaderDataLifecycleQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy,
    TimeBasedFilterQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
    WriterDataLifecycleQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
};

#[pyclass]
#[derive(Clone)]
pub struct DomainParticipantFactoryQos(dust_dds::infrastructure::qos::DomainParticipantFactoryQos);

impl From<DomainParticipantFactoryQos>
    for dust_dds::infrastructure::qos::DomainParticipantFactoryQos
{
    fn from(value: DomainParticipantFactoryQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::DomainParticipantFactoryQos>
    for DomainParticipantFactoryQos
{
    fn from(value: dust_dds::infrastructure::qos::DomainParticipantFactoryQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DomainParticipantFactoryQos {
    #[new]
    #[pyo3(signature = (entity_factory = EntityFactoryQosPolicy::default(), ))]
    pub fn new(entity_factory: EntityFactoryQosPolicy) -> Self {
        Self(dust_dds::infrastructure::qos::DomainParticipantFactoryQos {
            entity_factory: entity_factory.into(),
        })
    }

    fn get_entity_factory(&self) -> EntityFactoryQosPolicy {
        self.0.entity_factory.clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DomainParticipantQos(dust_dds::infrastructure::qos::DomainParticipantQos);

impl From<DomainParticipantQos> for dust_dds::infrastructure::qos::DomainParticipantQos {
    fn from(value: DomainParticipantQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::DomainParticipantQos> for DomainParticipantQos {
    fn from(value: dust_dds::infrastructure::qos::DomainParticipantQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DomainParticipantQos {
    #[new]
    #[pyo3(signature = (user_data= UserDataQosPolicy::default(), entity_factory = EntityFactoryQosPolicy::default()))]
    pub fn new(user_data: UserDataQosPolicy, entity_factory: EntityFactoryQosPolicy) -> Self {
        Self(dust_dds::infrastructure::qos::DomainParticipantQos {
            user_data: user_data.clone().into(),
            entity_factory: entity_factory.into(),
        })
    }

    fn get_user_data(&self) -> UserDataQosPolicy {
        self.0.user_data.clone().into()
    }

    fn get_entity_factory(&self) -> EntityFactoryQosPolicy {
        self.0.entity_factory.clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PublisherQos(dust_dds::infrastructure::qos::PublisherQos);

impl From<PublisherQos> for dust_dds::infrastructure::qos::PublisherQos {
    fn from(value: PublisherQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::PublisherQos> for PublisherQos {
    fn from(value: dust_dds::infrastructure::qos::PublisherQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl PublisherQos {
    #[new]
    #[pyo3(signature = (
        presentation = PresentationQosPolicy::default(),
        partition = PartitionQosPolicy::default(),
        group_data = GroupDataQosPolicy::default(),
        entity_factory = EntityFactoryQosPolicy::default(),
    ))]
    pub fn new(
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        group_data: GroupDataQosPolicy,
        entity_factory: EntityFactoryQosPolicy,
    ) -> Self {
        Self(dust_dds::infrastructure::qos::PublisherQos {
            presentation: presentation.into(),
            partition: partition.into(),
            group_data: group_data.into(),
            entity_factory: entity_factory.into(),
        })
    }

    pub fn get_presentation(&self) -> PresentationQosPolicy {
        self.0.presentation.clone().into()
    }

    pub fn set_presentation(&mut self, value: PresentationQosPolicy) {
        self.0.presentation = value.into()
    }

    pub fn get_partition(&self) -> PartitionQosPolicy {
        self.0.partition.clone().into()
    }

    pub fn set_partition(&mut self, value: PartitionQosPolicy) {
        self.0.partition = value.into()
    }

    pub fn get_group_data(&self) -> GroupDataQosPolicy {
        self.0.group_data.clone().into()
    }

    pub fn set_group_data(&mut self, value: GroupDataQosPolicy) {
        self.0.group_data = value.into()
    }

    pub fn get_entity_factory(&self) -> EntityFactoryQosPolicy {
        self.0.entity_factory.clone().into()
    }

    pub fn set_entity_factory(&mut self, value: EntityFactoryQosPolicy) {
        self.0.entity_factory = value.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SubscriberQos(dust_dds::infrastructure::qos::SubscriberQos);

impl From<SubscriberQos> for dust_dds::infrastructure::qos::SubscriberQos {
    fn from(value: SubscriberQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::SubscriberQos> for SubscriberQos {
    fn from(value: dust_dds::infrastructure::qos::SubscriberQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl SubscriberQos {
    #[new]
    #[pyo3(signature = (
        presentation = PresentationQosPolicy::default(),
        partition = PartitionQosPolicy::default(),
        group_data = GroupDataQosPolicy::default(),
        entity_factory = EntityFactoryQosPolicy::default(),
    ))]
    pub fn new(
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        group_data: GroupDataQosPolicy,
        entity_factory: EntityFactoryQosPolicy,
    ) -> Self {
        Self(dust_dds::infrastructure::qos::SubscriberQos {
            presentation: presentation.into(),
            partition: partition.into(),
            group_data: group_data.into(),
            entity_factory: entity_factory.into(),
        })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct TopicQos(dust_dds::infrastructure::qos::TopicQos);

impl From<TopicQos> for dust_dds::infrastructure::qos::TopicQos {
    fn from(value: TopicQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::TopicQos> for TopicQos {
    fn from(value: dust_dds::infrastructure::qos::TopicQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl TopicQos {
    #[new]
    #[pyo3(signature = (
        topic_data = TopicDataQosPolicy::default(),
        durability = DurabilityQosPolicy::default(),
        deadline = DeadlineQosPolicy::default(),
        latency_budget = LatencyBudgetQosPolicy::default(),
        liveliness = LivelinessQosPolicy::default(),
        reliability = DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        destination_order = DestinationOrderQosPolicy::default(),
        history = HistoryQosPolicy::default(),
        resource_limits = ResourceLimitsQosPolicy::default(),
        transport_priority = TransportPriorityQosPolicy::default(),
        lifespan = LifespanQosPolicy::default(),
        ownership = OwnershipQosPolicy::default(),
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        topic_data: TopicDataQosPolicy,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        history: HistoryQosPolicy,
        resource_limits: ResourceLimitsQosPolicy,
        transport_priority: TransportPriorityQosPolicy,
        lifespan: LifespanQosPolicy,
        ownership: OwnershipQosPolicy,
    ) -> Self {
        Self(dust_dds::infrastructure::qos::TopicQos {
            topic_data: topic_data.into(),
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: reliability.into(),
            destination_order: destination_order.into(),
            history: history.into(),
            resource_limits: resource_limits.into(),
            transport_priority: transport_priority.into(),
            lifespan: lifespan.into(),
            ownership: ownership.into(),
        })
    }

    fn get_topic_data(&self) -> TopicDataQosPolicy {
        self.0.topic_data.clone().into()
    }

    fn get_durability(&self) -> DurabilityQosPolicy {
        self.0.durability.clone().into()
    }

    fn get_deadline(&self) -> DeadlineQosPolicy {
        self.0.deadline.clone().into()
    }

    fn get_latency_budget(&self) -> LatencyBudgetQosPolicy {
        self.0.latency_budget.clone().into()
    }

    fn get_liveliness(&self) -> LivelinessQosPolicy {
        self.0.liveliness.clone().into()
    }

    fn get_reliability(&self) -> ReliabilityQosPolicy {
        self.0.reliability.clone().into()
    }

    fn get_destination_order(&self) -> DestinationOrderQosPolicy {
        self.0.destination_order.clone().into()
    }

    fn get_history(&self) -> HistoryQosPolicy {
        self.0.history.clone().into()
    }

    fn get_resource_limits(&self) -> ResourceLimitsQosPolicy {
        self.0.resource_limits.clone().into()
    }

    fn get_transport_priority(&self) -> TransportPriorityQosPolicy {
        self.0.transport_priority.clone().into()
    }

    fn get_lifespan(&self) -> LifespanQosPolicy {
        self.0.lifespan.clone().into()
    }

    fn get_ownership(&self) -> OwnershipQosPolicy {
        self.0.ownership.clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DataWriterQos(dust_dds::infrastructure::qos::DataWriterQos);

impl From<DataWriterQos> for dust_dds::infrastructure::qos::DataWriterQos {
    fn from(value: DataWriterQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::DataWriterQos> for DataWriterQos {
    fn from(value: dust_dds::infrastructure::qos::DataWriterQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DataWriterQos {
    #[new]
    #[pyo3(signature = (
        durability = DurabilityQosPolicy::default(),
        deadline = DeadlineQosPolicy::default(),
        latency_budget = LatencyBudgetQosPolicy::default(),
        liveliness = LivelinessQosPolicy::default(),
        reliability = DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        destination_order = DestinationOrderQosPolicy::default(),
        history = HistoryQosPolicy::default(),
        resource_limits = ResourceLimitsQosPolicy::default(),
        transport_priority = TransportPriorityQosPolicy::default(),
        lifespan = LifespanQosPolicy::default(),
        user_data = UserDataQosPolicy::default(),
        ownership = OwnershipQosPolicy::default(),
        writer_data_lifecycle = WriterDataLifecycleQosPolicy::default(),
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        history: HistoryQosPolicy,
        resource_limits: ResourceLimitsQosPolicy,
        transport_priority: TransportPriorityQosPolicy,
        lifespan: LifespanQosPolicy,
        user_data: UserDataQosPolicy,
        ownership: OwnershipQosPolicy,
        writer_data_lifecycle: WriterDataLifecycleQosPolicy,
    ) -> Self {
        Self(dust_dds::infrastructure::qos::DataWriterQos {
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: reliability.into(),
            destination_order: destination_order.into(),
            history: history.into(),
            resource_limits: resource_limits.into(),
            transport_priority: transport_priority.into(),
            lifespan: lifespan.into(),
            user_data: user_data.into(),
            ownership: ownership.into(),
            writer_data_lifecycle: writer_data_lifecycle.into(),
        })
    }

    fn get_durability(&self) -> DurabilityQosPolicy {
        self.0.durability.clone().into()
    }

    fn get_deadline(&self) -> DeadlineQosPolicy {
        self.0.deadline.clone().into()
    }

    fn get_latency_budget(&self) -> LatencyBudgetQosPolicy {
        self.0.latency_budget.clone().into()
    }

    fn get_liveliness(&self) -> LivelinessQosPolicy {
        self.0.liveliness.clone().into()
    }

    fn get_reliability(&self) -> ReliabilityQosPolicy {
        self.0.reliability.clone().into()
    }

    fn get_destination_order(&self) -> DestinationOrderQosPolicy {
        self.0.destination_order.clone().into()
    }

    fn get_history(&self) -> HistoryQosPolicy {
        self.0.history.clone().into()
    }

    fn get_resource_limits(&self) -> ResourceLimitsQosPolicy {
        self.0.resource_limits.clone().into()
    }

    fn get_transport_priority(&self) -> TransportPriorityQosPolicy {
        self.0.transport_priority.clone().into()
    }

    fn get_lifespan(&self) -> LifespanQosPolicy {
        self.0.lifespan.clone().into()
    }

    fn get_user_data(&self) -> UserDataQosPolicy {
        self.0.user_data.clone().into()
    }

    fn get_ownership(&self) -> OwnershipQosPolicy {
        self.0.ownership.clone().into()
    }

    fn get_writer_data_lifecycle(&self) -> WriterDataLifecycleQosPolicy {
        self.0.writer_data_lifecycle.clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DataReaderQos(dust_dds::infrastructure::qos::DataReaderQos);

impl From<DataReaderQos> for dust_dds::infrastructure::qos::DataReaderQos {
    fn from(value: DataReaderQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::DataReaderQos> for DataReaderQos {
    fn from(value: dust_dds::infrastructure::qos::DataReaderQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DataReaderQos {
    #[new]
    #[pyo3(signature = (
        durability = DurabilityQosPolicy::default(),
        deadline = DeadlineQosPolicy::default(),
        latency_budget = LatencyBudgetQosPolicy::default(),
        liveliness = LivelinessQosPolicy::default(),
        reliability = DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        destination_order = DestinationOrderQosPolicy::default(),
        history = HistoryQosPolicy::default(),
        resource_limits = ResourceLimitsQosPolicy::default(),
        user_data = UserDataQosPolicy::default(),
        ownership = OwnershipQosPolicy::default(),
        time_based_filter = TimeBasedFilterQosPolicy::default(),
        reader_data_lifecycle = ReaderDataLifecycleQosPolicy::default(),
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        history: HistoryQosPolicy,
        resource_limits: ResourceLimitsQosPolicy,
        user_data: UserDataQosPolicy,
        ownership: OwnershipQosPolicy,
        time_based_filter: TimeBasedFilterQosPolicy,
        reader_data_lifecycle: ReaderDataLifecycleQosPolicy,
    ) -> Self {
        Self(dust_dds::infrastructure::qos::DataReaderQos {
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: reliability.into(),
            destination_order: destination_order.into(),
            history: history.into(),
            resource_limits: resource_limits.into(),
            user_data: user_data.into(),
            ownership: ownership.into(),
            time_based_filter: time_based_filter.into(),
            reader_data_lifecycle: reader_data_lifecycle.into(),
        })
    }

    fn get_durability(&self) -> DurabilityQosPolicy {
        self.0.durability.clone().into()
    }

    fn get_deadline(&self) -> DeadlineQosPolicy {
        self.0.deadline.clone().into()
    }

    fn get_latency_budget(&self) -> LatencyBudgetQosPolicy {
        self.0.latency_budget.clone().into()
    }

    fn get_liveliness(&self) -> LivelinessQosPolicy {
        self.0.liveliness.clone().into()
    }

    fn get_reliability(&self) -> ReliabilityQosPolicy {
        self.0.reliability.clone().into()
    }

    fn get_destination_order(&self) -> DestinationOrderQosPolicy {
        self.0.destination_order.clone().into()
    }

    fn get_history(&self) -> HistoryQosPolicy {
        self.0.history.clone().into()
    }

    fn get_resource_limits(&self) -> ResourceLimitsQosPolicy {
        self.0.resource_limits.clone().into()
    }

    fn get_user_data(&self) -> UserDataQosPolicy {
        self.0.user_data.clone().into()
    }

    fn get_ownership(&self) -> OwnershipQosPolicy {
        self.0.ownership.clone().into()
    }

    fn get_time_based_filter(&self) -> TimeBasedFilterQosPolicy {
        self.0.time_based_filter.clone().into()
    }

    fn get_reader_data_lifecycle(&self) -> ReaderDataLifecycleQosPolicy {
        self.0.reader_data_lifecycle.clone().into()
    }
}
