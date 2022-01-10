use rust_dds_api::{
    dcps_psm::InstanceHandle,
    infrastructure::{entity::Entity, qos::DataWriterQos},
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener, publisher::Publisher,
    },
    return_type::{DDSError, DDSResult},
    topic::topic::Topic,
};
use rust_rtps_pim::{
    behavior::writer::writer::{RtpsWriterAttributes, RtpsWriterOperations},
    structure::{history_cache::RtpsHistoryCacheAddChange, types::ChangeKind},
};

use crate::{
    dds_type::{DdsSerialize, LittleEndian},
    rtps_impl::{
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
};

pub enum RtpsWriter {
    Stateless(RtpsStatelessWriterImpl),
    Stateful(RtpsStatefulWriterImpl),
}

impl RtpsWriter {
    pub fn try_as_stateless_writer(&mut self) -> DDSResult<&mut RtpsStatelessWriterImpl> {
        match self {
            RtpsWriter::Stateless(x) => Ok(x),
            RtpsWriter::Stateful(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateless writer".to_string(),
            )),
        }
    }
    pub fn try_as_stateful_writer(&mut self) -> DDSResult<&mut RtpsStatefulWriterImpl> {
        match self {
            RtpsWriter::Stateless(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateful writer".to_string(),
            )),
            RtpsWriter::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataWriterImpl<Foo> {
    _qos: DataWriterQos,
    rtps_writer: RtpsWriter,
    _listener: Option<Box<dyn DataWriterListener<DataType = Foo> + Send + Sync>>,
}

impl<Foo> DataWriterImpl<Foo>
where
    Foo: Send + 'static,
{
    pub fn new(qos: DataWriterQos, rtps_writer: RtpsWriter) -> Self {
        Self {
            _qos: qos,
            rtps_writer,
            _listener: None,
        }
    }
}

impl<Foo> AsRef<RtpsWriter> for DataWriterImpl<Foo> {
    fn as_ref(&self) -> &RtpsWriter {
        &self.rtps_writer
    }
}

impl<Foo> AsMut<RtpsWriter> for DataWriterImpl<Foo> {
    fn as_mut(&mut self) -> &mut RtpsWriter {
        &mut self.rtps_writer
    }
}

impl<Foo> DataWriter<Foo> for DataWriterImpl<Foo>
where
    Foo: DdsSerialize,
{
    fn register_instance(&mut self, _instance: Foo) -> DDSResult<Option<InstanceHandle>> {
        unimplemented!()
    }

    fn register_instance_w_timestamp(
        &mut self,
        _instance: Foo,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn unregister_instance(
        &mut self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        unimplemented!()
    }

    fn unregister_instance_w_timestamp(
        &mut self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&mut self, _data: &Foo, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn write_w_timestamp(
        &mut self,
        data: &Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)
            .unwrap();
        match &mut self.rtps_writer {
            RtpsWriter::Stateless(stateless_rtps_writer) => {
                let change =
                    stateless_rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                stateless_rtps_writer.writer_cache().add_change(change);
            }
            RtpsWriter::Stateful(stateful_rtps_writer) => {
                let change =
                    stateful_rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                stateful_rtps_writer.writer_cache().add_change(change);
            }
        }

        Ok(())
    }

    fn dispose(&mut self, _data: Foo, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn dispose_w_timestamp(
        &mut self,
        _data: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::LivelinessLostStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_topic(&self) -> &dyn Topic<Foo> {
        unimplemented!()
    }

    fn get_publisher(&self) -> &dyn Publisher {
        unimplemented!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: rust_dds_api::builtin_topics::SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<Foo> Entity for DataWriterImpl<Foo> {
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener<DataType = Foo>>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // let qos = qos.unwrap_or_default();
        // qos.is_consistent()?;
        // self.qos = qos;
        // Ok(())
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // &self.qos
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(
        &self,
    ) -> DDSResult<rust_dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<rust_dds_api::dcps_psm::StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {

//     use rust_rtps_pim::{
//         behavior::{
//             types::{Duration, DURATION_ZERO},
//             writer::{
//                 reader_proxy::RtpsReaderProxy,
//                 stateful_writer::{RtpsStatefulWriterConstructor, RtpsStatefulWriterOperations},
//             },
//         },
//         structure::types::{ReliabilityKind, TopicKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
//     };

//     use crate::{
//         dds_impl::publisher_impl, rtps_impl::rtps_stateful_writer_impl::RtpsStatefulWriterImpl, utils::clock::Timer,
//     };

//     use super::*;
//     struct EmptyTimer;

//     impl Timer for EmptyTimer {
//         fn reset(&mut self) {
//             unimplemented!()
//         }

//         fn elapsed(&self) -> std::time::Duration {
//             unimplemented!()
//         }
//     }

//     struct MockData(Vec<u8>);

//     impl DdsSerialize for MockData {
//         fn serialize<W: std::io::Write, R: crate::dds_type::Endianness>(
//             &self,
//             mut writer: W,
//         ) -> DDSResult<()> {
//             writer.write(&self.0).unwrap();
//             Ok(())
//         }
//     }

// struct MockCacheChange;

// #[test]
// fn write_w_timestamp() {
//     struct MockWriterCache;

//     impl RtpsHistoryCacheAddChange for MockWriterCache {
//         type CacheChangeType = MockWriterCache;

//         fn add_change(&mut self, _change: Self::CacheChangeType) {}
//     }

//     struct MockWriter {
//         cache: MockWriterCache,
//     }

//     impl RtpsWriterOperations for MockWriter {
//         type CacheChangeType = MockCacheChange;

//         fn new_change(
//             &mut self,
//             kind: ChangeKind,
//             data: <Self::CacheChangeType as RtpsCacheChangeConstructor>::DataType,
//             inline_qos: <Self::CacheChangeType as RtpsCacheChangeConstructor>::ParameterListType,
//             handle: InstanceHandle,
//         ) -> Self::CacheChangeType {
//             todo!()
//         }

//         // type CacheChangeType;

//         // fn new_change(
//         //     &mut self,
//         //     kind: ChangeKind,
//         //     data: D,
//         //     inline_qos: P,
//         //     handle: InstanceHandle,
//         // ) -> RtpsCacheChange<P, D> {
//         //     RtpsCacheChange {
//         //         data_value: data,
//         //         kind,
//         //         instance_handle: handle,
//         //         inline_qos,
//         //         sequence_number: 1,
//         //         writer_guid: GUID_UNKNOWN,
//         //     }
//         // }
//     }

//     let mut dds_data_writer = DataWriterImpl::new(
//         DataWriterQos::default(),
//         MockWriter {
//             cache: MockWriterCache,
//         },
//         Box::new(EmptyTimer),
//     );

//     let data_value = MockData(vec![0, 1, 0, 0, 7, 3]);
//     dds_data_writer
//         .write_w_timestamp(
//             &data_value,
//             None,
//             rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
//         )
//         .unwrap();
// }

//     #[test]
//     fn stateful_writer_heartbeat_send_timer() {
//         struct MockTimer;

//         impl Timer for MockTimer {
//             fn reset(&mut self) {}

//             fn elapsed(&self) -> std::time::Duration {
//                 std::time::Duration::new(5, 0)
//             }
//         }
//         let guid = GUID_UNKNOWN;
//         let topic_kind = TopicKind::WithKey;
//         let reliability_level = ReliabilityKind::Reliable;
//         let unicast_locator_list = &[];
//         let multicast_locator_list = &[];
//         let push_mode = true;
//         let heartbeat_period = Duration::new(2, 0);
//         let nack_response_delay = DURATION_ZERO;
//         let nack_suppression_duration = DURATION_ZERO;
//         let data_max_size_serialized = None;

//         let mut rtps_writer_impl = RtpsStatefulWriterImpl::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             unicast_locator_list,
//             multicast_locator_list,
//             push_mode,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//             data_max_size_serialized,
//         );
//         let reader_proxy =
//             RtpsReaderProxy::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, vec![], vec![], false);

//         rtps_writer_impl.matched_reader_add(reader_proxy);

//         let mut data_writer_impl: DataWriterImpl<MockData, _, _> =
//             DataWriterImpl::new(DataWriterQos::default(), rtps_writer_impl, MockTimer);

//         let destined_submessages1 =
//             publisher_impl::StatefulWriterSubmessageProducer::produce_submessages(
//                 &mut data_writer_impl,
//             );
//         let produced_submessages1 = &destined_submessages1[0].1;
//         assert_eq!(produced_submessages1.len(), 1);
//         if let RtpsSubmessageTypeWrite::Heartbeat(heartbeat_submessage) = &produced_submessages1[0]
//         {
//             assert_eq!(heartbeat_submessage.count.value, Count(1));
//         } else {
//             assert!(false, "Wrong submessage");
//         }

//         let destined_submessages2 = data_writer_impl.produce_submessages();
//         let produced_submessages2 = &destined_submessages2[0].1;
//         if let RtpsSubmessageTypeWrite::Heartbeat(heartbeat_submessage) = &produced_submessages2[0]
//         {
//             assert_eq!(heartbeat_submessage.count.value, Count(2));
//         } else {
//             assert!(false, "Wrong submessage");
//         }
//     }
// }
