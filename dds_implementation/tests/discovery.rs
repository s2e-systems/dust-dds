use dds_api::infrastructure::{entity::Entity, qos::DomainParticipantQos};
use dds_implementation::{
    dds_impl::domain_participant_impl::{
        CreateBuiltIns, DomainParticipantImpl, SendUserDefinedData,
    },
    transport::{RtpsMessage, TransportWrite},
};
use mockall::mock;
use rtps_pim::structure::types::{GuidPrefix, LOCATOR_KIND_UDPv4, Locator};

mock! {
    Transport{}

    impl TransportWrite for Transport {
        fn write<'a>(&'a mut self, message: &RtpsMessage<'a>, destination_locator: Locator);
    }
}

#[test]
fn participant_sends_spdp_discovery() {
    let dp = DomainParticipantImpl::new(
        GuidPrefix([0; 12]),
        1,
        String::new(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
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
