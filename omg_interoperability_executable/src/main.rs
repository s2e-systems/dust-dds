use clap::Parser;
use ctrlc;
use dust_dds::{
    domain::{
        domain_participant::DomainParticipant,
        domain_participant_factory::DomainParticipantFactory,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        error::DdsError,
        qos::{DataReaderQos, DataWriterQos, PublisherQos, QosKind, SubscriberQos},
        qos_policy::{
            self, DataRepresentationQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
            HistoryQosPolicyKind, OwnershipQosPolicy, OwnershipQosPolicyKind,
            OwnershipStrengthQosPolicy, PartitionQosPolicy, ReliabilityQosPolicy,
            XCDR2_DATA_REPRESENTATION, XCDR_DATA_REPRESENTATION,
        },
        status::{InconsistentTopicStatus, StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
    },
    publication::data_writer::DataWriter,
    subscription::{
        data_reader::DataReader,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
    topic_definition::topic::Topic,
};
use rand::{random, thread_rng, Rng};
use std::{
    fmt::Debug,
    io::Write,
    process::{ExitCode, Termination},
    sync::mpsc::Receiver,
};

include!(concat!(env!("OUT_DIR"), "/idl/shape.rs"));

fn qos_policy_name(id: i32) -> String {
    match id {
        qos_policy::DATA_REPRESENTATION_QOS_POLICY_ID => "DATAREPRESENTATION",
        qos_policy::DEADLINE_QOS_POLICY_ID => "DEADLINE",
        qos_policy::DESTINATIONORDER_QOS_POLICY_ID => "DESTINATIONORDER",
        qos_policy::DURABILITY_QOS_POLICY_ID => "DURABILITY",
        qos_policy::DURABILITYSERVICE_QOS_POLICY_ID => "DURABILITYSERVICE",
        qos_policy::ENTITYFACTORY_QOS_POLICY_ID => "ENTITYFACTORY",
        qos_policy::GROUPDATA_QOS_POLICY_ID => "GROUPDATA",
        qos_policy::HISTORY_QOS_POLICY_ID => "HISTORY",
        qos_policy::LATENCYBUDGET_QOS_POLICY_ID => "LATENCYBUDGET",
        qos_policy::LIFESPAN_QOS_POLICY_ID => "LIFESPAN",
        qos_policy::LIVELINESS_QOS_POLICY_ID => "LIVELINESS",
        qos_policy::OWNERSHIP_QOS_POLICY_ID => "OWNERSHIP",
        qos_policy::PARTITION_QOS_POLICY_ID => "PARTITION",
        qos_policy::PRESENTATION_QOS_POLICY_ID => "PRESENTATION",
        qos_policy::READERDATALIFECYCLE_QOS_POLICY_ID => "READERDATALIFECYCLE",
        qos_policy::RELIABILITY_QOS_POLICY_ID => "RELIABILITY",
        qos_policy::RESOURCELIMITS_QOS_POLICY_ID => "RESOURCELIMITS",
        qos_policy::TIMEBASEDFILTER_QOS_POLICY_ID => "TIMEBASEDFILTER",
        qos_policy::TOPICDATA_QOS_POLICY_ID => "TOPICDATA",
        qos_policy::TRANSPORTPRIORITY_QOS_POLICY_ID => "TRANSPORTPRIORITY",
        qos_policy::USERDATA_QOS_POLICY_ID => "USERDATA",
        qos_policy::WRITERDATALIFECYCLE_QOS_POLICY_ID => "WRITERDATALIFECYCLE",
        _ => "UNKNOWN",
    }
    .to_string()
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// publish samples
    #[clap(short = 'P', default_value_t = false)]
    publish: bool,

    /// subscribe samples
    #[clap(short = 'S', default_value_t = false)]
    subscribe: bool,

    /// domain id
    #[clap(short = 'd', default_value_t = 0)]
    domain_id: i32,

    /// BEST_EFFORT reliability
    #[clap(short = 'b', default_value_t = false)]
    best_effort_reliability: bool,

    /// RELIABLE reliability
    #[clap(short = 'r', default_value_t = false)]
    reliable_reliability: bool,

    /// keep history depth (-1: use default, 0: KEEP_ALL)
    #[clap(short = 'k', default_value_t = -1, allow_negative_numbers = true)]
    history_depth: i32,

    /// set a 'deadline' with interval (seconds) (0: OFF)
    #[clap(short = 'f', default_value_t = 0)]
    deadline_interval: i32,

    /// apply 'time based filter' with interval (seconds) (0: OFF)
    #[clap(short = 'i', default_value_t = 0)]
    timebasedfilter_interval: i32,

    /// set ownership strength (-1: SHARED)
    #[clap(short = 's', default_value_t = -1, allow_negative_numbers = true)]
    ownership_strength: i32,

    ///  set the topic name
    #[clap(short = 't', default_value = "Square")]
    topic_name: String,

    /// set color to publish (filter if subscriber)
    #[clap(short = 'c', default_value = None)]
    color: Option<String>,

    /// set a 'partition' string
    #[clap(short = 'p')]
    partition: Option<String>,

    /// set durability (v: VOLATILE,  l: TRANSIENT_LOCAL, t: TRANSIENT, p: PERSISTENT)
    #[clap(short = 'D', default_value_t = 'v')]
    durability_kind: char,

    /// set data representation (1: XCDR, 2: XCDR2)
    #[clap(short = 'x', default_value_t = 1)]
    data_representation: u16,

    /// print Publisher's samples
    #[clap(short = 'w', default_value_t = false)]
    print_writer_samples: bool,

    /// set shapesize (0: increase the size for every sample)
    #[clap(short = 'z', default_value_t = 20)]
    shapesize: i32,

    /// use 'read()' instead of 'take()'
    #[clap(short = 'R', default_value_t = false)]
    use_read: bool,

    /// waiting period between 'write()' operations in ms.
    #[clap(long = "write-period", default_value_t = 33)]
    write_period_ms: i32,

    /// waiting period between 'read()' or 'take()' operations in ms.
    #[clap(long = "read-period", default_value_t = 100)]
    read_period_ms: i32,

    /// set log message verbosity (e: ERROR, d: DEBUG)
    #[clap(short = 'v', default_value_t = 'e')]
    log_message_verbosity: char,
}

impl Options {
    fn validate(&self) -> Result<(), ParsingError> {
        if self.subscribe && self.publish || (!self.subscribe && !self.publish) {
            return Err(ParsingError(
                "must be either subscribe or publish".to_string(),
            ));
        }

        Ok(())
    }

    fn color_for_publisher(&self) -> String {
        match self.color.clone() {
            Some(color) => color,
            None => {
                let default_color = "BLUE".to_string();
                println!(
                    "warning: color was not specified, defaulting to \"{}\"",
                    default_color
                );
                default_color
            }
        }
    }

    fn reliability_qos_policy(&self) -> ReliabilityQosPolicy {
        let mut reliability = DataWriterQos::default().reliability;
        if self.best_effort_reliability {
            reliability.kind = qos_policy::ReliabilityQosPolicyKind::BestEffort;
        }
        if self.reliable_reliability {
            reliability.kind = qos_policy::ReliabilityQosPolicyKind::Reliable;
        }
        reliability
    }

    fn partition_qos_policy(&self) -> PartitionQosPolicy {
        if let Some(partition) = &self.partition {
            PartitionQosPolicy {
                name: vec![partition.to_owned()],
            }
        } else {
            PartitionQosPolicy::default()
        }
    }

    fn durability_qos_policy(&self) -> DurabilityQosPolicy {
        DurabilityQosPolicy {
            kind: match self.durability_kind {
                'v' => qos_policy::DurabilityQosPolicyKind::Volatile,
                'l' => qos_policy::DurabilityQosPolicyKind::TransientLocal,
                't' => qos_policy::DurabilityQosPolicyKind::Transient,
                'p' => qos_policy::DurabilityQosPolicyKind::Persistent,
                _ => panic!("durability not valid"),
            },
        }
    }

    fn data_representation_qos_policy(&self) -> DataRepresentationQosPolicy {
        let data_representation = match self.data_representation {
            1 => XCDR_DATA_REPRESENTATION,
            2 => XCDR2_DATA_REPRESENTATION,
            _ => panic!("Wrong data representation"),
        };
        qos_policy::DataRepresentationQosPolicy {
            value: vec![data_representation],
        }
    }

    fn ownership_qos_policy(&self) -> OwnershipQosPolicy {
        OwnershipQosPolicy {
            kind: match self.ownership_strength {
                -1 => qos_policy::OwnershipQosPolicyKind::Shared,
                _ => qos_policy::OwnershipQosPolicyKind::Exclusive,
            },
        }
    }

    fn history_depth_qos_policy(&self) -> HistoryQosPolicy {
        match self.history_depth {
            -1 => HistoryQosPolicy::default(),
            0 => HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
            },
            x if x >= 1 => HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(x as u32),
            },
            _ => panic!("history_depth not valid"),
        }
    }

    fn ownership_strength_qos_policy(&self) -> OwnershipStrengthQosPolicy {
        if self.ownership_strength < -1 {
            panic!("Ownership strength must be positive or zero")
        }
        OwnershipStrengthQosPolicy {
            value: self.ownership_strength,
        }
    }
}

