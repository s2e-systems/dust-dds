use dust_dds_derive::{ParameterListDeserialize, ParameterListSerialize};
use xtypes::{deserializer::DeserializeMutableStruct, serializer::SerializeMutableStruct};

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::{error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER},
    rtps::types::{EntityId, Guid, Locator},
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
    PID_DURABILITY, PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET,
    PID_LIFESPAN, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH,
    PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA,
    PID_TOPIC_NAME, PID_TYPE_NAME, PID_TYPE_REPRESENTATION, PID_UNICAST_LOCATOR, PID_USER_DATA,
};
#[derive(Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize)]
pub struct WriterProxy {
    #[parameter(id = PID_ENDPOINT_GUID, skip_serialize)]
    remote_writer_guid: Guid,
    #[parameter(id = PID_GROUP_ENTITYID, default=Default::default())]
    remote_group_entity_id: EntityId,
    #[parameter(id = PID_UNICAST_LOCATOR, collection)]
    unicast_locator_list: Vec<Locator>,
    #[parameter(id = PID_MULTICAST_LOCATOR, collection)]
    multicast_locator_list: Vec<Locator>,
    #[parameter(id = PID_DATA_MAX_SIZE_SERIALIZED, default=Default::default())]
    data_max_size_serialized: i32,
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            remote_writer_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized: data_max_size_serialized.unwrap_or_default(),
        }
    }

    pub fn remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
    }

    pub fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_ref()
    }

    pub fn data_max_size_serialized(&self) -> Option<i32> {
        Some(self.data_max_size_serialized)
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    ParameterListSerialize,
    ParameterListDeserialize,
    DdsSerialize,
    DdsDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct DiscoveredWriterData {
    dds_publication_data: PublicationBuiltinTopicData,
    writer_proxy: WriterProxy,
}
impl xtypes::serialize::XTypesSerialize for DiscoveredWriterData {
    fn serialize(
        &self,
        serializer: impl xtypes::serialize::XTypesSerializer,
    ) -> Result<(), xtypes::error::XcdrError> {
        let mut p = serializer.serialize_mutable_struct()?;
        p.serialize_field(
            &self.dds_publication_data.key,
            PID_ENDPOINT_GUID as u16,
            "key",
        )?;
        if self.dds_publication_data.participant_key != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.participant_key,
                PID_PARTICIPANT_GUID as u16,
                "participant_key",
            )?;
        }
        p.serialize_field(
            &self.dds_publication_data.topic_name.as_str(),
            PID_TOPIC_NAME as u16,
            "topic_name",
        )?;
        p.serialize_field(
            &self.dds_publication_data.type_name.as_str(),
            PID_TYPE_NAME as u16,
            "type_name",
        )?;
        if self.dds_publication_data.durability != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.durability,
                PID_DURABILITY as u16,
                "durability",
            )?;
        }
        if self.dds_publication_data.deadline != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.deadline,
                PID_DEADLINE as u16,
                "deadline",
            )?;
        }
        if self.dds_publication_data.latency_budget != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.latency_budget,
                PID_LATENCY_BUDGET as u16,
                "latency_budget",
            )?;
        }
        if self.dds_publication_data.liveliness != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.liveliness,
                PID_LIVELINESS as u16,
                "liveliness",
            )?;
        }
        if self.dds_publication_data.reliability != DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER {
            p.serialize_field(
                &self.dds_publication_data.reliability,
                PID_RELIABILITY as u16,
                "reliability",
            )?;
        }
        if self.dds_publication_data.lifespan != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.lifespan,
                PID_LIFESPAN as u16,
                "lifespan",
            )?;
        }
        if self.dds_publication_data.user_data != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.user_data,
                PID_USER_DATA as u16,
                "user_data",
            )?;
        }
        if self.dds_publication_data.ownership != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.ownership,
                PID_OWNERSHIP as u16,
                "ownership",
            )?;
        }
        if self.dds_publication_data.ownership_strength != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.ownership_strength,
                PID_OWNERSHIP_STRENGTH as u16,
                "ownership_strength",
            )?;
        }
        if self.dds_publication_data.destination_order != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.destination_order,
                PID_DESTINATION_ORDER as u16,
                "destination_order",
            )?;
        }
        if self.dds_publication_data.presentation != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.presentation,
                PID_PRESENTATION as u16,
                "presentation",
            )?;
        }
        if self.dds_publication_data.partition != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.partition,
                PID_PARTITION as u16,
                "partition",
            )?;
        }
        if self.dds_publication_data.topic_data != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.topic_data,
                PID_TOPIC_DATA as u16,
                "topic_data",
            )?;
        }
        if self.dds_publication_data.group_data != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.group_data,
                PID_GROUP_DATA as u16,
                "group_data",
            )?;
        }
        if !self.dds_publication_data.xml_type.is_empty() {
            p.serialize_field(
                &self.dds_publication_data.xml_type.as_str(),
                PID_TYPE_REPRESENTATION as u16,
                "xml_type",
            )?;
        }
        if self.dds_publication_data.representation != Default::default() {
            p.serialize_field(
                &self.dds_publication_data.representation,
                PID_DATA_REPRESENTATION as u16,
                "representation",
            )?;
        }

        //skip_serialize : self.writer_proxy.remote_writer_guid

        if self.writer_proxy.remote_group_entity_id != Default::default() {
            p.serialize_field(
                &self.writer_proxy.remote_group_entity_id,
                PID_GROUP_ENTITYID as u16,
                "remote_group_entity_id",
            )?;
        }
        for unicast_locator in &self.writer_proxy.unicast_locator_list {
            p.serialize_field(
                unicast_locator,
                PID_UNICAST_LOCATOR as u16,
                "unicast_locator_list",
            )?;
        }
        for multicast_locator in &self.writer_proxy.multicast_locator_list {
            p.serialize_field(
                multicast_locator,
                PID_MULTICAST_LOCATOR as u16,
                "multicast_locator_list",
            )?;
        }
        if self.writer_proxy.data_max_size_serialized != 0 {
            p.serialize_field(
                &self.writer_proxy.data_max_size_serialized,
                PID_DATA_MAX_SIZE_SERIALIZED as u16,
                "data_max_size_serialized",
            )?;
        }

        p.end()
    }
}

