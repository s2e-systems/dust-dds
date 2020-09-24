
use std::any::Any;
use std::sync::{Arc, Mutex};

use rust_dds_interface::types::{ReturnCode, Duration, InstanceHandle, Time, ReturnCodes};
use crate::infrastructure::status::{StatusKind, StatusMask };
use crate::topic::{Topic, TopicListener, TopicDescription};
use crate::subscription::{Subscriber, SubscriberListener};
use crate::publication::{Publisher, PublisherListener};
use crate::infrastructure::entity::StatusCondition;
use crate::domain::{DomainParticipantListener, DomainId};
use crate::builtin_topics::{TopicBuiltinTopicData, ParticipantBuiltinTopicData};
use crate::infrastructure::listener::NoListener;

use crate::implementation::publisher_impl::PublisherImpl;
use crate::implementation::subscriber_impl::SubscriberImpl;
use crate::implementation::topic_impl::TopicImpl;

use rust_dds_interface::protocol::ProtocolParticipant;
use rust_dds_interface::qos::{DomainParticipantQos, TopicQos, PublisherQos, SubscriberQos,};

pub struct DomainParticipantImpl{
    domain_id: DomainId,
    qos: DomainParticipantQos,
    a_listener: Box<dyn DomainParticipantListener>,
    mask: StatusMask,
    publisher_list: Mutex<Vec<Arc<PublisherImpl>>>,
    default_publisher_qos: Mutex<PublisherQos>,
    subscriber_list: Mutex<Vec<Arc<SubscriberImpl>>>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    topic_list: Mutex<Vec<Arc<TopicImpl>>>,
    default_topic_qos: Mutex<TopicQos>,
    protocol_participant: Box<dyn ProtocolParticipant>,
}

impl DomainParticipantImpl{
    pub(crate) fn create_publisher(
        this: &Arc<DomainParticipantImpl>,
        _qos_list: PublisherQos,
        _a_listener: impl PublisherListener,
        _mask: StatusMask
    ) -> Option<Publisher> {
        let protocol_group = this.protocol_participant.create_group();
        let publisher_impl = Arc::new(PublisherImpl::new(Arc::downgrade(this), protocol_group));
        let publisher = Publisher(Arc::downgrade(&publisher_impl));

        this.publisher_list.lock().ok()?.push(publisher_impl);

        Some(publisher)
    }

    pub(crate) fn delete_publisher(
        this: &Arc<DomainParticipantImpl>,
        a_publisher: &Publisher
    ) -> ReturnCode<()> {
        // TODO: Shouldn't be deleted if it still contains entities but can't yet be done because the publisher is not implemented
        let mut publisher_list = this.publisher_list.lock().unwrap();
        let index = publisher_list.iter().position(|x| std::ptr::eq(x.as_ref(), a_publisher.0.upgrade().unwrap().as_ref())).unwrap();
        publisher_list.swap_remove(index);
        Ok(())
    }

    pub(crate) fn create_subscriber(
        this: &Arc<DomainParticipantImpl>,
        _qos_list: SubscriberQos,
        _a_listener: impl SubscriberListener,
        _mask: StatusMask
    ) -> Option<Subscriber> {
        let protocol_group = this.protocol_participant.create_group();
        let subscriber_impl = Arc::new(SubscriberImpl::new(Arc::downgrade(this), protocol_group));
        let subscriber = Subscriber(Arc::downgrade(&subscriber_impl));

        this.subscriber_list.lock().ok()?.push(subscriber_impl);

        Some(subscriber)
    }

    pub(crate) fn delete_subscriber(
        this: &Arc<DomainParticipantImpl>,
        a_subscriber: &Subscriber,
    ) -> ReturnCode<()> {
        // TODO: Shouldn't be deleted if it still contains entities but can't yet be done because the subscriber is not implemented
        let mut subscriber_list = this.subscriber_list.lock().unwrap();
        let index = subscriber_list.iter().position(|x| std::ptr::eq(x.as_ref(), a_subscriber.0.upgrade().unwrap().as_ref())).unwrap();
        subscriber_list.swap_remove(index);
        Ok(())
    }