struct Listener;
impl DomainParticipantListener for Listener {
    fn on_inconsistent_topic(&mut self, the_topic: Topic, _status: InconsistentTopicStatus) {
        println!(
            "on_inconsistent_topic() topic: '{}'  type: '{}'",
            the_topic.get_name(),
            the_topic.get_type_name(),
        );
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<()>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
        let policy_name = qos_policy_name(status.last_policy_id);
        println!(
            "on_offered_incompatible_qos() topic: '{}'  type: '{}' : {:?} ({})",
            the_writer.get_topic().get_name(),
            the_writer.get_topic().get_type_name(),
            status.last_policy_id,
            policy_name
        );
    }

    fn on_publication_matched(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<()>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
        if !the_writer.get_topic().get_name().starts_with("DCPS") {
            println!(
            "on_publication_matched() topic: '{}'  type: '{}' : matched readers {} (change = {})",
            the_writer.get_topic().get_name(),
            the_writer.get_topic().get_type_name(),
            status.current_count,
            status.current_count_change
        );
        }
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<()>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
        println!(
            "on_offered_deadline_missed() topic: '{}'  type: '{}' : (total = {}, change = {})",
            the_writer.get_topic().get_name(),
            the_writer.get_topic().get_type_name(),
            status.total_count,
            status.total_count_change
        );
    }

