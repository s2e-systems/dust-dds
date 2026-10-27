use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use dust_dds::{
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        configuration::DustDdsConfiguration,
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::{NO_STATUS, StatusKind},
        time::{Duration as DdsDuration, DurationKind},
        type_support::DdsType,
    },
    security::plugins::types::DdsSecurityPlugins,
    std_runtime::StdRuntime,
    transport::{
        interface::{RtpsParticipant, Transport, Write},
        types::{BUILT_IN_PARTICIPANT, EntityId, Guid, GuidPrefix, Locator},
    },
    wait_set::{Condition, WaitSet},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use iceoryx2::{
    port::{notifier::Notifier, publisher::Publisher, update_connections::UpdateConnections},
    prelude::*,
};

const LOCATOR_KIND_ICEORYX2: i32 = 0x40;
const MAX_DATAGRAM_SIZE: usize = 65000;

pub struct Iceoryx2Writer<'a> {
    publisher: &'a Publisher<ipc_threadsafe::Service, [u8], ()>,
    notifier: &'a Notifier<ipc_threadsafe::Service>,
    buffer: &'a mut Vec<u8>,
}

impl<'a> Write for Iceoryx2Writer<'a> {
    fn write(&mut self, buf: &[u8]) {
        self.buffer.extend_from_slice(buf);
    }

    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        if let Ok(mut sample) = self.publisher.loan_slice(self.buffer.len()) {
            sample.payload_mut().copy_from_slice(self.buffer);
            if sample.send().is_ok() {
                let _ = self.notifier.notify();
            }
        }

        self.buffer.clear();
    }
}

pub struct Iceoryx2Participant {
    guid: Guid,
    locator_list: Vec<Locator>,
    publisher: Publisher<ipc_threadsafe::Service, [u8], ()>,
    notifier: Notifier<ipc_threadsafe::Service>,
    write_buffer: Vec<u8>,
    receive_channel: Arc<Channel<CriticalSectionRawMutex, Vec<u8>, 1024>>,
    receive_buffer: Vec<u8>,
}

impl RtpsParticipant for Iceoryx2Participant {
    type Writer<'a> = Iceoryx2Writer<'a>;

    fn guid(&self) -> Guid {
        self.guid
    }

    fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.locator_list
    }

    fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &self.locator_list
    }

    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.locator_list
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }

    fn fragment_size(&self) -> usize {
        MAX_DATAGRAM_SIZE
    }

    fn writer<'a>(&'a mut self, _locator: &'a [Locator]) -> Self::Writer<'a> {
        self.write_buffer.clear();
        Iceoryx2Writer {
            publisher: &self.publisher,
            notifier: &self.notifier,
            buffer: &mut self.write_buffer,
        }
    }

    async fn read(&mut self) -> &[u8] {
        self.receive_buffer = self.receive_channel.receive().await;
        &self.receive_buffer[..]
    }
}

struct ReactorHandle {
    is_running: Arc<AtomicBool>,
    notifier: Notifier<ipc_threadsafe::Service>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

#[derive(Default)]
pub struct Iceoryx2Transport {
    reactors: HashMap<Guid, ReactorHandle>,
}

impl Transport for Iceoryx2Transport {
    type Participant = Iceoryx2Participant;

    fn create_participant(&mut self, domain_id: i32) -> Self::Participant {
        let node = NodeBuilder::new()
            .create::<ipc_threadsafe::Service>()
            .expect("Failed to create iceoryx2 node");

        let data_service_name = format!("dust_dds_domain_{domain_id}");
        let data_service = node
            .service_builder(&data_service_name.as_str().try_into().unwrap())
            .publish_subscribe::<[u8]>()
            .subscriber_max_buffer_size(1024)
            .enable_safe_overflow(true)
            .open_or_create()
            .expect("Failed to open or create data service");

        let publisher = data_service
            .publisher_builder()
            .initial_max_slice_len(MAX_DATAGRAM_SIZE)
            .max_loaned_samples(256)
            .create()
            .expect("Failed to create publisher");

        let subscriber = data_service
            .subscriber_builder()
            .buffer_size(1024)
            .create()
            .expect("Failed to create subscriber");

        let event_service_name = format!("dust_dds_domain_{domain_id}_events");
        let event_service = node
            .service_builder(&event_service_name.as_str().try_into().unwrap())
            .event()
            .open_or_create()
            .expect("Failed to open or create event service");

        let notifier = event_service
            .notifier_builder()
            .create()
            .expect("Failed to create notifier");

        let delete_notifier = event_service
            .notifier_builder()
            .create()
            .expect("Failed to create delete notifier");

        let listener = event_service
            .listener_builder()
            .create()
            .expect("Failed to create listener");

        static GLOBAL_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let pid_bytes = std::process::id().to_ne_bytes();
        let counter_bytes = GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed).to_be_bytes();

