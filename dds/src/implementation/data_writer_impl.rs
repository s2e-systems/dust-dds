
use std::any::Any;
use std::sync::{Arc, Weak};
use std::marker::PhantomData;

use crate::types::DDSType;


use crate::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusKind};
use crate::domain::DomainParticipant;
use crate::topic::Topic;
use crate::publication::{Publisher, DataWriterListener};
use crate::builtin_topics::SubscriptionBuiltinTopicData;
use crate::implementation::publisher_impl::PublisherImpl;

use rust_dds_interface::types::{InstanceHandle, Time, ReturnCode, Duration, ReturnCodes};
use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::protocol::ProtocolWriter;


pub(crate) struct DataWriterImpl<T: DDSType+Any+Send+Sync> {
    parent_publisher: Weak<PublisherImpl>,
    protocol_writer: Weak<dyn ProtocolWriter>,
    value: PhantomData<T>,
}

impl<T: DDSType+Any+Send+Sync> DataWriterImpl<T> {
    pub fn register_instance(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T
    ) -> ReturnCode<Option<InstanceHandle>> {
        // ProtocolWriter::register(&concrete_writer);
        todo!()
    }

    pub fn register_instance_w_timestamp(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
        _timestamp: Time,
    ) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    pub fn unregister_instance(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
        _handle: Option<InstanceHandle>
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn unregister_instance_w_timestamp(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn get_key_value(
        _this: &Weak<DataWriterImpl<T>>,
        _key_holder: &mut T,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_instance(
        this: &Weak<DataWriterImpl<T>>,
        instance: &T,
    ) -> ReturnCode<Option<InstanceHandle>> {
        let dw = Self::upgrade_datawriter(this)?;
        let protocol_writer = Self::upgrade_protocol_writer(&dw.protocol_writer)?;

        let handle = instance.instance_handle();

        match protocol_writer.is_registered(handle) {
            true => Ok(Some(handle)),
            false => Ok(None),
        }
    }

    pub fn write (
        this: &Weak<DataWriterImpl<T>>,
        data: T,
        instance_handle: Option<InstanceHandle>,
    ) -> ReturnCode<()> {
        let timestamp = DomainParticipant::get_current_time()?;

        Self::write_w_timestamp(this, data, instance_handle, timestamp)
    }

    pub fn write_w_timestamp(
        this: &Weak<DataWriterImpl<T>>,
        data: T,
        instance_handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> ReturnCode<()> {
        let dw = Self::upgrade_datawriter(this)?;
        let protocol_writer = Self::upgrade_protocol_writer(&dw.protocol_writer)?;

        let handle = match instance_handle {
            None => data.instance_handle(),
            Some(handle) => {
                if let Some(existing_handle) = Self::lookup_instance(&this, &data)? {
                    if existing_handle != data.instance_handle() {
                        return Err(ReturnCodes::PreconditionNotMet);
                    }
                } else {
                    return Err(ReturnCodes::BadParameter);
                }
                handle
            },
        };

        let serialized_data = data.serialize();

        protocol_writer.write(handle, serialized_data, timestamp)
    }

    pub fn dispose(
        _this: &Weak<DataWriterImpl<T>>,
        _data: T,
        _instance_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn dispose_w_timestamp(
        _this: &Weak<DataWriterImpl<T>>,
        _data: T,
        _instance_handle: InstanceHandle,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn wait_for_acknowledgments(
        _this: &Weak<DataWriterImpl<T>>,
        _max_wait: Duration
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_liveliness_lost_status(
        _this: &Weak<DataWriterImpl<T>>,
        _status: &mut LivelinessLostStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_offered_deadline_missed_status(
        _this: &Weak<DataWriterImpl<T>>,
        _status: &mut OfferedDeadlineMissedStatus
    ) -> ReturnCode<()> {
        todo!()
    }


    pub fn get_offered_incompatible_qos_status(
        _this: &Weak<DataWriterImpl<T>>,
        _status: &mut OfferedIncompatibleQosStatus
    ) -> ReturnCode<()> {
        todo!()
    }


    pub fn get_publication_matched_status(
        _this: &Weak<DataWriterImpl<T>>,
        _status: &mut PublicationMatchedStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_topic(
        _this: &Weak<DataWriterImpl<T>>,
    ) -> Topic {
        todo!()
    }

    pub fn get_publisher(
        this: &Weak<DataWriterImpl<T>>,
    ) -> Publisher {
        Publisher(this.upgrade().unwrap().parent_publisher.clone())
    }

    pub fn assert_liveliness(_this: &Weak<DataWriterImpl<T>>,) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_subscription_data(
        _this: &Weak<DataWriterImpl<T>>,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_subscriptions(
        _this: &Weak<DataWriterImpl<T>>,
        _subscription_handles: &[InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_qos(_this: &Weak<DataWriterImpl<T>>, _qos_list: DataWriterQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_qos(_this: &Weak<DataWriterImpl<T>>, _qos_list: &mut DataWriterQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_listener(_this: &Weak<DataWriterImpl<T>>, _a_listener: Box<dyn DataWriterListener<T>>, _mask: &[StatusKind]) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_listener(_this: &Weak<DataWriterImpl<T>>,) -> Box<dyn DataWriterListener<T>> {
        todo!()
    }

    pub fn get_statuscondition(_this: &Weak<DataWriterImpl<T>>, ) -> crate::infrastructure::entity::StatusCondition {
        todo!()
    }

    pub fn get_status_changes(_this: &Weak<DataWriterImpl<T>>,) -> StatusKind {
        todo!()
    }

    pub fn enable(this: &Weak<DataWriterImpl<T>>,) -> ReturnCode<()> {
        let _dw = DataWriterImpl::upgrade_datawriter(this)?;

        // let guid = GUID::new([1;12],EntityId::new([1;3], EntityKind::UserDefinedWriterWithKey));
        // let topic_kind = TopicKind::WithKey;
        // let reliability_level = ReliabilityKind::Reliable;
        // let push_mode = true;
        // let heartbeat_period = crate::rtps::behavior::types::Duration::from_millis(100);
        // let nack_response_delay = crate::rtps::behavior::types::Duration::from_millis(100);
        // let nack_suppression_duration = crate::rtps::behavior::types::Duration::from_millis(100);

        // *dw.writer_interface.lock().unwrap() = Some(I::new());
        // StatefulWriter::new(
        //         guid,
        //         topic_kind,
        //         reliability_level,
        //         push_mode,
        //         heartbeat_period,
        //         nack_response_delay,
        //         nack_suppression_duration)
        //     );

        Ok(())
    }

    pub fn get_instance_handle(this: &Weak<DataWriterImpl<T>>) -> ReturnCode<InstanceHandle> {
        let datawriter = Self::upgrade_datawriter(this)?;
        let protocol_writer = Self::upgrade_protocol_writer(&datawriter.protocol_writer)?;
        Ok(protocol_writer.get_instance_handle())
    }

     //////////////// From here on are the functions that do not belong to the standard API
     pub(crate) fn new(parent_publisher: Weak<PublisherImpl>, protocol_writer: Weak<dyn ProtocolWriter>) -> Self {
         Self{
            parent_publisher,
            protocol_writer,
            value: PhantomData
         }
     }

    fn upgrade_datawriter(this: &Weak<DataWriterImpl<T>>) -> ReturnCode<Arc<DataWriterImpl<T>>> {
        this.upgrade().ok_or(ReturnCodes::AlreadyDeleted("Datawriter"))
    }

    fn upgrade_protocol_writer(protocol_writer: &Weak<dyn ProtocolWriter>) -> ReturnCode<Arc<dyn ProtocolWriter>> {
        protocol_writer.upgrade().ok_or(ReturnCodes::AlreadyDeleted("Protocol writer"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::types::DDSType;
    use crate::publication::AnyDataWriter;
    use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter};
    use rust_dds_interface::types::Data;

    struct MockProtocolWriter;
    impl ProtocolEntity for MockProtocolWriter{
        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }

        fn enable(&self) -> ReturnCode<()> {
            todo!()
        }
    }
    impl ProtocolWriter for MockProtocolWriter {
        fn write(&self, _instance_handle: InstanceHandle, _data: Data, _timestamp: Time) -> ReturnCode<()> {
            todo!()
        }

        fn dispose(&self, _instance_handle: InstanceHandle, _timestamp: Time) -> ReturnCode<()> {
            todo!()
        }

        fn unregister(&self, _instance_handle: InstanceHandle, _timestamp: Time) -> ReturnCode<()> {
            todo!()
        }

        fn register(&self, _instance_handle: InstanceHandle, _timestamp: Time) -> ReturnCode<()> {
            todo!()
        }

        fn is_registered(&self, _instance_handle: InstanceHandle) -> bool {
            todo!()
        }
    }


    #[derive(Debug)]
    struct  Foo {
        value: bool //@key   
    }

    impl DDSType for Foo {
        fn instance_handle(&self) -> InstanceHandle {
            todo!()
        }

        fn serialize(&self) -> Data {
            todo!()
        }

        fn deserialize(_data: Data) -> Self {
            todo!()
        }
    }

    #[derive(Debug)]
    struct  Bar {
        value: bool
    }
    impl DDSType for Bar {
        fn instance_handle(&self) -> InstanceHandle {
            todo!()
        }

        fn serialize(&self) -> Data {
            todo!()
        }

        fn deserialize(_data: Data) -> Self {
            todo!()
        }
    }

    #[derive(Debug)]
    struct  Baz {
        value: bool
    }
    impl DDSType for Baz {
        fn instance_handle(&self) -> InstanceHandle {
            todo!()
        }

        fn serialize(&self) -> Data {
            todo!()
        }

        fn deserialize(_data: Data) -> Self {
            todo!()
        }
    }

    #[test]
    fn get_single_anydatawriter_value() {
        let any_datawriter = AnyDataWriter(
            Arc::new(DataWriterImpl::<Foo>::new(Weak::new(), Weak::<MockProtocolWriter>::new()))
        );

        assert!(any_datawriter.get::<Foo>().is_some())
    }

    #[test]
    fn get_multiple_anydatawriter_values() {
        let mut datawriter_list = Vec::new();

        datawriter_list.push(AnyDataWriter(Arc::new(DataWriterImpl::<Foo>::new(Weak::new(), Weak::<MockProtocolWriter>::new()))));
        datawriter_list.push(AnyDataWriter(Arc::new(DataWriterImpl::<Bar>::new(Weak::new(), Weak::<MockProtocolWriter>::new()))));

        assert!(datawriter_list[0].get::<Foo>().is_some());
        assert!(datawriter_list[0].get::<Bar>().is_none());

        assert!(datawriter_list[1].get::<Foo>().is_none());
        assert!(datawriter_list[1].get::<Bar>().is_some());

        assert_eq!(datawriter_list.iter().position(|x| x.get::<Foo>().is_some()).unwrap(),0);
        assert_eq!(datawriter_list.iter().position(|x| x.get::<Bar>().is_some()).unwrap(),1);
    }

}