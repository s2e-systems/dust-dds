use crate::micro_runtime::MicroRuntime;
use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use chrono::NaiveDate;
use core::{future::Future, net::Ipv4Addr};
use defmt::{info, unwrap};
use dust_dds::{
    dds_async::{
        configuration::DustDdsConfiguration, data_writer::DataWriterAsync,
        data_writer_listener::DataWriterListener,
        domain_participant_factory::DomainParticipantFactoryAsync,
    },
    infrastructure::{
        self,
        listener::NO_LISTENER,
        qos::{DataReaderQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
    },
    transport::{
        interface::{TransportDataReceiver, WriteMessage},
        types::Locator,
    },
};
use embassy_executor::Spawner;
use embassy_net::{
    IpAddress, IpEndpoint, Ipv4Address, Stack, StackResources,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_stm32::{
    Config, bind_interrupts,
    eth::{self, Ethernet, GenericPhy, PacketQueue, Sma},
    gpio::{Input, Level, Output, Pull, Speed},
    peripherals::{self, ETH, ETH_SMA},
    rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, LsConfig, Pll, PllDiv, PllMul, PllPreDiv,
        PllSource, Sysclk, VoltageScale,
    },
    rng::{self, Rng},
    rtc::{Rtc, RtcConfig},
    time::Hertz,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use micro_dust_dds_interface::{ButtonState, ColorState};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

extern crate alloc;
extern crate core;

#[global_allocator]
static HEAP: embedded_alloc::LlffHeap = embedded_alloc::LlffHeap::empty();

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>;

type DomainId = i32;
const PB: i32 = 7400;
const DG: i32 = 250;
const D0: i32 = 0;
fn port_builtin_multicast(domain_id: DomainId) -> u16 {
    (PB + DG * domain_id + D0) as u16
}

type DataReceiverChannel =
    embassy_sync::watch::Watch<CriticalSectionRawMutex, TransportDataReceiver, 1>;
static DDS_DATA_RECEIVER_CHANNEL: DataReceiverChannel = DataReceiverChannel::new();

type SubscriptionSender = embassy_sync::channel::Sender<'static, CriticalSectionRawMutex, i32, 1>;
type SubscriptionChannel = embassy_sync::channel::Channel<CriticalSectionRawMutex, i32, 1>;
static SUBSCRIPTION_CHANNEL: SubscriptionChannel = SubscriptionChannel::new();

const WRITER_CHANNEL_SIZE: usize = 256;
type WriterChannelType = (Vec<u8>, Vec<Locator>);
type WriterChannel =
    embassy_sync::channel::Channel<CriticalSectionRawMutex, WriterChannelType, WRITER_CHANNEL_SIZE>;
static WRITER_CHANNEL: WriterChannel = WriterChannel::new();

const RX_BUFFER: usize = 2048;
const TX_BUFFER: usize = 2048;

const SEND_PORT: u16 = 3511;
const METATRAFFIC_UNICAST_PORT: u32 = 3512;
const DEFAULT_UNICAST_PORT: u32 = 3513;

#[embassy_executor::task]
async fn writer_task(stack: Stack<'static>) -> ! {
    let addr = if let Some(config) = stack.config_v4() {
        config.address.address()
    } else {
        Ipv4Address::UNSPECIFIED
    };

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; RX_BUFFER];
    let mut tx_buffer = [0; TX_BUFFER];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket
        .bind(IpEndpoint::new(IpAddress::Ipv4(addr), SEND_PORT))
        .unwrap();
    let own_address = stack.config_v4().unwrap().address;
    loop {
        let (datagram, locator_list) = WRITER_CHANNEL.receiver().receive().await;
        for destination_locator in locator_list {
            let udp_locator = UdpLocator(destination_locator);
            let addr = Ipv4Addr::from(&udp_locator);
            if own_address.contains_addr(&addr.into()) || addr.is_multicast() {
                info!(
                    "Send data to: {:?}, port: {}",
                    destination_locator.address(),
                    destination_locator.port()
                );
                socket.send_to(datagram.as_ref(), &udp_locator).await.ok();
            }
        }
    }
}

pub struct EmbeddedTransport {
    addr: Ipv4Addr,
}

impl dust_dds::transport::interface::TransportParticipantFactory for EmbeddedTransport {
    fn create_participant(
        &self,
        domain_id: i32,
        data_receiver: TransportDataReceiver,
    ) -> dust_dds::transport::interface::RtpsTransportParticipant {
        DDS_DATA_RECEIVER_CHANNEL.sender().send(data_receiver);

        let metatraffic_multicast_locator_list = vec![Locator::new(
            1,
            port_builtin_multicast(domain_id) as u32,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        )];
        let a = self.addr.octets();
        let local_addr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a[0], a[1], a[2], a[3]];
        let default_unicast_locator_list = vec![Locator::new(1, DEFAULT_UNICAST_PORT, local_addr)];
        let metatraffic_unicast_locator_list =
            vec![Locator::new(1, METATRAFFIC_UNICAST_PORT, local_addr)];

        dust_dds::transport::interface::RtpsTransportParticipant {
            message_writer: Box::new(MessageWriter {
                tx_buffer: [0; TX_BUFFER],
            }),
            default_unicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_multicast_locator_list: vec![],
            fragment_size: 1400,
        }
    }
}

