use dust_dds::{
    dds_async::{
        domain_participant_factory::DomainParticipantFactoryAsync,
        wait_set::{ConditionAsync, WaitSetAsync},
    },
    infrastructure::{
        qos::QosKind,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{StatusKind, NO_STATUS},
        time::Duration,
        type_support::DdsType,
    },
    listener::NO_LISTENER,
};

#[derive(Debug, PartialEq, DdsType)]
struct UserData {
    #[dust_dds(key)]
    id: u8,
    value: Vec<u8>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let domain_id = 100;

    let participant_factory = DomainParticipantFactoryAsync::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    let topic = participant
        .create_topic::<UserData>(
            "LargeDataTopic",
            "UserData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();

    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    let reader = subscriber
        .create_datareader::<UserData>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .await
        .unwrap();

    let mut wait_set = WaitSetAsync::new();
    wait_set
        .attach_condition(ConditionAsync::StatusCondition(cond))
        .await
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).await.unwrap();

    let data = UserData {
        id: 1,
        value: vec![8; 100],
    };

    writer.write(&data, None).await.unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::DataAvailable])
        .await
        .unwrap();
    let mut reader_wait_set = WaitSetAsync::new();
    reader_wait_set
        .attach_condition(ConditionAsync::StatusCondition(cond))
        .await
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).await.unwrap();

    let samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .await
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
}
