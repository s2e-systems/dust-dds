use std::collections::HashMap;

use crate::implementation::rtps::stateless_writer::RtpsStatelessWriter;
use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        messages::{
            overall_structure::RtpsMessageHeader,
            submessage_elements::{
                GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement,
                VendorIdSubmessageElement,
            },
            types::ProtocolId,
            RtpsMessage,
        },
        reader_locator::RtpsReaderLocator,
        transport::TransportWrite,
        types::{ChangeKind, Guid, Locator, TopicKind, PROTOCOLVERSION, VENDOR_ID_S2E},
        writer::RtpsWriter,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::DataWriterQos,
        time::Time,
    },
    infrastructure::{
        instance::{InstanceHandle, HANDLE_NIL},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind, LENGTH_UNLIMITED},
        time::DURATION_ZERO,
    },
    topic_definition::type_support::{DdsSerialize, DdsType, LittleEndian},
};

use crate::implementation::utils::shared_object::{DdsRwLock, DdsShared};

fn calculate_instance_handle(serialized_key: &[u8]) -> InstanceHandle {
    if serialized_key.len() <= 16 {
        let mut h = [0; 16];
        h[..serialized_key.len()].clone_from_slice(serialized_key);
        h.into()
    } else {
        <[u8; 16]>::from(md5::compute(serialized_key)).into()
    }
}

pub struct BuiltinStatelessWriter {
    rtps_writer: DdsRwLock<RtpsStatelessWriter>,
    registered_instance_list: DdsRwLock<HashMap<InstanceHandle, Vec<u8>>>,
    enabled: DdsRwLock<bool>,
}

impl BuiltinStatelessWriter {
    pub fn new(guid: Guid, discovery_locator_list: &[Locator]) -> DdsShared<Self> {
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];

        let mut spdp_builtin_participant_rtps_writer = RtpsStatelessWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                TopicKind::WithKey,
                unicast_locator_list,
                multicast_locator_list,
            ),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            None,
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DURATION_ZERO,
                },
                ..Default::default()
            },
        ));

        let spdp_reader_locators: Vec<RtpsReaderLocator> = discovery_locator_list
            .iter()
            .map(|&locator| RtpsReaderLocator::new(locator, false))
            .collect();

        for reader_locator in spdp_reader_locators {
            spdp_builtin_participant_rtps_writer.reader_locator_add(reader_locator);
        }

        DdsShared::new(BuiltinStatelessWriter {
            rtps_writer: DdsRwLock::new(spdp_builtin_participant_rtps_writer),
            registered_instance_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<BuiltinStatelessWriter> {
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
        let instance_handle = calculate_instance_handle(&serialized_key);

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

impl DdsShared<BuiltinStatelessWriter> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}

impl DdsShared<BuiltinStatelessWriter> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let guid_prefix = rtps_writer_lock.writer().guid().prefix();

        let destined_submessages = rtps_writer_lock.produce_submessages();
        for (reader_locator, submessages) in destined_submessages {
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
            transport.write(&rtps_message, reader_locator.locator())
        }
    }
}
