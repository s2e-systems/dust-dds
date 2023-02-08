use crate::implementation::rtps::stateless_writer::RtpsStatelessWriter;
use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        messages::{overall_structure::RtpsMessageHeader, RtpsMessage},
        reader_locator::RtpsReaderLocator,
        transport::TransportWrite,
        types::{Guid, Locator, TopicKind},
        writer::RtpsWriter,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::DataWriterQos,
        time::Time,
    },
    infrastructure::{
        instance::InstanceHandle,
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        time::DURATION_ZERO,
    },
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use crate::implementation::utils::shared_object::{DdsRwLock, DdsShared};

pub struct BuiltinStatelessWriter {
    rtps_writer: DdsRwLock<RtpsStatelessWriter>,
    enabled: DdsRwLock<bool>,
}

impl BuiltinStatelessWriter {
    pub fn new(guid: Guid, spdp_discovery_locator_list: &[Locator]) -> DdsShared<Self> {
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
            usize::MAX,
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DURATION_ZERO,
                },
                ..Default::default()
            },
        ));

        let spdp_reader_locators: Vec<RtpsReaderLocator> = spdp_discovery_locator_list
            .iter()
            .map(|&locator| RtpsReaderLocator::new(locator, false))
            .collect();

        for reader_locator in spdp_reader_locators {
            spdp_builtin_participant_rtps_writer.reader_locator_add(reader_locator);
        }

        DdsShared::new(BuiltinStatelessWriter {
            rtps_writer: DdsRwLock::new(spdp_builtin_participant_rtps_writer),
            enabled: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<BuiltinStatelessWriter> {
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
}

impl DdsShared<BuiltinStatelessWriter> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}

impl DdsShared<BuiltinStatelessWriter> {
    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();

        let destined_submessages = rtps_writer_lock.produce_submessages();
        for (reader_locator, submessages) in destined_submessages {
            let rtps_message = RtpsMessage {
                header,
                submessages,
            };
            transport.write(&rtps_message, reader_locator.locator())
        }
    }
}
