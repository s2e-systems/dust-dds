use rust_dds_api::{
    dcps_psm::{BuiltInTopicKey, Duration},
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DestinationOrderQosPolicyKind,
        DurabilityQosPolicy, DurabilityQosPolicyKind, DurabilityServiceQosPolicy,
        GroupDataQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind, LatencyBudgetQosPolicy,
        LifespanQosPolicy, LivelinessQosPolicy, LivelinessQosPolicyKind, OwnershipQosPolicy,
        OwnershipQosPolicyKind, OwnershipStrengthQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, PresentationQosPolicyAccessScopeKind, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind, ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy,
        TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
    },
};
use rust_rtps_psm::discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet};
use serde::Serialize;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "UserDataQosPolicy")]
pub struct UserDataQosPolicyDef {
    pub value: Vec<u8>,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct UserDataQosPolicySerialize<'a>(
    #[serde(with = "UserDataQosPolicyDef")] pub &'a UserDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct UserDataQosPolicyDeserialize(
    #[serde(with = "UserDataQosPolicyDef")] pub UserDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "BuiltinEndpointSet")]
pub struct BuiltinEndpointSetDef(pub u32);

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltinEndpointSetSerdeSerialize<'a>(
    #[serde(with = "BuiltinEndpointSetDef")] pub &'a BuiltinEndpointSet,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltinEndpointSetSerdeDeserialize(
    #[serde(with = "BuiltinEndpointSetDef")] pub BuiltinEndpointSet,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "BuiltinEndpointQos")]
pub struct BuiltinEndpointQosDef(pub u32);

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltinEndpointQosSerdeSerialize<'a>(
    #[serde(with = "BuiltinEndpointQosDef")] pub &'a BuiltinEndpointQos,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltinEndpointQosSerdeDeserialize(
    #[serde(with = "BuiltinEndpointQosDef")] pub BuiltinEndpointQos,
);

type BuiltInTopicKeyTypeNative = i32;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "BuiltInTopicKey")]
pub struct BuiltInTopicKeyDef {
    pub value: [BuiltInTopicKeyTypeNative; 4],
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltInTopicKeySerialize<'a>(
    #[serde(with = "BuiltInTopicKeyDef")] pub &'a BuiltInTopicKey,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltInTopicKeyDeserialize(#[serde(with = "BuiltInTopicKeyDef")] pub BuiltInTopicKey);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DurabilityQosPolicyKind")]
pub enum DurabilityQosPolicyKindDef {
    VolatileDurabilityQoS,
    TransientLocalDurabilityQoS,
    TransientDurabilityQoS,
    PersistentDurabilityQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DurabilityQosPolicy")]
pub struct DurabilityQosPolicyDef {
    #[serde(with = "DurabilityQosPolicyKindDef")]
    pub kind: DurabilityQosPolicyKind,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DurabilityQosPolicySerialize<'a>(
    #[serde(with = "DurabilityQosPolicyDef")] pub &'a DurabilityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DurabilityQosPolicyDeserialize(
    #[serde(with = "DurabilityQosPolicyDef")] pub DurabilityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "HistoryQosPolicyKind")]
pub enum HistoryQosPolicyKindDef {
    KeepLastHistoryQoS,
    KeepAllHistoryQos,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Duration")]
pub struct DcpsDurationDef {
    #[serde(getter = "Duration::sec")]
    sec: i32,
    #[serde(getter = "Duration::nanosec")]
    nanosec: u32,
}
impl From<DcpsDurationDef> for Duration {
    fn from(def: DcpsDurationDef) -> Duration {
        Duration::new(def.sec, def.nanosec)
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DurabilityServiceQosPolicy")]
pub struct DurabilityServiceQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub service_cleanup_delay: Duration,
    #[serde(with = "HistoryQosPolicyKindDef")]
    pub history_kind: HistoryQosPolicyKind,
    pub history_depth: i32,
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DurabilityServiceQosPolicySerialize<'a>(
    #[serde(with = "DurabilityServiceQosPolicyDef")] pub &'a DurabilityServiceQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DurabilityServiceQosPolicyDeserialize(
    #[serde(with = "DurabilityServiceQosPolicyDef")] pub DurabilityServiceQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DeadlineQosPolicy")]
pub struct DeadlineQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub period: Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DeadlineQosPolicySerialize<'a>(
    #[serde(with = "DeadlineQosPolicyDef")] pub &'a DeadlineQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DeadlineQosPolicyDeserialize(
    #[serde(with = "DeadlineQosPolicyDef")] pub DeadlineQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LatencyBudgetQosPolicy")]
pub struct LatencyBudgetQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub duration: Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LatencyBudgetQosPolicySerialize<'a>(
    #[serde(with = "LatencyBudgetQosPolicyDef")] pub &'a LatencyBudgetQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LatencyBudgetQosPolicyDeserialize(
    #[serde(with = "LatencyBudgetQosPolicyDef")] pub LatencyBudgetQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LivelinessQosPolicyKind")]
pub enum LivelinessQosPolicyKindDef {
    AutomaticLivelinessQoS,
    ManualByParticipantLivelinessQoS,
    ManualByTopicLivelinessQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LivelinessQosPolicy")]
pub struct LivelinessQosPolicyDef {
    #[serde(with = "LivelinessQosPolicyKindDef")]
    pub kind: LivelinessQosPolicyKind,
    #[serde(with = "DcpsDurationDef")]
    pub lease_duration: Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LivelinessQosPolicySerialize<'a>(
    #[serde(with = "LivelinessQosPolicyDef")] pub &'a LivelinessQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LivelinessQosPolicyDeserialize(
    #[serde(with = "LivelinessQosPolicyDef")] pub LivelinessQosPolicy,
);

#[derive(Debug, PartialEq)]
pub enum ReliabilityQosPolicyKindDef {
    BestEffortReliabilityQos,
    ReliableReliabilityQos,
}

const BEST_EFFORT: i32 = 1;
const RELIABLE: i32 = 2;
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
            _ => return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(value as u64), &self)),
        })
    }
}

impl<'de> ReliabilityQosPolicyKindDef {
    pub fn serialize<S>(this: &ReliabilityQosPolicyKind, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match this {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => BEST_EFFORT,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => RELIABLE,
        }
        .serialize(serializer)
    }

