use core::cmp::Ordering;

use crate::infrastructure::time::{Duration, DURATION_INFINITE, DURATION_ZERO};

pub type QosPolicyId = i32;
pub const LENGTH_UNLIMITED: i32 = -1;

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
const DURABILITYSERVICE_POLICY_NAME: &str = "DurabilityService";

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

/// The purpose of this QoS is to allow the application to attach additional information to the created Entity objects such that when
/// a remote application discovers their existence it can access that information and use it for its own purposes. One possible use
/// of this QoS is to attach security credentials or some other information that can be used by the remote application to
/// authenticate the source. In combination with operations such as ignore_participant, ignore_publication, ignore_subscription,
/// and ignore_topic these QoS can assist an application to define and enforce its own security policies. The use of this QoS is not
/// limited to security, rather it offers a simple, yet flexible extensibility mechanism.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserDataQosPolicy {
    pub value: Vec<u8>,
}

impl QosPolicy for UserDataQosPolicy {
    fn name(&self) -> &str {
        USERDATA_QOS_POLICY_NAME
    }
}

/// The purpose of this QoS is to allow the application to attach additional information to the created Topic such that when a
/// remote application discovers their existence it can examine the information and use it in an application-defined way. In
/// combination with the listeners on the DataReader and DataWriter as well as by means of operations such as ignore_topic,
/// these QoS can assist an application to extend the provided QoS.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicDataQosPolicy {
    pub value: Vec<u8>,
}

impl QosPolicy for TopicDataQosPolicy {
    fn name(&self) -> &str {
        TOPICDATA_QOS_POLICY_NAME
    }
}

/// The purpose of this QoS is to allow the application to attach additional information to the created Publisher or Subscriber. The
/// value of the GROUP_DATA is available to the application on the DataReader and DataWriter entities and is propagated by
/// means of the built-in topics.
/// This QoS can be used by an application combination with the DataReaderListener and DataWriterListener to implement
/// matching policies similar to those of the PARTITION QoS except the decision can be made based on an application-defined
/// policy.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupDataQosPolicy {
    pub value: Vec<u8>,
}

impl QosPolicy for GroupDataQosPolicy {
    fn name(&self) -> &str {
        GROUPDATA_QOS_POLICY_NAME
    }
}

/// The purpose of this QoS is to allow the application to take advantage of transports capable of sending messages with different
/// priorities.
/// This policy is considered a hint. The policy depends on the ability of the underlying transports to set a priority on the messages
/// they send. Any value within the range of a 32-bit signed integer may be chosen; higher values indicate higher priority.
/// However, any further interpretation of this policy is specific to a particular transport and a particular implementation of the
/// Service. For example, a particular transport is permitted to treat a range of priority values as equivalent to one another. It is
/// expected that during transport configuration the application would provide a mapping between the values of the
/// TRANSPORT_PRIORITY set on DataWriter and the values meaningful to each transport. This mapping would then be used
/// by the infrastructure when propagating the data written by the DataWriter.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportPriorityQosPolicy {
    pub value: i32,
}

impl QosPolicy for TransportPriorityQosPolicy {
    fn name(&self) -> &str {
        TRANSPORTPRIORITY_QOS_POLICY_NAME
    }
}

/// The purpose of this QoS is to avoid delivering “stale” data to the application.
/// Each data sample written by the DataWriter has an associated ‘expiration time’ beyond which the data should not be delivered
/// to any application. Once the sample expires, the data will be removed from the DataReader caches as well as from the
/// transient and persistent information caches.
/// The ‘expiration time’ of each sample is computed by adding the duration specified by the LIFESPAN QoS to the source
/// timestamp. As described in 2.2.2.4.2.11 and 2.2.2.4.2.12 the source timestamp is either automatically computed by the Service
/// each time the DataWriter write operation is called, or else supplied by the application by means of the write_w_timestamp
/// operation.
/// This QoS relies on the sender and receiving applications having their clocks sufficiently synchronized. If this is not the case
/// and the Service can detect it, the DataReader is allowed to use the reception timestamp instead of the source timestamp in its
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
    VolatileDurabilityQoS,
    TransientLocalDurabilityQoS,
}

impl PartialOrd for DurabilityQosPolicyKind {
    fn partial_cmp(&self, other: &DurabilityQosPolicyKind) -> Option<Ordering> {
        match self {
            DurabilityQosPolicyKind::VolatileDurabilityQoS => match other {
                DurabilityQosPolicyKind::VolatileDurabilityQoS => Some(Ordering::Equal),
                DurabilityQosPolicyKind::TransientLocalDurabilityQoS => Some(Ordering::Less),
            },
            DurabilityQosPolicyKind::TransientLocalDurabilityQoS => match other {
                DurabilityQosPolicyKind::VolatileDurabilityQoS => Some(Ordering::Greater),
                DurabilityQosPolicyKind::TransientLocalDurabilityQoS => Some(Ordering::Equal),
            },
        }
    }
}

