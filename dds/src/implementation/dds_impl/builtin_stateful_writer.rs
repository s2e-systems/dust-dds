use crate::implementation::rtps::{stateful_writer::RtpsStatefulWriter, utils::clock::StdTimer};
use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        messages::{
            overall_structure::RtpsMessageHeader,
            submessage_elements::{
                GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement,
                VendorIdSubmessageElement,
            },
            submessages::AckNackSubmessage,
            types::ProtocolId,
            RtpsMessage,
        },
        stateful_writer::{
            DEFAULT_HEARTBEAT_PERIOD, DEFAULT_NACK_RESPONSE_DELAY,
            DEFAULT_NACK_SUPPRESSION_DURATION,
        },
        transport::TransportWrite,
        types::{Guid, TopicKind, PROTOCOLVERSION, VENDOR_ID_S2E},
        writer::RtpsWriter,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::DataWriterQos,
        time::Time,
    },
    infrastructure::{instance::InstanceHandle, qos_policy::ReliabilityQosPolicyKind},
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use crate::implementation::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData, discovered_topic_data::DiscoveredTopicData,
        discovered_writer_data::DiscoveredWriterData,
    },
    utils::shared_object::{DdsRwLock, DdsShared},
};

use super::{
    message_receiver::MessageReceiver, participant_discovery::ParticipantDiscovery,
    topic_impl::TopicImpl,
};

pub struct BuiltinStatefulWriter {
    rtps_writer: DdsRwLock<RtpsStatefulWriter<StdTimer>>,
    topic: DdsShared<TopicImpl>,
    enabled: DdsRwLock<bool>,
}

impl BuiltinStatefulWriter {
    pub fn new(guid: Guid, topic: DdsShared<TopicImpl>) -> DdsShared<Self> {
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let topic_kind = TopicKind::WithKey;
        let push_mode = true;
        let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
        let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
        let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
        let data_max_size_serialized = None;
        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                unicast_locator_list,
                multicast_locator_list,
            ),
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
            DataWriterQos::default(),
        ));

        DdsShared::new(BuiltinStatefulWriter {
            rtps_writer: DdsRwLock::new(rtps_writer),
            topic,
            enabled: DdsRwLock::new(false),
        })
    }

    /// NOTE: This function is only useful for the SEDP writers so we probably need a separate
    /// type for those.
    pub fn add_matched_participant(&self, participant_discovery: &ParticipantDiscovery) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let type_name = self.topic.get_type_name();
        if type_name == DiscoveredWriterData::type_name() {
            participant_discovery
                .discovered_participant_add_publications_writer(&mut rtps_writer_lock);
        } else if type_name == DiscoveredReaderData::type_name() {
            participant_discovery
                .discovered_participant_add_subscriptions_writer(&mut rtps_writer_lock);
        } else if type_name == DiscoveredTopicData::type_name() {
            participant_discovery.discovered_participant_add_topics_writer(&mut rtps_writer_lock);
        }
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        if rtps_writer_lock.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            rtps_writer_lock.on_acknack_submessage_received(
                acknack_submessage,
                message_receiver.dest_guid_prefix(),
            );
        }
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn write_w_timestamp<Foo>(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .write_w_timestamp(data, handle, timestamp)
    }

    pub fn dispose_w_timestamp<Foo>(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        self.rtps_writer
            .write_lock()
            .dispose_w_timestamp(data, handle, timestamp)
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let guid_prefix = rtps_writer_lock.guid().prefix();

        let destined_submessages = rtps_writer_lock.produce_submessages();

        for (reader_proxy, submessages) in destined_submessages {
            let header = RtpsMessageHeader {
                protocol: ProtocolId::PROTOCOL_RTPS,
                version: ProtocolVersionSubmessageElement {
                    value: PROTOCOLVERSION,
                },
                vendor_id: VendorIdSubmessageElement {
                    value: VENDOR_ID_S2E,
                },
                guid_prefix: GuidPrefixSubmessageElement {
                    value: guid_prefix,
                },
            };

            let rtps_message = RtpsMessage {
                header,
                submessages,
            };
            for locator in reader_proxy.unicast_locator_list() {
                transport.write(&rtps_message, *locator);
            }
        }
    }
}
