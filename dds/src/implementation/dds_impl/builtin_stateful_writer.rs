use std::collections::HashMap;

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
        types::{ChangeKind, Guid, TopicKind, PROTOCOLVERSION, VENDOR_ID_S2E},
        writer::RtpsWriter,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::DataWriterQos,
        time::Time,
    },
    infrastructure::{
        instance::{InstanceHandle, HANDLE_NIL},
        qos_policy::{ReliabilityQosPolicyKind, LENGTH_UNLIMITED},
    },
    topic_definition::type_support::{DdsSerialize, DdsType, LittleEndian},
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
    registered_instance_list: DdsRwLock<HashMap<InstanceHandle, Vec<u8>>>,
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
            registered_instance_list: DdsRwLock::new(HashMap::new()),
            topic,
            enabled: DdsRwLock::new(false),
        })
    }

    /// NOTE: This function is only useful for the SEDP writers so we probably need a separate
    /// type for those.
    pub fn add_matched_participant(&self, participant_discovery: &ParticipantDiscovery) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        if !rtps_writer_lock
            .matched_readers()
            .iter_mut()
            .any(|r| r.remote_reader_guid().prefix() == participant_discovery.guid_prefix())
        {
            let type_name = self.topic.get_type_name().unwrap();
            if type_name == DiscoveredWriterData::type_name() {
                participant_discovery
                    .discovered_participant_add_publications_writer(&mut rtps_writer_lock);
            } else if type_name == DiscoveredReaderData::type_name() {
                participant_discovery
                    .discovered_participant_add_subscriptions_writer(&mut rtps_writer_lock);
            } else if type_name == DiscoveredTopicData::type_name() {
                participant_discovery
                    .discovered_participant_add_topics_writer(&mut rtps_writer_lock);
            }
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
        if rtps_writer_lock.writer().get_qos().reliability.kind
            == ReliabilityQosPolicyKind::Reliable
        {
            rtps_writer_lock.on_acknack_submessage_received(
                acknack_submessage,
                message_receiver.source_guid_prefix(),
            );
        }
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn register_instance_w_timestamp<Foo>(
        &self,
        instance: &Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let serialized_key = instance.get_serialized_key::<LittleEndian>();
        let instance_handle = serialized_key.as_slice().into();

        let mut registered_instances_lock = self.registered_instance_list.write_lock();
        let rtps_writer_lock = self.rtps_writer.read_lock();
        if !registered_instances_lock.contains_key(&instance_handle) {
            if rtps_writer_lock
                .writer()
                .get_qos()
                .resource_limits
                .max_instances
                == LENGTH_UNLIMITED
                || (registered_instances_lock.len() as i32)
                    < rtps_writer_lock
                        .writer()
                        .get_qos()
                        .resource_limits
                        .max_instances
            {
                registered_instances_lock.insert(instance_handle, serialized_key);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }
        Ok(Some(instance_handle))
    }

    pub fn write_w_timestamp<Foo>(
        &self,
        data: &Foo,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)?;
        let handle = self
            .register_instance_w_timestamp(data, timestamp)?
            .unwrap_or(HANDLE_NIL);
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let change = rtps_writer_lock.writer_mut().new_change(
            ChangeKind::Alive,
            serialized_data,
            vec![],
            handle,
            timestamp,
        );
        rtps_writer_lock.add_change(change);

        Ok(())
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
        let guid_prefix = rtps_writer_lock.writer().guid().prefix();

        let destined_submessages = rtps_writer_lock.produce_submessages();

        for (reader_proxy, submessages) in destined_submessages {
            let header = RtpsMessageHeader {
                protocol: ProtocolId::PROTOCOL_RTPS,
                version: ProtocolVersionSubmessageElement {
                    value: PROTOCOLVERSION.into(),
                },
                vendor_id: VendorIdSubmessageElement {
                    value: VENDOR_ID_S2E,
                },
                guid_prefix: GuidPrefixSubmessageElement {
                    value: guid_prefix.into(),
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