/// The decoupling between DataReader and DataWriter offered by the Publish/Subscribe paradigm allows an application to
/// write data even if there are no current readers on the network. Moreover, a DataReader that joins the network after some data
/// has been written could potentially be interested in accessing the most current values of the data as well as potentially some
/// history. This QoS policy controls whether the Service will actually make data available to late-joining readers. Note that
/// although related, this does not strictly control what data the Service will maintain internally. That is, the Service may choose to
/// maintain some data for its own purposes (e.g., flow control) and yet not make it available to late-joining readers if the
/// DURABILITY QoS policy is set to VOLATILE.
/// The value offered is considered compatible with the value requested if and only if the inequality “offered kind >= requested
/// kind” evaluates to ‘TRUE.’ For the purposes of this inequality, the values of DURABILITY kind are considered ordered such
/// that VOLATILE < TRANSIENT_LOCAL < TRANSIENT < PERSISTENT.
/// For the purpose of implementing the DURABILITY QoS kind TRANSIENT or PERSISTENT, the service behaves “as if” for
/// each Topic that has TRANSIENT or PERSISTENT DURABILITY kind there was a corresponding “built-in” DataReader and
/// DataWriter configured to have the same DURABILITY kind. In other words, it is “as if” somewhere in the system (possibly
/// on a remote node) there was a “built-in durability DataReader” that subscribed to that Topic and a “built-in durability
/// DataWriter” that published that Topic as needed for the new subscribers that join the system.
/// For each Topic, the built-in fictitious “persistence service” DataReader and DataWriter has its QoS configured from the Topic
/// QoS of the corresponding Topic. In other words, it is “as-if” the service first did find_topic to access the Topic, and then used
/// the QoS from the Topic to configure the fictitious built-in entities.
/// A consequence of this model is that the transient or persistence serviced can be configured by means of setting the proper QoS
/// on the Topic.
/// For a given Topic, the usual request/offered semantics apply to the matching between any DataWriter in the system that writes
/// the Topic and the built-in transient/persistent DataReader for that Topic; similarly for the built-in transient/persistent
/// DataWriter for a Topic and any DataReader for the Topic. As a consequence, a DataWriter that has an incompatible QoS with
/// respect to what the Topic specified will not send its data to the transient/persistent service, and a DataReader that has an
/// incompatible QoS with respect to the specified in the Topic will not get data from it.
/// Incompatibilities between local DataReader/DataWriter entities and the corresponding fictitious “built-in transient/persistent
/// entities” cause the REQUESTED_INCOMPATIBLE_QOS/OFFERED_INCOMPATIBLE_QOS status to change and the
/// corresponding Listener invocations and/or signaling of Condition and WaitSet objects as they would with non-fictitious
/// entities.
/// The setting of the service_cleanup_delay controls when the TRANSIENT or PERSISTENT service is able to remove all
/// information regarding a data-instances. Information on a data-instances is maintained until the following conditions are met:
/// 1. the instance has been explicitly disposed (instance_state = NOT_ALIVE_DISPOSED), and
/// 2. while in the NOT_ALIVE_DISPOSED state the system detects that there are no more “live” DataWriter entities
/// writing the instance, that is, all existing writers either unregister the instance (call unregister) or lose their liveliness,
/// and
/// 3. a time interval longer that service_cleanup_delay has elapsed since the moment the service detected that the previous
/// two conditions were met.
/// The utility of the service_cleanup_delay is apparent in the situation where an application disposes an instance and it crashes
/// before it has a chance to complete additional tasks related to the disposition. Upon re-start the application may ask for initial
/// data to regain its state and the delay introduced by the service_cleanup_delay will allow the restarted application to receive
/// the information on the disposed instance and complete the interrupted tasks.
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
            kind: DurabilityQosPolicyKind::VolatileDurabilityQoS,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum PresentationQosPolicyAccessScopeKind {
    InstancePresentationQoS,
    TopicPresentationQoS,
}

impl PartialOrd for PresentationQosPolicyAccessScopeKind {
    fn partial_cmp(&self, other: &PresentationQosPolicyAccessScopeKind) -> Option<Ordering> {
        match self {
            PresentationQosPolicyAccessScopeKind::InstancePresentationQoS => match other {
                PresentationQosPolicyAccessScopeKind::InstancePresentationQoS => {
                    Some(Ordering::Equal)
                }
                PresentationQosPolicyAccessScopeKind::TopicPresentationQoS => Some(Ordering::Less),
            },
            PresentationQosPolicyAccessScopeKind::TopicPresentationQoS => match other {
                PresentationQosPolicyAccessScopeKind::InstancePresentationQoS => {
                    Some(Ordering::Greater)
                }
                PresentationQosPolicyAccessScopeKind::TopicPresentationQoS => Some(Ordering::Equal),
            },
        }
    }
}

