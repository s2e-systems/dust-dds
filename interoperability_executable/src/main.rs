use clap::Parser;

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
    #[clap(short = 't')]
    topic_name: Option<String>,

    /// set color to publish (filter if subscriber)
    #[clap(short = 'c')]
    color: Option<String>,

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

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    println!("{:?} {:?}", cli.color, cli.ownership_strength);
    Ok(())
}