struct UdpLocator(Locator);
impl From<&UdpLocator> for Ipv4Addr {
    fn from(value: &UdpLocator) -> Self {
        Ipv4Addr::new(
            value.0.address()[12],
            value.0.address()[13],
            value.0.address()[14],
            value.0.address()[15],
        )
    }
}
impl From<&UdpLocator> for IpEndpoint {
    fn from(value: &UdpLocator) -> Self {
        let address = Ipv4Addr::from(value);
        IpEndpoint::new(address.into(), value.0.port() as u16)
    }
}

#[derive(Clone)]
struct MessageWriter {
    tx_buffer: [u8; TX_BUFFER],
}

impl WriteMessage for MessageWriter {
    fn write_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.tx_buffer
    }

    fn write_message(&mut self, len: usize, locators: &[Locator]) {
        if WRITER_CHANNEL
            .sender()
            .try_send((self.tx_buffer[..len].to_vec(), locators.to_vec()))
            .is_err()
        {
            defmt::warn!("Dropped a message");
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn blinky() -> ! {
    loop {
        Timer::after_millis(500).await;
        if HEAP.used() > HEAP_SIZE * 90 / 100 {
            defmt::warn!("{}B of {}B heap used", HEAP.used(), HEAP_SIZE);
        }
    }
}

#[embassy_executor::task]
async fn metatraffic_multicast(stack: Stack<'static>) -> ! {
    let addr = if let Some(config) = stack.config_v4() {
        config.address.address()
    } else {
        Ipv4Address::UNSPECIFIED
    };

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; RX_BUFFER];
    let mut tx_buffer = [0; TX_BUFFER];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    unwrap!(socket.bind(IpEndpoint::new(
        IpAddress::Ipv4(addr),
        port_builtin_multicast(0),
    )));
    let dds_data_receiver = unwrap!(DDS_DATA_RECEIVER_CHANNEL.receiver()).get().await;
    let mut buf = [0; RX_BUFFER];
    loop {
        let (size, _metadata) = unwrap!(socket.recv_from(&mut buf).await);
        // info!("Received {:?} B of data on metatraffic_multicast", size);
        dds_data_receiver
            .receive_message(buf[..size].to_vec())
            .await;
    }
}

#[embassy_executor::task]
async fn metatraffic_unicast(stack: Stack<'static>) -> ! {
    let addr = if let Some(config) = stack.config_v4() {
        config.address.address()
    } else {
        Ipv4Address::UNSPECIFIED
    };

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; RX_BUFFER];
    let mut tx_buffer = [0; TX_BUFFER];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket
        .bind(IpEndpoint::new(
            IpAddress::Ipv4(addr),
            METATRAFFIC_UNICAST_PORT as u16,
        ))
        .unwrap();

    let dds_data_receiver = unwrap!(DDS_DATA_RECEIVER_CHANNEL.receiver()).get().await;
    let mut buf = Box::new([0; RX_BUFFER]);

    loop {
        let (size, _metadata) = unwrap!(socket.recv_from(buf.as_mut()).await);
        // info!("received metatraffic_unicast {:?} B", size,);
        dds_data_receiver.receive_message(buf[..size].into()).await;
    }
}

#[embassy_executor::task]
async fn default_unicast(stack: Stack<'static>) -> ! {
    let addr = if let Some(config) = stack.config_v4() {
        config.address.address()
    } else {
        Ipv4Address::UNSPECIFIED
    };

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; RX_BUFFER];
    let mut tx_buffer = [0; TX_BUFFER];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    unwrap!(socket.bind(IpEndpoint::new(
        IpAddress::Ipv4(addr),
        DEFAULT_UNICAST_PORT as u16,
    )));

    let dds_data_receiver = unwrap!(DDS_DATA_RECEIVER_CHANNEL.receiver()).get().await;
    let mut buf = [0; RX_BUFFER];

    loop {
        let (size, _metadata) = unwrap!(socket.recv_from(buf.as_mut()).await);
        // info!("received unicast {:?} B", size,);
        dds_data_receiver.receive_message(buf[..size].into()).await;
    }
}

struct Listener {
    sender: SubscriptionSender,
}
impl<Foo> DataWriterListener<Foo> for Listener {
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriterAsync<Foo>,
        status: infrastructure::status::PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        self.sender.send(status.current_count)
    }
}

