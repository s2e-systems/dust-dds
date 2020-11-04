use std::sync::{Arc, Mutex, };

use crate::types::{GUID, GuidPrefix, ProtocolVersion, VendorId};
use crate::types::constants::{
    ENTITYID_PARTICIPANT,
    PROTOCOL_VERSION_2_4,};

use super::{RtpsGroup, RtpsEntity};

use rust_dds_interface::types::DomainId;

pub struct RtpsParticipant {
    guid: GUID,
    domain_id: DomainId,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,

    groups: Vec<Arc<Mutex<RtpsGroup>>>,
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        guid_prefix: GuidPrefix,
    ) -> Self {
        let protocol_version = PROTOCOL_VERSION_2_4;
        let vendor_id = [99,99];

        Self {
            guid: GUID::new(guid_prefix,ENTITYID_PARTICIPANT ),
            domain_id,
            protocol_version,
            vendor_id,
            groups: Vec::new(),
        }
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
    
    pub fn mut_groups(&mut self) -> &mut Vec<Arc<Mutex<RtpsGroup>>> {
        &mut self.groups
    }
}

impl RtpsEntity for RtpsParticipant {
    fn guid(&self) -> GUID {
        self.guid
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::Locator;

//     struct MockTransport{
//         multicast_locator_list: Vec<Locator>,
//     }

//     impl MockTransport{
//         fn new() -> Self {
//             Self {
//                 multicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
//             }
//         }
//     }

//     impl Transport for MockTransport {
//         fn write(&self, _message: crate::RtpsMessage, _destination_locator: &Locator) {
//             todo!()
//         }

//         fn read(&self) -> crate::transport::TransportResult<Option<(crate::RtpsMessage, Locator)>> {
//             todo!()
//         }

//         fn unicast_locator_list(&self) -> &Vec<Locator> {
//             todo!()
//         }

//         fn multicast_locator_list(&self) -> &Vec<Locator> {
//             &self.multicast_locator_list
//         }
//     }

//     #[test]
//     fn create_publisher() {
//         let guid_prefix = [1;12];
//         let mut participant = RtpsParticipant::new(0, guid_prefix, MockTransport::new(), MockTransport::new());
//         let participant_guid_prefix = &participant.get_instance_handle()[0..12];

//         let publisher1 = participant.create_publisher();
//         let publisher1 = publisher1.lock().unwrap();
//         let publisher1_entityid = [0,0,0,8];
//         assert_eq!(&publisher1.get_instance_handle()[0..12], participant_guid_prefix); 
//         assert_eq!(publisher1.get_instance_handle()[12..16], publisher1_entityid);

//         let publisher2 = participant.create_publisher();
//         let publisher2 = publisher2.lock().unwrap();
//         let publisher2_entityid = [1,0,0,8];
//         assert_eq!(&publisher2.get_instance_handle()[0..12], participant_guid_prefix); 
//         assert_eq!(publisher2.get_instance_handle()[12..16], publisher2_entityid);

//         std::mem::drop(publisher1);

//         let publisher3 = participant.create_publisher();
//         let publisher3 = publisher3.lock().unwrap();
//         let publisher3_entityid = [0,0,0,8];
//         assert_eq!(&publisher3.get_instance_handle()[0..12], participant_guid_prefix); 
//         assert_eq!(publisher3.get_instance_handle()[12..16], publisher3_entityid);
//     }

//     #[test]
//     fn create_subscriber() {
//         let guid_prefix = [1;12];
//         let mut participant = RtpsParticipant::new(0, guid_prefix, MockTransport::new(), MockTransport::new());
//         let participant_guid_prefix = &participant.get_instance_handle()[0..12];

//         let subscriber1 = participant.create_subscriber();
//         let subscriber1 = subscriber1.lock().unwrap();
//         let subscriber1_entityid = [0,0,0,9];
//         assert_eq!(&subscriber1.get_instance_handle()[0..12], participant_guid_prefix); 
//         assert_eq!(subscriber1.get_instance_handle()[12..16], subscriber1_entityid);

//         let subscriber2 = participant.create_subscriber();
//         let subscriber2 = subscriber2.lock().unwrap();
//         let subscriber2_entityid = [1,0,0,9];
//         assert_eq!(&subscriber2.get_instance_handle()[0..12], participant_guid_prefix); 
//         assert_eq!(subscriber2.get_instance_handle()[12..16], subscriber2_entityid);

//         std::mem::drop(subscriber1);

//         let subscriber3 = participant.create_subscriber();
//         let subscriber3 = subscriber3.lock().unwrap();
//         let subscriber3_entityid = [0,0,0,9];
//         assert_eq!(&subscriber3.get_instance_handle()[0..12], participant_guid_prefix); 
//         assert_eq!(subscriber3.get_instance_handle()[12..16], subscriber3_entityid);
//     }
// }