    fn on_liveliness_lost(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<()>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
        println!(
            "on_liveliness_lost() topic: '{}'  type: '{}' : (total = {}, change = {})",
            the_writer.get_topic().get_name(),
            the_writer.get_topic().get_type_name(),
            status.total_count,
            status.total_count_change
        );
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReader<()>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
        let policy_name = qos_policy_name(status.last_policy_id);
        println!(
            "on_requested_incompatible_qos() topic: '{}'  type: '{}' : {} ({})\n",
            the_reader.get_topicdescription().get_name(),
            the_reader.get_topicdescription().get_type_name(),
            status.last_policy_id,
            policy_name
        );
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: DataReader<()>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        if !the_reader
            .get_topicdescription()
            .get_name()
            .starts_with("DCPS")
        {
            println!(
            "on_subscription_matched() topic: '{}'  type: '{}' : matched writers {} (change = {})",
            the_reader.get_topicdescription().get_name(),
            the_reader.get_topicdescription().get_type_name(),
            status.current_count,
            status.current_count_change
        );
        }
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: DataReader<()>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
        println!(
            "on_requested_deadline_missed() topic: '{}'  type: '{}' : (total = {}, change = {})\n",
            the_reader.get_topicdescription().get_name(),
            the_reader.get_topicdescription().get_type_name(),
            status.total_count,
            status.total_count_change
        );
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: DataReader<()>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
        println!(
            "on_liveliness_changed() topic: '{}'  type: '{}' : (alive = {}, not_alive = {})",
            the_reader.get_topicdescription().get_name(),
            the_reader.get_topicdescription().get_type_name(),
            status.alive_count,
            status.not_alive_count,
        );
    }
}

fn move_shape(
    shape: &mut ShapeType,
    x_vel: &mut i32,
    y_vel: &mut i32,
    da_width: i32,
    da_height: i32,
) {
    shape.x = shape.x + *x_vel;
    shape.y = shape.y + *y_vel;
    if shape.x < 0 {
        shape.x = 0;
        *x_vel = -*x_vel;
    }
    if shape.x > da_width {
        shape.x = da_width;
        *x_vel = -*x_vel;
    }
    if shape.y < 0 {
        shape.y = 0;
        *y_vel = -*y_vel;
    }
    if shape.y > da_height {
        shape.y = da_height;
        *y_vel = -*y_vel;
    }
}