    pub(crate) fn create_topic(
        this: &Arc<DomainParticipantImpl>,
        topic_name: String,
        type_name: String,
        _qos_list: TopicQos,
        _a_listener: impl TopicListener,
        _mask: StatusMask
    ) -> Option<Topic> {
        let topic_impl = Arc::new(TopicImpl::new(Arc::downgrade(this), topic_name, type_name));
        let topic = Topic(Arc::downgrade(&topic_impl));

        this.topic_list.lock().ok()?.push(topic_impl);

        Some(topic)
    }

    pub(crate) fn delete_topic(
        this: &Arc<DomainParticipantImpl>,
        a_topic: &Topic,
    ) -> ReturnCode<()> {
        // TODO: Shouldn't be deleted if there are any existing DataReader, DataWriter, ContentFilteredTopic, or MultiTopic
        // objects that are using the Topic. It can't yet be done because the functionality is not implemented
        let mut topic_list = this.topic_list.lock().unwrap();
        let index = topic_list.iter().position(|x| std::ptr::eq(x.as_ref(), a_topic.0.upgrade().unwrap().as_ref())).unwrap();
        topic_list.swap_remove(index);
        Ok(())
    }

    pub(crate) fn find_topic(
        _this: &Arc<DomainParticipantImpl>,
        _topic_name: String,
        _timeout: Duration,
    ) -> Option<Topic> {
        todo!()
    }

    pub(crate) fn lookup_topicdescription(
        _this: &Arc<DomainParticipantImpl>,
        _name: String,
    ) -> Option<&dyn TopicDescription> {
        todo!()
    }

    pub(crate) fn get_builtin_subscriber(_this: &Arc<DomainParticipantImpl>,) -> Subscriber {
        todo!()
    }

