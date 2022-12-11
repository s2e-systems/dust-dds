use core::cmp::Ordering;

use crate::infrastructure::time::{Duration, DURATION_INFINITE, DURATION_ZERO};

pub type QosPolicyId = i32;
#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Length {
    Unlimited,
    Limited(u32),
}

impl PartialOrd for Length {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Length::Unlimited => match other {
                Length::Unlimited => Some(Ordering::Equal),
                Length::Limited(_) => Some(Ordering::Greater),
            },
            Length::Limited(value) => match other {
                Length::Unlimited => Some(Ordering::Less),
                Length::Limited(other) => value.partial_cmp(other),
            },
        }
    }
}

impl PartialEq<usize> for Length {
    fn eq(&self, other: &usize) -> bool {
        match self {
            Length::Unlimited => false,
            Length::Limited(value) => (*value as usize).eq(other),
        }
    }
}

impl PartialOrd<usize> for Length {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        match self {
            Length::Unlimited => Some(Ordering::Greater),
            Length::Limited(value) => (*value as usize).partial_cmp(other),
        }
    }
}

impl PartialEq<Length> for usize {
    fn eq(&self, other: &Length) -> bool {
        match other {
            Length::Unlimited => false,
            Length::Limited(value) => self.eq(&(*value as usize)),
        }
    }
}

impl PartialOrd<Length> for usize {
    fn partial_cmp(&self, other: &Length) -> Option<Ordering> {
        match other {
            Length::Unlimited => Some(Ordering::Less),
            Length::Limited(value) => self.partial_cmp(&(*value as usize)),
        }
    }
}

/// This class is the abstract root for all the QoS policies.
/// It provides the basic mechanism for an application to specify quality of service parameters. It has an attribute name that is used
/// to identify uniquely each QoS policy. All concrete QosPolicy classes derive from this root and include a value whose type
/// depends on the concrete QoS policy.
/// The type of a QosPolicy value may be atomic, such as an integer or float, or compound (a structure). Compound types are used
/// whenever multiple parameters must be set coherently to define a consistent value for a QosPolicy.
/// Each Entity can be configured with a list of QosPolicy. However, any Entity cannot support any QosPolicy. For instance, a
/// DomainParticipant supports different QosPolicy than a Topic or a Publisher.
/// QosPolicy can be set when the Entity is created, or modified with the set_qos method. Each QosPolicy in the list is treated
/// independently from the others. This approach has the advantage of being very extensible. However, there may be cases where
/// several policies are in conflict. Consistency checking is performed each time the policies are modified via the set_qos
/// operation.
/// When a policy is changed after being set to a given value, it is not required that the new value be applied instantaneously; the
/// Service is allowed to apply it after a transition phase. In addition, some QosPolicy have “immutable” semantics meaning that
/// they can only be specified either at Entity creation time or else prior to calling the enable operation on the Entity.
/// Sub clause 2.2.3, Supported QoS provides the list of all QosPolicy, their meaning, characteristics and possible values, as well
/// as the concrete Entity to which they apply.
pub trait QosPolicy {
    fn name(&self) -> &str;
}

const USERDATA_QOS_POLICY_NAME: &str = "UserData";
const DURABILITY_QOS_POLICY_NAME: &str = "Durability";
const PRESENTATION_QOS_POLICY_NAME: &str = "Presentation";
const DEADLINE_QOS_POLICY_NAME: &str = "Deadline";
const LATENCYBUDGET_QOS_POLICY_NAME: &str = "LatencyBudget";
const OWNERSHIP_QOS_POLICY_NAME: &str = "Ownership";
const LIVELINESS_QOS_POLICY_NAME: &str = "Liveliness";
const TIMEBASEDFILTER_QOS_POLICY_NAME: &str = "TimeBasedFilter";
const PARTITION_QOS_POLICY_NAME: &str = "Partition";
const RELIABILITY_QOS_POLICY_NAME: &str = "Reliability";
const DESTINATIONORDER_QOS_POLICY_NAME: &str = "DestinationOrder";
const HISTORY_QOS_POLICY_NAME: &str = "History";
const RESOURCELIMITS_QOS_POLICY_NAME: &str = "ResourceLimits";
const ENTITYFACTORY_QOS_POLICY_NAME: &str = "EntityFactory";
const WRITERDATALIFECYCLE_QOS_POLICY_NAME: &str = "WriterDataLifecycle";
const READERDATALIFECYCLE_QOS_POLICY_NAME: &str = "ReaderDataLifecycle";
const TOPICDATA_QOS_POLICY_NAME: &str = "TopicData";
const TRANSPORTPRIORITY_QOS_POLICY_NAME: &str = "TransportPriority";
const GROUPDATA_QOS_POLICY_NAME: &str = "GroupData";
const LIFESPAN_QOS_POLICY_NAME: &str = "Lifespan";

pub const INVALID_QOS_POLICY_ID: QosPolicyId = 0;
pub const USERDATA_QOS_POLICY_ID: QosPolicyId = 1;
pub const DURABILITY_QOS_POLICY_ID: QosPolicyId = 2;
pub const PRESENTATION_QOS_POLICY_ID: QosPolicyId = 3;
pub const DEADLINE_QOS_POLICY_ID: QosPolicyId = 4;
pub const LATENCYBUDGET_QOS_POLICY_ID: QosPolicyId = 5;
pub const OWNERSHIP_QOS_POLICY_ID: QosPolicyId = 6;
pub const LIVELINESS_QOS_POLICY_ID: QosPolicyId = 8;
pub const TIMEBASEDFILTER_QOS_POLICY_ID: QosPolicyId = 9;
pub const PARTITION_QOS_POLICY_ID: QosPolicyId = 10;
pub const RELIABILITY_QOS_POLICY_ID: QosPolicyId = 11;
pub const DESTINATIONORDER_QOS_POLICY_ID: QosPolicyId = 12;
pub const HISTORY_QOS_POLICY_ID: QosPolicyId = 13;
pub const RESOURCELIMITS_QOS_POLICY_ID: QosPolicyId = 14;
pub const ENTITYFACTORY_QOS_POLICY_ID: QosPolicyId = 15;
pub const WRITERDATALIFECYCLE_QOS_POLICY_ID: QosPolicyId = 16;
pub const READERDATALIFECYCLE_QOS_POLICY_ID: QosPolicyId = 17;
pub const TOPICDATA_QOS_POLICY_ID: QosPolicyId = 18;
pub const GROUPDATA_QOS_POLICY_ID: QosPolicyId = 19;
pub const TRANSPORTPRIORITY_QOS_POLICY_ID: QosPolicyId = 20;
pub const LIFESPAN_QOS_POLICY_ID: QosPolicyId = 21;
pub const DURABILITYSERVICE_QOS_POLICY_ID: QosPolicyId = 22;