fn init_publisher(
    participant: &DomainParticipant,
    options: Options,
) -> Result<DataWriter<ShapeType>, InitializeError> {
    let topic = participant
        .lookup_topicdescription(&options.topic_name)
        .expect("lookup_topicdescription succeeds")
        .expect("topic existes");
    let publisher_qos = QosKind::Specific(PublisherQos {
        partition: options.partition_qos_policy(),
        ..Default::default()
    });
    let publisher = participant.create_publisher(publisher_qos, None, NO_STATUS)?;
    println!(
        "Create writer for topic: {} color: {}",
        options.topic_name,
        options.color_for_publisher()
    );

    let mut data_writer_qos = DataWriterQos {
        durability: options.durability_qos_policy(),
        reliability: options.reliability_qos_policy(),
        representation: options.data_representation_qos_policy(),
        ownership: options.ownership_qos_policy(),
        history: options.history_depth_qos_policy(),
        ..Default::default()
    };
    if options.deadline_interval > 0 {
        data_writer_qos.deadline.period =
            DurationKind::Finite(Duration::new(options.deadline_interval, 0));
    }
    if options.ownership_qos_policy().kind == OwnershipQosPolicyKind::Exclusive {
        data_writer_qos.ownership_strength = options.ownership_strength_qos_policy();
    }

    let data_writer = publisher.create_datawriter::<ShapeType>(
        &topic,
        QosKind::Specific(data_writer_qos),
        None,
        NO_STATUS,
    )?;

    Ok(data_writer)
}