const HEAP_SIZE: usize = 100000;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    spawner.spawn(unwrap!(blinky()));

    let mut config = Config::default();
    config.rcc.hsi = None;
    config.rcc.hsi48 = Some(Default::default()); // needed for RNG
    config.rcc.hse = Some(Hse {
        freq: Hertz(8_000_000),
        mode: HseMode::BypassDigital,
    });
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV2,
        mul: PllMul::MUL125,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV2),
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb3_pre = APBPrescaler::DIV1;
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.voltage_scale = VoltageScale::Scale0;

    config.rcc.ls = LsConfig::default_lse();

    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2025, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();
    let (mut rtc, rtc_time_provider) = Rtc::new(p.RTC, RtcConfig::default());
    info!("Got RTC! {:?}", now.and_utc().timestamp());
    unwrap!(rtc.set_datetime(now.into()));

    let mut green_led = Output::new(p.PB0, Level::High, Speed::Low);
    let mut yellow_led = Output::new(p.PF4, Level::High, Speed::Low);
    let mut red_led = Output::new(p.PG4, Level::High, Speed::Low);

    let button: Input<'_> = Input::new(p.PC13, Pull::Down);

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        p.PB15,
        p.PG11,
        mac_addr,
        p.ETH_SMA,
        p.PA2,
        p.PC1,
    );

    // Init network stack
    const NUMBER_OF_SOCKETS: usize = 4;
    const NUMBER_OF_SOCKETS_INCL_DHCP: usize = NUMBER_OF_SOCKETS + 1;
    static RESOURCES: StaticCell<StackResources<NUMBER_OF_SOCKETS_INCL_DHCP>> = StaticCell::new();
    // let net_config = embassy_net::Config::dhcpv4(Default::default());

    let net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 204), 24),
        dns_servers: Default::default(),
        gateway: None,
    });

    let (stack, runner) = embassy_net::new(
        device,
        net_config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;

    info!("Network task initialized");
    stack
        .join_multicast_group(Ipv4Addr::new(239, 255, 0, 1))
        .unwrap();
    info!("Joined multicast group");

    let addr = if let Some(config) = stack.config_v4() {
        config.address.address()
    } else {
        Ipv4Address::UNSPECIFIED
    };
    info!("Got addr: {:?}", addr);

    let runtime = MicroRuntime {
        rtc_time_provider: Arc::new(rtc_time_provider),
        spawner: spawner.make_send(),
    };

    let mut app_id = [0; 4];
    rng.fill_bytes(&mut app_id);

    let participant_factory = DomainParticipantFactoryAsync::new(
        runtime,
        app_id,
        [5, 6, 7, 8],
        EmbeddedTransport { addr },
        DustDdsConfiguration::default(),
    );

    info!("Created participant factory");

    let participant = participant_factory
        .create_participant(0, QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    info!("Created participant");

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    let topic = participant
        .create_topic::<ButtonState>(
            "Button",
            "ButtonState",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();

    let writer: DataWriterAsync<ButtonState> = publisher
        .create_datawriter(
            &topic,
            QosKind::Default,
            Some(Listener {
                sender: SUBSCRIPTION_CHANNEL.sender(),
            }),
            &[StatusKind::PublicationMatched],
        )
        .await
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();

    let green_reader_topic = participant
        .create_topic::<ColorState>(
            "GreenLed",
            "ColorState",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();
    let green_reader = subscriber
        .create_datareader::<ColorState>(
            &green_reader_topic,
            QosKind::Specific(DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
                },
                ..Default::default()
            }),
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();
    let yellow_reader_topic = participant
        .create_topic::<ColorState>(
            "YellowLed",
            "ColorState",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();
    let yellow_reader = subscriber
        .create_datareader::<ColorState>(
            &yellow_reader_topic,
            QosKind::Specific(DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
                },
                ..Default::default()
            }),
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();

    green_led.set_low();
    yellow_led.set_low();
    red_led.set_high();

    spawner.spawn(unwrap!(default_unicast(stack)));
    spawner.spawn(unwrap!(metatraffic_unicast(stack)));
    spawner.spawn(unwrap!(metatraffic_multicast(stack)));

    spawner.spawn(unwrap!(writer_task(stack)));

    loop {
        if let Ok(sample) = green_reader.take_next_sample().await {
            if let Some(sample) = sample.data {
                if sample.on {
                    green_led.set_high();
                } else {
                    green_led.set_low();
                }
            }
        }
        if let Ok(sample) = yellow_reader.take_next_sample().await {
            if let Some(sample) = sample.data {
                if sample.on {
                    yellow_led.set_high();
                } else {
                    yellow_led.set_low();
                }
            }
        }
        let pressed = button.is_high();
        writer.write(ButtonState { pressed }, None).await.ok();

        Timer::after_millis(20).await;
        red_led.toggle();
    }
}
