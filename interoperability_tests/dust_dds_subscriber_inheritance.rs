use self::interoperability::test::Cat;
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE, InstanceStateKind},
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
    },
    listener::NO_LISTENER,
    wait_set::{Condition, WaitSet},
};

// TODO: remove when dust_dds_gen adds support for inheritance
pub mod interoperability {
    pub mod test {
        use dust_dds::infrastructure::type_support::DdsType;

        #[derive(DdsType, Default, Debug, Clone, PartialEq, Eq)]
        #[dust_dds(name = "interoperability::test::Animal")]
        pub struct Animal {
            #[dust_dds(key)]
            pub id: u32,
            pub name: String,
            pub age: u8,
        }

        #[derive(DdsType, Default, Debug, Clone, PartialEq, Eq)]
        #[dust_dds(name = "interoperability::test::Cat")]
        pub struct Cat {
            #[dust_dds(key)]
            pub parent: Animal,
            pub lives: u8,
        }
    }
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .find_topic::<Cat>("Inheritance", Duration::new(120, 0))
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<Cat>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(reader_cond.clone()))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    wait_set.wait(Duration::new(30, 0)).unwrap();

    let mut samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.size(), 1);
    println!("read: {samples:?}");

    if samples[0].sample_info.instance_state == InstanceStateKind::Alive {
        let instance_handle = samples[0].sample_info.instance_handle;
        assert!(samples[0].data.is_some());

        wait_set.wait(Duration::new(30, 0)).unwrap();

        samples = reader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();
        assert_eq!(samples.size(), 1);
        println!("read: {samples:?}");
        assert_eq!(samples[0].sample_info.instance_handle, instance_handle);
    }

    assert!(samples[0].data.is_none());
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed,
    );

    // Sleep to allow sending acknowledgements
    std::thread::sleep(std::time::Duration::from_secs(5));
}