    pub fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ReliabilityQosPolicyKind, D::Error> {
        deserializer.deserialize_i32(ReliabilityQosPolicyKindVisitor)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(remote = "ReliabilityQosPolicy")]
pub struct ReliabilityQosPolicyDef {
    #[serde(with = "ReliabilityQosPolicyKindDef")]
    pub kind: ReliabilityQosPolicyKind,
    #[serde(with = "DcpsDurationDef")]
    pub max_blocking_time: Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ReliabilityQosPolicySerialize<'a>(
    #[serde(with = "ReliabilityQosPolicyDef")] pub &'a ReliabilityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct ReliabilityQosPolicyDeserialize(#[serde(with = "ReliabilityQosPolicyDef")] pub ReliabilityQosPolicy);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "TransportPriorityQosPolicy")]
pub struct TransportPriorityQosPolicyDef {
    pub value: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TransportPriorityQosPolicySerialize<'a>(
    #[serde(with = "TransportPriorityQosPolicyDef")] pub &'a TransportPriorityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct TransportPriorityQosPolicyDeserialize(
    #[serde(with = "TransportPriorityQosPolicyDef")] pub TransportPriorityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LifespanQosPolicy")]
pub struct LifespanQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub duration: Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LifespanQosPolicySerialize<'a>(
    #[serde(with = "LifespanQosPolicyDef")] pub &'a LifespanQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LifespanQosPolicyDeserialize(
    #[serde(with = "LifespanQosPolicyDef")] pub LifespanQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DestinationOrderQosPolicyKind")]
pub enum DestinationOrderQosPolicyKindDef {
    ByReceptionTimestampDestinationOrderQoS,
    BySourceTimestampDestinationOrderQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DestinationOrderQosPolicy")]
pub struct DestinationOrderQosPolicyDef {
    #[serde(with = "DestinationOrderQosPolicyKindDef")]
    pub kind: DestinationOrderQosPolicyKind,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DestinationOrderQosPolicySerialize<'a>(
    #[serde(with = "DestinationOrderQosPolicyDef")] pub &'a DestinationOrderQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DestinationOrderQosPolicyDeserialize(
    #[serde(with = "DestinationOrderQosPolicyDef")] pub DestinationOrderQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "HistoryQosPolicy")]
pub struct HistoryQosPolicyDef {
    #[serde(with = "HistoryQosPolicyKindDef")]
    pub kind: HistoryQosPolicyKind,
    pub depth: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct HistoryQosPolicySerialize<'a>(
    #[serde(with = "HistoryQosPolicyDef")] pub &'a HistoryQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct HistoryQosPolicyDeserialize(#[serde(with = "HistoryQosPolicyDef")] pub HistoryQosPolicy);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "ResourceLimitsQosPolicy")]
pub struct ResourceLimitsQosPolicyDef {
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ResourceLimitsQosPolicySerialize<'a>(
    #[serde(with = "ResourceLimitsQosPolicyDef")] pub &'a ResourceLimitsQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct ResourceLimitsQosPolicyDeserialize(
    #[serde(with = "ResourceLimitsQosPolicyDef")] pub ResourceLimitsQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "OwnershipQosPolicyKind")]
pub enum OwnershipQosPolicyKindDef {
    SharedOwnershipQoS,
    ExclusiveOwnershipQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "OwnershipQosPolicy")]
pub struct OwnershipQosPolicyDef {
    #[serde(with = "OwnershipQosPolicyKindDef")]
    pub kind: OwnershipQosPolicyKind,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct OwnershipQosPolicySerialize<'a>(
    #[serde(with = "OwnershipQosPolicyDef")] pub &'a OwnershipQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct OwnershipQosPolicyDeserialize(
    #[serde(with = "OwnershipQosPolicyDef")] pub OwnershipQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "TopicDataQosPolicy")]
pub struct TopicDataQosPolicyDef {
    pub value: Vec<u8>,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TopicDataQosPolicySerialize<'a>(
    #[serde(with = "TopicDataQosPolicyDef")] pub &'a TopicDataQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct TopicDataQosPolicyDeserialize(
    #[serde(with = "TopicDataQosPolicyDef")] pub TopicDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "OwnershipStrengthQosPolicy")]
pub struct OwnershipStrengthQosPolicyDef {
    pub value: i32,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct OwnershipStrengthQosPolicySerialize<'a>(
    #[serde(with = "OwnershipStrengthQosPolicyDef")] pub &'a OwnershipStrengthQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct OwnershipStrengthQosPolicyDeserialize(
    #[serde(with = "OwnershipStrengthQosPolicyDef")] pub OwnershipStrengthQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "PresentationQosPolicyAccessScopeKind")]
pub enum PresentationQosPolicyAccessScopeKindDef {
    InstancePresentationQoS,
    TopicPresentationQoS,
    GroupPresentationQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "PresentationQosPolicy")]
pub struct PresentationQosPolicyDef {
    #[serde(with = "PresentationQosPolicyAccessScopeKindDef")]
    pub access_scope: PresentationQosPolicyAccessScopeKind,
    pub coherent_access: bool,
    pub ordered_access: bool,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PresentationQosPolicySerialize<'a>(
    #[serde(with = "PresentationQosPolicyDef")] pub &'a PresentationQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PresentationQosPolicyDeserialize(
    #[serde(with = "PresentationQosPolicyDef")] pub PresentationQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "PartitionQosPolicy")]
pub struct PartitionQosPolicyDef {
    pub name: String,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PartitionQosPolicySerialize<'a>(
    #[serde(with = "PartitionQosPolicyDef")] pub &'a PartitionQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PartitionQosPolicyDeserialize(
    #[serde(with = "PartitionQosPolicyDef")] pub PartitionQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "GroupDataQosPolicy")]
pub struct GroupDataQosPolicyDef {
    pub value: Vec<u8>,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct GroupDataQosPolicySerialize<'a>(
    #[serde(with = "GroupDataQosPolicyDef")] pub &'a GroupDataQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct GroupDataQosPolicyDeserialize(
    #[serde(with = "GroupDataQosPolicyDef")] pub GroupDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "TimeBasedFilterQosPolicy")]
pub struct TimeBasedFilterQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub minimum_separation: Duration,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TimeBasedFilterQosPolicySerialize<'a>(
    #[serde(with = "TimeBasedFilterQosPolicyDef")] pub &'a TimeBasedFilterQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct TimeBasedFilterQosPolicyDeserialize(
    #[serde(with = "TimeBasedFilterQosPolicyDef")] pub TimeBasedFilterQosPolicy,
);



#[cfg(test)]
mod tests {
    use super::*;
    fn to_bytes_le<S: serde::Serialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        let mut serializer = cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut writer);
        serde::Serialize::serialize(value, &mut serializer).unwrap();
        writer
    }
    fn from_bytes_le<'de, D: serde::Deserialize<'de>>(buf: &'de [u8]) -> D {
        let mut deserializer =
            cdr::de::Deserializer::<_, _, byteorder::LittleEndian>::new(buf, cdr::Infinite);
        serde::Deserialize::deserialize(&mut deserializer).unwrap()
    }

    #[test]
    fn serialize_reliability_qos_policy_def() {
        let value = ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: Duration::new(0, 100),
        };
        let result = to_bytes_le(&ReliabilityQosPolicySerialize(&value));
        assert_eq!(
            result,
            vec![
                2, 0, 0, 0, // kind
                0, 0, 0, 0, // max_blocking_time:  sec: i32,
                100, 0, 0, 0, // max_blocking_time:  nanosec: u32,
            ]
        );
    }
    #[test]
    fn deserialize_reliability_qos_policy_def() {
        let data = &[
            2u8, 0, 0, 0, // kind
            0, 0, 0, 0, // max_blocking_time:  sec: i32,
            100, 0, 0, 0, // max_blocking_time:  nanosec: u32,
        ];
        let expected = ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: Duration::new(0, 100),
        };
        let result: ReliabilityQosPolicyDeserialize = from_bytes_le(data);
        assert_eq!(result.0, expected);
    }
}