impl<'de> xtypes::deserialize::XTypesDeserialize<'de> for DiscoveredWriterData {
    fn deserialize(
        deserializer: impl xtypes::deserializer::XTypesDeserializer<'de>,
    ) -> Result<Self, xtypes::error::XcdrError> {
        let mut m = deserializer.deserialize_mutable_struct()?;
        Ok(Self {
            dds_publication_data: PublicationBuiltinTopicData {
                key: m.deserialize_field(PID_ENDPOINT_GUID as u16, "key")?,
                participant_key: m
                    .deserialize_optional_field(PID_PARTICIPANT_GUID as u16, "participant_key")?
                    .unwrap_or_default(),
                topic_name: m
                    .deserialize_field::<&str>(PID_TOPIC_NAME as u16, "topic_name")?
                    .to_owned(),
                type_name: m
                    .deserialize_field::<&str>(PID_TYPE_NAME as u16, "type_name")?
                    .to_owned(),
                durability: m
                    .deserialize_optional_field(PID_DURABILITY as u16, "durability")?
                    .unwrap_or_default(),
                deadline: m
                    .deserialize_optional_field(PID_DEADLINE as u16, "deadline")?
                    .unwrap_or_default(),
                latency_budget: m
                    .deserialize_optional_field(PID_LATENCY_BUDGET as u16, "latency_budget")?
                    .unwrap_or_default(),
                liveliness: m
                    .deserialize_optional_field(PID_LIVELINESS as u16, "liveliness")?
                    .unwrap_or_default(),
                reliability: m
                    .deserialize_optional_field(PID_RELIABILITY as u16, "reliability")?
                    .unwrap_or(DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER),
                lifespan: m
                    .deserialize_optional_field(PID_LIFESPAN as u16, "lifespan")?
                    .unwrap_or_default(),
                user_data: m
                    .deserialize_optional_field(PID_USER_DATA as u16, "user_data")?
                    .unwrap_or_default(),
                ownership: m
                    .deserialize_optional_field(PID_OWNERSHIP as u16, "ownership")?
                    .unwrap_or_default(),
                ownership_strength: m
                    .deserialize_optional_field(
                        PID_OWNERSHIP_STRENGTH as u16,
                        "ownership_strength",
                    )?
                    .unwrap_or_default(),
                destination_order: m
                    .deserialize_optional_field(PID_DESTINATION_ORDER as u16, "destination_order")?
                    .unwrap_or_default(),
                presentation: m
                    .deserialize_optional_field(PID_PRESENTATION as u16, "presentation")?
                    .unwrap_or_default(),
                partition: m
                    .deserialize_optional_field(PID_PARTITION as u16, "partition")?
                    .unwrap_or_default(),
                topic_data: m
                    .deserialize_optional_field(PID_TOPIC_DATA as u16, "topic_data")?
                    .unwrap_or_default(),
                group_data: m
                    .deserialize_optional_field(PID_GROUP_DATA as u16, "group_data")?
                    .unwrap_or_default(),
                xml_type: m
                    .deserialize_optional_field::<&str>(PID_TYPE_REPRESENTATION as u16, "xml_type")?
                    .unwrap_or_default()
                    .to_owned(),
                representation: m
                    .deserialize_optional_field(PID_DATA_REPRESENTATION as u16, "representation")?
                    .unwrap_or_default(),
            },
            writer_proxy: WriterProxy {
                remote_writer_guid: m
                    .deserialize_field(PID_ENDPOINT_GUID as u16, "remote_writer_guid")?,
                remote_group_entity_id: m
                    .deserialize_optional_field(
                        PID_GROUP_ENTITYID as u16,
                        "remote_group_entity_id",
                    )?
                    .unwrap_or_default(),
                unicast_locator_list: m
                    .deserialize_list_field(PID_UNICAST_LOCATOR as u16, "unicast_locator_list")
                    .collect(),
                multicast_locator_list: m
                    .deserialize_list_field(PID_MULTICAST_LOCATOR as u16, "multicast_locator_list")
                    .collect(),
                data_max_size_serialized: m
                    .deserialize_optional_field(
                        PID_DATA_MAX_SIZE_SERIALIZED as u16,
                        "data_max_size_serialized",
                    )?
                    .unwrap_or_default(),
            },
        })
    }
}

