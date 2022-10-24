use crate::infrastructure::{instance::InstanceHandle, time::Time};

// Sample states to support reads
pub type SampleStateKind = u32;
pub const READ_SAMPLE_STATE: SampleStateKind = 0x0001 << 0;
pub const NOT_READ_SAMPLE_STATE: SampleStateKind = 0x0001 << 1;

// This is a bit-mask SampleStateKind
pub type SampleStateMask = u32;
pub const ANY_SAMPLE_STATE: SampleStateMask = 0xffff;

// View states to support reads
pub type ViewStateKind = u32;
pub const NEW_VIEW_STATE: ViewStateKind = 0x0001 << 0;
pub const NOT_NEW_VIEW_STATE: ViewStateKind = 0x0001 << 1;

// This is a bit-mask ViewStateKind
pub type ViewStateMask = u32;
pub const ANY_VIEW_STATE: ViewStateMask = 0xffff;

// Instance states to support reads
pub type InstanceStateKind = u32;
pub const ALIVE_INSTANCE_STATE: InstanceStateKind = 0x0001 << 0;
pub const NOT_ALIVE_DISPOSED_INSTANCE_STATE: InstanceStateKind = 0x0001 << 1;
pub const NOT_ALIVE_NO_WRITERS_INSTANCE_STATE: InstanceStateKind = 0x0001 << 2;

// This is a bit-mask InstanceStateKind
pub type InstanceStateMask = u32;
pub const ANY_INSTANCE_STATE: InstanceStateMask = 0xffff;
pub const NOT_ALIVE_INSTANCE_STATE: InstanceStateMask = 0x006;

pub struct SampleInfo {
    /// The sample_state (READ or NOT_READ) - indicates whether or not the corresponding data sample has already been read.
    pub sample_state: SampleStateKind,
    /// The view_state, (NEW, or NOT_NEW) - indicates whether the DataReader has already seen samples for the most current generation of the related instance
    pub view_state: ViewStateKind,
    /// The instance_state (ALIVE, NOT_ALIVE_DISPOSED, or NOT_ALIVE_NO_WRITERS) - indicates whether the
    /// instance is currently in existence or, if it has been disposed, the reason why it was disposed.
    /// • ALIVE if this instance is currently in existence.
    /// • NOT_ALIVE_DISPOSED if this instance was disposed by the a DataWriter.
    /// • NOT_ALIVE_NO_WRITERS if the instance has been disposed by the DataReader because none of the
    /// DataWriter objects currently “alive” (according to the LIVELINESS QoS) are writing the instance.
    pub instance_state: InstanceStateKind,
    /// The disposed_generation_count that indicates the number of times the instance had become alive after it was disposed
    /// explicitly by a DataWriter, at the time the sample was received.
    pub disposed_generation_count: i32,
    /// The no_writers_generation_count that indicates the number of times the instance had become alive after it was
    /// disposed because there were no writers, at the time the sample was received.
    pub no_writers_generation_count: i32,
    /// The sample_rank that indicates the number of samples related to the same instance that follow in the collection
    /// returned by read or take.
    pub sample_rank: i32,
    /// The generation_rank that indicates the generation difference (number of times the instance was disposed and become
    /// alive again) between the time the sample was received, and the time the most recent sample in the collection related to
    /// the same instance was received.
    pub generation_rank: i32,
    /// The absolute_generation_rank that indicates the generation difference (number of times the instance was disposed and
    /// become alive again) between the time the sample was received, and the time the most recent sample (which may not be
    /// in the returned collection) related to the same instance was received.
    pub absolute_generation_rank: i32,
    /// the source_timestamp that indicates the time provided by the DataWriter when the sample was written.
    pub source_timestamp: Option<Time>,
    /// the instance_handle that identifies locally the corresponding instance
    pub instance_handle: InstanceHandle,
    /// the publication_handle that identifies locally the DataWriter that modified the instance. The publication_handle is the
    /// same InstanceHandle_t that is returned by the operation get_matched_publications on the DataReader and can also
    /// be used as a parameter to the DataReader operation get_matched_publication_data.
    pub publication_handle: InstanceHandle,
    /// the valid_data flag that indicates whether the DataSample contains data or else it is only used to communicate of a
    /// change in the instance_state of the instance.
    pub valid_data: bool,
}
