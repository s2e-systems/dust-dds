use crate::api::dcps_psm::{InstanceHandle, InstanceStateKind, SampleStateKind, Time, ViewStateKind};

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
    pub source_timestamp: Time,
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