impl DiscoveredWriterData {
    pub fn new(
        dds_publication_data: PublicationBuiltinTopicData,
        writer_proxy: WriterProxy,
    ) -> Self {
        Self {
            dds_publication_data,
            writer_proxy,
        }
    }

    pub fn dds_publication_data(&self) -> &PublicationBuiltinTopicData {
        &self.dds_publication_data
    }

    pub fn writer_proxy(&self) -> &WriterProxy {
        &self.writer_proxy
    }
}

pub const DCPS_PUBLICATION: &str = "DCPSPublication";

impl DdsHasKey for DiscoveredWriterData {
    const HAS_KEY: bool = true;
}

impl DdsKey for DiscoveredWriterData {
    type Key = [u8; 16];

    fn get_key(&self) -> DdsResult<Self::Key> {
        Ok(self.dds_publication_data.key().value)
    }

    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> DdsResult<Self::Key> {
        Ok(Self::deserialize_data(serialized_foo)?
            .dds_publication_data
            .key()
            .value)
    }
}

impl DdsTypeXml for DiscoveredWriterData {
    fn get_type_xml() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        infrastructure::{
            qos::{DataWriterQos, PublisherQos},
            qos_policy::TopicDataQosPolicy,
        },
        rtps::types::{
            BUILT_IN_PARTICIPANT, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_WITH_KEY,
            USER_DEFINED_UNKNOWN,
        },
    };
    use xtypes::{
        deserialize::XTypesDeserialize, error::XcdrError, serialize::XTypesSerialize,
        xcdr_deserializer::Xcdr1LeDeserializer, xcdr_serializer::Xcdr1LeSerializer,
    };

    fn serialize_v1_le<T: XTypesSerialize, const N: usize>(v: &T) -> Result<[u8; N], XcdrError> {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr1LeSerializer::new(&mut buffer))?;
        Ok(buffer)
    }

    #[test]
    fn xtypes_serialize_all_default() {
        let data = DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DataWriterQos::default(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::default(),
            ),
            WriterProxy::new(
                Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], BUILT_IN_WRITER_WITH_KEY),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_READER_GROUP),
                vec![],
                vec![],
                None,
            ),
        );

        let expected = Ok([
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x07, 0x00, // PID_TOPIC_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x07, 0x00, // PID_TYPE_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ]);
        assert_eq!(serialize_v1_le(&data), expected);
    }

    #[test]
    fn xtypes_deserialize_all_default() {
        let expected = Ok(DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DataWriterQos::default(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::default(),
            ),
            WriterProxy::new(
                // must correspond to publication_builtin_topic_data.key
                Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_PARTICIPANT),
                vec![],
                vec![],
                None,
            ),
        ));

        let data = [
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc1, // u8[3], u8
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x07, 0x00, // PID_TOPIC_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x07, 0x00, // PID_TYPE_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(
            XTypesDeserialize::deserialize(&mut Xcdr1LeDeserializer::new(&data)),
            expected
        );
    }

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DataWriterQos::default(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::default(),
            ),
            WriterProxy::new(
                Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], BUILT_IN_WRITER_WITH_KEY),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_READER_GROUP),
                vec![],
                vec![],
                None,
            ),
        );

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DataWriterQos::default(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::default(),
            ),
            WriterProxy::new(
                // must correspond to publication_builtin_topic_data.key
                Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_PARTICIPANT),
                vec![],
                vec![],
                None,
            ),
        );

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc1, // u8[3], u8
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result = DiscoveredWriterData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
