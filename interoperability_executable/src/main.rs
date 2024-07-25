use clap::Parser;
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        qos::QosKind,
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    }, publication::data_writer_listener::DataWriterListener,
};

#[derive(Parser)]
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
    #[clap(short = 'c', default_value = "BLUE")]
    color: String,

    /// set a 'partition' string
    #[clap(short = 'p')]
    partition: Option<String>,

    /// set durability (v: VOLATILE,  l: TRANSIENT_LOCAL, t: TRANSIENT, p: PERSISTENT)
    #[clap(short = 'D', default_value_t = 'v')]
    durability_kind: char,

    /// set data representation (1: XCDR, 2: XCDR2)
    #[clap(short = 'x', default_value_t = 1)]
    data_representation: i32,

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
impl DataWriterListener<'_> for Listener {
    type Foo = ShapeType;
    
    fn on_liveliness_lost(
        &mut self,
        _the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        _status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
    }
    
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        _status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
    }
    
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        _status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
    }
    
    fn on_publication_matched(
        &mut self,
        _the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        _status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
    }
    
}




fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory.create_participant(
        cli.domain_id,
        QosKind::Default,
        None,
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

    if cli.publish {

        let publisher = participant.create_publisher(QosKind::Default, None, NO_STATUS)?;
        println!("Create writer for topic: {} color: {}", cli.topic_name, cli.color);
        let data_writer =
            publisher.create_datawriter::<ShapeType>(&topic, QosKind::Default, None, NO_STATUS)?;
        let writer_cond = data_writer.get_statuscondition();
        writer_cond.set_enabled_statuses(&[StatusKind::PublicationMatched])?;
        let mut wait_set = WaitSet::new();
        wait_set.attach_condition(Condition::StatusCondition(writer_cond))?;
        wait_set.wait(Duration::new(60, 0))?;

        let sample = ShapeType {
            color: cli.color,
            x: 0,
            y: 0,
            shapesize: cli.shapesize,
        };
        data_writer.write(&sample, None)?;



        // Todo: Maybe wait_for_ackn for reliablity or subcription matched decreases
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // let reader_cond = reader.get_statuscondition();
        // reader_cond
        //     .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        //     .unwrap();
        // let mut wait_set = WaitSet::new();
        // wait_set
        //     .attach_condition(Condition::StatusCondition(reader_cond.clone()))
        //     .unwrap();
        // wait_set.wait(Duration::new(600, 0)).unwrap();
    }
    Ok(())
}