    pub(crate) fn ignore_participant(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn ignore_topic(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn ignore_publication(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn ignore_subscription(
        _this: &Arc<DomainParticipantImpl>,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_domain_id(this: &Arc<DomainParticipantImpl>) -> DomainId {
        this.domain_id
    }

    pub(crate) fn delete_contained_entities(_this: &Arc<DomainParticipantImpl>) -> ReturnCode<()> {
        todo!()   
    }

    pub(crate) fn assert_liveliness(_this: &Arc<DomainParticipantImpl>) -> ReturnCode<()> {
        todo!()   
    }

    pub(crate) fn set_default_publisher_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: PublisherQos,
    ) -> ReturnCode<()> {
        *this.default_publisher_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub(crate) fn get_default_publisher_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: &mut PublisherQos,
    ) -> ReturnCode<()> {
        qos.clone_from(&this.default_publisher_qos.lock().unwrap());
        Ok(())
    }

    pub(crate) fn set_default_subscriber_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: SubscriberQos,
    ) -> ReturnCode<()> {
        *this.default_subscriber_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub(crate) fn get_default_subscriber_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: &mut SubscriberQos,
    ) -> ReturnCode<()> {
        qos.clone_from(&this.default_subscriber_qos.lock().unwrap());
        Ok(())
    }

    pub(crate) fn set_default_topic_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: TopicQos,
    ) -> ReturnCode<()> {
        if qos.is_consistent() {
            *this.default_topic_qos.lock().unwrap() = qos;
        } else {
            return Err(ReturnCodes::InconsistentPolicy);
        }
            
        Ok(())
    }

    pub(crate) fn get_default_topic_qos(
        this: &Arc<DomainParticipantImpl>,
        qos: &mut TopicQos,
    ) -> ReturnCode<()> {
        qos.clone_from(&this.default_topic_qos.lock().unwrap());
        Ok(())
    }

    pub(crate) fn get_discovered_participants(
        _this: &Arc<DomainParticipantImpl>,
        _participant_handles: &mut [InstanceHandle]
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_discovered_participant_data(
        _this: &Arc<DomainParticipantImpl>,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_discovered_topics(
        _this: &Arc<DomainParticipantImpl>,
        _topic_handles: &mut [InstanceHandle]
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_discovered_topic_data(
        _this: &Arc<DomainParticipantImpl>,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn contains_entity(
        _this: &Arc<DomainParticipantImpl>,
        _a_handle: InstanceHandle
    ) -> bool {
        todo!()
    }

    pub(crate) fn get_current_time(
        _this: &Arc<DomainParticipantImpl>,
        _current_time: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    //////////////// Entity trait methods
    pub(crate) fn set_qos(_this: &Arc<DomainParticipantImpl>, _qos_list: DomainParticipantQos) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_qos(_this: &Arc<DomainParticipantImpl>, _qos_list: &mut DomainParticipantQos) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn set_listener(_this: &Arc<DomainParticipantImpl>, _a_listener: Box<dyn DomainParticipantListener>, _mask: &[StatusKind]) -> ReturnCode<()> {
        todo!()
    }

    pub(crate) fn get_listener(_this: &Arc<DomainParticipantImpl>, ) -> Box<dyn DomainParticipantListener> {
        todo!()
    }

    pub(crate) fn get_statuscondition(_this: &Arc<DomainParticipantImpl>) -> StatusCondition {
        todo!()
    }

    pub(crate) fn get_status_changes(_this: &Arc<DomainParticipantImpl>) -> StatusKind {
        todo!()
    }

    pub(crate) fn enable(_this: &Arc<DomainParticipantImpl>) -> ReturnCode<()> {
        //TODO: This is to prevent the ParticipantFactory test from panicking
        Ok(())
    }

    pub(crate) fn get_instance_handle(this: &Arc<DomainParticipantImpl>) -> ReturnCode<InstanceHandle> {
        Ok(this.protocol_participant.get_instance_handle())
    }



    //////////////// From here on are the functions that do not belong to the standard API
    pub(crate) fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        a_listener: impl DomainParticipantListener,
        mask: StatusMask,
        protocol_participant: Box<dyn ProtocolParticipant>,
    ) -> Self {
        
        if !Any::is::<NoListener>(&a_listener) {
            println!("TODO: Use the real listener")
        }

        
        

        Self {
            domain_id,
            qos,
            a_listener: Box::new(a_listener),
            mask,
            publisher_list: Mutex::new(Vec::new()),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            subscriber_list: Mutex::new(Vec::new()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            topic_list: Mutex::new(Vec::new()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            protocol_participant,
        }
    }

 
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Weak;
    use crate::infrastructure::listener::NoListener;
    use rust_dds_interface::protocol::ProtocolEntity;
    use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

    struct MockProtocolParticipant;
    impl ProtocolEntity for MockProtocolParticipant{
        fn get_instance_handle(&self) -> InstanceHandle {
            todo!()
        }
    }

    impl ProtocolParticipant for MockProtocolParticipant {
        fn create_group(&self) -> Weak<dyn rust_dds_interface::protocol::ProtocolGroup> {
            todo!()
        }
    }

    #[test]
    fn create_publisher() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        assert_eq!(domain_participant_impl.publisher_list.lock().unwrap().len(), 0);
        let publisher = DomainParticipantImpl::create_publisher(&domain_participant_impl,PublisherQos::default(), NoListener, 0).unwrap();
        assert_eq!(domain_participant_impl.publisher_list.lock().unwrap().len(), 1);

        DomainParticipantImpl::delete_publisher(&domain_participant_impl, &publisher).unwrap();

        assert_eq!(domain_participant_impl.publisher_list.lock().unwrap().len(), 0);
    }

    #[test]
    fn create_subscriber() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        assert_eq!(domain_participant_impl.subscriber_list.lock().unwrap().len(), 0);
        let subscriber = DomainParticipantImpl::create_subscriber(&domain_participant_impl,SubscriberQos::default(), NoListener, 0).unwrap();
        assert_eq!(domain_participant_impl.subscriber_list.lock().unwrap().len(), 1);

        DomainParticipantImpl::delete_subscriber(&domain_participant_impl, &subscriber).unwrap();

        assert_eq!(domain_participant_impl.subscriber_list.lock().unwrap().len(), 0);
    }

    #[test]
    fn create_topic() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        assert_eq!(domain_participant_impl.topic_list.lock().unwrap().len(), 0);
        let topic = DomainParticipantImpl::create_topic(&domain_participant_impl,"name".to_string(), "type".to_string(), TopicQos::default(), NoListener, 0).unwrap();
        assert_eq!(domain_participant_impl.topic_list.lock().unwrap().len(), 1);

        DomainParticipantImpl::delete_topic(&domain_participant_impl, &topic).unwrap();

        assert_eq!(domain_participant_impl.topic_list.lock().unwrap().len(), 0);
    }

    #[test]
    fn set_and_get_default_publisher_qos() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.partition.name = String::from("test");
        publisher_qos.entity_factory.autoenable_created_entities = false;

        DomainParticipantImpl::set_default_publisher_qos(&domain_participant_impl, publisher_qos.clone()).unwrap();
        assert_eq!(*domain_participant_impl.default_publisher_qos.lock().unwrap(), publisher_qos);

        let mut read_publisher_qos = PublisherQos::default();
        DomainParticipantImpl::get_default_publisher_qos(&domain_participant_impl, &mut read_publisher_qos).unwrap();

        assert_eq!(read_publisher_qos, publisher_qos);
    }

    #[test]
    fn set_and_get_default_subscriber_qos() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.partition.name = String::from("test");
        subscriber_qos.entity_factory.autoenable_created_entities = false;

        DomainParticipantImpl::set_default_subscriber_qos(&domain_participant_impl, subscriber_qos.clone()).unwrap();
        assert_eq!(*domain_participant_impl.default_subscriber_qos.lock().unwrap(), subscriber_qos);

        let mut read_subscriber_qos = SubscriberQos::default();
        DomainParticipantImpl::get_default_subscriber_qos(&domain_participant_impl, &mut read_subscriber_qos).unwrap();

        assert_eq!(read_subscriber_qos, subscriber_qos);
    }

    #[test]
    fn set_and_get_default_topic_qos() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        let mut topic_qos = TopicQos::default();
        topic_qos.topic_data.value = vec![1,2,3,4];
        topic_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        DomainParticipantImpl::set_default_topic_qos(&domain_participant_impl, topic_qos.clone()).unwrap();
        assert_eq!(*domain_participant_impl.default_topic_qos.lock().unwrap(), topic_qos);

        let mut read_topic_qos = TopicQos::default();
        DomainParticipantImpl::get_default_topic_qos(&domain_participant_impl, &mut read_topic_qos).unwrap();

        assert_eq!(read_topic_qos, topic_qos);
    }

    #[test]
    fn inconsistent_datareader_qos() {
        let domain_participant_impl = Arc::new(DomainParticipantImpl::new(0, DomainParticipantQos::default(), NoListener, 0, Box::new(MockProtocolParticipant)));

        let mut topic_qos = TopicQos::default();
        topic_qos.resource_limits.max_samples = 5;
        topic_qos.resource_limits.max_samples_per_instance = 15;

        let error = DomainParticipantImpl::set_default_topic_qos(&domain_participant_impl, topic_qos.clone());
        assert_eq!(error, Err(ReturnCodes::InconsistentPolicy));

        assert_eq!(*domain_participant_impl.default_topic_qos.lock().unwrap(), TopicQos::default());
    }
}