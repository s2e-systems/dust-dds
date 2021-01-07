// use crate::dds_rtps_implementation::rtps_data_reader::RtpsDataReaderInner;
// use crate::dds::publication::data_writer::RtpsDataWriterInner;
// use crate::dds::topic::topic::RtpsTopicInner;
// use crate::rtps::behavior::types::Duration;
// use crate::rtps::behavior::{StatefulReader, StatefulWriter};
// use crate::rtps::types::constants::{
//     ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
//     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
//     ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
// };
// use crate::rtps::types::{GuidPrefix, ReliabilityKind, GUID};
// use crate::types::{ReturnCode, TopicKind};

// pub struct SimpleEndpointDiscoveryProtocol {
//     sedp_builtin_publications_writer: StatefulWriter,
//     sedp_builtin_publications_reader: StatefulReader,
//     sedp_builtin_subscriptions_writer: StatefulWriter,
//     sedp_builtin_subscriptions_reader: StatefulReader,
//     sedp_builtin_topics_writer: StatefulWriter,
//     sedp_builtin_topics_reader: StatefulReader,
// }

// impl SimpleEndpointDiscoveryProtocol {
//     pub fn new(guid_prefix: GuidPrefix) -> Self {
//         let topic_kind = TopicKind::WithKey;
//         let reliability_level = ReliabilityKind::Reliable;
//         let expects_inline_qos = false;
//         let push_mode = true;
//         let data_max_sized_serialized = None;
//         let heartbeat_period = Duration::from_millis(100);
//         let nack_response_delay = Duration::from_millis(100);
//         let nack_suppression_duration = Duration::from_millis(100);
//         let heartbeat_response_delay = Duration::from_millis(500);

//         let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
//         let sedp_builtin_publications_writer = StatefulWriter::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             push_mode,
//             data_max_sized_serialized,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//         );

//         let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
//         let sedp_builtin_publications_reader = StatefulReader::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             expects_inline_qos,
//             heartbeat_response_delay,
//         );

//         let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//         let sedp_builtin_subscriptions_writer = StatefulWriter::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             push_mode,
//             data_max_sized_serialized,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//         );

//         let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
//         let sedp_builtin_subscriptions_reader = StatefulReader::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             expects_inline_qos,
//             heartbeat_response_delay,
//         );

//         let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
//         let sedp_builtin_topics_writer = StatefulWriter::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             push_mode,
//             data_max_sized_serialized,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//         );

//         let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
//         let sedp_builtin_topics_reader = StatefulReader::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             expects_inline_qos,
//             heartbeat_response_delay,
//         );

//         Self {
//             sedp_builtin_publications_writer,
//             sedp_builtin_publications_reader,
//             sedp_builtin_subscriptions_writer,
//             sedp_builtin_subscriptions_reader,
//             sedp_builtin_topics_writer,
//             sedp_builtin_topics_reader,
//         }
//     }

//     pub fn writers(&mut self) -> Vec<&mut StatefulWriter> {
//         vec![
//             &mut self.sedp_builtin_publications_writer,
//             &mut self.sedp_builtin_subscriptions_writer,
//             &mut self.sedp_builtin_topics_writer,
//         ]
//     }

//     pub fn insert_writer(&self, _a_datawriter: &RtpsDataWriterInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn update_writer(&self, _a_datawriter: &RtpsDataWriterInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn remove_writer(&self, _a_datawriter: &RtpsDataWriterInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn insert_reader(&self, _a_datareader: &RtpsDataReaderInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn update_reader(&self, _a_datareader: &RtpsDataReaderInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn remove_reader(&self, _a_datareader: &RtpsDataReaderInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn insert_topic(&self, _a_topic: &RtpsTopicInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn update_topic(&self, _a_topic: &RtpsTopicInner) -> ReturnCode<()> {
//         Ok(())
//     }

//     pub fn remove_topic(&self, _a_topic: &RtpsTopicInner) -> ReturnCode<()> {
//         Ok(())
//     }
// }