/// This QoS policy controls the extent to which changes to data-instances can be made dependent on each other and also the kind
/// of dependencies that can be propagated and maintained by the Service.
/// The setting of coherent_access controls whether the Service will preserve the groupings of changes made by the publishing
/// application by means of the operations begin_coherent_change and end_coherent_change.
/// The setting of ordered_access controls whether the Service will preserve the order of changes.
/// The granularity is controlled by the setting of the access_scope.
/// If coherent_access is set, then the access_scope controls the maximum extent of coherent changes. The behavior is as follows:
/// • If access_scope is set to INSTANCE, the use of begin_coherent_change and end_coherent_change has no effect on
/// how the subscriber can access the data because with the scope limited to each instance, changes to separate instances
/// are considered independent and thus cannot be grouped by a coherent change.
/// • If access_scope is set to TOPIC, then coherent changes (indicated by their enclosure within calls to
/// begin_coherent_change and end_coherent_change) will be made available as such to each remote DataReader
/// independently. That is, changes made to instances within each individual DataWriter will be available as coherent with
/// respect to other changes to instances in that same DataWriter, but will not be grouped with changes made to instances
/// belonging to a different DataWriter.
/// • If access_scope is set to GROUP, then coherent changes made to instances through a DataWriter attached to a common
/// Publisher are made available as a unit to remote subscribers.
/// If ordered_access is set, then the access_scope controls the maximum extent for which order will be preserved by the Service.
/// • If access_scope is set to INSTANCE (the lowest level), then changes to each instance are considered unordered relative
/// to changes to any other instance. That means that changes (creations, deletions, modifications) made to two instances
/// are not necessarily seen in the order they occur. This is the case even if it is the same application thread making the
/// changes using the same DataWriter.
/// • If access_scope is set to TOPIC, changes (creations, deletions, modifications) made by a single DataWriter are made
/// available to subscribers in the same order they occur. Changes made to instances through different DataWriter entities
/// are not necessarily seen in the order they occur. This is the case, even if the changes are made by a single application
/// thread using DataWriter objects attached to the same Publisher.
/// • Finally, if access_scope is set to GROUP, changes made to instances via DataWriter entities attached to the same
/// Publisher object are made available to subscribers on the same order they occur.
/// Note that this QoS policy controls the scope at which related changes are made available to the subscriber. This means the
/// subscriber can access the changes in a coherent manner and in the proper order; however, it does not necessarily imply that the
/// Subscriber will indeed access the changes in the correct order. For that to occur, the application at the subscriber end must use
/// the proper logic in reading the DataReader objects, as described in 2.2.2.5.1, Access to the data.
/// The value offered is considered compatible with the value requested if and only if the following conditions are met:
/// 1. The inequality “offered access_scope >= requested access_scope” evaluates to ‘TRUE.’ For the purposes of this
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
            access_scope: PresentationQosPolicyAccessScopeKind::InstancePresentationQoS,
            coherent_access: false,
            ordered_access: false,
        }
    }
}

/// This policy is useful for cases where a Topic is expected to have each instance updated periodically. On the publishing side this
/// setting establishes a contract that the application must meet. On the subscribing side the setting establishes a minimum
/// requirement for the remote publishers that are expected to supply the data values.
/// When the Service ‘matches’ a DataWriter and a DataReader it checks whether the settings are compatible (i.e., offered
/// deadline period<= requested deadline period) if they are not, the two entities are informed (via the listener or condition
/// mechanism) of the incompatibility of the QoS settings and communication will not occur.
/// Assuming that the reader and writer ends have compatible settings, the fulfillment of this contract is monitored by the Service
/// and the application is informed of any violations by means of the proper listener or condition.
/// The value offered is considered compatible with the value requested if and only if the inequality “offered deadline period <=
/// requested deadline period” evaluates to ‘TRUE.’
/// The setting of the DEADLINE policy must be set consistently with that of the TIME_BASED_FILTER. For these two policies
/// to be consistent the settings must be such that “deadline period>= minimum_separation.”
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

/// This policy provides a means for the application to indicate to the middleware the “urgency” of the data-communication. By
/// having a non-zero duration the Service can optimize its internal operation.
/// This policy is considered a hint. There is no specified mechanism as to how the service should take advantage of this hint.
/// The value offered is considered compatible with the value requested if and only if the inequality “offered duration <=
/// requested duration” evaluates to ‘TRUE.’
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
    SharedOwnershipQoS,
}

