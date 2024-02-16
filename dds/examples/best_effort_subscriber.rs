use std::{
    sync::mpsc::{sync_channel, SyncSender},
    time::Duration,
};

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listeners::NoOpListener,
        qos::QosKind,
        status::{StatusKind, NO_STATUS},
    },
    subscription::{
        data_reader::DataReader,
        data_reader_listener::DataReaderListener,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
    topic_definition::type_support::DdsType,
};

#[derive(DdsType, Debug)]
struct BestEffortExampleType {
    id: i32,
}

struct Listener {
    sender: SyncSender<()>,
}

impl DataReaderListener for Listener {
    type Foo = BestEffortExampleType;
    fn on_data_available(&mut self, the_reader: &DataReader<BestEffortExampleType>) {
        if let Ok(samples) =
            the_reader.take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        {
            let sample = samples[0].data().unwrap();
            println!("Read sample: {:?}", sample);
        }
    }
    fn on_subscription_matched(
        &mut self,
        _the_reader: &DataReader<BestEffortExampleType>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        if status.current_count == 0 {
            self.sender.send(()).unwrap();
        }
    }
}

fn main() {
    let domain_id = 1;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<BestEffortExampleType>(
            "BestEffortExampleTopic",
            "BestEffortExampleType",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let (sender, receiver) = sync_channel(0);
    let listener = Listener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Default,
            listener,
            &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
        )
        .unwrap();
    receiver.recv_timeout(Duration::from_secs(20)).ok();
}
