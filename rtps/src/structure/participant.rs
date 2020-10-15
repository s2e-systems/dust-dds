use std::sync::{Arc, Weak, Mutex};
use crate::types::{GUID, ProtocolVersion, VendorId, EntityId, EntityKind};
use crate::types::constants::{
    ENTITYID_PARTICIPANT,
    PROTOCOL_VERSION_2_4,};
use crate::transport::Transport;

use super::publisher::RtpsPublisher;
use super::subscriber::RtpsSubscriber;
use super::builtin_publisher::BuiltinPublisher;
use super::builtin_subscriber::BuiltinSubscriber;

use rust_dds_interface::types::{DomainId, InstanceHandle, ReturnCode};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolParticipant, ProtocolPublisher, ProtocolSubscriber};

pub struct RtpsParticipant {
    guid: GUID,
    domain_id: DomainId,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    builtin_publisher: BuiltinPublisher,
    builtin_subscriber: Arc<BuiltinSubscriber>, 
    publisher_list: [Weak<Mutex<RtpsPublisher>>;32],
    subscriber_list:[Weak<Mutex<RtpsSubscriber>>;32],
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        userdata_transport: impl Transport + 'static,
        metatraffic_transport: impl Transport + 'static,
    ) -> Self {
        let protocol_version = PROTOCOL_VERSION_2_4;
        let vendor_id = [99,99];
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];   // TODO: Should be uniquely generated
        let builtin_publisher = BuiltinPublisher;
        let builtin_subscriber = Arc::new(BuiltinSubscriber);

        Self {
            guid: GUID::new(guid_prefix,ENTITYID_PARTICIPANT ),
            domain_id,
            protocol_version,
            vendor_id,
            userdata_transport: Box::new(userdata_transport),
            metatraffic_transport: Box::new(metatraffic_transport),
            builtin_subscriber,
            builtin_publisher,
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
        }
    }

    pub fn send(&self) {
        let valid_publishers = self.publisher_list.iter().filter_map(|p|p.upgrade());
        valid_publishers.for_each(|p|p.lock().unwrap().send(self.userdata_transport.as_ref()));
    }

    pub fn guid(&self) -> GUID {
        self.guid
    }

    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn userdata_transport(&self) -> &Box<dyn Transport> {
        &self.userdata_transport
    }

    pub fn metatraffic_transport(&self) -> &Box<dyn Transport> {
        &self.metatraffic_transport
    }    
}

impl ProtocolEntity for RtpsParticipant {
    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }
}

impl ProtocolParticipant for RtpsParticipant {
    fn create_publisher(&mut self) -> Arc<Mutex<dyn ProtocolPublisher>> {
        let index = self.publisher_list.iter().position(|x| x.strong_count() == 0).unwrap();

        let guid_prefix = self.guid.prefix();
        let entity_id = EntityId::new([index as u8,0,0], EntityKind::UserDefinedWriterGroup);
        let publisher_guid = GUID::new(guid_prefix, entity_id);
        let new_publisher = Arc::new(Mutex::new(RtpsPublisher::new(publisher_guid)));
        self.publisher_list[index] = Arc::downgrade(&new_publisher);

        new_publisher
    }

    fn create_subscriber(&mut self) -> Arc<Mutex<dyn ProtocolSubscriber>> {
        let index = self.subscriber_list.iter().position(|x| x.strong_count() == 0).unwrap();

        let guid_prefix = self.guid.prefix();
        let entity_id = EntityId::new([index as u8,0,0], EntityKind::UserDefinedReaderGroup);
        let subscriber_guid = GUID::new(guid_prefix, entity_id);
        let new_subscriber = Arc::new(Mutex::new(RtpsSubscriber::new(subscriber_guid)));
        self.subscriber_list[index] = Arc::downgrade(&new_subscriber);

        new_subscriber
    }

