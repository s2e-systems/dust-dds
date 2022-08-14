use crate::implementation::dds_impl::domain_participant_impl::{
    CreateBuiltIns, DomainParticipantImpl, SendUserDefinedData,
};
use crate::implementation::rtps::messages::RtpsMessage;
use crate::implementation::rtps::participant::RtpsParticipant;
use crate::implementation::rtps::transport::TransportWrite;
use crate::implementation::rtps::types::{
    GuidPrefix, LOCATOR_KIND_UDPv4, Locator, PROTOCOLVERSION, VENDOR_ID_S2E,
};
use crate::infrastructure::{entity::Entity, qos::DomainParticipantQos};

use mockall::mock;

mock! {
    Transport{}

    impl TransportWrite for Transport {
        fn write<'a>(&'a mut self, message: &RtpsMessage<'a>, destination_locator: Locator);
    }
}

#[test]
fn participant_sends_spdp_discovery() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let dp = DomainParticipantImpl::new(
        rtps_participant,
        1,
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
    );

    dp.enable().unwrap();
    dp.create_builtins().unwrap();

    let mut mock_transport = MockTransport::new();
    let expected_locator = Locator::new(
        LOCATOR_KIND_UDPv4,
        7400,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
    );
    mock_transport
        .expect_write()
        .withf(move |message, locator| {
            message.submessages.len() == 2 && locator == &expected_locator
        })
        .return_const(());
    dp.send_user_defined_data(&mut mock_transport);
}
