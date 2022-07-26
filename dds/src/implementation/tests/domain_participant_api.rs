use crate::implementation::{
    data_representation_builtin_endpoints::spdp_discovered_participant_data::{
        ParticipantProxy, SpdpDiscoveredParticipantData,
    },
    dds_impl::domain_participant_impl::{
        AddDiscoveredParticipant, CreateBuiltIns, DomainParticipantImpl,
    },
};
use crate::{
    dds_type::DdsType,
    return_type::DdsError,
    {
        builtin_topics::ParticipantBuiltinTopicData,
        dcps_psm::{BuiltInTopicKey, DomainId},
        infrastructure::{
            entity::Entity, qos::DomainParticipantQos, qos_policy::UserDataQosPolicy,
        },
    },
};
use rtps_pim::{
    discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet},
    messages::types::Count,
    structure::types::{GuidPrefix, PROTOCOLVERSION, VENDOR_ID_S2E},
};

struct Foo;

impl DdsType for Foo {
    fn type_name() -> &'static str {
        "Foo"
    }

    fn has_key() -> bool {
        false
    }
}

struct Bar;

impl DdsType for Bar {
    fn type_name() -> &'static str {
        "Bar"
    }
}

#[test]
fn domain_participant_create_and_delete_topic() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    domain_participant.delete_topic::<Foo>(&topic).unwrap();
}

#[test]
fn not_allowed_to_delete_topic_from_other_participant() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let domain_participant2 = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    assert!(matches!(
        domain_participant2.delete_topic::<Foo>(&topic),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn domain_participant_lookup_topic_without_creating_any_topic() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    assert!(domain_participant
        .lookup_topicdescription::<Foo>("topic")
        .is_err());
}

#[test]
fn domain_participant_lookup_single_existing_topic() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    assert!(
        domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .unwrap()
            == topic
    );
}

#[test]
fn domain_participant_lookup_topic_with_wrong_type() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    domain_participant
        .create_topic::<Bar>("topic", None, None, 0)
        .unwrap();

    assert!(domain_participant
        .lookup_topicdescription::<Foo>("topic")
        .is_err());
}

#[test]
fn domain_participant_lookup_topic_with_wrong_name() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    domain_participant
        .create_topic::<Foo>("other_topic", None, None, 0)
        .unwrap();

    assert!(domain_participant
        .lookup_topicdescription::<Foo>("topic")
        .is_err());
}

#[test]
fn domain_participant_lookup_topic_with_two_topics_with_different_types() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let topic_foo = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic", None, None, 0)
        .unwrap();

    assert!(
        domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .unwrap()
            == topic_foo
    );

    assert!(
        domain_participant
            .lookup_topicdescription::<Bar>("topic")
            .unwrap()
            == topic_bar
    );
}

#[test]
fn domain_participant_lookup_topic_with_two_topics_with_different_names() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let topic1 = domain_participant
        .create_topic::<Foo>("topic1", None, None, 0)
        .unwrap();
    let topic2 = domain_participant
        .create_topic::<Foo>("topic2", None, None, 0)
        .unwrap();

    assert!(
        domain_participant
            .lookup_topicdescription::<Foo>("topic1")
            .unwrap()
            == topic1
    );

    assert!(
        domain_participant
            .lookup_topicdescription::<Foo>("topic2")
            .unwrap()
            == topic2
    );
}

#[test]
fn get_instance_handle() {
    let guid_prefix = GuidPrefix([1; 12]);
    let domain_participant = DomainParticipantImpl::new(
        guid_prefix,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    domain_participant.enable().unwrap();

    domain_participant.get_instance_handle().unwrap();
}

#[test]
fn domain_participant_get_discovered_participant_data() {
    let guid_prefix = GuidPrefix([1; 12]);
    let domain_id = DomainId::default();
    let domain_tag = "".to_string();
    let domain_participant = DomainParticipantImpl::new(
        guid_prefix,
        domain_id,
        domain_tag.clone(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    domain_participant.enable().unwrap();
    domain_participant.create_builtins().unwrap();

    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey { value: [2; 16] },
        user_data: UserDataQosPolicy { value: vec![] },
    };
    let discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data: dds_participant_data.clone(),
        participant_proxy: ParticipantProxy {
            domain_id: domain_id as u32,
            domain_tag: domain_tag,
            protocol_version: PROTOCOLVERSION,
            guid_prefix,
            vendor_id: VENDOR_ID_S2E,
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: vec![],
            metatraffic_multicast_locator_list: vec![],
            default_unicast_locator_list: vec![],
            default_multicast_locator_list: vec![],
            available_builtin_endpoints: BuiltinEndpointSet::default(),
            manual_liveliness_count: Count(0),
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        },
        lease_duration: rtps_pim::behavior::types::Duration {
            seconds: 30,
            fraction: 0,
        },
    };
    domain_participant.add_discovered_participant(&discovered_participant_data);

    let discovered_participants = domain_participant.get_discovered_participants().unwrap();
    assert_eq!(discovered_participants.len(), 1);
    assert_eq!(discovered_participants[0], [2; 16]);

    let discovered_participant_data = domain_participant
        .get_discovered_participant_data(discovered_participants[0])
        .unwrap();
    assert_eq!(discovered_participant_data, dds_participant_data);
}