/// This policy controls whether the Service allows multiple DataWriter objects to update the same instance (identified by Topic +
/// key) of a data-object.
/// There are two kinds of OWNERSHIP selected by the setting of the kind: SHARED and EXCLUSIVE.
/// SHARED kind
/// This setting indicates that the Service does not enforce unique ownership for each instance. In this case, multiple writers can
/// update the same data-object instance. The subscriber to the Topic will be able to access modifications from all DataWriter
/// objects, subject to the settings of other QoS that may filter particular samples (e.g., the TIME_BASED_FILTER or HISTORY
/// QoS policy). In any case there is no “filtering” of modifications made based on the identity of the DataWriter that causes the
/// modification.
/// EXCLUSIVE kind
/// This setting indicates that each instance of a data-object can only be modified by one DataWriter. In other words, at any point
/// in time a single DataWriter “owns” each instance and is the only one whose modifications will be visible to the DataReader
/// objects. The owner is determined by selecting the DataWriter with the highest value of the strength that is both “alive” as
/// defined by the LIVELINESS QoS policy and has not violated its DEADLINE contract with regards to the data-instance.
/// Ownership can therefore change as a result of (a) a DataWriter in the system with a higher value of the strength that modifies
/// the instance, (b) a change in the strength value of the DataWriter that owns the instance, (c) a change in the liveliness of the
/// DataWriter that owns the instance, and (d) a deadline with regards to the instance that is missed by the DataWriter that owns
/// the instance.
/// The behavior of the system is as if the determination was made independently by each DataReader. Each DataReader may
/// detect the change of ownership at a different time. It is not a requirement that at a particular point in time all the DataReader
/// objects for that Topic have a consistent picture of who owns each instance.
/// It is also not a requirement that the DataWriter objects are aware of whether they own a particular instance. There is no error or
/// notification given to a DataWriter that modifies an instance it does not currently own.
/// The requirements are chosen to (a) preserve the decoupling of publishers and subscriber, and (b) allow the policy to be
/// implemented efficiently.
/// It is possible that multiple DataWriter objects with the same strength modify the same instance. If this occurs the Service will
/// pick one of the DataWriter objects as the “owner.” It is not specified how the owner is selected. However, it is required that the
/// policy used to select the owner is such that all DataReader objects will make the same choice of the particular DataWriter that
/// is the owner. It is also required that the owner remains the same until there is a change in strength, liveliness, the owner misses
/// a deadline on the instance, a new DataWriter with higher strength modifies the instance, or another DataWriter with the same
/// strength that is deemed by the Service to be the new owner modifies the instance
/// Exclusive ownership is on an instance-by-instance basis. That is, a subscriber can receive values written by a lower
/// strength DataWriter as long as they affect instances whose values have not been set by the higher-strength
/// DataWriter.
/// The value of the OWNERSHIP kind offered must exactly match the one requested or else they are considered
/// incompatible.
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
            kind: OwnershipQosPolicyKind::SharedOwnershipQoS,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum LivelinessQosPolicyKind {
    AutomaticLivelinessQoS,
    ManualByParticipantLivelinessQoS,
    ManualByTopicLivelinessQoS,
}

