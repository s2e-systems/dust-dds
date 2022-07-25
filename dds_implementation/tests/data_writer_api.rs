use dds_api::{
    dcps_psm::{DomainId, Time, TIME_INVALID},
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::Entity,
        qos::{DataWriterQos, DomainParticipantQos},
        qos_policy::ResourceLimitsQosPolicy,
    },
    publication::{data_writer::FooDataWriter, publisher::Publisher},
    return_type::{DdsError, DdsResult},
};
use dds_implementation::{
    dds_impl::domain_participant_impl::DomainParticipantImpl,
    dds_type::{DdsSerialize, DdsType, Endianness},
};
use rtps_pim::structure::types::GuidPrefix;

struct Foo;

impl DdsType for Foo {
    fn type_name() -> &'static str {
        "Foo"
    }
}

impl DdsSerialize for Foo {
    fn serialize<W: std::io::Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
        Ok(())
    }
}

struct KeyedFoo {
    key: Vec<u8>,
}

impl DdsType for KeyedFoo {
    fn type_name() -> &'static str {
        "KeyedFoo"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        self.key.clone()
    }

    fn set_key_fields_from_serialized_key(&mut self, key: &[u8]) -> DdsResult<()> {
        self.key = key.to_vec();
        Ok(())
    }
}

impl DdsSerialize for KeyedFoo {
    fn serialize<W: std::io::Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
        Ok(())
    }
}

#[test]
fn register_instance_w_timestamp_different_keys() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();

    let instance_handle = data_writer
        .register_instance_w_timestamp(&KeyedFoo { key: vec![1, 2] }, Time { sec: 0, nanosec: 0 })
        .unwrap();
    assert_eq!(
        instance_handle,
        Some([1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    );

    let instance_handle = data_writer
        .register_instance_w_timestamp(
            &KeyedFoo {
                key: vec![1, 2, 3, 4, 5, 6],
            },
            Time { sec: 0, nanosec: 0 },
        )
        .unwrap();
    assert_eq!(
        instance_handle,
        Some([1, 2, 3, 4, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    );

    let instance_handle = data_writer
        .register_instance_w_timestamp(
            &KeyedFoo {
                key: vec![b'1'; 20],
            },
            Time { sec: 0, nanosec: 0 },
        )
        .unwrap();
    assert_eq!(
        instance_handle,
        Some([
            0x50, 0x20, 0x7f, 0xa2, 0x81, 0x4e, 0x81, 0xa0, 0x67, 0xbd, 0x26, 0x62, 0xba, 0x10,
            0xb0, 0xf1
        ])
    );
}

#[test]
fn register_instance_w_timestamp_no_key() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();

    let instance_handle = data_writer
        .register_instance_w_timestamp(&Foo {}, TIME_INVALID)
        .unwrap();
    assert_eq!(instance_handle, None);
}

#[test]
fn register_instance_w_timestamp_out_of_resources() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(
            &topic,
            Some(DataWriterQos {
                resource_limits: ResourceLimitsQosPolicy {
                    max_instances: 2,
                    ..Default::default()
                },
                ..Default::default()
            }),
            None,
            0,
        )
        .unwrap();

    data_writer
        .register_instance_w_timestamp(&KeyedFoo { key: vec![1] }, TIME_INVALID)
        .unwrap();
    data_writer
        .register_instance_w_timestamp(&KeyedFoo { key: vec![2] }, TIME_INVALID)
        .unwrap();
    let instance_handle_result =
        data_writer.register_instance_w_timestamp(&KeyedFoo { key: vec![3] }, TIME_INVALID);
    assert_eq!(instance_handle_result, Err(DdsError::OutOfResources));

    // Already registered sample does not cause OutOfResources error
    data_writer
        .register_instance_w_timestamp(&KeyedFoo { key: vec![2] }, TIME_INVALID)
        .unwrap();
}

#[test]
fn lookup_instance() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();

    let instance1 = KeyedFoo { key: vec![1] };
    let instance2 = KeyedFoo { key: vec![2] };

    let instance_handle1 = data_writer
        .register_instance_w_timestamp(&instance1, TIME_INVALID)
        .unwrap();

    assert_eq!(
        data_writer.lookup_instance(&instance1),
        Ok(instance_handle1)
    );
    assert_eq!(data_writer.lookup_instance(&instance2), Ok(None));
}

