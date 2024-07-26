use clap::Parser;
use dust_dds::{
    domain::{
        domain_participant_factory::DomainParticipantFactory,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        error::DdsError,
        qos::{DataReaderQos, DataWriterQos, PublisherQos, QosKind, SubscriberQos},
        qos_policy::{self, DurabilityQosPolicy, PartitionQosPolicy},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    publication::data_writer::DataWriter,
    subscription::{
        data_reader::DataReader,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
};
use rand::{random, thread_rng, Rng};

fn qos_policy_name(id: i32) -> String {
    match id {
        qos_policy::USERDATA_QOS_POLICY_ID => "USERDATA",
        qos_policy::DURABILITY_QOS_POLICY_ID => "DURABILITY",
        qos_policy::PRESENTATION_QOS_POLICY_ID => "PRESENTATION",
        qos_policy::DEADLINE_QOS_POLICY_ID => "DEADLINE",
        qos_policy::LATENCYBUDGET_QOS_POLICY_ID => "LATENCYBUDGET",
        qos_policy::OWNERSHIP_QOS_POLICY_ID => "OWNERSHIP",
        qos_policy::LIVELINESS_QOS_POLICY_ID => "LIVELINESS",
        qos_policy::TIMEBASEDFILTER_QOS_POLICY_ID => "TIMEBASEDFILTER",
        qos_policy::PARTITION_QOS_POLICY_ID => "PARTITION",
        qos_policy::RELIABILITY_QOS_POLICY_ID => "RELIABILITY",
        qos_policy::DESTINATIONORDER_QOS_POLICY_ID => "DESTINATIONORDER",
        qos_policy::HISTORY_QOS_POLICY_ID => "HISTORY",
        qos_policy::RESOURCELIMITS_QOS_POLICY_ID => "RESOURCELIMITS",
        qos_policy::ENTITYFACTORY_QOS_POLICY_ID => "ENTITYFACTORY",
        qos_policy::WRITERDATALIFECYCLE_QOS_POLICY_ID => "WRITERDATALIFECYCLE",
        qos_policy::READERDATALIFECYCLE_QOS_POLICY_ID => "READERDATALIFECYCLE",
        qos_policy::TOPICDATA_QOS_POLICY_ID => "TOPICDATA",
        qos_policy::GROUPDATA_QOS_POLICY_ID => "GROUPDATA",
        qos_policy::TRANSPORTPRIORITY_QOS_POLICY_ID => "TRANSPORTPRIORITY",
        qos_policy::LIFESPAN_QOS_POLICY_ID => "LIFESPAN",
        qos_policy::DURABILITYSERVICE_QOS_POLICY_ID => "DURABILITYSERVICE",
        _ => "UNKNOWN",
    }
    .to_string()
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
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
    #[clap(short = 'k', default_value_t = -1)]
    history_depth: i32,

    /// set a 'deadline' with interval (seconds) (0: OFF)
    #[clap(short = 'f', default_value_t = 0)]
    deadline_interval: i32,

    /// apply 'time based filter' with interval (seconds) (0: OFF)
    #[clap(short = 'i', default_value_t = 0)]
    timebasedfilter_interval: i32,

    /// set ownership strength (-1: SHARED)
    #[clap(short = 's', default_value_t = -1)]
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

#[derive(Debug)]
struct Error(String);
impl From<DdsError> for Error {
    fn from(value: DdsError) -> Self {
        Self(format!("DDS error: {:?}", value))
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
pub struct ShapeType {
    #[dust_dds(key)]
    pub color: String,
    pub x: i32,
    pub y: i32,
    pub shapesize: i32,
}

struct Listener;
impl DomainParticipantListener for Listener {
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
        println!(
            "on_publication_matched() topic: '{}'  type: '{}' : matched readers {} (change = {})",
            the_writer.get_topic().get_name(),
            the_writer.get_topic().get_type_name(),
            status.current_count,
            status.current_count_change
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

#[test]
fn te() {
    //%-10s %-10s %03d %03d [%d]
    println!(
        "{:10} {:10} {:03} {:03} [{:}]",
        "hellohellohello", "world", 3, 4, 5
    );
}

fn run_publisher(data_writer: &DataWriter<ShapeType>, cli: Cli) {
    let mut random_gen = thread_rng();

    let da_width = 240;
    let da_height = 270;
    let mut shape = ShapeType {
        color: cli.color.unwrap_or("BLUE".to_string()),
        x: random::<i32>() % da_width,
        y: random::<i32>() % da_height,
        shapesize: cli.shapesize,
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

    loop {
        if cli.shapesize == 0 {
            shape.shapesize += 1;
        }

        move_shape(&mut shape, &mut x_vel, &mut y_vel, da_width, da_height);
        data_writer
            .write(&shape, None)
            .expect("write shape sample succeeds");
        if cli.print_writer_samples {
            println!(
                "{:10} {:10} {:03} {:03} [{:}]",
                cli.topic_name.as_str(),
                shape.color,
                shape.x,
                shape.y,
                shape.shapesize
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(cli.write_period_ms as u64));
    }
}

fn run_subscriber(data_reader: &DataReader<ShapeType>, cli: Cli) {
    loop {
        let mut previous_handle = None;
        loop {
            let max_samples = i32::MAX;
            let result = if cli.use_read {
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
            match result {
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
                        }
                        previous_handle = Some(sample.sample_info().instance_handle);
                    }
                }
                Err(_) => break,
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(cli.read_period_ms as u64));
    }
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory.create_participant(
        cli.domain_id,
        QosKind::Default,
        Some(Box::new(Listener)),
        NO_STATUS,
    )?;
    println!("Create topic: {}", cli.topic_name);
    let topic = participant.create_topic::<ShapeType>(
        &cli.topic_name,
        "ShapeType",
        QosKind::Default,
        None,
        NO_STATUS,
    )?;
    if cli.subscribe && cli.publish || (!cli.subscribe && !cli.publish) {
        panic!("must be either subscribe or publish")
    }
    if cli.best_effort_reliability && cli.reliable_reliability
        || (!cli.best_effort_reliability && !cli.reliable_reliability)
    {
        panic!("reliability must be either best effort or reliable")
    }
    let mut reliability = DataWriterQos::default().reliability;
    if cli.best_effort_reliability {
        reliability.kind = qos_policy::ReliabilityQosPolicyKind::BestEffort;
    }
    if cli.reliable_reliability {
        reliability.kind = qos_policy::ReliabilityQosPolicyKind::Reliable;
    }
    let partition = if let Some(partition) = &cli.partition {
        PartitionQosPolicy {
            name: vec![partition.to_owned()],
        }
    } else {
        PartitionQosPolicy::default()
    };

    let durability = DurabilityQosPolicy {
        kind: match cli.durability_kind {
            'v' => qos_policy::DurabilityQosPolicyKind::Volatile,
            'l' => qos_policy::DurabilityQosPolicyKind::TransientLocal,
            't' => qos_policy::DurabilityQosPolicyKind::Volatile, // Todo: TRANSIENT
            'p' => qos_policy::DurabilityQosPolicyKind::Volatile, // Todo: PERSISTENT
            _ => panic!("durability not valid"),
        },
    };

    let _representation = qos_policy::DataRepresentationQosPolicy {
        value: vec![cli.data_representation],
    };

    let ownership = qos_policy::OwnershipQosPolicy {
        kind: match cli.ownership_strength {
            -1 => qos_policy::OwnershipQosPolicyKind::Shared,
            _ => qos_policy::OwnershipQosPolicyKind::Shared, //Todo: Exclusive,
        },
    };

    if cli.publish {
        let cli = cli.clone();
        let publisher_qos = QosKind::Specific(PublisherQos {
            partition: partition.clone(),
            ..Default::default()
        });
        let publisher = participant.create_publisher(publisher_qos, None, NO_STATUS)?;
        println!(
            "Create writer for topic: {} color: {}",
            cli.topic_name,
            cli.color.clone().expect("Color must be set for publish")
        );

        let mut data_writer_qos = DataWriterQos {
            durability: durability.clone(),
            reliability: reliability.clone(),
            // representation: representation.clone(),
            ownership: ownership.clone(),
            ..Default::default()
        };
        if cli.deadline_interval > 0 {
            data_writer_qos.deadline.period =
                DurationKind::Finite(Duration::new(cli.deadline_interval, 0));
        }

        let data_writer = publisher.create_datawriter::<ShapeType>(
            &topic,
            QosKind::Specific(data_writer_qos),
            None,
            NO_STATUS,
        )?;
        let writer_cond = data_writer.get_statuscondition();
        writer_cond.set_enabled_statuses(&[StatusKind::PublicationMatched])?;
        let mut wait_set = WaitSet::new();
        wait_set.attach_condition(Condition::StatusCondition(writer_cond))?;
        wait_set.wait(Duration::new(60, 0))?;
        
        run_publisher(&data_writer, cli);
    }

    if cli.subscribe {
        let subscriber_qos = QosKind::Specific(SubscriberQos {
            partition: partition.clone(),
            ..Default::default()
        });
        let subscriber = participant.create_subscriber(subscriber_qos, None, NO_STATUS)?;
        match &cli.color {
            Some(color) => println!(
                "Create reader for topic: {} color: {}",
                cli.topic_name, color
            ),
            None => println!("Create reader for topic: {} ", cli.topic_name),
        }

        let mut data_reader_qos = DataReaderQos {
            durability: durability.clone(),
            reliability: reliability.clone(),
            // representation: representation.clone(),
            ownership: ownership.clone(),
            ..Default::default()
        };
        if cli.deadline_interval > 0 {
            data_reader_qos.deadline.period =
                DurationKind::Finite(Duration::new(cli.deadline_interval, 0));
        }

        let data_reader = subscriber.create_datareader::<ShapeType>(
            &topic,
            QosKind::Specific(data_reader_qos),
            None,
            NO_STATUS,
        )?;
        let condition = data_reader.get_statuscondition();
        condition.set_enabled_statuses(&[StatusKind::SubscriptionMatched])?;
        let mut wait_set = WaitSet::new();
        wait_set.attach_condition(Condition::StatusCondition(condition))?;
        wait_set.wait(Duration::new(60, 0))?;
        run_subscriber(&data_reader, cli);
    }
    println!("Done.");
    Ok(())
}