    fn get_builtin_subscriber(&self) -> Arc<dyn ProtocolSubscriber> {
        self.builtin_subscriber.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Locator;

    struct MockTransport{
        multicast_locator_list: Vec<Locator>,
    }

    impl MockTransport{
        fn new() -> Self {
            Self {
                multicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
            }
        }
    }

    impl Transport for MockTransport {
        fn write(&self, _message: crate::RtpsMessage, _destination_locator_list: &[Locator]) {
            todo!()
        }

        fn read(&self) -> crate::transport::TransportResult<Option<(crate::RtpsMessage, Locator)>> {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<Locator> {
            todo!()
        }

        fn multicast_locator_list(&self) -> &Vec<Locator> {
            &self.multicast_locator_list
        }

        fn as_any(&self) -> &dyn std::any::Any {
            todo!()
        }
    }

    #[test]
    fn create_publisher() {
        let mut participant = RtpsParticipant::new(0, MockTransport::new(), MockTransport::new());
        let participant_guid_prefix = &participant.get_instance_handle()[0..12];

        assert_eq!(participant.publisher_list[0].strong_count(),0);
        assert_eq!(participant.publisher_list[1].strong_count(),0);

        let publisher1_arc = participant.create_publisher();
        let publisher1 = publisher1_arc.lock().unwrap();
        let publisher1_entityid = [0,0,0,8];
        assert_eq!(&publisher1.get_instance_handle()[0..12], participant_guid_prefix); 
        assert_eq!(publisher1.get_instance_handle()[12..16], publisher1_entityid);

        assert_eq!(participant.publisher_list[0].strong_count(),1);
        assert_eq!(participant.publisher_list[1].strong_count(),0);

        let publisher2 = participant.create_publisher();
        let publisher2 = publisher2.lock().unwrap();
        let publisher2_entityid = [1,0,0,8];
        assert_eq!(&publisher2.get_instance_handle()[0..12], participant_guid_prefix); 
        assert_eq!(publisher2.get_instance_handle()[12..16], publisher2_entityid);

        assert_eq!(participant.publisher_list[0].strong_count(),1);
        assert_eq!(participant.publisher_list[1].strong_count(),1);

        std::mem::drop(publisher1);
        std::mem::drop(publisher1_arc);

        assert_eq!(participant.publisher_list[0].strong_count(),0);
        assert_eq!(participant.publisher_list[1].strong_count(),1);

        let publisher3 = participant.create_publisher();
        let publisher3 = publisher3.lock().unwrap();
        let publisher3_entityid = [0,0,0,8];
        assert_eq!(&publisher3.get_instance_handle()[0..12], participant_guid_prefix); 
        assert_eq!(publisher3.get_instance_handle()[12..16], publisher3_entityid);

        assert_eq!(participant.publisher_list[0].strong_count(),1);
        assert_eq!(participant.publisher_list[1].strong_count(),1);
    }

    #[test]
    fn create_subscriber() {
        let mut participant = RtpsParticipant::new(0, MockTransport::new(), MockTransport::new());
        let participant_guid_prefix = &participant.get_instance_handle()[0..12];

        assert_eq!(participant.subscriber_list[0].strong_count(),0);
        assert_eq!(participant.subscriber_list[1].strong_count(),0);

        let subscriber1_arc = participant.create_subscriber();
        let subscriber1 = subscriber1_arc.lock().unwrap();
        let subscriber1_entityid = [0,0,0,9];
        assert_eq!(&subscriber1.get_instance_handle()[0..12], participant_guid_prefix); 
        assert_eq!(subscriber1.get_instance_handle()[12..16], subscriber1_entityid);

        assert_eq!(participant.subscriber_list[0].strong_count(),1);
        assert_eq!(participant.subscriber_list[1].strong_count(),0);

        let subscriber2 = participant.create_subscriber();
        let subscriber2 = subscriber2.lock().unwrap();
        let subscriber2_entityid = [1,0,0,9];
        assert_eq!(&subscriber2.get_instance_handle()[0..12], participant_guid_prefix); 
        assert_eq!(subscriber2.get_instance_handle()[12..16], subscriber2_entityid);

        assert_eq!(participant.subscriber_list[0].strong_count(),1);
        assert_eq!(participant.subscriber_list[1].strong_count(),1);

        std::mem::drop(subscriber1);
        std::mem::drop(subscriber1_arc);

        assert_eq!(participant.subscriber_list[0].strong_count(),0);
        assert_eq!(participant.subscriber_list[1].strong_count(),1);

        let subscriber3 = participant.create_subscriber();
        let subscriber3 = subscriber3.lock().unwrap();
        let subscriber3_entityid = [0,0,0,9];
        assert_eq!(&subscriber3.get_instance_handle()[0..12], participant_guid_prefix); 
        assert_eq!(subscriber3.get_instance_handle()[12..16], subscriber3_entityid);

        assert_eq!(participant.subscriber_list[0].strong_count(),1);
        assert_eq!(participant.subscriber_list[1].strong_count(),1);
    }
}