/// This policy allows the application to attach additional information to the created Entity objects such that when
/// a remote application discovers their existence it can access that information and use it for its own purposes.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserDataQosPolicy {
    pub value: Vec<u8>,
}

impl QosPolicy for UserDataQosPolicy {
    fn name(&self) -> &str {
        USERDATA_QOS_POLICY_NAME
    }
}

/// This policy allows the application to attach additional information to the created Topic such that when a
/// remote application discovers their existence it can examine the information and use it in an application-defined way.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicDataQosPolicy {
    pub value: Vec<u8>,
}

impl QosPolicy for TopicDataQosPolicy {
    fn name(&self) -> &str {
        TOPICDATA_QOS_POLICY_NAME
    }
}

/// This policy allows the application to attach additional information to the created
/// [`Publisher`](crate::publication::publisher::Publisher) or [`Subscriber`](crate::subscription::subscriber::Subscriber).
///
/// The value is available to the application on the
/// [`DataReader`](crate::subscription::data_reader::DataReader) and [`DataWriter`](crate::publication::data_writer::DataWriter) entities and is propagated by
/// means of the built-in topics.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupDataQosPolicy {
    pub value: Vec<u8>,
}

impl QosPolicy for GroupDataQosPolicy {
    fn name(&self) -> &str {
        GROUPDATA_QOS_POLICY_NAME
    }
}

/// This policy allows the application to take advantage of transports capable of sending messages with different priorities.
///
/// This policy is considered a hint. The policy depends on the ability of the underlying transports to set a priority on the messages
/// they send. Any value within the range of a 32-bit signed integer may be chosen; higher values indicate higher priority.
/// However, any further interpretation of this policy is specific to a particular transport and a particular implementation of the
/// Service. For example, a particular transport is permitted to treat a range of priority values as equivalent to one another. It is
/// expected that during transport configuration the application would provide a mapping between the values of the
/// [`TransportPriorityQosPolicy`] set on [`DataWriter`](crate::publication::data_writer::DataWriter) and the values meaningful to each transport.
/// This mapping would then be used by the infrastructure when propagating the data written by the [`DataWriter`](crate::publication::data_writer::DataWriter).
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportPriorityQosPolicy {
    pub value: i32,
}

impl QosPolicy for TransportPriorityQosPolicy {
    fn name(&self) -> &str {
        TRANSPORTPRIORITY_QOS_POLICY_NAME
    }
}

/// This policy is used to avoid delivering “stale” data to the application.
///
/// Each data sample written by the [`DataWriter`](crate::publication::data_writer::DataWriter) has an associated ‘expiration time’ beyond which the data should not be delivered
/// to any application. Once the sample expires, the data will be removed from the [`DataReader`](crate::subscription::data_reader::DataReader) caches as well as from the
/// transient and persistent information caches.
/// The ‘expiration time’ of each sample is computed by adding the duration specified by the [`LifespanQosPolicy`] to the source
/// timestamp. The source timestamp is either automatically computed by the Service
/// each time the [`DataWriter::write()`](crate::publication::data_writer::DataWriter) operation is called, or else supplied by the application by means
/// of the  [`DataWriter::write_w_timestamp()`](crate::publication::data_writer::DataWriter)
/// operation.
/// This QoS relies on the sender and receiving applications having their clocks sufficiently synchronized. If this is not the case
/// and the Service can detect it, the [`DataReader`](crate::subscription::data_reader::DataReader) is allowed to use the reception timestamp instead of the source timestamp in its
/// computation of the ‘expiration time.’
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LifespanQosPolicy {
    pub duration: Duration,
}

impl QosPolicy for LifespanQosPolicy {
    fn name(&self) -> &str {
        LIFESPAN_QOS_POLICY_NAME
    }
}

