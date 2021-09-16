// use rust_rtps_pim::structure::{
//     types::{Guid, Locator, ProtocolVersion, VendorId, PROTOCOLVERSION_2_4},
//     RtpsEntity, RtpsParticipant,
// };

// pub struct RtpsParticipantImpl {
//     guid: Guid,
//     protocol_version: ProtocolVersion,
//     vendor_id: VendorId,
// }

// impl RtpsParticipantImpl {
//     pub fn new(guid_prefix: rust_rtps_pim::structure::types::GuidPrefix) -> Self {
//         let guid = Guid::new(
//             guid_prefix,
//             rust_rtps_pim::structure::types::ENTITYID_PARTICIPANT,
//         );

//         Self {
//             guid,
//             protocol_version: PROTOCOLVERSION_2_4,
//             vendor_id: [99, 99],
//         }
//     }
// }

// impl RtpsEntity for RtpsParticipantImpl {
//     fn guid(&self) -> &Guid {
//         &self.guid
//     }
// }

// impl RtpsParticipant for RtpsParticipantImpl {
//     fn protocol_version(&self) -> &ProtocolVersion {
//         &self.protocol_version
//     }

//     fn vendor_id(&self) -> &VendorId {
//         &self.vendor_id
//     }

//     fn default_unicast_locator_list(&self) -> &[Locator] {
//         todo!()
//     }

//     fn default_multicast_locator_list(&self) -> &[Locator] {
//         todo!()
//     }
// }
