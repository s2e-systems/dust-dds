use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
    PID_DURABILITY, PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET,
    PID_LIFESPAN, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH,
    PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA,
    PID_TOPIC_NAME, PID_TYPE_NAME, PID_TYPE_REPRESENTATION, PID_UNICAST_LOCATOR, PID_USER_DATA,
};
use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::payload_serializer_deserializer::parameter_list_serializer::ParameterListCdrSerializer,
    infrastructure::{error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER},
    rtps::types::{EntityId, Guid, Locator},
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};
use dust_dds_derive::ParameterListDeserialize;
#[derive(Debug, PartialEq, Eq, Clone, ParameterListDeserialize)]
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

impl dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize
    for WriterProxy
{
    fn serialize(&self, serializer: &mut ParameterListCdrSerializer) -> Result<(), std::io::Error> {
        serializer.write_with_default(
            PID_GROUP_ENTITYID,
            &self.remote_group_entity_id,
            &Default::default(),
        )?;
        serializer.write_collection(PID_UNICAST_LOCATOR, &self.unicast_locator_list)?;
        serializer.write_collection(PID_MULTICAST_LOCATOR, &self.multicast_locator_list)?;
        serializer.write_with_default(
            PID_DATA_MAX_SIZE_SERIALIZED,
            &self.data_max_size_serialized,
            &Default::default(),
        )?;
        Ok(())
    }
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

#[derive(Debug, PartialEq, Eq, Clone, ParameterListDeserialize, DdsDeserialize)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct DiscoveredWriterData {
    dds_publication_data: PublicationBuiltinTopicData,
    writer_proxy: WriterProxy,
}

impl DdsSerialize for DiscoveredWriterData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::new();
        serializer.write_header()?;

        // dds_publication_data: PublicationBuiltinTopicData:

        serializer.write(PID_ENDPOINT_GUID, &self.dds_publication_data.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.write_with_default(
            PID_PARTICIPANT_GUID,
            &self.dds_publication_data.participant_key,
            &Default::default(),
        )?;
        serializer.write(PID_TOPIC_NAME, &self.dds_publication_data.topic_name)?;
        serializer.write(PID_TYPE_NAME, &self.dds_publication_data.type_name)?;
        serializer.write_with_default(
            PID_DURABILITY,
            &self.dds_publication_data.durability,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DEADLINE,
            &self.dds_publication_data.deadline,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.dds_publication_data.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIVELINESS,
            &self.dds_publication_data.liveliness,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.dds_publication_data.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        )?;
        serializer.write_with_default(
            PID_LIFESPAN,
            &self.dds_publication_data.lifespan,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_USER_DATA,
            &self.dds_publication_data.user_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP,
            &self.dds_publication_data.ownership,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP_STRENGTH,
            &self.dds_publication_data.ownership_strength,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.dds_publication_data.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PRESENTATION,
            &self.dds_publication_data.presentation,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PARTITION,
            &self.dds_publication_data.partition,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TOPIC_DATA,
            &self.dds_publication_data.topic_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_GROUP_DATA,
            &self.dds_publication_data.group_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TYPE_REPRESENTATION,
            &self.dds_publication_data.xml_type,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.dds_publication_data.representation,
            &Default::default(),
        )?;

        // writer_proxy: WriterProxy:

        // skip serilize:
        // writer_proxy.remote_writer_guid: Guid,

        serializer.write_with_default(
            PID_GROUP_ENTITYID,
            &self.writer_proxy.remote_group_entity_id,
            &Default::default(),
        )?;
        serializer
            .write_collection(PID_UNICAST_LOCATOR, &self.writer_proxy.unicast_locator_list)?;
        serializer.write_collection(
            PID_MULTICAST_LOCATOR,
            &self.writer_proxy.multicast_locator_list,
        )?;
        serializer.write_with_default(
            PID_DATA_MAX_SIZE_SERIALIZED,
            &self.writer_proxy.data_max_size_serialized,
            &Default::default(),
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
}

impl dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize
    for DiscoveredWriterData
{
    fn serialize(&self, serializer: &mut ParameterListCdrSerializer) -> Result<(), std::io::Error> {
        dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize::serialize(
            &self.dds_publication_data,
            serializer,
        )?;
        dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize::serialize(
            &self.writer_proxy,
            serializer,
        )?;
        Ok(())
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
        // assert_eq!(serialize_v1_le(&data), expected);
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
