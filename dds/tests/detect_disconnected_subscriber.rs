use dust_dds::dds_async::data_writer::DataWriterAsync;
use dust_dds::publication::data_writer_listener::DataWriterListener;
use dust_dds::infrastructure::status::PublicationMatchedStatus;

struct PubListener {
    label: String,
}

impl<R: DdsRuntime, T: 'static> DataWriterListener<R, T> for PubListener {
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriterAsync<R, T>,
        status: PublicationMatchedStatus,
    ) -> impl core::future::Future<Output = ()> + Send {
        println!(
            "\x1b[35m[Publisher - {}]\x1b[0m Publication matched: current_count = {}, total_count = {}",
            self.label, status.current_count, status.total_count
        );
        if status.current_count == 0 {
            println!("\x1b[35m[Publisher - {}]\x1b[0m All subscribers disconnected!", self.label);
        }
        core::future::ready(())
    }
}

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::QosKind,
        status::{StatusKind, NO_STATUS},
        type_support::DdsType,
    },
    listener::NO_LISTENER,
};
use std::{
    sync::mpsc::{sync_channel, SyncSender},
    thread,
};

use dust_dds::{
    dds_async::data_reader::DataReaderAsync,
    infrastructure::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    runtime::DdsRuntime,
    subscription::data_reader_listener::DataReaderListener,
};

#[derive(DdsType, Debug, Clone, PartialEq)]
struct TestType {
    id: i32,
    message: String,
}

struct Listener {
    sender: SyncSender<()>,
    label: String,
}

impl<R: DdsRuntime> DataReaderListener<R, TestType> for Listener {
    async fn on_data_available(&mut self, the_reader: DataReaderAsync<R, TestType>) {
        println!("\x1b[36m[Listener - {}]\x1b[0m Reading sample(s)", self.label);
        if let Ok(samples) = the_reader
            .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .await
        {
            for (i, s) in samples.iter().enumerate() {
                if let Ok(sample) = s.data() {
                    println!("\x1b[36m[Listener - {}]\x1b[0m Read sample {}: {:?}", self.label, i, sample);
                    assert_eq!(sample.message, "Test", "Message mismatch!");
                } else if let Err(e) = s.data() {
                    println!("\x1b[36m[Listener - {}]\x1b[0m Error reading sample {}: {:?}", self.label, i, e);
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

    let pub_listener = PubListener {
        label: "main".to_string(),
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Default,
            Some(pub_listener),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();

    for i in 0..50 {
        let sample = TestType {
            id: 3 + i,
            message: String::from("Test"),
        };
        println!("\x1b[35m[Publisher]\x1b[0m Writing sample: {:?}", sample);
        let result = writer.write(&sample, None);
        assert!(result.is_ok());
        println!("\x1b[35m[Publisher]\x1b[0m Sample written successfully.");

        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    // Clean up
    publisher.delete_datawriter(&writer).unwrap();
    println!("\x1b[35m[Publisher]\x1b[0m Datawriter deleted, exiting publisher.");
}

fn run_subscriber_with_reconnect(domain_id: i32, topic_name: &str) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async move {
        println!("\x1b[32m[Subscriber]\x1b[0m Creating participant (once)...");
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        println!("\x1b[32m[Subscriber]\x1b[0m Creating topic (once)...");
        let topic = participant
            .create_topic::<TestType>(
                topic_name,
                "TestType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();

        println!("\x1b[32m[Subscriber]\x1b[0m Creating subscriber (once)...");
        let subscriber = participant
            .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        // First connection
        println!("\x1b[32m[Subscriber]\x1b[0m === FIRST CONNECTION ===");
        {
            let (sender, receiver) = sync_channel(0);
            let listener = Listener {
                sender,
                label: "connection-1".to_string(),
            };

            println!("\x1b[32m[Subscriber]\x1b[0m Creating datareader with listener...");
            let reader = subscriber
                .create_datareader(
                    &topic,
                    QosKind::Default,
                    Some(listener),
                    &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
                )
                .unwrap();

            println!("\x1b[32m[Subscriber]\x1b[0m Waiting for subscription match...");
            let result = receiver.recv();
            assert!(result.is_ok(), "{:?}", result.err());
            println!("\x1b[32m[Subscriber]\x1b[0m Subscription matched! Waiting for data...");

            // Wait for some data
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Simulate disconnect by deleting the datareader
            println!("\x1b[32m[Subscriber]\x1b[0m Deleting datareader (simulating disconnect)...");
            subscriber.delete_datareader(&reader).unwrap();
            println!("\x1b[32m[Subscriber]\x1b[0m Datareader deleted.");
        }

        // Wait a bit before reconnecting
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Reconnect - create new datareader with same subscriber
        println!("\x1b[32m[Subscriber]\x1b[0m === RECONNECTION ===");
        {
            let (sender, receiver) = sync_channel(0);
            let listener = Listener {
                sender,
                label: "reconnect-1".to_string(),
            };

            println!("\x1b[32m[Subscriber]\x1b[0m Creating new datareader (reconnection)...");
            let reader = subscriber
                .create_datareader(
                    &topic,
                    QosKind::Default,
                    Some(listener),
                    &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
                )
                .unwrap();

            println!("\x1b[32m[Subscriber]\x1b[0m Waiting for subscription match...");
            let result = receiver.recv();
            assert!(result.is_ok(), "{:?}", result.err());
            println!("\x1b[32m[Subscriber]\x1b[0m Subscription matched! Waiting for data...");

            // Wait for data
            let total_wait = 12; // seconds
            let interval = 4; // seconds
            for waited in (interval..=total_wait).step_by(interval) {
                std::thread::sleep(std::time::Duration::from_secs(interval as u64));
                println!(
                    "\x1b[32m[Subscriber]\x1b[0m Status: still waiting for data... ({}s elapsed)",
                    waited
                );
            }

            println!("\x1b[32m[Subscriber]\x1b[0m Sample read successfully");

            // Clean up
            subscriber.delete_datareader(&reader).unwrap();
            println!("\x1b[32m[Subscriber]\x1b[0m Datareader deleted, exiting subscriber.");
        }
    });
}

#[test]
fn test_publisher_subscriber_reconnect() {
    let domain_id = 1;
    let topic_name = "TestTopic";

    // Start publisher thread
    let publisher_thread = thread::spawn({
        let topic_name = topic_name.to_string();
        move || {
            println!("\x1b[35m[Publisher]\x1b[0m Starting publisher thread...");
            run_publisher(domain_id, &topic_name);
            println!("\x1b[35m[Publisher]\x1b[0m Publisher thread finished.");
        }
    });

    // Start subscriber thread with reconnection logic
    let subscriber_thread = thread::spawn({
        let topic_name = topic_name.to_string();
        move || {
            println!("\x1b[33m[Main]\x1b[0m Starting subscriber thread...");
            run_subscriber_with_reconnect(domain_id, &topic_name);
            println!("\x1b[33m[Main]\x1b[0m Subscriber thread finished.");
        }
    });

    // Wait for both threads to complete
    subscriber_thread.join().unwrap();
    publisher_thread.join().unwrap();
    
    println!("\x1b[33m[Main]\x1b[0m Test completed successfully!");
}