fn run_publisher(
    data_writer: &DataWriter<ShapeType>,
    options: Options,
    all_done: Receiver<()>,
) -> Result<(), RunningError> {
    let mut random_gen = thread_rng();

    let da_width = 240;
    let da_height = 270;
    let mut shape = ShapeType {
        color: options.color_for_publisher(),
        x: random::<i32>() % da_width,
        y: random::<i32>() % da_height,
        shapesize: options.shapesize,
    };

    // get random non-zero velocity.
    let mut x_vel = if random() {
        random_gen.gen_range(1..5)
    } else {
        random_gen.gen_range(-5..-1)
    };
    let mut y_vel = if random() {
        random_gen.gen_range(1..5)
    } else {
        random_gen.gen_range(-5..-1)
    };

    while all_done.try_recv().is_err() {
        if options.shapesize == 0 {
            shape.shapesize += 1;
        }

        move_shape(&mut shape, &mut x_vel, &mut y_vel, da_width, da_height);
        data_writer.write(&shape, None).ok();
        if options.print_writer_samples {
            println!(
                "{:10} {:10} {:03} {:03} [{:}]",
                options.topic_name.as_str(),
                shape.color,
                shape.x,
                shape.y,
                shape.shapesize
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(
            options.write_period_ms as u64,
        ));
    }
    Ok(())
}

fn init_subscriber(
    participant: &DomainParticipant,
    options: Options,
) -> Result<DataReader<ShapeType>, InitializeError> {
    let topic = participant
        .lookup_topicdescription(&options.topic_name)
        .expect("lookup_topicdescription succeeds")
        .expect("topic existes");
    let subscriber_qos = QosKind::Specific(SubscriberQos {
        partition: options.partition_qos_policy(),
        ..Default::default()
    });
    let subscriber = participant.create_subscriber(subscriber_qos, None, NO_STATUS)?;

    let mut data_reader_qos = DataReaderQos {
        durability: options.durability_qos_policy(),
        reliability: options.reliability_qos_policy(),
        representation: options.data_representation_qos_policy(),
        ownership: options.ownership_qos_policy(),
        history: options.history_depth_qos_policy(),
        ..Default::default()
    };
    if options.deadline_interval > 0 {
        data_reader_qos.deadline.period =
            DurationKind::Finite(Duration::new(options.deadline_interval, 0));
    }

    let data_reader = match &options.color {
        // filter on specified color
        Some(color) => {
            let filtered_topic_name = options.topic_name + "_filtered";
            // return Err(InitializeError(
            //     "contenfilter topic not implemented".to_string(),
            // ));
            println!(
                "Create reader for topic: {} color: {}",
                filtered_topic_name, color
            );
            subscriber.create_datareader::<ShapeType>(
                &topic,
                QosKind::Specific(data_reader_qos),
                None,
                NO_STATUS,
            )?
        }
        // No filter on specified color
        None => {
            println!("Create reader for topic: {} ", options.topic_name);
            subscriber.create_datareader::<ShapeType>(
                &topic,
                QosKind::Specific(data_reader_qos),
                None,
                NO_STATUS,
            )?
        }
    };

    Ok(data_reader)
}

fn run_subscriber(
    data_reader: &DataReader<ShapeType>,
    options: Options,
    all_done: Receiver<()>,
) -> Result<(), RunningError> {
    while all_done.try_recv().is_err() {
        let mut previous_handle = None;
        loop {
            let max_samples = i32::MAX;
            let read_result = if options.use_read {
                data_reader.read_next_instance(
                    max_samples,
                    previous_handle,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    ANY_INSTANCE_STATE,
                )
            } else {
                data_reader.take_next_instance(
                    max_samples,
                    previous_handle,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    ANY_INSTANCE_STATE,
                )
            };
            match read_result {
                Ok(samples) => {
                    for sample in samples {
                        if sample.sample_info().valid_data {
                            let smaple_data = sample.data().expect("data present");
                            println!(
                                "{:10} {:10} {:03} {:03} [{}]",
                                data_reader.get_topicdescription().get_name(),
                                smaple_data.color,
                                smaple_data.x,
                                smaple_data.y,
                                smaple_data.shapesize
                            );
                            std::io::stdout().flush().expect("flush stdout succeeds");
                        }
                        previous_handle = Some(sample.sample_info().instance_handle);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(
                        options.read_period_ms as u64,
                    ));
                }
                Err(_) => break,
            }
        }
    }
    Ok(())
}

fn initialize(options: &Options) -> Result<DomainParticipant, InitializeError> {
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory.create_participant(
        options.domain_id,
        QosKind::Default,
        Some(Box::new(Listener)),
        &[
            StatusKind::InconsistentTopic,
            StatusKind::OfferedIncompatibleQos,
            StatusKind::PublicationMatched,
            StatusKind::OfferedDeadlineMissed,
            StatusKind::LivelinessLost,
            StatusKind::RequestedIncompatibleQos,
            StatusKind::SubscriptionMatched,
            StatusKind::RequestedDeadlineMissed,
            StatusKind::LivelinessChanged,
        ],
    )?;
    println!("Create topic: {}", options.topic_name);
    let _topic = participant.create_topic::<ShapeType>(
        &options.topic_name,
        "ShapeType",
        QosKind::Default,
        None,
        NO_STATUS,
    )?;

    Ok(participant)
}

struct ParsingError(String);
struct InitializeError(String);
struct RunningError(String);

impl From<DdsError> for InitializeError {
    fn from(value: DdsError) -> Self {
        Self(format!("DdsError: {:?}", value))
    }
}
impl From<DdsError> for RunningError {
    fn from(value: DdsError) -> Self {
        Self(format!("DdsError: {:?}", value))
    }
}

struct Return {
    code: u8,
    describtion: String,
}
impl Debug for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("code {}: {}", self.code, self.describtion))
    }
}

impl Termination for Return {
    fn report(self) -> ExitCode {
        self.code.into()
    }
}

impl From<ParsingError> for Return {
    fn from(value: ParsingError) -> Self {
        Self {
            code: 1,
            describtion: value.0,
        }
    }
}

impl From<InitializeError> for Return {
    fn from(value: InitializeError) -> Self {
        Self {
            code: 2,
            describtion: value.0,
        }
    }
}

impl From<RunningError> for Return {
    fn from(value: RunningError) -> Self {
        Self {
            code: 3,
            describtion: value.0,
        }
    }
}

fn main() -> Result<(), Return> {
    let (tx, rx) = std::sync::mpsc::channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    let options = Options::parse();
    options.validate()?;
    let participant = initialize(&options)?;
    if options.publish {
        let data_writer = init_publisher(&participant, options.clone())?;
        run_publisher(&data_writer, options.clone(), rx)?;
    } else {
        let data_reader = init_subscriber(&participant, options.clone())?;
        run_subscriber(&data_reader, options.clone(), rx)?;
    }
    participant
        .delete_contained_entities()
        .expect("Entites beeing deleted");
    println!("Done.");
    Ok(())
}
