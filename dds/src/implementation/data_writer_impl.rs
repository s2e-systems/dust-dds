
use std::sync::Weak;
use std::marker::PhantomData;

use crate::types::{InstanceHandle, Time, ReturnCode, Duration};

use crate::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::topic::topic::Topic;
use crate::publication::publisher::Publisher;
use crate::publication::data_writer_listener::DataWriterListener;
use crate::builtin_topics::SubscriptionBuiltinTopicData;

use crate::implementation::publisher_impl::PublisherImpl;

use crate::publication::data_writer::qos::DataWriterQos;

pub(crate) struct DataWriterImpl<T> {
    parent_publisher: Weak<PublisherImpl>,
    // rtps_writer: Mutex<Option<StatefulWriter>>,
    value: PhantomData<T>,
}

impl<T> DataWriterImpl<T> {
    pub fn register_instance(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T
    ) -> InstanceHandle {
        todo!()
    }

    pub fn register_instance_w_timestamp(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
        _timestamp: Time,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn unregister_instance(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn unregister_instance_w_timestamp(
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
        _handle: InstanceHandle,
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
        _this: &Weak<DataWriterImpl<T>>,
        _instance: T,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn write (
        _this: &Weak<DataWriterImpl<T>>,
        _data: T,
        _instance_handle: Option<InstanceHandle>,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn write_w_timestamp(
        _this: &Weak<DataWriterImpl<T>>,
        _data: T,
        _instance_handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        // let dw = this.upgrade().ok_or(ReturnCodes::AlreadyDeleted)?;
        
        // // let datawriter_lock = dw.rtps_writer.lock().unwrap();
        // // let rtps_datawriter = datawriter_lock.as_ref().ok_or(ReturnCodes::NotEnabled)?;
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

        // Ok(())
        todo!()
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

    pub fn set_listener(_this: &Weak<DataWriterImpl<T>>, _a_listener: Box<dyn DataWriterListener<T>>, _mask: &[crate::types::StatusKind]) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_listener(_this: &Weak<DataWriterImpl<T>>,) -> Box<dyn DataWriterListener<T>> {
        todo!()
    }

    pub fn get_statuscondition(_this: &Weak<DataWriterImpl<T>>, ) -> crate::infrastructure::entity::StatusCondition {
        todo!()
    }

    pub fn get_status_changes(_this: &Weak<DataWriterImpl<T>>,) -> crate::types::StatusKind {
        todo!()
    }

    pub fn enable(_this: &Weak<DataWriterImpl<T>>,) -> ReturnCode<()> {
        // let dw = this.upgrade().ok_or(ReturnCodes::AlreadyDeleted)?;

        // let guid = GUID::new([1;12],EntityId::new([1;3], EntityKind::UserDefinedWriterWithKey));
        // let topic_kind = TopicKind::WithKey;
        // let reliability_level = ReliabilityKind::Reliable;
        // let push_mode = true;
        // let heartbeat_period = crate::rtps::behavior::types::Duration::from_millis(100);
        // let nack_response_delay = crate::rtps::behavior::types::Duration::from_millis(100);
        // let nack_suppression_duration = crate::rtps::behavior::types::Duration::from_millis(100);

        // *dw.rtps_writer.lock().unwrap() = Some(
        // StatefulWriter::new(
        //         guid,
        //         topic_kind,
        //         reliability_level,
        //         push_mode,
        //         heartbeat_period,
        //         nack_response_delay,
        //         nack_suppression_duration)
        //     );

        // Ok(())

        todo!()
            
    }

    pub fn get_instance_handle(_this: &Weak<DataWriterImpl<T>>,) -> InstanceHandle {
        todo!()
    }

     //////////////// From here on are the functions that do not belong to the standard API
     pub(crate) fn new(parent_publisher: Weak<PublisherImpl>
     ) -> Self {
         Self{
            parent_publisher,
            // rtps_writer: Mutex::new(None),
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
    
    #[derive(Debug)]
    struct  Foo {
        value: bool
    }
    impl DDSType for Foo {
        fn key(&self) -> InstanceHandle {
            todo!()
        }

        fn data(&self) -> Vec<u8> {
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

        fn data(&self) -> Vec<u8> {
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

        fn data(&self) -> Vec<u8> {
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

        struct Foo {
            value: u8,
        }

        let timestamp = Time {
            sec: 100,
            nanosec: 20000,
        };

        let dw = Arc::new(DataWriterImpl::new(Weak::new()));
        let dw_weak = Arc::downgrade(&dw);

        let new_foo = Foo{value:1};

        DataWriterImpl::write_w_timestamp(&dw_weak, new_foo, None, timestamp).unwrap();
    }
}