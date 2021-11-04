use std::io::Write;

use rust_dds_api::{builtin_topics::PublicationBuiltinTopicData, return_type::DDSResult};
use rust_rtps_pim::{behavior::reader::writer_proxy::RtpsWriterProxy, structure::types::Locator};

use crate::{data_serialize_deserialize::ParameterSerializer, dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness}};

use super::{dds_serialize_deserialize_impl::BuiltInTopicKeySerialize, parameter_id_values::{PID_ENDPOINT_GUID, PID_PARTICIPANT_GUID}};

pub struct SedpDiscoveredWriterData {
    pub writer_proxy: RtpsWriterProxy<Vec<Locator>>,
    pub publication_builtin_topic_data: PublicationBuiltinTopicData,
}

impl DdsType for SedpDiscoveredWriterData {
    fn type_name() -> &'static str {
        "SedpDiscoveredWriterData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredWriterData {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DDSResult<()> {
        let mut parameter_list_serializer = ParameterSerializer::<_, E>::new(writer);

        parameter_list_serializer
            .serialize_parameter(PID_ENDPOINT_GUID, &BuiltInTopicKeySerialize(&self.publication_builtin_topic_data.key))
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(PID_PARTICIPANT_GUID, &BuiltInTopicKeySerialize(&self.publication_builtin_topic_data.participant_key))
            .unwrap();
        Ok(())
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredWriterData {
    fn deserialize(_buf: &mut &'_ [u8]) -> DDSResult<Self> {
        todo!()
    }
}