impl PartialOrd for LivelinessQosPolicyKind {
    fn partial_cmp(&self, other: &LivelinessQosPolicyKind) -> Option<Ordering> {
        match self {
            LivelinessQosPolicyKind::AutomaticLivelinessQoS => match other {
                LivelinessQosPolicyKind::AutomaticLivelinessQoS => Some(Ordering::Equal),
                LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS => Some(Ordering::Less),
                LivelinessQosPolicyKind::ManualByTopicLivelinessQoS => Some(Ordering::Less),
            },
            LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS => match other {
                LivelinessQosPolicyKind::AutomaticLivelinessQoS => Some(Ordering::Greater),
                LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS => Some(Ordering::Equal),
                LivelinessQosPolicyKind::ManualByTopicLivelinessQoS => Some(Ordering::Less),
            },
            LivelinessQosPolicyKind::ManualByTopicLivelinessQoS => match other {
                LivelinessQosPolicyKind::AutomaticLivelinessQoS => Some(Ordering::Greater),
                LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS => {
                    Some(Ordering::Greater)
                }
                LivelinessQosPolicyKind::ManualByTopicLivelinessQoS => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy controls the mechanism and parameters used by the Service to ensure that particular entities on the network are
/// still “alive.” The liveliness can also affect the ownership of a particular instance, as determined by the OWNERSHIP QoS
/// policy.
/// This policy has several settings to support both data-objects that are updated periodically as well as those that are changed
/// sporadically. It also allows customizing for different application requirements in terms of the kinds of failures that will be
/// detected by the liveliness mechanism.
/// The AUTOMATIC liveliness setting is most appropriate for applications that only need to detect failures at the processlevel27,
/// but not application-logic failures within a process. The Service takes responsibility for renewing the leases at the
/// required rates and thus, as long as the local process where a DomainParticipant is running and the link connecting it to remote
/// participants remains connected, the entities within the DomainParticipant will be considered alive. This requires the lowest
/// overhead.
/// The MANUAL settings (MANUAL_BY_PARTICIPANT, MANUAL_BY_TOPIC), require the application on the publishing
/// side to periodically assert the liveliness before the lease expires to indicate the corresponding Entity is still alive. The action
/// can be explicit by calling the assert_liveliness operations, or implicit by writing some data.
/// The two possible manual settings control the granularity at which the application must assert liveliness.
/// • The setting MANUAL_BY_PARTICIPANT requires only that one Entity within the publisher is asserted to be alive to
/// deduce all other Entity objects within the same DomainParticipant are also alive.
/// • The setting MANUAL_BY_TOPIC requires that at least one instance within the DataWriter is asserted.
/// The value offered is considered compatible with the value requested if and only if the following conditions are met:
/// 1. the inequality “offered kind >= requested kind” evaluates to ‘TRUE’. For the purposes of this inequality, the values
/// of LIVELINESS kind are considered ordered such that:
/// AUTOMATIC < MANUAL_BY_PARTICIPANT < MANUAL_BY_TOPIC.
/// 2. the inequality “offered lease_duration <= requested lease_duration” evaluates to TRUE.
/// Changes in LIVELINESS must be detected by the Service with a time-granularity greater or equal to the lease_duration. This
/// ensures that the value of the LivelinessChangedStatus is updated at least once during each lease_duration and the related
/// Listeners and WaitSets are notified within a lease_duration from the time the LIVELINESS changed.
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
            kind: LivelinessQosPolicyKind::AutomaticLivelinessQoS,
            lease_duration: DURATION_INFINITE,
        }
    }
}

/// This policy allows a DataReader to indicate that it does not necessarily want to see all values of each instance published under
/// the Topic. Rather, it wants to see at most one change every minimum_separation period.
/// The TIME_BASED_FILTER applies to each instance separately, that is, the constraint is that the DataReader does not want to
/// see more than one sample of each instance per minimum_separation period.
/// This setting allows a DataReader to further decouple itself from the DataWriter objects. It can be used to protect applications
/// that are running on a heterogeneous network where some nodes are capable of generating data much faster than others can
/// consume it. It also accommodates the fact that for fast-changing data different subscribers may have different requirements as
/// to how frequently they need to be notified of the most current values.
/// The setting of a TIME_BASED_FILTER, that is, the selection of a minimum_separation with a value greater than zero is
/// compatible with all settings of the HISTORY and RELIABILITY QoS. The TIME_BASED_FILTER specifies the samples
/// that are of interest to the DataReader. The HISTORY and RELIABILITY QoS affect the behavior of the middleware with
/// respect to the samples that have been determined to be of interest to the DataReader, that is, they apply after the
/// TIME_BASED_FILTER has been applied.
/// In the case where the reliability QoS kind is RELIABLE then in steady-state, defined as the situation where the DataWriter
/// does not write new samples for a period “long” compared to the minimum_separation, the system should guarantee delivery
/// the last sample to the DataReader.
/// The setting of the TIME_BASED_FILTER minimum_separation must be consistent with the DEADLINE period. For these
/// two QoS policies to be consistent they must verify that "period >= minimum_separation." An attempt to set these policies in
/// an inconsistent manner when an entity is created of via a set_qos operation will cause the operation to fail.
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
/// For a DataReader to see the changes made to an instance by a DataWriter, not only the Topic must match, but also they must
/// share a common partition. Each string in the list that defines this QoS policy defines a partition name. A partition name may
/// contain wildcards. Sharing a common partition means that one of the partition names matches.
/// Failure to match partitions is not considered an “incompatible” QoS and does not trigger any listeners nor conditions.
/// This policy is changeable. A change of this policy can potentially modify the “match” of existing DataReader and DataWriter
/// entities. It may establish new “matchs” that did not exist before, or break existing matchs.
/// PARTITION names can be regular expressions and include wildcards as defined by the POSIX fnmatch API (1003.2-1992
/// section B.6). Either Publisher or Subscriber may include regular expressions in partition names, but no two names that both
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
    BestEffortReliabilityQos,
    ReliableReliabilityQos,
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
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => BEST_EFFORT,
                ReliabilityQosPolicyKind::ReliableReliabilityQos => RELIABLE,
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
            BEST_EFFORT => ReliabilityQosPolicyKind::BestEffortReliabilityQos,
            RELIABLE => ReliabilityQosPolicyKind::ReliableReliabilityQos,
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
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => match other {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => Some(Ordering::Equal),
                ReliabilityQosPolicyKind::ReliableReliabilityQos => Some(Ordering::Less),
            },
            ReliabilityQosPolicyKind::ReliableReliabilityQos => match other {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => Some(Ordering::Greater),
                ReliabilityQosPolicyKind::ReliableReliabilityQos => Some(Ordering::Equal),
            },
        }
    }
}

/// This policy indicates the level of reliability requested by a DataReader or offered by a DataWriter. These levels are ordered,
/// BEST_EFFORT being lower than RELIABLE. A DataWriter offering a level is implicitly offering all levels below.
/// The setting of this policy has a dependency on the setting of the RESOURCE_LIMITS policy. In case the RELIABILITY kind
/// is set to RELIABLE the write operation on the DataWriter may block if the modification would cause data to be lost or else
/// cause one of the limits in specified in the RESOURCE_LIMITS to be exceeded. Under these circumstances, the
/// RELIABILITY max_blocking_time configures the maximum duration the write operation may block.
/// If the RELIABILITY kind is set to RELIABLE, data-samples originating from a single DataWriter cannot be made available
/// to the DataReader if there are previous data-samples that have not been received yet due to a communication error. In other
/// words, the service will repair the error and re-transmit data-samples as needed in order to re-construct a correct snapshot of the
/// DataWriter history before it is accessible by the DataReader.
/// If the RELIABILITY kind is set to BEST_EFFORT, the service will not re-transmit missing data-samples. However for datasamples
/// originating from any one DataWriter the service will ensure they are stored in the DataReader history in the same
/// order they originated in the DataWriter. In other words, the DataReader may miss some data-samples but it will never see the
/// value of a data-object change from a newer value to an order value.
/// The value offered is considered compatible with the value requested if and only if the inequality “offered kind >= requested
/// kind” evaluates to ‘TRUE.’ For the purposes of this inequality, the values of RELIABILITY kind are considered ordered such
/// that BEST_EFFORT < RELIABLE.
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
        kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
        max_blocking_time: DEFAULT_MAX_BLOCKING_TIME,
    };
