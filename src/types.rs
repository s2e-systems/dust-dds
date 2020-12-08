pub trait DDSType: 'static {
    fn type_name() -> &'static str;

    fn topic_kind() -> TopicKind;
    
    fn instance_handle(&self) -> InstanceHandle;

    fn serialize(&self) -> Data;

    fn deserialize(data: Data) -> Self;
}


pub type DomainId = i32;
pub type InstanceHandle = [u8; 16];
pub type Data = Vec<u8>;

pub type ReturnCode<T> = Result<T, ReturnCodes>;

#[derive(Debug, PartialEq)]
pub enum ReturnCodes {
    Error,
    Unsupported,
    BadParameter,
    PreconditionNotMet(&'static str),
    OutOfResources,
    NotEnabled,
    ImmutablePolicy,
    InconsistentPolicy,
    AlreadyDeleted,
    Timeout,
    NoData,
    IllegalOperation,
}

pub enum EntityType {
    BuiltIn,
    UserDefined,
}

//// From DDS
#[derive(PartialOrd, PartialEq, Debug, Clone)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}

pub const DURATION_INFINITE: Duration = Duration{sec: 0x7fffffff, nanosec:0x7fffffff};
pub const DURATION_ZERO: Duration = Duration{sec: 0, nanosec:0};


pub struct Time {
    pub sec: i32,
    pub nanosec: u32,
}

pub const TIME_INVALID: Time = Time{sec: -1, nanosec:0xffffffff};

pub type Length = i32;
pub const LENGTH_UNLIMITED: i32 = -1;  

pub struct ResourceLimits {
    pub max_samples: Length,
    pub max_instances: Length,
    pub max_samples_per_instance: Length,
}

pub enum HistoryKind {
    KeepAll,
    KeepLast(i32),
}

pub type SequenceNumber = i64;

pub const SEQUENCE_NUMBER_UNKNOWN : SequenceNumber = std::i64::MIN;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

pub type ParameterId = i16;

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    parameter_id: ParameterId,
    length: i16, // length is rounded up to multple of 4
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id,
            length: (value.len() + 3 & !3) as i16,
            value,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    pub fn length(&self) -> i16 {
        self.length
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ParameterList {
    pub parameter: Vec<Parameter>,
}

impl ParameterList {
    pub const PID_SENTINEL : ParameterId = 0x0001;
    
    pub fn new() -> Self {
        Self {
            parameter: Vec::new(),
        }
    }
}

//// From RTPS
#[derive(Copy, Clone)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

 
