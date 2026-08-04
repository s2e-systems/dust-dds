use crate::infrastructure::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, DurabilityServiceQosPolicy,
    EntityFactoryQosPolicy, GroupDataQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy,
    LifespanQosPolicy, LivelinessQosPolicy, OctetSeq, OwnershipQosPolicy,
    OwnershipStrengthQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
    ReaderDataLifecycleQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy, StringSeq,
    TimeBasedFilterQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
    WriterDataLifecycleQosPolicy, DDS_octet_seq_free, DDS_String_seq_free,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DomainParticipantFactoryQos {
    pub entity_factory: EntityFactoryQosPolicy,
}

impl From<DomainParticipantFactoryQos>
    for dust_dds::infrastructure::qos::DomainParticipantFactoryQos
{
    fn from(val: DomainParticipantFactoryQos) -> Self {
        Self {
            entity_factory: val.entity_factory.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::DomainParticipantFactoryQos>
    for DomainParticipantFactoryQos
{
    fn from(val: dust_dds::infrastructure::qos::DomainParticipantFactoryQos) -> Self {
        Self {
            entity_factory: val.entity_factory.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DomainParticipantQos {
    pub user_data: UserDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl From<DomainParticipantQos> for dust_dds::infrastructure::qos::DomainParticipantQos {
    fn from(val: DomainParticipantQos) -> Self {
        Self {
            user_data: val.user_data.into(),
            entity_factory: val.entity_factory.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::DomainParticipantQos> for DomainParticipantQos {
    fn from(val: dust_dds::infrastructure::qos::DomainParticipantQos) -> Self {
        Self {
            user_data: val.user_data.into(),
            entity_factory: val.entity_factory.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TopicQos {
    pub topic_data: TopicDataQosPolicy,
    pub durability: DurabilityQosPolicy,
    pub durability_service: DurabilityServiceQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub transport_priority: TransportPriorityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub ownership: OwnershipQosPolicy,
}

impl From<TopicQos> for dust_dds::infrastructure::qos::TopicQos {
    fn from(val: TopicQos) -> Self {
        Self {
            topic_data: val.topic_data.into(),
            durability: val.durability.into(),
            deadline: val.deadline.into(),
            latency_budget: val.latency_budget.into(),
            liveliness: val.liveliness.into(),
            reliability: val.reliability.into(),
            destination_order: val.destination_order.into(),
            history: val.history.into(),
            resource_limits: val.resource_limits.into(),
            transport_priority: val.transport_priority.into(),
            lifespan: val.lifespan.into(),
            ownership: val.ownership.into(),
            representation: Default::default(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::TopicQos> for TopicQos {
    fn from(val: dust_dds::infrastructure::qos::TopicQos) -> Self {
        Self {
            topic_data: val.topic_data.into(),
            durability: val.durability.into(),
            durability_service: Default::default(),
            deadline: val.deadline.into(),
            latency_budget: val.latency_budget.into(),
            liveliness: val.liveliness.into(),
            reliability: val.reliability.into(),
            destination_order: val.destination_order.into(),
            history: val.history.into(),
            resource_limits: val.resource_limits.into(),
            transport_priority: val.transport_priority.into(),
            lifespan: val.lifespan.into(),
            ownership: val.ownership.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DataWriterQos {
    pub durability: DurabilityQosPolicy,
    pub durability_service: DurabilityServiceQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub transport_priority: TransportPriorityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub ownership_strength: OwnershipStrengthQosPolicy,
    pub writer_data_lifecycle: WriterDataLifecycleQosPolicy,
}

impl From<DataWriterQos> for dust_dds::infrastructure::qos::DataWriterQos {
    fn from(val: DataWriterQos) -> Self {
        Self {
            durability: val.durability.into(),
            deadline: val.deadline.into(),
            latency_budget: val.latency_budget.into(),
            liveliness: val.liveliness.into(),
            reliability: val.reliability.into(),
            destination_order: val.destination_order.into(),
            history: val.history.into(),
            resource_limits: val.resource_limits.into(),
            transport_priority: val.transport_priority.into(),
            lifespan: val.lifespan.into(),
            user_data: val.user_data.into(),
            ownership: val.ownership.into(),
            ownership_strength: val.ownership_strength.into(),
            writer_data_lifecycle: val.writer_data_lifecycle.into(),
            representation: Default::default(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::DataWriterQos> for DataWriterQos {
    fn from(val: dust_dds::infrastructure::qos::DataWriterQos) -> Self {
        Self {
            durability: val.durability.into(),
            durability_service: Default::default(),
            deadline: val.deadline.into(),
            latency_budget: val.latency_budget.into(),
            liveliness: val.liveliness.into(),
            reliability: val.reliability.into(),
            destination_order: val.destination_order.into(),
            history: val.history.into(),
            resource_limits: val.resource_limits.into(),
            transport_priority: val.transport_priority.into(),
            lifespan: val.lifespan.into(),
            user_data: val.user_data.into(),
            ownership: val.ownership.into(),
            ownership_strength: val.ownership_strength.into(),
            writer_data_lifecycle: val.writer_data_lifecycle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PublisherQos {
    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub group_data: GroupDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl From<PublisherQos> for dust_dds::infrastructure::qos::PublisherQos {
    fn from(val: PublisherQos) -> Self {
        Self {
            presentation: val.presentation.into(),
            partition: val.partition.into(),
            group_data: val.group_data.into(),
            entity_factory: val.entity_factory.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::PublisherQos> for PublisherQos {
    fn from(val: dust_dds::infrastructure::qos::PublisherQos) -> Self {
        Self {
            presentation: val.presentation.into(),
            partition: val.partition.into(),
            group_data: val.group_data.into(),
            entity_factory: val.entity_factory.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DataReaderQos {
    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub time_based_filter: TimeBasedFilterQosPolicy,
    pub reader_data_lifecycle: ReaderDataLifecycleQosPolicy,
}

impl From<DataReaderQos> for dust_dds::infrastructure::qos::DataReaderQos {
    fn from(val: DataReaderQos) -> Self {
        Self {
            durability: val.durability.into(),
            deadline: val.deadline.into(),
            latency_budget: val.latency_budget.into(),
            liveliness: val.liveliness.into(),
            reliability: val.reliability.into(),
            destination_order: val.destination_order.into(),
            history: val.history.into(),
            resource_limits: val.resource_limits.into(),
            user_data: val.user_data.into(),
            ownership: val.ownership.into(),
            time_based_filter: val.time_based_filter.into(),
            reader_data_lifecycle: val.reader_data_lifecycle.into(),
            representation: Default::default(),
            type_consistency: Default::default(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::DataReaderQos> for DataReaderQos {
    fn from(val: dust_dds::infrastructure::qos::DataReaderQos) -> Self {
        Self {
            durability: val.durability.into(),
            deadline: val.deadline.into(),
            latency_budget: val.latency_budget.into(),
            liveliness: val.liveliness.into(),
            reliability: val.reliability.into(),
            destination_order: val.destination_order.into(),
            history: val.history.into(),
            resource_limits: val.resource_limits.into(),
            user_data: val.user_data.into(),
            ownership: val.ownership.into(),
            time_based_filter: val.time_based_filter.into(),
            reader_data_lifecycle: val.reader_data_lifecycle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubscriberQos {
    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub group_data: GroupDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl From<SubscriberQos> for dust_dds::infrastructure::qos::SubscriberQos {
    fn from(val: SubscriberQos) -> Self {
        Self {
            presentation: val.presentation.into(),
            partition: val.partition.into(),
            group_data: val.group_data.into(),
            entity_factory: val.entity_factory.into(),
        }
    }
}
impl From<dust_dds::infrastructure::qos::SubscriberQos> for SubscriberQos {
    fn from(val: dust_dds::infrastructure::qos::SubscriberQos) -> Self {
        Self {
            presentation: val.presentation.into(),
            partition: val.partition.into(),
            group_data: val.group_data.into(),
            entity_factory: val.entity_factory.into(),
        }
    }
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_qos_default() -> DomainParticipantFactoryQos
{
    DomainParticipantFactoryQos::default()
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipant_qos_default() -> DomainParticipantQos {
    DomainParticipantQos::default()
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_qos_default() -> PublisherQos {
    PublisherQos::default()
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Subscriber_qos_default() -> SubscriberQos {
    SubscriberQos::default()
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Topic_qos_default() -> TopicQos {
    TopicQos::default()
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataWriter_qos_default() -> DataWriterQos {
    DataWriterQos::default()
}
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_qos_default() -> DataReaderQos {
    DataReaderQos::default()
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `_qos` must be a valid pointer to a `DomainParticipantFactoryQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_qos_cleanup(
    _qos: *mut DomainParticipantFactoryQos,
) {
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `qos` must be a valid pointer to a `DomainParticipantQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipant_qos_cleanup(qos: *mut DomainParticipantQos) {
    if !qos.is_null() {
        let q = unsafe { &mut *qos };
        unsafe { DDS_octet_seq_free(q.user_data.value) };
        q.user_data.value = OctetSeq::default();
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `qos` must be a valid pointer to a `PublisherQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_qos_cleanup(qos: *mut PublisherQos) {
    if !qos.is_null() {
        let q = unsafe { &mut *qos };
        unsafe {
            DDS_String_seq_free(q.partition.name);
            DDS_octet_seq_free(q.group_data.value);
        }
        q.partition.name = StringSeq::default();
        q.group_data.value = OctetSeq::default();
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `qos` must be a valid pointer to a `SubscriberQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Subscriber_qos_cleanup(qos: *mut SubscriberQos) {
    if !qos.is_null() {
        let q = unsafe { &mut *qos };
        unsafe {
            DDS_String_seq_free(q.partition.name);
            DDS_octet_seq_free(q.group_data.value);
        }
        q.partition.name = StringSeq::default();
        q.group_data.value = OctetSeq::default();
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `qos` must be a valid pointer to a `TopicQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Topic_qos_cleanup(qos: *mut TopicQos) {
    if !qos.is_null() {
        let q = unsafe { &mut *qos };
        unsafe { DDS_octet_seq_free(q.topic_data.value) };
        q.topic_data.value = OctetSeq::default();
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `qos` must be a valid pointer to a `DataWriterQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataWriter_qos_cleanup(qos: *mut DataWriterQos) {
    if !qos.is_null() {
        let q = unsafe { &mut *qos };
        unsafe { DDS_octet_seq_free(q.user_data.value) };
        q.user_data.value = OctetSeq::default();
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `qos` must be a valid pointer to a `DataReaderQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_qos_cleanup(qos: *mut DataReaderQos) {
    if !qos.is_null() {
        let q = unsafe { &mut *qos };
        unsafe { DDS_octet_seq_free(q.user_data.value) };
        q.user_data.value = OctetSeq::default();
    }
}
