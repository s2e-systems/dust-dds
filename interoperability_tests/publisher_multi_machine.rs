use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    topic_definition::type_support::{DdsSerializeKey, DdsType},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, DdsType)]
struct HelloWorldType {
    #[key]
    id: u8,
    msg: String,
}

impl DdsSerializeKey for HelloWorldType {
    fn dds_serialize_key(
        &self,
        writer: impl std::io::Write,
    ) -> dust_dds::infrastructure::error::DdsResult<()> {
        #[derive(serde::Serialize)]
        struct HelloWorldTypeKeyHolder<'a> {
            id: &'a u8,
        }

        let key_holder = HelloWorldTypeKeyHolder { id: &self.id };

        dust_dds::topic_definition::type_support::serialize_key_cdr(&key_holder, writer)
    }
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic(
            "HelloWorld",
            "HelloWorldType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
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
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();
    let writer_cond = writer.get_statuscondition().unwrap();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();

    let number_of_subscribers_to_find = 2;
    for _ in 0..number_of_subscribers_to_find {
        wait_set.wait(Duration::new(60, 0)).unwrap();
        writer.get_publication_matched_status().unwrap();
    }

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world".to_string(),
    };
    writer.write(&hello_world, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();
}
