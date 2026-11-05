use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, PublisherQos, QosKind, SubscriberQos},
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, PresentationQosPolicy,
            PresentationQosPolicyAccessScopeKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
    },
    wait_set::{Condition, WaitSet},
};
use dust_dds_derive::DdsType;

mod domain_id_generator;
use domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Clone, Debug, PartialEq, DdsType)]
struct KeyedData {
    #[dust_dds(key)]
    id: u8,
    value: String,
}

#[derive(Clone, Debug, PartialEq, DdsType)]
struct ShapeType {
    #[dust_dds(key)]
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

#[derive(Clone, Debug, PartialEq, DdsType)]
struct CoherentData {
    value: i32,
}

#[test]
fn samples_should_be_content_filtered() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "MyTopicFiltered",
            &topic,
            String::from("value = %0"),
            vec![String::from("RED")],
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData {
        id: 1,
        value: String::from("BLUE"),
    };
    let data2 = KeyedData {
        id: 2,
        value: String::from("RED"),
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_shapesize() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "ShapeTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "ShapeTopicFiltered",
            &topic,
            String::from("shapesize <= 20"),
            vec![],
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = ShapeType {
        color: String::from("BLUE"),
        x: 100,
        y: 100,
        shapesize: 30,
    };
    let data2 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 20,
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_parameterized_shapesize() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "ShapeTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "ShapeTopicFiltered",
            &topic,
            String::from("shapesize <= %0"),
            vec![String::from("20")],
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = ShapeType {
        color: String::from("BLUE"),
        x: 100,
        y: 100,
        shapesize: 30,
    };
    let data2 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 20,
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_literal_string() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "MyTopicFiltered",
            &topic,
            String::from("value = 'RED'"),
            vec![],
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData {
        id: 1,
        value: String::from("BLUE"),
    };
    let data2 = KeyedData {
        id: 2,
        value: String::from("RED"),
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_compound_expression() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "ShapeTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "ShapeTopicFiltered",
            &topic,
            String::from("(color = 'RED') AND (shapesize <= 20)"),
            vec![],
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
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 15,
    };
    let data2 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 25,
    };
    let data3 = ShapeType {
        color: String::from("BLUE"),
        x: 100,
        y: 100,
        shapesize: 15,
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
}

// Test_CoherentSets_10: Test the behavior of coherent access with INSTANCE_PRESENTATION with several instances
#[test]
fn coherent_sets_with_multiple_instances_and_topics() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic_names = ["Square", "Square1", "Square2"];
    let mut topics = Vec::new();
    for topic_name in topic_names {
        let topic = participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        topics.push(topic);
    }

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let mut writers = Vec::new();
    for topic in &topics {
        let writer = publisher
            .create_datawriter(
                topic,
                QosKind::Specific(writer_qos.clone()),
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        writers.push(writer);
    }

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let mut readers = Vec::new();
    for topic in &topics {
        let reader = subscriber
            .create_datareader::<ShapeType>(
                topic,
                QosKind::Specific(reader_qos.clone()),
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        readers.push(reader);
    }

    for writer in &writers {
        let cond = writer.get_statuscondition();
        cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
            .unwrap();

        let mut wait_set = WaitSet::new();
        wait_set
            .attach_condition(Condition::StatusCondition(cond))
            .unwrap();
        wait_set.wait(Duration::new(10, 0)).unwrap();
    }

    let instance_colors = ["BLUE", "BLUE1", "BLUE2", "BLUE3"];
    let coherent_sample_count = 3;

    publisher.begin_coherent_changes().unwrap();

    for n in 1..=coherent_sample_count {
        for writer in &writers {
            for &color in &instance_colors {
                let data = ShapeType {
                    color: color.to_string(),
                    x: 100,
                    y: 100,
                    shapesize: n,
                };
                writer.write(data, None).unwrap();
            }
        }
    }

    for writer in &writers {
        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();
    }

    for reader in &readers {
        assert!(
            reader
                .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .is_err()
        );
    }

    publisher.end_coherent_changes().unwrap();

    for writer in &writers {
        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();
    }

    for reader in &readers {
        let samples = reader
            .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(
            samples.len(),
            instance_colors.len() * coherent_sample_count as usize
        );

        for &color in &instance_colors {
            let instance_samples: Vec<_> = samples
                .iter()
                .filter(|s| s.data.as_ref().unwrap().color == color)
                .collect();
            assert_eq!(instance_samples.len(), coherent_sample_count as usize);
            let sizes: Vec<i32> = instance_samples
                .iter()
                .map(|s| s.data.as_ref().unwrap().shapesize)
                .collect();
            assert_eq!(sizes, vec![1, 2, 3]);
        }
    }
}

#[test]
fn coherent_sets_topic_presentation() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "CoherentTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
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

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    publisher.begin_coherent_changes().unwrap();

    for i in 1..=3 {
        let data = ShapeType {
            color: "RED".to_string(),
            x: 10,
            y: 20,
            shapesize: i,
        };
        writer.write(data, None).unwrap();
    }

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert!(
        reader
            .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .is_err()
    );

    publisher.end_coherent_changes().unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 3);
}

