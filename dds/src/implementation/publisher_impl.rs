use std::sync::{Arc, Weak, Mutex};

use crate::types::DDSType;
use rust_dds_interface::types::{ReturnCode, Duration, InstanceHandle, ReturnCodes};
use crate::infrastructure::status::StatusMask;
use crate::domain::DomainParticipant;
use crate::topic::Topic;
use crate::infrastructure::entity::StatusCondition;
use crate::publication::{PublisherListener, DataWriter, AnyDataWriter, DataWriterListener};
use crate::implementation::domain_participant_impl::DomainParticipantImpl;
use crate::implementation::data_writer_impl::DataWriterImpl;

use rust_dds_interface::qos::{TopicQos, PublisherQos, DataWriterQos};

pub struct PublisherImpl{
    parent_participant: Weak<DomainParticipantImpl>,
    instance_handle: InstanceHandle,
    datawriter_list: Mutex<Vec<Box<dyn AnyDataWriter>>>,
    default_datawriter_qos: Mutex<DataWriterQos>,
}

impl PublisherImpl {
    pub(crate) fn create_datawriter<T: DDSType>(
        this: &Weak<PublisherImpl>,
        _a_topic: Topic,
        qos: DataWriterQos,
        _a_listener: Box<dyn DataWriterListener<T>>,
        _mask: StatusMask,
    ) -> Option<DataWriter<T>> {
        let publisher = PublisherImpl::upgrade_publisher(this).ok()?;
        let protocol_writer = publisher
            .parent_participant
            .upgrade()
            .unwrap()
            .protocol_participant()
            .lock()
            .unwrap()
            .create_writer(T::topic_kind(), &qos);
        let datawriter_impl = Arc::new(DataWriterImpl::new(this.clone(), protocol_writer));
        let datawriter = DataWriter(Arc::downgrade(&datawriter_impl)); 
        let datawriter_2 = DataWriter(Arc::downgrade(&datawriter_impl)); 

        publisher.datawriter_list.lock().ok()?.push(Box::new(datawriter_2));

        Some(datawriter)
    }

