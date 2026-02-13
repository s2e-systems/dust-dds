use self::interoperability::test::{Animal, Cat};
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
        type_support::TypeSupport,
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
        .create_topic::<Cat>(
            "Inheritance",
            Cat::get_type_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    let data = Cat {
        parent: Animal {
            id: 1,
            name: "Zoe".to_string(),
            age: 1,
        },
        lives: 7,
    };
    println!("write: {data:?}");
    writer.write(data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();

    let data_to_dispose = Cat {
        parent: Animal {
            id: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    println!("dispose: {data_to_dispose:?}");
    writer.dispose(data_to_dispose, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();
}