impl Default for LifespanQosPolicy {
    fn default() -> Self {
        Self {
            duration: DURATION_INFINITE,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum DurabilityQosPolicyKind {
    Volatile,
    TransientLocal,
}

impl PartialOrd for DurabilityQosPolicyKind {
    fn partial_cmp(&self, other: &DurabilityQosPolicyKind) -> Option<Ordering> {
        match self {
            DurabilityQosPolicyKind::Volatile => match other {
                DurabilityQosPolicyKind::Volatile => Some(Ordering::Equal),
                DurabilityQosPolicyKind::TransientLocal => Some(Ordering::Less),
            },
            DurabilityQosPolicyKind::TransientLocal => match other {
                DurabilityQosPolicyKind::Volatile => Some(Ordering::Greater),
                DurabilityQosPolicyKind::TransientLocal => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy controls whether the Service will actually make data available to late-joining readers.
///
/// The decoupling between [`DataReader`](crate::subscription::data_reader::DataReader) and [`DataWriter`](crate::publication::data_writer::DataWriter)
/// offered by the Publish/Subscribe paradigm allows an application to write data even if there are
/// no current readers on the network. Moreover, a [`DataReader`](crate::subscription::data_reader::DataReader) that joins the network after some data
/// has been written could potentially be interested in accessing the most current values of the data as well as potentially some
/// history.
/// Note that although related, this does not strictly control what data the Service will maintain internally.
/// That is, the Service may choose to maintain some data for its own purposes (e.g., flow control)
/// and yet not make it available to late-joining readers if the [`DurabilityQosPolicy`] is set to [`DurabilityQosPolicyKind::Volatile`].
/// The value offered is considered compatible with the value requested if and only if the *offered kind >= requested
/// kind* is true. For the purposes of this inequality, the values of [`DurabilityQosPolicyKind`] kind are considered ordered such
/// that *Volatile < TransientLocal*.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
pub struct DurabilityQosPolicy {
    pub kind: DurabilityQosPolicyKind,
}

impl QosPolicy for DurabilityQosPolicy {
    fn name(&self) -> &str {
        DURABILITY_QOS_POLICY_NAME
    }
}

impl Default for DurabilityQosPolicy {
    fn default() -> Self {
        Self {
            kind: DurabilityQosPolicyKind::Volatile,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum PresentationQosPolicyAccessScopeKind {
    Instance,
    Topic,
}

impl PartialOrd for PresentationQosPolicyAccessScopeKind {
    fn partial_cmp(&self, other: &PresentationQosPolicyAccessScopeKind) -> Option<Ordering> {
        match self {
            PresentationQosPolicyAccessScopeKind::Instance => match other {
                PresentationQosPolicyAccessScopeKind::Instance => Some(Ordering::Equal),
                PresentationQosPolicyAccessScopeKind::Topic => Some(Ordering::Less),
            },
            PresentationQosPolicyAccessScopeKind::Topic => match other {
                PresentationQosPolicyAccessScopeKind::Instance => Some(Ordering::Greater),
                PresentationQosPolicyAccessScopeKind::Topic => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy controls the extent to which changes to data-instances can be made dependent on each other and also the kind
/// of dependencies that can be propagated and maintained by the Service.
///
/// The setting of [`PresentationQosPolicy::coherent_access`] controls whether the Service will preserve the groupings of changes made by the publishing
/// application by means of the operations `begin_coherent_change()` and `end_coherent_change()`.
/// The setting of  [`PresentationQosPolicy::ordered_access`] controls whether the Service will preserve the order of changes.
/// The granularity is controlled by the setting of the  [`PresentationQosPolicy::access_scope`].
/// If [`PresentationQosPolicy::coherent_access`] is set, then the [`PresentationQosPolicy::access_scope`] controls the maximum extent of coherent changes.
/// The behavior is as follows:
/// - If access_scope is set to INSTANCE, the use of begin_coherent_change and end_coherent_change has no effect on
/// how the subscriber can access the data because with the scope limited to each instance, changes to separate instances
/// are considered independent and thus cannot be grouped by a coherent change.
/// - If access_scope is set to TOPIC, then coherent changes (indicated by their enclosure within calls to
/// begin_coherent_change and end_coherent_change) will be made available as such to each remote DataReader
/// independently. That is, changes made to instances within each individual DataWriter will be available as coherent with
/// respect to other changes to instances in that same DataWriter, but will not be grouped with changes made to instances
/// belonging to a different DataWriter.
/// If ordered_access is set, then the access_scope controls the maximum extent for which order will be preserved by the Service.
/// - If access_scope is set to INSTANCE (the lowest level), then changes to each instance are considered unordered relative
/// to changes to any other instance. That means that changes (creations, deletions, modifications) made to two instances
/// are not necessarily seen in the order they occur. This is the case even if it is the same application thread making the
/// changes using the same DataWriter.
/// - If access_scope is set to TOPIC, changes (creations, deletions, modifications) made by a single DataWriter are made
/// available to subscribers in the same order they occur. Changes made to instances through different DataWriter entities
/// are not necessarily seen in the order they occur. This is the case, even if the changes are made by a single application
/// thread using DataWriter objects attached to the same Publisher.
/// Note that this QoS policy controls the scope at which related changes are made available to the subscriber. This means the
/// subscriber can access the changes in a coherent manner and in the proper order; however, it does not necessarily imply that the
/// Subscriber will indeed access the changes in the correct order. For that to occur, the application at the subscriber end must use
/// the proper logic in reading the DataReader objects.
/// The value offered is considered compatible with the value requested if and only if the following conditions are met:
/// 1. The inequality “offered access_scope >= requested access_scope” is true. For the purposes of this
/// inequality, the values of PRESENTATION access_scope are considered ordered such that INSTANCE < TOPIC <
/// GROUP.
/// 2. Requested coherent_access is FALSE, or else both offered and requested coherent_access are TRUE.
/// 3. Requested ordered_access is FALSE, or else both offered and requested ordered _access are TRUE.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct PresentationQosPolicy {
    pub access_scope: PresentationQosPolicyAccessScopeKind,
    pub coherent_access: bool,
    pub ordered_access: bool,
}

impl QosPolicy for PresentationQosPolicy {
    fn name(&self) -> &str {
        PRESENTATION_QOS_POLICY_NAME
    }
}

impl Default for PresentationQosPolicy {
    fn default() -> Self {
        Self {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: false,
            ordered_access: false,
        }
    }
}

/// This policy is useful for cases where a [`Topic`](crate::topic_definition::topic::Topic) is expected to have each instance updated periodically. On the publishing side this
/// setting establishes a contract that the application must meet.
///
/// On the subscribing side the setting establishes a minimum
/// requirement for the remote publishers that are expected to supply the data values.
/// When the Service ‘matches’ a [`DataWriter`](crate::publication::data_writer::DataWriter) and a [`DataReader`](crate::subscription::data_reader::DataReader) it checks whether the settings are compatible (i.e., *offered
/// deadline period <= requested deadline period*) if they are not, the two entities are informed (via the listener or condition
/// mechanism) of the incompatibility of the QoS settings and communication will not occur.
/// Assuming that the reader and writer ends have compatible settings, the fulfillment of this contract is monitored by the Service
/// and the application is informed of any violations by means of the proper listener or condition.
/// The value offered is considered compatible with the value requested if and only if the *offered deadline period <=
/// requested deadline period* is true.
/// The setting of the [`DeadlineQosPolicy`] policy must be set consistently with that of the [`TimeBasedFilterQosPolicy`]. For these two policies
/// to be consistent the settings must be such that *deadline period >= minimum_separation*.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeadlineQosPolicy {
    pub period: Duration,
}

impl QosPolicy for DeadlineQosPolicy {
    fn name(&self) -> &str {
        DEADLINE_QOS_POLICY_NAME
    }
}

impl Default for DeadlineQosPolicy {
    fn default() -> Self {
        Self {
            period: DURATION_INFINITE,
        }
    }
}

/// This policy provides a means for the application to indicate to the middleware the “urgency” of the data-communication.
///
/// By having a non-zero duration the Service can optimize its internal operation.
/// This policy is considered a hint. There is no specified mechanism as to how the service should take advantage of this hint.
/// The value offered is considered compatible with the value requested if and only if the *offered duration <=
/// requested duration* is true.
#[derive(PartialOrd, PartialEq, Eq, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LatencyBudgetQosPolicy {
    pub duration: Duration,
}

impl QosPolicy for LatencyBudgetQosPolicy {
    fn name(&self) -> &str {
        LATENCYBUDGET_QOS_POLICY_NAME
    }
}

impl Default for LatencyBudgetQosPolicy {
    fn default() -> Self {
        Self {
            duration: DURATION_ZERO,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum OwnershipQosPolicyKind {
    Shared,
}

/// This policy controls whether the Service allows multiple [`DataWriter`](crate::publication::data_writer::DataWriter)
/// objects to update the same instance (identified by Topic + key) of a data-object.
///
/// Only [`OwnershipQosPolicyKind::Shared`] can be selected. This setting indicates that the Service does not enforce unique ownership for each instance.
/// In this case, multiple writers can update the same data-object instance. The subscriber to the Topic will be able to access modifications from all DataWriter
/// objects, subject to the settings of other QoS that may filter particular samples (e.g., the [`TimeBasedFilterQosPolicy`] or [`HistoryQosPolicy`]).
/// In any case there is no “filtering” of modifications made based on the identity of the DataWriter that causes the
/// modification.

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct OwnershipQosPolicy {
    pub kind: OwnershipQosPolicyKind,
}

impl QosPolicy for OwnershipQosPolicy {
    fn name(&self) -> &str {
        OWNERSHIP_QOS_POLICY_NAME
    }
}

impl Default for OwnershipQosPolicy {
    fn default() -> Self {
        Self {
            kind: OwnershipQosPolicyKind::Shared,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum LivelinessQosPolicyKind {
    Automatic,
    ManualByParticipant,
    ManualByTopic,
}

impl PartialOrd for LivelinessQosPolicyKind {
    fn partial_cmp(&self, other: &LivelinessQosPolicyKind) -> Option<Ordering> {
        match self {
            LivelinessQosPolicyKind::Automatic => match other {
                LivelinessQosPolicyKind::Automatic => Some(Ordering::Equal),
                LivelinessQosPolicyKind::ManualByParticipant => Some(Ordering::Less),
                LivelinessQosPolicyKind::ManualByTopic => Some(Ordering::Less),
            },
            LivelinessQosPolicyKind::ManualByParticipant => match other {
                LivelinessQosPolicyKind::Automatic => Some(Ordering::Greater),
                LivelinessQosPolicyKind::ManualByParticipant => Some(Ordering::Equal),
                LivelinessQosPolicyKind::ManualByTopic => Some(Ordering::Less),
            },
            LivelinessQosPolicyKind::ManualByTopic => match other {
                LivelinessQosPolicyKind::Automatic => Some(Ordering::Greater),
                LivelinessQosPolicyKind::ManualByParticipant => Some(Ordering::Greater),
                LivelinessQosPolicyKind::ManualByTopic => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy controls the mechanism and parameters used by the Service to ensure that particular entities on the network are
/// still “alive”.
///
/// The liveliness can also affect the ownership of a particular instance, as determined by the [`OwnershipQosPolicy`].
/// This policy has several settings to support both data-objects that are updated periodically as well as those that are changed
/// sporadically. It also allows customizing for different application requirements in terms of the kinds of failures that will be
/// detected by the liveliness mechanism.
/// The [`LivelinessQosPolicyKind::Automatic`] liveliness setting is most appropriate for applications that only need to detect failures at the process level,
/// but not application-logic failures within a process. The Service takes responsibility for renewing the leases at the
/// required rates and thus, as long as the local process where a [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant) is running and the link connecting it to remote
/// participants remains connected, the entities within the [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant) will be considered alive. This requires the lowest
/// overhead.
/// The manual settings ([`LivelinessQosPolicyKind::ManualByParticipant`], [`LivelinessQosPolicyKind::ManualByTopic`]), require the application on the publishing
/// side to periodically assert the liveliness before the lease expires to indicate the corresponding Entity is still alive. The action
/// can be explicit by calling the `assert_liveliness()` operations, or implicit by writing some data.
/// The two possible manual settings control the granularity at which the application must assert liveliness.
/// The setting [`LivelinessQosPolicyKind::ManualByParticipant`] requires only that one Entity within the publisher is asserted to be alive to
/// deduce all other Entity objects within the same [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant) are also alive.
/// The setting [`LivelinessQosPolicyKind::ManualByTopic`] requires that at least one instance within the [`DataWriter`](crate::publication::data_writer::DataWriter) is asserted.
/// The value offered is considered compatible with the value requested if and only if the inequality *offered kind >= requested kind* is true. For the purposes of this inequality, the values
/// of [`LivelinessQosPolicyKind`] kind are considered ordered such that *Automatic < ManualByParticipant < ManualByTopic*.
/// and the inequality *offered lease_duration <= requested lease_duration* is true.
/// Changes in liveliness must be detected by the Service with a time-granularity greater or equal to the [`LivelinessQosPolicy::lease_duration`]. This
/// ensures that the value of the LivelinessChangedStatus is updated at least once during each [`LivelinessQosPolicy::lease_duration`] and the related
/// Listeners and WaitSets are notified within a [`LivelinessQosPolicy::lease_duration`] from the time the liveliness changed.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
pub struct LivelinessQosPolicy {
    pub kind: LivelinessQosPolicyKind,
    pub lease_duration: Duration,
}

impl QosPolicy for LivelinessQosPolicy {
    fn name(&self) -> &str {
        LIVELINESS_QOS_POLICY_NAME
    }
}

impl Default for LivelinessQosPolicy {
    fn default() -> Self {
        Self {
            kind: LivelinessQosPolicyKind::Automatic,
            lease_duration: DURATION_INFINITE,
        }
    }
}

/// This policy allows a [`DataReader`](crate::subscription::data_reader::DataReader) to indicate that it does not necessarily want to
/// see all values of each instance published under the [`Topic`](crate::topic_definition::topic::Topic).
/// Rather, it wants to see at most one change every [`TimeBasedFilterQosPolicy::minimum_separation`] period.
///
/// The [`TimeBasedFilterQosPolicy`] applies to each instance separately, that is, the constraint is that the [`DataReader`](crate::subscription::data_reader::DataReader)
/// does not want to see more than one sample of each instance per [`TimeBasedFilterQosPolicy::minimum_separation`] period.
/// This setting allows a [`DataReader`](crate::subscription::data_reader::DataReader) to further decouple itself from the
/// [`DataWriter`](crate::publication::data_writer::DataWriter) objects. It can be used to protect applications
/// that are running on a heterogeneous network where some nodes are capable of generating data much faster than others can
/// consume it. It also accommodates the fact that for fast-changing data different subscribers may have different requirements as
/// to how frequently they need to be notified of the most current values.
/// The setting of a [`TimeBasedFilterQosPolicy`], that is, the selection of a  [`TimeBasedFilterQosPolicy::minimum_separation`] with a value greater
/// than zero is compatible with all settings of the [`HistoryQosPolicy`] and [`ReliabilityQosPolicy`].
/// The [`TimeBasedFilterQosPolicy`] specifies the samples that are of interest to the [`DataReader`](crate::subscription::data_reader::DataReader).
/// The [`HistoryQosPolicy`] and [`ReliabilityQosPolicy`] affect the behavior of the middleware with
/// respect to the samples that have been determined to be of interest to the [`DataReader`](crate::subscription::data_reader::DataReader),
/// that is, they apply after the [`TimeBasedFilterQosPolicy`] has been applied.
/// In the case where the reliability [`ReliabilityQosPolicyKind::Reliable`]  then in steady-state, defined as the situation where
/// the [`DataWriter`](crate::publication::data_writer::DataWriter) does not write new samples for a period “long” compared to
/// the [`TimeBasedFilterQosPolicy::minimum_separation`], the system should guarantee delivery the last sample to the [`DataReader`](crate::subscription::data_reader::DataReader).
/// The setting of the  [`TimeBasedFilterQosPolicy::minimum_separation`] minimum_separation must be consistent with the [`DeadlineQosPolicy::period`]. For these
/// two QoS policies to be consistent they must verify that *[`DeadlineQosPolicy::period`] >= [`TimeBasedFilterQosPolicy::minimum_separation`]*.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimeBasedFilterQosPolicy {
    pub minimum_separation: Duration,
}

impl QosPolicy for TimeBasedFilterQosPolicy {
    fn name(&self) -> &str {
        TIMEBASEDFILTER_QOS_POLICY_NAME
    }
}

impl Default for TimeBasedFilterQosPolicy {
    fn default() -> Self {
        Self {
            minimum_separation: DURATION_ZERO,
        }
    }
}

/// This policy allows the introduction of a logical partition concept inside the ‘physical’ partition induced by a domain.
///
/// For a [`DataReader`](crate::subscription::data_reader::DataReader) to see the changes made to an instance by a [`DataWriter`](crate::publication::data_writer::DataWriter),
/// not only the [`Topic`](crate::topic_definition::topic::Topic) must match, but also they must share a common partition.
/// Each string in the list that defines this QoS policy defines a partition name. A partition name may
/// contain wildcards. Sharing a common partition means that one of the partition names matches.
/// Failure to match partitions is not considered an “incompatible” QoS and does not trigger any listeners nor conditions.
/// This policy is changeable. A change of this policy can potentially modify the “match” of existing [`DataReader`](crate::subscription::data_reader::DataReader)
/// and [`DataWriter`](crate::publication::data_writer::DataWriter) entities. It may establish new “matchs” that did not exist before, or break existing matchs.
/// Partition names can be regular expressions and include wildcards as defined by the POSIX fnmatch API (1003.2-1992
/// section B.6). Either [`Publisher`](crate::publication::publisher::Publisher) or [`Subscriber`](crate::subscription::subscriber::Subscriber)
/// may include regular expressions in partition names, but no two names that both
/// contain wildcards will ever be considered to match. This means that although regular expressions may be used both at
/// publisher as well as subscriber side, the service will not try to match two regular expressions (between publishers and
/// subscribers).
/// Partitions are different from creating Entity objects in different domains in several ways. First, entities belonging to different
/// domains are completely isolated from each other; there is no traffic, meta-traffic or any other way for an application or the
/// Service itself to see entities in a domain it does not belong to. Second, an Entity can only belong to one domain whereas an
/// Entity can be in multiple partitions. Finally, as far as the DDS Service is concerned, each unique data instance is identified by
/// the tuple (domainId, Topic, key). Therefore two Entity objects in different domains cannot refer to the same data instance. On
/// the other hand, the same data-instance can be made available (published) or requested (subscribed) on one or more partitions.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartitionQosPolicy {
    pub name: String,
}

impl QosPolicy for PartitionQosPolicy {
    fn name(&self) -> &str {
        PARTITION_QOS_POLICY_NAME
    }
}

impl Default for PartitionQosPolicy {
    fn default() -> Self {
        Self {
            name: "".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ReliabilityQosPolicyKind {
    BestEffort,
    Reliable,
}

const BEST_EFFORT: i32 = 1;
const RELIABLE: i32 = 2;

impl serde::Serialize for ReliabilityQosPolicyKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(
            &match self {
                ReliabilityQosPolicyKind::BestEffort => BEST_EFFORT,
                ReliabilityQosPolicyKind::Reliable => RELIABLE,
            },
            serializer,
        )
    }
}
struct ReliabilityQosPolicyKindVisitor;

impl<'de> serde::de::Visitor<'de> for ReliabilityQosPolicyKindVisitor {
    type Value = ReliabilityQosPolicyKind;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("value `{:}` or `{:}`", BEST_EFFORT, RELIABLE))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match value {
            BEST_EFFORT => ReliabilityQosPolicyKind::BestEffort,
            RELIABLE => ReliabilityQosPolicyKind::Reliable,
            _ => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Unsigned(value as u64),
                    &self,
                ))
            }
        })
    }
}

impl<'de> serde::Deserialize<'de> for ReliabilityQosPolicyKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i32(ReliabilityQosPolicyKindVisitor)
    }
}

impl PartialOrd for ReliabilityQosPolicyKind {
    fn partial_cmp(&self, other: &ReliabilityQosPolicyKind) -> Option<Ordering> {
        match self {
            ReliabilityQosPolicyKind::BestEffort => match other {
                ReliabilityQosPolicyKind::BestEffort => Some(Ordering::Equal),
                ReliabilityQosPolicyKind::Reliable => Some(Ordering::Less),
            },
            ReliabilityQosPolicyKind::Reliable => match other {
                ReliabilityQosPolicyKind::BestEffort => Some(Ordering::Greater),
                ReliabilityQosPolicyKind::Reliable => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy indicates the level of reliability requested by a [`DataReader`](crate::subscription::data_reader::DataReader)
/// or offered by a [`DataWriter`](crate::publication::data_writer::DataWriter).
///
/// These levels are ordered, [`ReliabilityQosPolicyKind::BestEffort`] being lower than [`ReliabilityQosPolicyKind::Reliable`].
/// A [`DataWriter`](crate::publication::data_writer::DataWriter) offering a level is implicitly offering all levels below.
/// The setting of this policy has a dependency on the setting of the [`ResourceLimitsQosPolicy`] policy.
/// In case the [`ReliabilityQosPolicyKind`] kind is set to [`ReliabilityQosPolicyKind::Reliable`] the write operation
/// on the [`DataWriter`](crate::publication::data_writer::DataWriter) may block if the modification would cause data to be lost or else
/// cause one of the limits in specified in the [`ResourceLimitsQosPolicy`] to be exceeded. Under these circumstances, the
///  [`ReliabilityQosPolicy::max_blocking_time`] configures the maximum duration the write operation may block.
/// If the [`ReliabilityQosPolicyKind`] kind is set to [`ReliabilityQosPolicyKind::Reliable`], data-samples originating from a
/// single [`DataWriter`](crate::publication::data_writer::DataWriter) cannot be made available
/// to the [`DataReader`](crate::subscription::data_reader::DataReader) if there are previous data-samples that have not been received
/// yet due to a communication error. In other words, the service will repair the error and re-transmit data-samples as needed
/// in order to re-construct a correct snapshot of the [`DataWriter`](crate::publication::data_writer::DataWriter) history before
/// it is accessible by the [`DataReader`](crate::subscription::data_reader::DataReader).
/// If the [`ReliabilityQosPolicyKind`] is set to [`ReliabilityQosPolicyKind::BestEffort`], the service will not re-transmit missing data samples.
/// However for data samples originating from any one [`DataWriter`](crate::publication::data_writer::DataWriter) the service will ensure
/// they are stored in the [`DataReader`](crate::subscription::data_reader::DataReader) history in the same
/// order they originated in the [`DataWriter`](crate::publication::data_writer::DataWriter).
/// In other words, the [`DataReader`](crate::subscription::data_reader::DataReader) may miss some data samples but it will never see the
/// value of a data-object change from a newer value to an older value.
/// The value offered is considered compatible with the value requested if and only if the inequality *offered kind >= requested
/// kind* is true. For the purposes of this inequality, the values of [`ReliabilityQosPolicyKind`] are considered ordered such
/// that *BestEffort < Reliable*.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReliabilityQosPolicy {
    pub kind: ReliabilityQosPolicyKind,
    pub max_blocking_time: Duration,
}

impl QosPolicy for ReliabilityQosPolicy {
    fn name(&self) -> &str {
        RELIABILITY_QOS_POLICY_NAME
    }
}

impl PartialOrd for ReliabilityQosPolicy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.kind.partial_cmp(&other.kind)
    }
}

// default for Reliability is differnet for reader and writer, hence
// add here as constants
const DEFAULT_MAX_BLOCKING_TIME: Duration = Duration::new(0, 100);
pub const DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS: ReliabilityQosPolicy =
    ReliabilityQosPolicy {
        kind: ReliabilityQosPolicyKind::BestEffort,
        max_blocking_time: DEFAULT_MAX_BLOCKING_TIME,
    };
pub const DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER: ReliabilityQosPolicy = ReliabilityQosPolicy {
    kind: ReliabilityQosPolicyKind::Reliable,
    max_blocking_time: DEFAULT_MAX_BLOCKING_TIME,
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum DestinationOrderQosPolicyKind {
    ByReceptionTimestamp,
    BySourceTimestamp,
}

impl PartialOrd for DestinationOrderQosPolicyKind {
    fn partial_cmp(&self, other: &DestinationOrderQosPolicyKind) -> Option<Ordering> {
        match self {
            DestinationOrderQosPolicyKind::ByReceptionTimestamp => match other {
                DestinationOrderQosPolicyKind::ByReceptionTimestamp => Some(Ordering::Equal),
                DestinationOrderQosPolicyKind::BySourceTimestamp => Some(Ordering::Less),
            },
            DestinationOrderQosPolicyKind::BySourceTimestamp => match other {
                DestinationOrderQosPolicyKind::ByReceptionTimestamp => Some(Ordering::Greater),
                DestinationOrderQosPolicyKind::BySourceTimestamp => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy controls how each subscriber resolves the final value of a data instance that is written by multiple [`DataWriter`](crate::publication::data_writer::DataWriter)
/// objects (which may be associated with different [`Publisher`](crate::publication::publisher::Publisher) objects) running on different nodes.
///
/// The setting [`DestinationOrderQosPolicyKind::ByReceptionTimestamp`] indicates that, assuming the [`OwnershipQosPolicy`] policy allows it, the latest received
/// value for the instance should be the one whose value is kept. This is the default value.
/// The setting [`DestinationOrderQosPolicyKind::BySourceTimestamp`] indicates that, assuming the [`OwnershipQosPolicy`] policy allows it, a timestamp placed at
/// the source should be used. This is the only setting that, in the case of concurrent same-strength [`DataWriter`](crate::publication::data_writer::DataWriter) objects updating the
/// same instance, ensures all subscribers will end up with the same final value for the instance. The mechanism to set the source
/// timestamp is middleware dependent.
/// The value offered is considered compatible with the value requested if and only if the inequality *offered kind >= requested
/// kind* is true. For the purposes of this inequality, the values of [`DestinationOrderQosPolicyKind`] kind are considered
/// ordered such that *DestinationOrderQosPolicyKind::ByReceptionTimestamp < DestinationOrderQosPolicyKind::BySourceTimestamp*.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, serde::Serialize, serde::Deserialize)]
pub struct DestinationOrderQosPolicy {
    pub kind: DestinationOrderQosPolicyKind,
}

impl QosPolicy for DestinationOrderQosPolicyKind {
    fn name(&self) -> &str {
        DESTINATIONORDER_QOS_POLICY_NAME
    }
}

impl Default for DestinationOrderQosPolicy {
    fn default() -> Self {
        Self {
            kind: DestinationOrderQosPolicyKind::ByReceptionTimestamp,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoryQosPolicyKind {
    KeepLast,
    KeepAll,
}

/// This policy controls the behavior of the Service when the value of an instance changes before it is finally
/// communicated to some of its existing [`DataReader`](crate::subscription::data_reader::DataReader) entities.
///
/// If the kind is set to [`HistoryQosPolicyKind::KeepLast`], then the Service will only attempt to keep the latest values of the instance and
/// discard the older ones. In this case, the value of depth regulates the maximum number of values (up to and including
/// the most current one) the Service will maintain and deliver. The default (and most common setting) for depth is one,
/// indicating that only the most recent value should be delivered.
/// If the kind is set to [`HistoryQosPolicyKind::KeepAll`], then the Service will attempt to maintain and deliver all the values of the instance to
/// existing subscribers. The resources that the Service can use to keep this history are limited by the settings of the
/// [`ResourceLimitsQosPolicy`]. If the limit is reached, then the behavior of the Service will depend on the
/// [`ReliabilityQosPolicy`]. If the reliability kind is [`ReliabilityQosPolicyKind::BestEffort`], then the old values will be discarded. If reliability is
/// [`ReliabilityQosPolicyKind::Reliable`], then the Service will block the [`DataWriter`](crate::publication::data_writer::DataWriter) until it can deliver the necessary old values to all subscribers.
/// The setting of [`HistoryQosPolicy`] depth must be consistent with the [`ResourceLimitsQosPolicy::max_samples_per_instance`]. For these two
/// QoS to be consistent, they must verify that *depth <= max_samples_per_instance*.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryQosPolicy {
    pub kind: HistoryQosPolicyKind,
    pub depth: i32,
}

impl QosPolicy for HistoryQosPolicy {
    fn name(&self) -> &str {
        HISTORY_QOS_POLICY_NAME
    }
}

impl Default for HistoryQosPolicy {
    fn default() -> Self {
        Self {
            kind: HistoryQosPolicyKind::KeepLast,
            depth: 1,
        }
    }
}

/// This policy controls the resources that the Service can use in order to meet the requirements imposed by the application and
/// other QoS settings.
///
/// If the [`DataWriter`](crate::publication::data_writer::DataWriter) objects are communicating samples faster than they are ultimately
/// taken by the [`DataReader`](crate::subscription::data_reader::DataReader) objects, the
/// middleware will eventually hit against some of the QoS-imposed resource limits. Note that this may occur when just a single
/// [`DataReader`](crate::subscription::data_reader::DataReader) cannot keep up with its corresponding [`DataWriter`](crate::publication::data_writer::DataWriter).
/// The behavior in this case depends on the setting for the [`ReliabilityQosPolicy`].
/// If reliability is [`ReliabilityQosPolicyKind::BestEffort`] then the Service is allowed to drop samples. If the reliability is
/// [`ReliabilityQosPolicyKind::Reliable`], the Service will block the DataWriter or discard the sample at the
/// [`DataReader`](crate::subscription::data_reader::DataReader) in order not to lose existing samples.
/// The constant [`LENGTH_UNLIMITED`] may be used to indicate the absence of a particular limit. For example setting
/// [`ResourceLimitsQosPolicy::max_samples_per_instance`] to [`LENGTH_UNLIMITED`] will cause the middleware to not enforce
/// this particular limit.
/// The setting of [`ResourceLimitsQosPolicy::max_samples`] must be consistent with the [`ResourceLimitsQosPolicy::max_samples_per_instance`].
/// For these two values to be consistent they must verify that *max_samples >= max_samples_per_instance*.
/// The setting of [`ResourceLimitsQosPolicy::max_samples_per_instance`] must be consistent with the
/// [`HistoryQosPolicy`] depth. For these two QoS to be consistent, they must verify
/// that *[`HistoryQosPolicy::depth`] <= [`ResourceLimitsQosPolicy::max_samples_per_instance`]*.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimitsQosPolicy {
    pub max_samples: Length,
    pub max_instances: Length,
    pub max_samples_per_instance: Length,
}

impl QosPolicy for ResourceLimitsQosPolicy {
    fn name(&self) -> &str {
        RESOURCELIMITS_QOS_POLICY_NAME
    }
}

impl Default for ResourceLimitsQosPolicy {
    fn default() -> Self {
        Self {
            max_samples: Length::Unlimited,
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Unlimited,
        }
    }
}

/// This policy controls the behavior of the Entity as a factory for other entities.
///
/// This policy concerns only DomainParticipant (as factory for Publisher, Subscriber, and Topic), Publisher (as factory for
/// DataWriter), and Subscriber (as factory for DataReader).
/// This policy is mutable. A change in the policy affects only the entities created after the change; not the previously created
/// entities.
/// The setting of `autoenable_created_entities` to [`true`] indicates that the factory `create_<entity>` operation will automatically
/// invoke the enable operation each time a new Entity is created. Therefore, the Entity returned by `create_...` will already
/// be enabled. A setting of [`false`] indicates that the Entity will not be automatically enabled. The application will need to enable
/// it explicitly by means of the `enable()` operation.
/// The default setting of `autoenable_created_entities` is [`true`] which means that, by default, it is not necessary to explicitly call `enable()`
/// on newly created entities.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EntityFactoryQosPolicy {
    pub autoenable_created_entities: bool,
}

impl QosPolicy for EntityFactoryQosPolicy {
    fn name(&self) -> &str {
        ENTITYFACTORY_QOS_POLICY_NAME
    }
}

impl Default for EntityFactoryQosPolicy {
    fn default() -> Self {
        Self {
            autoenable_created_entities: true,
        }
    }
}

/// This policy controls the behavior of the [`DataWriter`](crate::publication::data_writer::DataWriter) with regards to the lifecycle
/// of the data-instances it manages, that is, the data-instances that have been either explicitly registered with the
/// [`DataWriter::register`](crate::publication::data_writer::DataWriter) or implicitly by using [`DataWriter::write`](crate::publication::data_writer::DataWriter)
///
/// The [`WriterDataLifecycleQosPolicy::autodispose_unregistered_instances`] flag controls the behavior when the
/// DataWriter unregisters an instance by means of the [`DataWriter::unregister_instance`](crate::publication::data_writer::DataWriter) operations:
/// - The setting [`WriterDataLifecycleQosPolicy::autodispose_unregistered_instances`] = [`true`] causes the [`DataWriter::unregister_instance`](crate::publication::data_writer::DataWriter)
///   to dispose the instance each time it is unregistered.
///   The behavior is identical to explicitly calling one of the [`DataWriter::dispose`](crate::publication::data_writer::DataWriter) operations on the
///   instance prior to calling the unregister operation.
/// - The setting [`WriterDataLifecycleQosPolicy::autodispose_unregistered_instances`] = [`false`]  will not cause this automatic disposition upon unregistering.
///   The application can still call one of the dispose operations prior to unregistering the instance and accomplish the same effect.
///
/// Note that the deletion of a [`DataWriter`](crate::publication::data_writer::DataWriter) automatically unregisters all data-instances it manages.
/// Therefore the setting of the [`WriterDataLifecycleQosPolicy::autodispose_unregistered_instances`] flag will determine whether instances are ultimately disposed when the
/// [`DataWriter`](crate::publication::data_writer::DataWriter) is deleted.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WriterDataLifecycleQosPolicy {
    pub autodispose_unregistered_instances: bool,
}

impl QosPolicy for WriterDataLifecycleQosPolicy {
    fn name(&self) -> &str {
        WRITERDATALIFECYCLE_QOS_POLICY_NAME
    }
}

impl Default for WriterDataLifecycleQosPolicy {
    fn default() -> Self {
        Self {
            autodispose_unregistered_instances: true,
        }
    }
}

/// This policy controls the behavior of the [`DataReader`](crate::subscription::data_reader::DataReader) with regards to the lifecycle of the data-instances it manages, that is, the
/// data-instances that have been received and for which the [`DataReader`](crate::subscription::data_reader::DataReader) maintains some internal resources.
///
/// The [`DataReader`](crate::subscription::data_reader::DataReader) internally maintains the samples that have not been taken by the application, subject to the constraints
/// imposed by other QoS policies such as [`HistoryQosPolicy`] and [`ResourceLimitsQosPolicy`].
/// The [`DataReader`](crate::subscription::data_reader::DataReader) also maintains information regarding the identity, view_state and instance_state
/// of data-instances even after all samples have been ‘taken.’ This is needed to properly compute the states when future samples arrive.
/// Under normal circumstances the [`DataReader`](crate::subscription::data_reader::DataReader) can only reclaim all resources for instances for which there are no writers and for
/// which all samples have been ‘taken’. The last sample the [`DataReader`](crate::subscription::data_reader::DataReader) will have taken for that instance will have an
/// `instance_state` of either [`InstanceStateKind::NotAliveNoWriters`](crate::subscription::sample_info::InstanceStateKind) or
/// [`InstanceStateKind::NotAliveDisposed`](crate::subscription::sample_info::InstanceStateKind) depending on whether the last writer
/// that had ownership of the instance disposed it or not.  In the absence of the [`ReaderDataLifecycleQosPolicy`] this behavior could cause problems if the
/// application “forgets” to ‘take’ those samples. The ‘untaken’ samples will prevent the [`DataReader`](crate::subscription::data_reader::DataReader) from reclaiming the
/// resources and they would remain in the [`DataReader`](crate::subscription::data_reader::DataReader) indefinitely.
/// The [`ReaderDataLifecycleQosPolicy::autopurge_nowriter_samples_delay`] defines the maximum duration for which the [`DataReader`](crate::subscription::data_reader::DataReader) will maintain information
/// regarding an instance once its `instance_state` becomes [`InstanceStateKind::NotAliveNoWriters`](crate::subscription::sample_info::InstanceStateKind). After this time elapses, the [`DataReader`](crate::subscription::data_reader::DataReader)
/// will purge all internal information regarding the instance, any untaken samples will also be lost.
/// The [`ReaderDataLifecycleQosPolicy::autopurge_disposed_samples_delay`] defines the maximum duration for which the [`DataReader`](crate::subscription::data_reader::DataReader) will maintain samples for
/// an instance once its `instance_state` becomes [`InstanceStateKind::NotAliveDisposed`](crate::subscription::sample_info::InstanceStateKind). After this time elapses, the [`DataReader`](crate::subscription::data_reader::DataReader) will purge all
/// samples for the instance.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReaderDataLifecycleQosPolicy {
    pub autopurge_nowriter_samples_delay: Duration,
    pub autopurge_disposed_samples_delay: Duration,
}

impl QosPolicy for ReaderDataLifecycleQosPolicy {
    fn name(&self) -> &str {
        READERDATALIFECYCLE_QOS_POLICY_NAME
    }
}

impl Default for ReaderDataLifecycleQosPolicy {
    fn default() -> Self {
        Self {
            autopurge_nowriter_samples_delay: DURATION_INFINITE,
            autopurge_disposed_samples_delay: DURATION_INFINITE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durability_qos_policy_kind_ordering() {
        assert!(DurabilityQosPolicyKind::Volatile < DurabilityQosPolicyKind::TransientLocal);

        assert!(DurabilityQosPolicyKind::Volatile == DurabilityQosPolicyKind::Volatile);
        assert!(DurabilityQosPolicyKind::Volatile < DurabilityQosPolicyKind::TransientLocal);

        assert!(DurabilityQosPolicyKind::TransientLocal > DurabilityQosPolicyKind::Volatile);
        assert!(DurabilityQosPolicyKind::TransientLocal == DurabilityQosPolicyKind::TransientLocal);
    }

    #[test]
    fn presentation_qos_policy_access_scope_kind_ordering() {
        assert!(
            PresentationQosPolicyAccessScopeKind::Instance
                < PresentationQosPolicyAccessScopeKind::Topic
        );

        assert!(
            PresentationQosPolicyAccessScopeKind::Instance
                == PresentationQosPolicyAccessScopeKind::Instance
        );
        assert!(
            PresentationQosPolicyAccessScopeKind::Instance
                < PresentationQosPolicyAccessScopeKind::Topic
        );

        assert!(
            PresentationQosPolicyAccessScopeKind::Topic
                > PresentationQosPolicyAccessScopeKind::Instance
        );
        assert!(
            PresentationQosPolicyAccessScopeKind::Topic
                == PresentationQosPolicyAccessScopeKind::Topic
        );
    }

    #[test]
    fn liveliness_qos_policy_kind_ordering() {
        assert!(LivelinessQosPolicyKind::Automatic < LivelinessQosPolicyKind::ManualByParticipant);
        assert!(
            LivelinessQosPolicyKind::ManualByParticipant < LivelinessQosPolicyKind::ManualByTopic
        );

        assert!(LivelinessQosPolicyKind::Automatic == LivelinessQosPolicyKind::Automatic);
        assert!(LivelinessQosPolicyKind::Automatic < LivelinessQosPolicyKind::ManualByParticipant);
        assert!(LivelinessQosPolicyKind::Automatic < LivelinessQosPolicyKind::ManualByTopic);

        assert!(LivelinessQosPolicyKind::ManualByParticipant > LivelinessQosPolicyKind::Automatic);
        assert!(
            LivelinessQosPolicyKind::ManualByParticipant
                == LivelinessQosPolicyKind::ManualByParticipant
        );
        assert!(
            LivelinessQosPolicyKind::ManualByParticipant < LivelinessQosPolicyKind::ManualByTopic
        );

        assert!(LivelinessQosPolicyKind::ManualByTopic > LivelinessQosPolicyKind::Automatic);
        assert!(
            LivelinessQosPolicyKind::ManualByTopic > LivelinessQosPolicyKind::ManualByParticipant
        );
        assert!(LivelinessQosPolicyKind::ManualByTopic == LivelinessQosPolicyKind::ManualByTopic);
    }

    #[test]
    fn reliability_qos_policy_kind_ordering() {
        assert!(ReliabilityQosPolicyKind::BestEffort < ReliabilityQosPolicyKind::Reliable);

        assert!(ReliabilityQosPolicyKind::BestEffort == ReliabilityQosPolicyKind::BestEffort);
        assert!(ReliabilityQosPolicyKind::BestEffort < ReliabilityQosPolicyKind::Reliable);

        assert!(ReliabilityQosPolicyKind::Reliable > ReliabilityQosPolicyKind::BestEffort);
        assert!(ReliabilityQosPolicyKind::Reliable == ReliabilityQosPolicyKind::Reliable);
    }

    #[test]
    fn destination_order_qos_policy_kind_ordering() {
        assert!(
            DestinationOrderQosPolicyKind::ByReceptionTimestamp
                < DestinationOrderQosPolicyKind::BySourceTimestamp
        );

        assert!(
            DestinationOrderQosPolicyKind::ByReceptionTimestamp
                == DestinationOrderQosPolicyKind::ByReceptionTimestamp
        );
        assert!(
            DestinationOrderQosPolicyKind::ByReceptionTimestamp
                < DestinationOrderQosPolicyKind::BySourceTimestamp
        );

        assert!(
            DestinationOrderQosPolicyKind::BySourceTimestamp
                > DestinationOrderQosPolicyKind::ByReceptionTimestamp
        );
        assert!(
            DestinationOrderQosPolicyKind::BySourceTimestamp
                == DestinationOrderQosPolicyKind::BySourceTimestamp
        );
    }
}