    pub(crate) fn delete_datawriter<T: DDSType>(
        this: &Weak<PublisherImpl>,
        a_datawriter: &DataWriter<T>
    ) -> ReturnCode<()> {
        let publisher = PublisherImpl::upgrade_publisher(this)?;
        let mut datawriter_list = publisher.datawriter_list.lock().unwrap();
        let index = datawriter_list.iter().position(|x| 
            match x.get::<T>() {
                Some(dw) => dw.0.ptr_eq(&a_datawriter.0),
                None => false,
        });
        
        if let Some(index) = index{
            datawriter_list.swap_remove(index);
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet("Data writer not found in publisher"))
        }
    }

    pub(crate) fn lookup_datawriter<T: DDSType>(
        _this: &Weak<PublisherImpl>,
        _topic_name: String,
    ) -> Option<DataWriter<T>> {
        todo!()
    }

    pub(crate) fn suspend_publications(_this: &Weak<PublisherImpl>) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn resume_publications(_this: &Weak<PublisherImpl>) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn begin_coherent_changes(_this: &Weak<PublisherImpl>) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn end_coherent_changes(_this: &Weak<PublisherImpl>) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn wait_for_acknowledgments(
        _this: &Weak<PublisherImpl>,
        _max_wait: Duration
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_participant(this: &Weak<PublisherImpl>) -> DomainParticipant {
        DomainParticipant(this.upgrade().unwrap().parent_participant.upgrade().unwrap())
    }

    pub(crate) fn delete_contained_entities(_this: &Weak<PublisherImpl>) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn set_default_datawriter_qos(
        this: &Weak<PublisherImpl>,
        qos: DataWriterQos,
    ) -> ReturnCode<()> {
        let publisher = PublisherImpl::upgrade_publisher(this)?;

        if qos.is_consistent() {
            *publisher.default_datawriter_qos.lock().unwrap() = qos;
        } else {
            return Err(ReturnCodes::InconsistentPolicy);
        }
        
        Ok(())
    }

    pub(crate) fn get_default_datawriter_qos (
        this: &Weak<PublisherImpl>,
        qos: &mut DataWriterQos,
    ) -> ReturnCode<()> {
        let publisher = PublisherImpl::upgrade_publisher(this)?;

        qos.clone_from(&publisher.default_datawriter_qos.lock().unwrap());
        Ok(())
    }

    pub(crate) fn copy_from_topic_qos(
        _this: &Weak<PublisherImpl>,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    ///////////////// Entity trait methods
    pub(crate) fn set_qos(_this: &Weak<PublisherImpl>, _qos_list: PublisherQos) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_qos(_this: &Weak<PublisherImpl>, _qos_list: &mut PublisherQos) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn set_listener(_this: &Weak<PublisherImpl>, _a_listener: Box<dyn PublisherListener>, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_listener(_this: &Weak<PublisherImpl>, ) -> Box<dyn PublisherListener> {
        todo!()
    }

    pub(crate) fn get_statuscondition(_this: &Weak<PublisherImpl>, ) -> StatusCondition {
        todo!()
    }

    pub(crate) fn get_status_changes(_this: &Weak<PublisherImpl>, ) -> StatusMask {
        todo!()
    }

    pub(crate) fn enable(_this: &Weak<PublisherImpl>, ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_instance_handle(this: &Weak<PublisherImpl>) -> ReturnCode<InstanceHandle> {
        let publisher = PublisherImpl::upgrade_publisher(this)?;
        Ok(publisher.instance_handle)
    }

    //////////////// From here on are the functions that do not belong to the standard API
    pub(crate) fn new(parent_participant: Weak<DomainParticipantImpl>, instance_handle: InstanceHandle) -> Self {
        Self{
            parent_participant,
            datawriter_list: Mutex::new(Vec::new()),
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            instance_handle
        }
    }

    fn upgrade_publisher(this: &Weak<PublisherImpl>) -> ReturnCode<Arc<PublisherImpl>> {
        this.upgrade().ok_or(ReturnCodes::AlreadyDeleted("Publisher"))
    }

    pub fn has_no_datawriters(&self) -> bool {
        self.datawriter_list.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::listener::NoListener;
    use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter};
    use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;
    use rust_dds_interface::types::{Data, TopicKind};

    struct MockWriter;
    impl ProtocolEntity for MockWriter {
        fn enable(&self) -> ReturnCode<()> {
            todo!()
        }

        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }
    }
    impl ProtocolWriter for MockWriter {
        fn write(&mut self, _instance_handle: InstanceHandle, _data: Data, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
            todo!()
        }

        fn dispose(&self, _instance_handle: InstanceHandle, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
            todo!()
        }

        fn unregister(&self, _instance_handle: InstanceHandle, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
            todo!()
        }

        fn register(&self, _instance_handle: InstanceHandle, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<Option<InstanceHandle>> {
            todo!()
        }

        fn lookup_instance(&self, _instance_handle: InstanceHandle) -> Option<InstanceHandle> {
            todo!()
        }
    }

    struct MockWriterProtocolGroup;
    impl ProtocolEntity for MockWriterProtocolGroup{
        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }

        fn enable(&self) -> ReturnCode<()> {
            todo!()
        }
    }

    #[derive(Debug)]
    struct  Foo {
        value: bool
    }

    impl DDSType for Foo {
        fn topic_kind() -> TopicKind {
            TopicKind::NoKey
        }

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
    struct Bar {
        value: i32
    }

    impl DDSType for Bar {
        fn topic_kind() -> TopicKind {
            TopicKind::NoKey
        }

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
    fn create_delete_datawriter() {
        let instance_handle = [1;16];
        let publisher_impl = Arc::new(PublisherImpl::new(Weak::new(), instance_handle));
        let topic = Topic(Weak::new());
        
        assert_eq!(publisher_impl.datawriter_list.lock().unwrap().len(), 0);
        let datawriter = PublisherImpl::create_datawriter::<Foo>(&Arc::downgrade(&publisher_impl),topic, DataWriterQos::default(), Box::new(NoListener), 0).unwrap();
        assert_eq!(publisher_impl.datawriter_list.lock().unwrap().len(), 1);
        
        PublisherImpl::delete_datawriter(&Arc::downgrade(&publisher_impl), &datawriter).unwrap();
        assert_eq!(publisher_impl.datawriter_list.lock().unwrap().len(), 0);
    }

    #[test]
    fn any_datawriter() {
        let instance_handle = [1;16];
        let publisher_impl = Arc::new(PublisherImpl::new(Weak::new(), instance_handle));
        let topic_foo = Topic(Weak::new());
        let topic_bar = Topic(Weak::new());
        
        let _datawriter_foo = PublisherImpl::create_datawriter::<Foo>(&Arc::downgrade(&publisher_impl),topic_foo, DataWriterQos::default(), Box::new(NoListener), 0).unwrap();
        let _datawriter_bar = PublisherImpl::create_datawriter::<Bar>(&Arc::downgrade(&publisher_impl),topic_bar, DataWriterQos::default(), Box::new(NoListener), 0).unwrap();

        let list = publisher_impl.datawriter_list.lock().unwrap();
        assert!(list[0].get::<Foo>().is_some());
        assert!(list[0].get::<Bar>().is_none());

        assert!(list[1].get::<Foo>().is_none());
        assert!(list[1].get::<Bar>().is_some());

        // PublisherImpl::delete_datawriter(&Arc::downgrade(&publisher_impl), &datawriter_bar).unwrap();
        // assert_eq!(publisher_impl.datawriter_list.lock().unwrap().len(), 0);

        // PublisherImpl::delete_datawriter(&Arc::downgrade(&publisher_impl), &datawriter_foo).unwrap();
        // assert_eq!(publisher_impl.datawriter_list.lock().unwrap().len(), 0);
        
    }

    #[test]
    fn set_and_get_default_datawriter_qos() {
        let instance_handle = [1;16];
        let publisher_impl = Arc::new(PublisherImpl::new(Weak::new(), instance_handle));
        let publisher = Arc::downgrade(&publisher_impl);

        let mut datawriter_qos = DataWriterQos::default();
        datawriter_qos.user_data.value = vec![1,2,3,4];
        datawriter_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        PublisherImpl::set_default_datawriter_qos(&publisher, datawriter_qos.clone()).unwrap();
        assert_eq!(*publisher_impl.default_datawriter_qos.lock().unwrap(), datawriter_qos);

        let mut read_datawriter_qos = DataWriterQos::default();
        PublisherImpl::get_default_datawriter_qos(&publisher, &mut read_datawriter_qos).unwrap();

        assert_eq!(read_datawriter_qos, datawriter_qos);
    }

    #[test]
    fn inconsistent_datawriter_qos() {
        let instance_handle = [1;16];
        let publisher_impl = Arc::new(PublisherImpl::new(Weak::new(), instance_handle));
        let publisher = Arc::downgrade(&publisher_impl);

        let mut datawriter_qos = DataWriterQos::default();
        datawriter_qos.resource_limits.max_samples = 5;
        datawriter_qos.resource_limits.max_samples_per_instance = 15;

        let error = PublisherImpl::set_default_datawriter_qos(&publisher, datawriter_qos.clone());
        assert_eq!(error, Err(ReturnCodes::InconsistentPolicy));

        assert_eq!(*publisher_impl.default_datawriter_qos.lock().unwrap(), DataWriterQos::default());
    }
}