#[test]
fn unregister_registered_instance() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance = KeyedFoo { key: vec![1] };
    data_writer
        .register_instance_w_timestamp(&instance, TIME_INVALID)
        .unwrap();
    data_writer
        .unregister_instance_w_timestamp(&instance, None, TIME_INVALID)
        .unwrap();
    assert!(data_writer.lookup_instance(&instance).unwrap().is_none());
}

#[test]
fn unregister_instance_not_registered() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance = KeyedFoo { key: vec![1] };
    let result = data_writer.unregister_instance_w_timestamp(&instance, None, TIME_INVALID);
    assert_eq!(
        result,
        Err(DdsError::PreconditionNotMet(
            "Instance not registered with this DataWriter".to_string()
        ))
    );
}

#[test]
fn unregister_instance_non_registered_handle() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance = KeyedFoo { key: vec![1] };
    data_writer
        .register_instance_w_timestamp(&instance, TIME_INVALID)
        .unwrap();
    let result = data_writer.unregister_instance_w_timestamp(
        &instance,
        Some([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        TIME_INVALID,
    );
    assert_eq!(result, Err(DdsError::BadParameter));
}

#[test]
fn unregister_instance_not_matching_handle() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance1 = KeyedFoo { key: vec![1] };
    let instance2 = KeyedFoo { key: vec![2] };
    data_writer
        .register_instance_w_timestamp(&instance1, TIME_INVALID)
        .unwrap();
    data_writer
        .register_instance_w_timestamp(&instance2, TIME_INVALID)
        .unwrap();
    let result = data_writer.unregister_instance_w_timestamp(
        &instance1,
        Some([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        TIME_INVALID,
    );
    assert_eq!(
        result,
        Err(DdsError::PreconditionNotMet(
            "Handle does not match instance".to_string()
        ))
    );
}

#[test]
fn dispose_not_registered() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance = KeyedFoo { key: vec![1] };
    let result = data_writer.dispose_w_timestamp(&instance, None, TIME_INVALID);
    assert_eq!(
        result,
        Err(DdsError::PreconditionNotMet(
            "Instance not registered with this DataWriter".to_string()
        ))
    );
}

#[test]
fn dispose_non_registered_handle() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance = KeyedFoo { key: vec![1] };
    data_writer
        .register_instance_w_timestamp(&instance, TIME_INVALID)
        .unwrap();
    let result = data_writer.dispose_w_timestamp(
        &instance,
        Some([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        TIME_INVALID,
    );
    assert_eq!(result, Err(DdsError::BadParameter));
}

#[test]
fn dispose_not_matching_handle() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();
    let instance1 = KeyedFoo { key: vec![1] };
    let instance2 = KeyedFoo { key: vec![2] };
    data_writer
        .register_instance_w_timestamp(&instance1, TIME_INVALID)
        .unwrap();
    data_writer
        .register_instance_w_timestamp(&instance2, TIME_INVALID)
        .unwrap();
    let result = data_writer.dispose_w_timestamp(
        &instance1,
        Some([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        TIME_INVALID,
    );
    assert_eq!(
        result,
        Err(DdsError::PreconditionNotMet(
            "Handle does not match instance".to_string()
        ))
    );
}

#[test]
fn get_key_value_known_instance() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();

    let instance_handle = data_writer
        .register_instance_w_timestamp(&KeyedFoo { key: vec![1, 2] }, Time { sec: 0, nanosec: 0 })
        .unwrap()
        .unwrap();

    let mut keyed_foo = KeyedFoo { key: vec![] };
    data_writer
        .get_key_value(&mut keyed_foo, instance_handle)
        .unwrap();
    assert_eq!(keyed_foo.key, vec![1, 2]);
}

#[test]
fn get_key_value_unknown_instance() {
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
    domain_participant.enable().unwrap();

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<KeyedFoo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedFoo>(&topic, None, None, 0)
        .unwrap();

    data_writer
        .register_instance_w_timestamp(&KeyedFoo { key: vec![1, 2] }, Time { sec: 0, nanosec: 0 })
        .unwrap()
        .unwrap();

    let mut keyed_foo = KeyedFoo { key: vec![] };
    assert_eq!(
        data_writer.get_key_value(&mut keyed_foo, [1; 16]),
        Err(DdsError::BadParameter)
    );
}
