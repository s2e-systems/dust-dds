use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory, infrastructure::{
        qos::{DataReaderQos, QosKind}, qos_policy::{DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind}, status::{StatusKind, NO_STATUS}, time::{Duration, DurationKind}, type_support::DdsType
    }, listener::NO_LISTENER
};
use std::{
    sync::mpsc::{sync_channel, SyncSender,},
    thread,
};
    
use dust_dds::{
    runtime::DdsRuntime,
    dds_async::data_reader::DataReaderAsync,
    infrastructure::{
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
    subscription::data_reader_listener::DataReaderListener,
};

#[derive(DdsType, Debug, Clone, PartialEq)]
struct TestType {
    id: i32,
    message: String,
}

struct Listener {
    sender: SyncSender<()>,
}

impl<R: DdsRuntime> DataReaderListener<R, TestType> for Listener {
    async fn on_data_available(&mut self, the_reader: DataReaderAsync<R, TestType>) {
        println!("Reading sample(s)");
        if let Ok(samples) = the_reader
            .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .await
        {
            for (i, s) in samples.iter().enumerate() {
                if let Ok(sample) = s.data() {
                    println!("Read sample {}: {:?}", i, sample);
                    assert_eq!(sample.message, "Test", "Message mismatch!");
                } else if let Err(e) = s.data() {
                    println!("Error reading sample {}: {:?}", i, e);
                }
            }
        }
    }
    async fn on_subscription_matched(
        &mut self,
        _the_reader: DataReaderAsync<R, TestType>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        if status.current_count > 0 {
            self.sender.send(()).unwrap();
        }
    }
}

fn run_publisher(domain_id: i32, topic_name: &str) {
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<TestType>(
            topic_name,
            "TestType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    for i in 0..10 {
        let sample = TestType {
            id: 3 + i,
            message: String::from("Test"),
        };
        println!("Writing sample: {:?}", sample);
        let result = writer.write(&sample, None);
        assert!(result.is_ok());
        println!("Sample written successfully.");

        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    // Clean up
    publisher.delete_datawriter(&writer).unwrap();
}

fn run_subscriber(domain_id: i32, topic_name: &str, label: &str) {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        println!("[Subscriber-{label}] Creating participant...");
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        println!("[Subscriber-{label}] Creating topic...");
        let topic = participant
            .create_topic::<TestType>(
                topic_name,
                "TestType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();

        println!("[Subscriber-{label}] Creating subscriber...");
        let subscriber = participant
            .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        let (sender, receiver) = sync_channel(0);
        let listener = Listener { sender };

        println!("[Subscriber-{label}] Creating datareader with listener...");
        let reader = subscriber
            .create_datareader(
                &topic,
                QosKind::Default,
                Some(listener),
                &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
            )
            .unwrap();

        println!("[Subscriber-{label}] Waiting for subscription match...");
        let result = receiver.recv();
        assert!(result.is_ok(), "{:?}", result.err());
        println!("[Subscriber-{label}] Subscription matched! Waiting for data...");

        // Give more time for the async listener to process data
        std::thread::sleep(std::time::Duration::from_secs(5));

        println!("[Subscriber-{label}] Sample read successfully");

        // Clean up
        subscriber.delete_datareader(&reader).unwrap();
        println!("[Subscriber-{label}] Datareader deleted, exiting subscriber.");
    });
}

#[test]
fn test_publisher_subscriber_threads() {
    let domain_id = 1;
    let topic_name = "TestTopic";

    // Start publisher thread
    let publisher_thread = thread::spawn({
        let topic_name = topic_name.to_string();
        move || {
            run_publisher(domain_id, &topic_name);
        }
    });

    // Wait a bit to ensure publisher starts first
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Start subscriber thread (first connection)
    let topic_name1 = topic_name.to_string();
    let subscriber1 = thread::spawn(move || {
        println!("[Main] Starting subscriber (first connection)...");
        run_subscriber(domain_id, &topic_name1, "1");
        println!("[Main] Subscriber (first connection) finished (simulating disconnect)");
    });

    // Wait for the subscriber to process some data and "disconnect"
    std::thread::sleep(std::time::Duration::from_secs(7));

    // Simulate time before reconnect
    println!("[Main] Simulating subscriber disconnection...");
    subscriber1.join().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Start subscriber thread (reconnection)
    let topic_name2 = topic_name.to_string();
    let subscriber2 = thread::spawn(move || {
        println!("[Main] Starting subscriber (reconnection)...");
        run_subscriber(domain_id, &topic_name2, "2");
        println!("[Main] Subscriber (reconnection) finished");
    });

    publisher_thread.join().unwrap();
    subscriber2.join().unwrap();
}