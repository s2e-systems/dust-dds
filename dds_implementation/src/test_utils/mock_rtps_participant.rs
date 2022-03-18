use mockall::mock;
use rtps_pim::structure::{
    entity::RtpsEntityAttributes,
    participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
    types::{Guid, Locator, ProtocolVersion, VendorId},
};

mock! {
    pub RtpsParticipant{}

    impl RtpsEntityAttributes for RtpsParticipant {
        fn guid(&self) -> Guid;
    }

    impl RtpsParticipantAttributes for RtpsParticipant {
        fn default_unicast_locator_list(&self) -> &[Locator];
        fn default_multicast_locator_list(&self) -> &[Locator];
        fn protocol_version(&self) -> ProtocolVersion;
        fn vendor_id(&self) -> VendorId;
    }
}

impl RtpsParticipantConstructor for MockRtpsParticipant {
    fn new(
        _guid: Guid,
        _default_unicast_locator_list: &[Locator],
        _default_multicast_locator_list: &[Locator],
        _protocol_version: ProtocolVersion,
        _vendor_id: VendorId,
    ) -> Self {
        MockRtpsParticipant::new()
    }
}