pub const DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER: ReliabilityQosPolicy = ReliabilityQosPolicy {
    kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
    max_blocking_time: DEFAULT_MAX_BLOCKING_TIME,
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum DestinationOrderQosPolicyKind {
    ByReceptionTimestampDestinationOrderQoS,
    BySourceTimestampDestinationOrderQoS,
}

impl PartialOrd for DestinationOrderQosPolicyKind {
    fn partial_cmp(&self, other: &DestinationOrderQosPolicyKind) -> Option<Ordering> {
        match self {
            DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS => match other {
                DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS => {
                    Some(Ordering::Equal)
                }
                DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS => {
                    Some(Ordering::Less)
                }
            },
            DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS => match other {
                DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS => {
                    Some(Ordering::Greater)
                }
                DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS => {
                    Some(Ordering::Equal)
                }
            },
        }
    }
}

/// This policy controls how each subscriber resolves the final value of a data instance that is written by multiple DataWriter
/// objects (which may be associated with different Publisher objects) running on different nodes.
/// The setting BY_RECEPTION_TIMESTAMP indicates that, assuming the OWNERSHIP policy allows it, the latest received
/// value for the instance should be the one whose value is kept. This is the default value.
/// The setting BY_SOURCE_TIMESTAMP indicates that, assuming the OWNERSHIP policy allows it, a timestamp placed at
/// the source should be used. This is the only setting that, in the case of concurrent same-strength DataWriter objects updating the
/// same instance, ensures all subscribers will end up with the same final value for the instance. The mechanism to set the source
/// timestamp is middleware dependent.
/// The value offered is considered compatible with the value requested if and only if the inequality “offered kind >= requested
/// kind” evaluates to ‘TRUE.’ For the purposes of this inequality, the values of DESTINATION_ORDER kind are considered
/// ordered such that BY_RECEPTION_TIMESTAMP < BY_SOURCE_TIMESTAMP.
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
            kind: DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum HistoryQosPolicyKind {
    KeepLastHistoryQoS,
    KeepAllHistoryQos,
}

/// 1. This policy controls the behavior of the Service when the value of an instance changes before it is finally
/// communicated to some of its existing DataReader entities.
/// 2. If the kind is set to KEEP_LAST, then the Service will only attempt to keep the latest values of the instance and
/// discard the older ones. In this case, the value of depth regulates the maximum number of values (up to and including
/// the most current one) the Service will maintain and deliver. The default (and most common setting) for depth is one,
/// indicating that only the most recent value should be delivered.
/// 3. If the kind is set to KEEP_ALL, then the Service will attempt to maintain and deliver all the values of the instance to
/// existing subscribers. The resources that the Service can use to keep this history are limited by the settings of the
/// RESOURCE_LIMITS QoS. If the limit is reached, then the behavior of the Service will depend on the
/// RELIABILITY QoS. If the reliability kind is BEST_EFFORT, then the old values will be discarded. If reliability is
/// RELIABLE, then the Service will block the DataWriter until it can deliver the necessary old values to all subscribers.
/// The setting of HISTORY depth must be consistent with the RESOURCE_LIMITS max_samples_per_instance. For these two
/// QoS to be consistent, they must verify that depth <= max_samples_per_instance.
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
            kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
            depth: 1,
        }
    }
}

/// This policy controls the resources that the Service can use in order to meet the requirements imposed by the application and
/// other QoS settings.
/// If the DataWriter objects are communicating samples faster than they are ultimately taken by the DataReader objects, the
/// middleware will eventually hit against some of the QoS-imposed resource limits. Note that this may occur when just a single
/// DataReader cannot keep up with its corresponding DataWriter. The behavior in this case depends on the setting for the
/// RELIABILITY QoS. If reliability is BEST_EFFORT then the Service is allowed to drop samples. If the reliability is
/// RELIABLE, the Service will block the DataWriter or discard the sample at the DataReader in order not to lose existing
/// samples.
/// The constant LENGTH_UNLIMITED may be used to indicate the absence of a particular limit. For example setting
/// max_samples_per_instance to LENGH_UNLIMITED will cause the middleware to not enforce this particular limit.
/// The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
/// values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
/// The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
/// QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
/// An attempt to set this policy to inconsistent values when an entity is created of via a set_qos operation will cause the operation
/// to fail.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimitsQosPolicy {
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

impl QosPolicy for ResourceLimitsQosPolicy {
    fn name(&self) -> &str {
        RESOURCELIMITS_QOS_POLICY_NAME
    }
}

impl Default for ResourceLimitsQosPolicy {
    fn default() -> Self {
        Self {
            max_samples: LENGTH_UNLIMITED,
            max_instances: LENGTH_UNLIMITED,
            max_samples_per_instance: LENGTH_UNLIMITED,
        }
    }
}

impl ResourceLimitsQosPolicy {
    pub fn is_consistent(&self) -> bool {
        let no_sample_limit = self.max_samples == LENGTH_UNLIMITED;
        let samples_per_instance_within_sample_limit = self.max_samples_per_instance
            != LENGTH_UNLIMITED
            && self.max_samples_per_instance <= self.max_samples;

        no_sample_limit || samples_per_instance_within_sample_limit
    }
}

