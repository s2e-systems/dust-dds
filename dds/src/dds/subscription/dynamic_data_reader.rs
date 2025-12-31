use super::subscriber::Subscriber;
use crate::{
    condition::StatusCondition,
    dds_async::dynamic_data_reader::{DynamicDataReaderAsync, DynamicSample},
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::DataReaderQos,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
        status::StatusKind,
    },
    std_runtime::executor::block_on,
    xtypes::dynamic_type::DynamicType,
};
use alloc::vec::Vec;

/// A [`DynamicDataReader`] allows the application to subscribe to a topic and access received data using runtime type information.
///
/// Unlike [`DataReader`](crate::subscription::data_reader::DataReader), this reader works with
/// [`DynamicData`](crate::xtypes::dynamic_type::DynamicData) and does not require compile-time type knowledge.
/// This is useful for generic monitoring tools, data recorders, or dynamic language bindings.
pub struct DynamicDataReader {
    reader_async: DynamicDataReaderAsync,
}

impl DynamicDataReader {
    pub(crate) fn reader_async(&self) -> &DynamicDataReaderAsync {
        &self.reader_async
    }
}

impl From<DynamicDataReaderAsync> for DynamicDataReader {
    fn from(value: DynamicDataReaderAsync) -> Self {
        Self {
            reader_async: value,
        }
    }
}

impl Clone for DynamicDataReader {
    fn clone(&self) -> Self {
        Self {
            reader_async: self.reader_async.clone(),
        }
    }
}

impl DynamicDataReader {
    /// Returns the dynamic type associated with this reader.
    pub fn get_type(&self) -> &DynamicType {
        self.reader_async.get_type()
    }

    /// Returns the topic name this reader is subscribed to.
    pub fn get_topic_name(&self) -> &str {
        self.reader_async.get_topic_name()
    }

    /// Read samples with full filtering options.
    ///
    /// This operation accesses samples from the reader without removing them from the
    /// internal cache. Subsequent reads will return the same samples (unless they are
    /// taken or the cache overflows).
    #[tracing::instrument(skip(self))]
    pub fn read(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<DynamicSample>> {
        block_on(
            self.reader_async
                .read(max_samples, sample_states, view_states, instance_states),
        )
    }

    /// Take samples with full filtering options.
    ///
    /// This operation accesses samples from the reader and removes them from the
    /// internal cache. Subsequent takes will not return the same samples.
    #[tracing::instrument(skip(self))]
    pub fn take(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<DynamicSample>> {
        block_on(
            self.reader_async
                .take(max_samples, sample_states, view_states, instance_states),
        )
    }

    /// Read the next unread sample.
    #[tracing::instrument(skip(self))]
    pub fn read_next_sample(&self) -> DdsResult<DynamicSample> {
        block_on(self.reader_async.read_next_sample())
    }

    /// Take the next unread sample.
    #[tracing::instrument(skip(self))]
    pub fn take_next_sample(&self) -> DdsResult<DynamicSample> {
        block_on(self.reader_async.take_next_sample())
    }

    /// Get the status condition for this reader.
    pub fn get_statuscondition(&self) -> StatusCondition {
        StatusCondition::new(self.reader_async.get_statuscondition())
    }

    /// Get the subscriber that created this reader.
    pub fn get_subscriber(&self) -> Subscriber {
        Subscriber::from(self.reader_async.get_subscriber())
    }

    /// Get the instance handle of this reader.
    pub fn get_instance_handle(&self) -> InstanceHandle {
        block_on(self.reader_async.get_instance_handle())
    }

    /// Get the QoS settings for this reader.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        block_on(self.reader_async.get_qos())
    }

    /// Enable this reader.
    #[tracing::instrument(skip(self))]
    pub fn enable(&self) -> DdsResult<()> {
        block_on(self.reader_async.enable())
    }

    /// Get the status changes for this reader.
    #[tracing::instrument(skip(self))]
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        block_on(self.reader_async.get_status_changes())
    }
}