        let guid_prefix: GuidPrefix = [
            0x10,
            0x20,
            pid_bytes[0],
            pid_bytes[1],
            pid_bytes[2],
            pid_bytes[3],
            0,
            0,
            counter_bytes[0],
            counter_bytes[1],
            counter_bytes[2],
            counter_bytes[3],
        ];

        let guid = Guid::new(
            guid_prefix,
            EntityId::new([0, 0, 0x01], BUILT_IN_PARTICIPANT),
        );

        let locator = Locator::new(LOCATOR_KIND_ICEORYX2, domain_id as u32, [0; 16]);

        let is_running = Arc::new(AtomicBool::new(true));
        let receive_channel = Arc::new(Channel::new());

        let is_running_clone = is_running.clone();
        let channel_sender = receive_channel.clone();

        let thread_handle = std::thread::spawn(move || {
            while is_running_clone.load(Ordering::Relaxed) {
                let _ = listener.timed_wait(|_| {}, Duration::from_millis(10));
                if !is_running_clone.load(Ordering::Relaxed) {
                    break;
                }

                let _ = subscriber.update_connections();
                while let Ok(Some(sample)) = subscriber.receive() {
                    let data = sample.payload().to_vec();
                    let _ = channel_sender.try_send(data);
                }
            }
        });

        self.reactors.insert(
            guid,
            ReactorHandle {
                is_running,
                notifier: delete_notifier,
                thread_handle: Some(thread_handle),
            },
        );

        Iceoryx2Participant {
            guid,
            locator_list: vec![locator],
            publisher,
            notifier,
            write_buffer: Vec::with_capacity(MAX_DATAGRAM_SIZE),
            receive_channel,
            receive_buffer: Vec::new(),
        }
    }

    fn delete_participant(&mut self, handle: Guid) {
        if let Some(mut reactor) = self.reactors.remove(&handle) {
            reactor.is_running.store(false, Ordering::Relaxed);
            let _ = reactor.notifier.notify();
            if let Some(handle) = reactor.thread_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

#[derive(DdsType, Debug, PartialEq)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}

fn create_iceoryx2_participant_factory() -> DomainParticipantFactory {
    let factory_async = Box::leak(Box::new(DomainParticipantFactoryAsync::new(
        DustDdsConfiguration::default(),
        StdRuntime::default(),
        Iceoryx2Transport::default(),
        DdsSecurityPlugins::disabled(),
    )));
    DomainParticipantFactory::new(factory_async)
}

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let domain_id = 0;

    println!("Running publisher & subscriber communication over iceoryx2 shared memory...");

    let subscriber_thread = std::thread::spawn(move || {
        let factory = create_iceoryx2_participant_factory();
        let participant = factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        let topic = participant
            .create_topic::<HelloWorldType>(
                "HelloWorld",
                "HelloWorldType",
                QosKind::Default,
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
                max_blocking_time: DurationKind::Finite(DdsDuration::new(1, 0)),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            ..Default::default()
        };

        let reader = subscriber
            .create_datareader::<HelloWorldType>(
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
        wait_set.wait(DdsDuration::new(15, 0)).unwrap();

        reader_cond
            .set_enabled_statuses(&[StatusKind::DataAvailable])
            .unwrap();
        wait_set.wait(DdsDuration::new(15, 0)).unwrap();

        let sample = reader.read_next_sample().unwrap();
        println!("[Subscriber] Received: {:?}", sample.data.unwrap());

        participant.delete_contained_entities().unwrap();
        factory.delete_participant(&participant).unwrap();
    });

    let publisher_thread = std::thread::spawn(move || {
        let factory = create_iceoryx2_participant_factory();
        let participant = factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        let topic = participant
            .create_topic::<HelloWorldType>(
                "HelloWorld",
                "HelloWorldType",
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
                max_blocking_time: DurationKind::Finite(DdsDuration::new(1, 0)),
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
        wait_set.wait(DdsDuration::new(15, 0)).unwrap();

        let data = HelloWorldType {
            id: 42,
            msg: "Hello from iceoryx2 shared memory transport!".to_string(),
        };

        println!("[Publisher] Writing: {:?}", data);
        writer.write(data, None).unwrap();
        writer
            .wait_for_acknowledgments(DdsDuration::new(5, 0))
            .unwrap();

        std::thread::sleep(Duration::from_millis(500));

        participant.delete_contained_entities().unwrap();
        factory.delete_participant(&participant).unwrap();
    });

    subscriber_thread
        .join()
        .expect("Subscriber thread panicked");
    publisher_thread.join().expect("Publisher thread panicked");

    println!("Example finished successfully.");
}
