
use std::any::Any;
use std::sync::{Weak, Mutex};
use std::marker::PhantomData;

use crate::types::{InstanceHandle, Time, ReturnCode, Duration, ReturnCodes, DDSType};

use crate::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::topic::topic::Topic;
use crate::publication::publisher::Publisher;
use crate::publication::data_writer_listener::DataWriterListener;
use crate::builtin_topics::SubscriptionBuiltinTopicData;

use crate::implementation::publisher_impl::PublisherImpl;

use crate::publication::data_writer::qos::DataWriterQos;

use rust_dds_interface::protocol::WriterInterface;
use rust_rtps::StatefulWriter;

pub(crate) struct DataWriterImpl<T: DDSType+Any+Send+Sync, I: WriterInterface = StatefulWriter> {
    parent_publisher: Weak<PublisherImpl>,
    writer_interface: Mutex<Option<I>>,
    value: PhantomData<T>,
}

impl<T: DDSType+Any+Send+Sync, I: WriterInterface> DataWriterImpl<T,I> {
    pub fn register_instance(
        _this: &Weak<DataWriterImpl<T,I>>,
        _instance: T
    ) -> InstanceHandle {
        // WriterInterface::register(&concrete_writer);
        todo!()
    }

    pub fn register_instance_w_timestamp(
        _this: &Weak<DataWriterImpl<T,I>>,
        _instance: T,
        _timestamp: Time,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn unregister_instance(
        _this: &Weak<DataWriterImpl<T,I>>,
        _instance: T,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn unregister_instance_w_timestamp(
        _this: &Weak<DataWriterImpl<T,I>>,
        _instance: T,
        _handle: InstanceHandle,
        _timestamp: Time,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn get_key_value(
        _this: &Weak<DataWriterImpl<T,I>>,
        _key_holder: &mut T,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_instance(
        _this: &Weak<DataWriterImpl<T,I>>,
        _instance: T,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn write (
        _this: &Weak<DataWriterImpl<T,I>>,
        _data: T,
        _instance_handle: Option<InstanceHandle>,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn write_w_timestamp(
        this: &Weak<DataWriterImpl<T,I>>,
        _data: T,
        _instance_handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        let dw = this.upgrade().ok_or(ReturnCodes::AlreadyDeleted)?;
        
        let writer_interface_lock = dw.writer_interface.lock().unwrap();
        let writer_interface = writer_interface_lock.as_ref().ok_or(ReturnCodes::NotEnabled)?;
        writer_interface.write([0;16], vec![]);
        // // let history_cache = rtps_datawriter.writer_cache();

        // // let kind = ChangeKind::Alive;
        // // let data = Some(vec![1,2,3]);
        // // let inline_qos = None;
        // // let handle = match instance_handle {
        // //     Some(handle) => handle,
        // //     None => todo!()
        // // };
        
        // let change = rtps_datawriter.new_change(kind, data, inline_qos, handle);
       
        // history_cache.add_change(change);

        Ok(())
    }

    pub fn dispose(
        _this: &Weak<DataWriterImpl<T,I>>,
        _data: T,
        _instance_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn dispose_w_timestamp(
        _this: &Weak<DataWriterImpl<T,I>>,
        _data: T,
        _instance_handle: InstanceHandle,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn wait_for_acknowledgments(
        _this: &Weak<DataWriterImpl<T,I>>,
        _max_wait: Duration
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_liveliness_lost_status(
        _this: &Weak<DataWriterImpl<T,I>>,
        _status: &mut LivelinessLostStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_offered_deadline_missed_status(
        _this: &Weak<DataWriterImpl<T,I>>,
        _status: &mut OfferedDeadlineMissedStatus
    ) -> ReturnCode<()> {
        todo!()
    }


    pub fn get_offered_incompatible_qos_status(
        _this: &Weak<DataWriterImpl<T,I>>,
        _status: &mut OfferedIncompatibleQosStatus
    ) -> ReturnCode<()> {
        todo!()
    }


    pub fn get_publication_matched_status(
        _this: &Weak<DataWriterImpl<T,I>>,
        _status: &mut PublicationMatchedStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_topic(
        _this: &Weak<DataWriterImpl<T,I>>,
    ) -> Topic {
        todo!()
    }

    pub fn get_publisher(
        this: &Weak<DataWriterImpl<T,I>>,
    ) -> Publisher {
        Publisher(this.upgrade().unwrap().parent_publisher.clone())
    }

    pub fn assert_liveliness(_this: &Weak<DataWriterImpl<T,I>>,) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_subscription_data(
        _this: &Weak<DataWriterImpl<T,I>>,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_subscriptions(
        _this: &Weak<DataWriterImpl<T,I>>,
        _subscription_handles: &[InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_qos(_this: &Weak<DataWriterImpl<T,I>>, _qos_list: DataWriterQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_qos(_this: &Weak<DataWriterImpl<T,I>>, _qos_list: &mut DataWriterQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_listener(_this: &Weak<DataWriterImpl<T,I>>, _a_listener: Box<dyn DataWriterListener<T>>, _mask: &[crate::types::StatusKind]) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_listener(_this: &Weak<DataWriterImpl<T,I>>,) -> Box<dyn DataWriterListener<T>> {
        todo!()
    }

    pub fn get_statuscondition(_this: &Weak<DataWriterImpl<T,I>>, ) -> crate::infrastructure::entity::StatusCondition {
        todo!()
    }

    pub fn get_status_changes(_this: &Weak<DataWriterImpl<T,I>>,) -> crate::types::StatusKind {
        todo!()
    }

    pub fn enable(this: &Weak<DataWriterImpl<T,I>>,) -> ReturnCode<()> {
        let dw = this.upgrade().ok_or(ReturnCodes::AlreadyDeleted)?;

        // let guid = GUID::new([1;12],EntityId::new([1;3], EntityKind::UserDefinedWriterWithKey));
        // let topic_kind = TopicKind::WithKey;
        // let reliability_level = ReliabilityKind::Reliable;
        // let push_mode = true;
        // let heartbeat_period = crate::rtps::behavior::types::Duration::from_millis(100);
        // let nack_response_delay = crate::rtps::behavior::types::Duration::from_millis(100);
        // let nack_suppression_duration = crate::rtps::behavior::types::Duration::from_millis(100);

        *dw.writer_interface.lock().unwrap() = Some(I::new());
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

    pub fn get_instance_handle(_this: &Weak<DataWriterImpl<T,I>>,) -> InstanceHandle {
        todo!()
    }

     //////////////// From here on are the functions that do not belong to the standard API
     pub(crate) fn new(parent_publisher: Weak<PublisherImpl>
     ) -> Self {
         Self{
            parent_publisher,
            writer_interface: Mutex::new(None),
            value: PhantomData
         }
     }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::types::DDSType;
    use crate::publication::data_writer::AnyDataWriter;
    use crate::types::Data;

    #[derive(Debug)]
    struct  Foo {
        value: bool //@key   
    }

    impl DDSType for Foo {
        fn key(&self) -> InstanceHandle {
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
        fn key(&self) -> InstanceHandle {
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
        fn key(&self) -> InstanceHandle {
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
            Arc::new(DataWriterImpl::<Foo>::new(Weak::new()))
        );

        assert!(any_datawriter.get::<Foo>().is_some())
    }

    #[test]
    fn get_multiple_anydatawriter_values() {
        let mut datawriter_list = Vec::new();

        datawriter_list.push(AnyDataWriter(Arc::new(DataWriterImpl::<Foo>::new(Weak::new()))));
        datawriter_list.push(AnyDataWriter(Arc::new(DataWriterImpl::<Bar>::new(Weak::new()))));

        assert!(datawriter_list[0].get::<Foo>().is_some());
        assert!(datawriter_list[0].get::<Bar>().is_none());

        assert!(datawriter_list[1].get::<Foo>().is_none());
        assert!(datawriter_list[1].get::<Bar>().is_some());

        assert_eq!(datawriter_list.iter().position(|x| x.get::<Foo>().is_some()).unwrap(),0);
        assert_eq!(datawriter_list.iter().position(|x| x.get::<Bar>().is_some()).unwrap(),1);
    }

    #[test]
    fn write_w_timestamp() {
        use crate::types::Time;
        use rust_dds_interface::types::Data;
        struct MockInterface {

        }

        impl WriterInterface for MockInterface {
            fn new() -> Self {
                MockInterface{}
            }

            fn write(&self, _instance_handle: InstanceHandle, _data: Data) {
                println!("Mock interface");
            }
        
            fn dispose(&self, _instance_handle: InstanceHandle) {
                todo!()
            }
        
            fn unregister(&self, _instance_handle: InstanceHandle) {
                todo!()
            }
        
            fn register(&self, _instance_handle: InstanceHandle) {
                todo!()
            }
        }


        let timestamp = Time {
            sec: 100,
            nanosec: 20000,
        };

        let dw = Arc::new(DataWriterImpl::<Foo, MockInterface>::new(Weak::new()));
        let dw_weak = Arc::downgrade(&dw);

        let new_foo = Foo{value:false};

        DataWriterImpl::enable(&dw_weak).unwrap();

        DataWriterImpl::write_w_timestamp(&dw_weak, new_foo, None, timestamp).unwrap();
    }
}