/// This policy controls the behavior of the Entity as a factory for other entities.
/// This policy concerns only DomainParticipant (as factory for Publisher, Subscriber, and Topic), Publisher (as factory for
/// DataWriter), and Subscriber (as factory for DataReader).
/// This policy is mutable. A change in the policy affects only the entities created after the change; not the previously created
/// entities.
/// The setting of autoenable_created_entities to TRUE indicates that the factory create_<entity> operation will automatically
/// invoke the enable operation each time a new Entity is created. Therefore, the Entity returned by create_<entity> will already
/// be enabled. A setting of FALSE indicates that the Entity will not be automatically enabled. The application will need to enable
/// it explicitly by means of the enable operation (see 2.2.2.1.1.7).
/// The default setting of autoenable_created_entities = TRUE means that, by default, it is not necessary to explicitly call enable
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

/// This policy controls the behavior of the DataWriter with regards to the lifecycle of the data-instances it manages, that is, the
/// data-instances that have been either explicitly registered with the DataWriter using the register operations (see 2.2.2.4.2.5 and
///     2.2.2.4.2.6) or implicitly by directly writing the data (see 2.2.2.4.2.11 and 2.2.2.4.2.12).
///     The autodispose_unregistered_instances flag controls the behavior when the DataWriter unregisters an instance by means of
///     the unregister operations (see 2.2.2.4.2.7 and 2.2.2.4.2.8):
///     • The setting ‘autodispose_unregistered_instances = TRUE’ causes the DataWriter to dispose the instance each time it
///     is unregistered. The behavior is identical to explicitly calling one of the dispose operations (2.2.2.4.2.13 and
///     2.2.2.4.2.14) on the instance prior to calling the unregister operation.
///     • The setting ‘autodispose_unregistered_instances = FALSE’ will not cause this automatic disposition upon
///     unregistering. The application can still call one of the dispose operations prior to unregistering the instance and
///     accomplish the same effect. Refer to 2.2.3.23.3 for a description of the consequences of disposing and unregistering
///     instances.
/// Note that the deletion of a DataWriter automatically unregisters all data-instances it manages (2.2.2.4.1.6). Therefore the
/// setting of the autodispose_unregistered_instances flag will determine whether instances are ultimately disposed when the
/// DataWriter is deleted either directly by means of the Publisher::delete_datawriter operation or indirectly as a consequence of
/// calling delete_contained_entities on the Publisher or the DomainParticipant that contains the DataWriter.
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

/// This policy controls the behavior of the DataReader with regards to the lifecycle of the data-instances it manages, that is, the
/// data-instances that have been received and for which the DataReader maintains some internal resources.
/// The DataReader internally maintains the samples that have not been taken by the application, subject to the constraints
/// imposed by other QoS policies such as HISTORY and RESOURCE_LIMITS.
/// The DataReader also maintains information regarding the identity, view_state and instance_state of data-instances even after
/// all samples have been ‘taken.’ This is needed to properly compute the states when future samples arrive.
/// Under normal circumstances the DataReader can only reclaim all resources for instances for which there are no writers and for
/// which all samples have been ‘taken.’ The last sample the DataReader will have taken for that instance will have an
/// instance_state of either NOT_ALIVE_NO_WRITERS or NOT_ALIVE_DISPOSED depending on whether the last writer
/// that had ownership of the instance disposed it or not. Refer to Figure 2.11for a statechart describing the transitions possible for
/// the instance_state. In the absence of the READER_DATA_LIFECYCLE QoS this behavior could cause problems if the
/// application “forgets” to ‘take’ those samples. The ‘untaken’ samples will prevent the DataReader from reclaiming the
/// resources and they would remain in the DataReader indefinitely.
/// The autopurge_nowriter_samples_delay defines the maximum duration for which the DataReader will maintain information
/// regarding an instance once its instance_state becomes NOT_ALIVE_NO_WRITERS. After this time elapses, the DataReader
/// will purge all internal information regarding the instance, any untaken samples will also be lost.
/// The autopurge_disposed_samples_delay defines the maximum duration for which the DataReader will maintain samples for
/// an instance once its instance_state becomes NOT_ALIVE_DISPOSED. After this time elapses, the DataReader will purge all
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

