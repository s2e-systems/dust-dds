use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        entity::Entity,
        wait_set::{Condition, WaitSet},
    },
};

#[derive(serde::Serialize, serde::Deserialize)]
struct UserType(i32);

impl dust_dds::dds_type::DdsSerde for UserType {}
impl dust_dds::dds_type::DdsType for UserType {
    fn type_name() -> &'static str {
        "UserType"
    }
}

#[test]
fn discovery_of_reader_and_writer_in_same_participant() {
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(0, None, None, 0)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", None, None, 0)
        .unwrap();
    let publisher = dp.create_publisher(None, None, 0).unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, None, None, 0)
        .unwrap();
    let subscriber = dp.create_subscriber(None, None, 0).unwrap();
    let _data_reader = subscriber
        .create_datareader::<UserType>(&topic, None, None, 0)
        .unwrap();
    let mut cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(dust_dds::dcps_psm::SUBSCRIPTION_MATCHED_STATUS)
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set
        .wait(dust_dds::dcps_psm::Duration::new(5, 0))
        .unwrap();

    assert_eq!(data_writer.get_matched_subscriptions().unwrap().len(), 1);
}