#[test]
fn coherent_changes_topic_access_scope() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<CoherentData>(
            "CoherentTopic",
            "CoherentData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
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

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<CoherentData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    // 1. Begin coherent set and write 2 samples
    publisher.begin_coherent_changes().unwrap();
    writer.write(CoherentData { value: 10 }, None).unwrap();
    writer.write(CoherentData { value: 20 }, None).unwrap();

    // Reader should not see data before coherent set ends
    let data_cond = reader.get_statuscondition();
    data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut data_wait_set = WaitSet::new();
    data_wait_set
        .attach_condition(Condition::StatusCondition(data_cond.clone()))
        .unwrap();
    assert!(data_wait_set.wait(Duration::new(0, 200_000_000)).is_err());

    // 2. End coherent changes -> triggers DataAvailable
    publisher.end_coherent_changes().unwrap();
    data_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 10);
    assert_eq!(samples[1].data.as_ref().unwrap().value, 20);

    // 3. Subsequent sample outside coherent set is received immediately
    writer.write(CoherentData { value: 30 }, None).unwrap();
    data_wait_set.wait(Duration::new(5, 0)).unwrap();
    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 30);
}

#[test]
fn coherent_changes_instance_access_scope() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<CoherentData>(
            "CoherentInstanceTopic",
            "CoherentData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
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

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<CoherentData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    publisher.begin_coherent_changes().unwrap();
    writer.write(CoherentData { value: 100 }, None).unwrap();
    writer.write(CoherentData { value: 200 }, None).unwrap();

    let data_cond = reader.get_statuscondition();
    data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut data_wait_set = WaitSet::new();
    data_wait_set
        .attach_condition(Condition::StatusCondition(data_cond))
        .unwrap();
    assert!(data_wait_set.wait(Duration::new(0, 200_000_000)).is_err());

    publisher.end_coherent_changes().unwrap();
    data_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 100);
    assert_eq!(samples[1].data.as_ref().unwrap().value, 200);
}

#[test]
fn multiple_writers_coherent_and_non_coherent() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<CoherentData>(
            "CoherentMultiWriterTopic",
            "CoherentData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let pub_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };

    // Publisher 1: coherent access enabled
    let pub1 = participant
        .create_publisher(QosKind::Specific(pub_qos.clone()), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1 = pub1
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Publisher 2: also coherent access enabled
    let pub2 = participant
        .create_publisher(QosKind::Specific(pub_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer2 = pub2
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Subscriber: coherent access
    let sub_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(sub_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<CoherentData>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Wait for matching
    let cond1 = writer1.get_statuscondition();
    cond1
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut ws1 = WaitSet::new();
    ws1.attach_condition(Condition::StatusCondition(cond1))
        .unwrap();
    ws1.wait(Duration::new(10, 0)).unwrap();

    let cond2 = writer2.get_statuscondition();
    cond2
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut ws2 = WaitSet::new();
    ws2.attach_condition(Condition::StatusCondition(cond2))
        .unwrap();
    ws2.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut rws = WaitSet::new();
    rws.attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    while reader
        .get_subscription_matched_status()
        .unwrap()
        .current_count
        < 2
    {
        rws.wait(Duration::new(10, 0)).unwrap();
    }

    let data_cond = reader.get_statuscondition();
    data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut dws = WaitSet::new();
    dws.attach_condition(Condition::StatusCondition(data_cond))
        .unwrap();

    // Writer 1 begins coherent set and writes
    pub1.begin_coherent_changes().unwrap();
    writer1.write(CoherentData { value: 1 }, None).unwrap();

    // Writer 2 writes a non-coherent sample
    writer2.write(CoherentData { value: 99 }, None).unwrap();

    // Reader should receive Writer 2's sample even though Writer 1 is in an open coherent set
    dws.wait(Duration::new(5, 0)).unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 99);

    // Writer 1 ends coherent changes
    pub1.end_coherent_changes().unwrap();
    dws.wait(Duration::new(5, 0)).unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 1);
}