/// This policy is used to configure the HISTORY QoS and the RESOURCE_LIMITS QoS used by the fictitious DataReader and
/// DataWriter used by the “persistence service.” The “persistence service” is the one responsible for implementing the
/// DURABILITY kinds TRANSIENT and PERSISTENCE (see 2.2.3.4).
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct DurabilityServiceQosPolicy {
    pub service_cleanup_delay: Duration,
    pub history_kind: HistoryQosPolicyKind,
    pub history_depth: i32,
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

impl QosPolicy for DurabilityServiceQosPolicy {
    fn name(&self) -> &str {
        DURABILITYSERVICE_POLICY_NAME
    }
}

impl Default for DurabilityServiceQosPolicy {
    fn default() -> Self {
        Self {
            service_cleanup_delay: DURATION_ZERO,
            history_kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
            history_depth: 1,
            max_samples: LENGTH_UNLIMITED,
            max_instances: LENGTH_UNLIMITED,
            max_samples_per_instance: LENGTH_UNLIMITED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durability_qos_policy_kind_ordering() {
        assert!(
            DurabilityQosPolicyKind::VolatileDurabilityQoS
                < DurabilityQosPolicyKind::TransientLocalDurabilityQoS
        );

        assert!(
            DurabilityQosPolicyKind::VolatileDurabilityQoS
                == DurabilityQosPolicyKind::VolatileDurabilityQoS
        );
        assert!(
            DurabilityQosPolicyKind::VolatileDurabilityQoS
                < DurabilityQosPolicyKind::TransientLocalDurabilityQoS
        );

        assert!(
            DurabilityQosPolicyKind::TransientLocalDurabilityQoS
                > DurabilityQosPolicyKind::VolatileDurabilityQoS
        );
        assert!(
            DurabilityQosPolicyKind::TransientLocalDurabilityQoS
                == DurabilityQosPolicyKind::TransientLocalDurabilityQoS
        );
    }

    #[test]
    fn presentation_qos_policy_access_scope_kind_ordering() {
        assert!(
            PresentationQosPolicyAccessScopeKind::InstancePresentationQoS
                < PresentationQosPolicyAccessScopeKind::TopicPresentationQoS
        );

        assert!(
            PresentationQosPolicyAccessScopeKind::InstancePresentationQoS
                == PresentationQosPolicyAccessScopeKind::InstancePresentationQoS
        );
        assert!(
            PresentationQosPolicyAccessScopeKind::InstancePresentationQoS
                < PresentationQosPolicyAccessScopeKind::TopicPresentationQoS
        );

        assert!(
            PresentationQosPolicyAccessScopeKind::TopicPresentationQoS
                > PresentationQosPolicyAccessScopeKind::InstancePresentationQoS
        );
        assert!(
            PresentationQosPolicyAccessScopeKind::TopicPresentationQoS
                == PresentationQosPolicyAccessScopeKind::TopicPresentationQoS
        );
    }

    #[test]
    fn liveliness_qos_policy_kind_ordering() {
        assert!(
            LivelinessQosPolicyKind::AutomaticLivelinessQoS
                < LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
                < LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
        );

        assert!(
            LivelinessQosPolicyKind::AutomaticLivelinessQoS
                == LivelinessQosPolicyKind::AutomaticLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::AutomaticLivelinessQoS
                < LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::AutomaticLivelinessQoS
                < LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
        );

        assert!(
            LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
                > LivelinessQosPolicyKind::AutomaticLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
                == LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
                < LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
        );

        assert!(
            LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
                > LivelinessQosPolicyKind::AutomaticLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
                > LivelinessQosPolicyKind::ManualByParticipantLivelinessQoS
        );
        assert!(
            LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
                == LivelinessQosPolicyKind::ManualByTopicLivelinessQoS
        );
    }

    #[test]
    fn reliability_qos_policy_kind_ordering() {
        assert!(
            ReliabilityQosPolicyKind::BestEffortReliabilityQos
                < ReliabilityQosPolicyKind::ReliableReliabilityQos
        );

        assert!(
            ReliabilityQosPolicyKind::BestEffortReliabilityQos
                == ReliabilityQosPolicyKind::BestEffortReliabilityQos
        );
        assert!(
            ReliabilityQosPolicyKind::BestEffortReliabilityQos
                < ReliabilityQosPolicyKind::ReliableReliabilityQos
        );

        assert!(
            ReliabilityQosPolicyKind::ReliableReliabilityQos
                > ReliabilityQosPolicyKind::BestEffortReliabilityQos
        );
        assert!(
            ReliabilityQosPolicyKind::ReliableReliabilityQos
                == ReliabilityQosPolicyKind::ReliableReliabilityQos
        );
    }

    #[test]
    fn destination_order_qos_policy_kind_ordering() {
        assert!(
            DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS
                < DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
        );

        assert!(
            DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS
                == DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS
        );
        assert!(
            DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS
                < DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
        );

        assert!(
            DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
                > DestinationOrderQosPolicyKind::ByReceptionTimestampDestinationOrderQoS
        );
        assert!(
            DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
                == DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
        );
    }

    #[test]
    fn resource_limit_consistency() {
        let mut resource_limits = ResourceLimitsQosPolicy {
            max_samples: LENGTH_UNLIMITED,
            max_instances: LENGTH_UNLIMITED,
            max_samples_per_instance: LENGTH_UNLIMITED,
        };
        assert_eq!(resource_limits.is_consistent(), true);

        resource_limits.max_samples = 5;
        assert_eq!(resource_limits.is_consistent(), false); // Inconsistent: Max samples per instance bigger than max samples

        resource_limits.max_samples_per_instance = 5;
        assert_eq!(resource_limits.is_consistent(), true);

        resource_limits.max_samples = 4;
        assert_eq!(resource_limits.is_consistent(), false); // Inconsistent: Max samples per instance bigger than max samples

        resource_limits.max_samples = LENGTH_UNLIMITED;
        assert_eq!(resource_limits.is_consistent(), true);

        assert_eq!(ResourceLimitsQosPolicy::default().is_consistent(), true);
    }
}
