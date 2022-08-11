use dds::{
    dcps_psm::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    dds_type::{DdsSerde, DdsType, Endianness},
    domain::domain_participant_factory::{DdsDomainParticipantFactory, DomainParticipantFactory},
    return_type::DdsResult,
};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct KeyedData {
    id: u8,
    value: u8,
}

impl DdsType for KeyedData {
    fn type_name() -> &'static str {
        "KeyedData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        vec![self.id]
    }

    fn set_key_fields_from_serialized_key(&mut self, key: &[u8]) -> DdsResult<()> {
        self.id = key[0];
        Ok(())
    }
}

impl DdsSerde for KeyedData {}

#[test]
fn write_read_keyed_topic() {
    let domain_id = 20;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let topic = participant1
        .create_topic::<KeyedData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let reader = subscriber.create_datareader(&topic, None, None, 0).unwrap();

    //Wait for reader to be aware of the user writer
    while reader
        .get_subscription_matched_status()
        .unwrap()
        .total_count
        < 1
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };
    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(
        samples[0].sample_info.instance_handle,
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    );

    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
    assert_eq!(
        samples[1].sample_info.instance_handle,
        [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    );
    
    assert_eq!(samples[2].data.as_ref().unwrap(), &data3);
    assert_eq!(
        samples[2].sample_info.instance_handle,
        [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    );
}
