use std::{
    collections::HashMap,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};

use dust_dds::{
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::configuration::DustDdsConfiguration,
    security::plugins::types::DdsSecurityPlugins,
    std_runtime::StdRuntime,
    transport::{
        interface::{RtpsParticipant, Transport, Write},
        types::{BUILT_IN_PARTICIPANT, EntityId, Guid, GuidPrefix, Locator},
    },
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use iceoryx2::{
    port::{notifier::Notifier, publisher::Publisher, update_connections::UpdateConnections},
    prelude::*,
};

const LOCATOR_KIND_ICEORYX2: i32 = 0x40;
const MAX_DATAGRAM_SIZE: usize = 65536;

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
    fragment_size: usize,
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
        self.fragment_size
    }

    fn writer<'a>(&'a mut self, _locator: &'a [Locator]) -> Self::Writer<'a> {
        self.write_buffer.clear();
        Iceoryx2Writer {
            publisher: &self.publisher,
            notifier: &self.notifier,
            buffer: &mut self.write_buffer,
        }
    }

    fn read(&mut self) -> impl Future<Output = &[u8]> + Send {
        async {
            self.receive_buffer = self.receive_channel.receive().await;
            &self.receive_buffer[..]
        }
    }
}

struct ReactorHandle {
    is_running: Arc<AtomicBool>,
    notifier: Notifier<ipc_threadsafe::Service>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

pub struct Iceoryx2Transport {
    reactors: HashMap<Guid, ReactorHandle>,
    fragment_size: usize,
}

impl Default for Iceoryx2Transport {
    fn default() -> Self {
        Self {
            reactors: HashMap::new(),
            fragment_size: MAX_DATAGRAM_SIZE,
        }
    }
}

impl Iceoryx2Transport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_fragment_size(&mut self, fragment_size: usize) {
        self.fragment_size = fragment_size;
    }
}

impl Transport for Iceoryx2Transport {
    type Participant = Iceoryx2Participant;

    fn create_participant(&mut self, domain_id: i32) -> Self::Participant {
        let node = NodeBuilder::new()
            .create::<ipc_threadsafe::Service>()
            .expect("Failed to create iceoryx2 node");

        let data_service_name = format!("dust_dds_bench_domain_{domain_id}");
        let data_service = node
            .service_builder(&data_service_name.as_str().try_into().unwrap())
            .publish_subscribe::<[u8]>()
            .subscriber_max_buffer_size(1024)
            .subscriber_max_borrowed_samples(256)
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

        let event_service_name = format!("dust_dds_bench_domain_{domain_id}_events");
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

        static GLOBAL_COUNTER: AtomicU32 = AtomicU32::new(1);
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
                let _ = subscriber.update_connections();
                let mut received_any = false;
                while let Ok(Some(sample)) = subscriber.receive() {
                    received_any = true;
                    let data = sample.payload().to_vec();
                    let _ = channel_sender.try_send(data);
                }

                if !received_any {
                    let _ = listener.timed_wait(|_| {}, Duration::from_millis(10));
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
            fragment_size: self.fragment_size,
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

pub fn create_iceoryx2_participant_factory() -> DomainParticipantFactory {
    create_iceoryx2_participant_factory_with_transport(Iceoryx2Transport::default())
}

pub fn create_iceoryx2_participant_factory_with_transport(
    transport: Iceoryx2Transport,
) -> DomainParticipantFactory {
    let factory_async = Box::leak(Box::new(DomainParticipantFactoryAsync::new(
        DustDdsConfiguration::default(),
        StdRuntime::default(),
        transport,
        DdsSecurityPlugins::disabled(),
    )));
    DomainParticipantFactory::new(factory_async)
}
