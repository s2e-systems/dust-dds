use rust_dds_api::{builtin_topics::ParticipantBuiltinTopicData, dcps_psm::BuiltInTopicKey, infrastructure::{qos::{DataWriterQos, PublisherQos}, qos_policy::UserDataQosPolicy}, publication::data_writer::DataWriter};
use rust_rtps_pim::{behavior::types::Duration, discovery::{
        spdp::{
            builtin_endpoints::SpdpBuiltinParticipantWriter, participant_proxy::ParticipantProxy,
        },
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    }, messages::types::Count, structure::{types::{BUILT_IN_WRITER_GROUP, EntityId, Guid, Locator, ProtocolVersion}}};

use crate::{data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData, dds_impl::{data_writer_impl::DataWriterImpl, publisher_impl::PublisherImpl}, rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_writer_impl::RtpsWriterImpl}, utils::shared_object::RtpsShared};

#[test]
fn send_discovery_data_happy_path() {
    let spdp_discovery_locator = Locator::new(11, 12, [1; 16]);

    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey { value: [8, 8, 8] },
        user_data: UserDataQosPolicy { value: &[] },
    };
    let participant_proxy = ParticipantProxy {
        domain_id: 1,
        domain_tag: "ab",
        protocol_version: ProtocolVersion { major: 2, minor: 4 },
        guid_prefix: [8; 12],
        vendor_id: [73, 74],
        expects_inline_qos: false,
        metatraffic_unicast_locator_list: vec![Locator::new(11, 12, [1; 16])],
        metatraffic_multicast_locator_list: vec![],
        default_unicast_locator_list: vec![],
        default_multicast_locator_list: vec![],
        available_builtin_endpoints: BuiltinEndpointSet::new(
            BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
        ),
        manual_liveliness_count: Count(0),
        builtin_endpoint_qos: BuiltinEndpointQos::default(),
    };
    let lease_duration = Duration {
        seconds: 100,
        fraction: 0,
    };

    let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data,
        participant_proxy,
        lease_duration,
    };

    let spdp_builtin_participant_rtps_writer: RtpsWriterImpl =
        SpdpBuiltinParticipantWriter::create([3; 12], &[], &[], &[spdp_discovery_locator]);

    let mut data_writer = DataWriterImpl::new(
        DataWriterQos::default(),
        spdp_builtin_participant_rtps_writer,
    );

    data_writer
        .write_w_timestamp(
            spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();

    let _publisher = PublisherImpl::new(
        PublisherQos::default(),
        RtpsGroupImpl::new(Guid::new(
            [4; 12],
            EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
        )),
        vec![RtpsShared::new(data_writer)],
    );


    // publisher.send_data(participant, transport)
}
