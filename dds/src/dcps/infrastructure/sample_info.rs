use alloc::sync::Arc;

use crate::{
    infrastructure::{instance::InstanceHandle, time::Time},
    xtypes::{dynamic_type::DynamicType, type_support::TypeSupport},
};

/// A [`Sample`] contains the data and [`SampleInfo`] read by the [`DataReader`].
#[derive(Debug, PartialEq)]
pub struct Sample<Foo> {
    /// Data received by the [`DataReader`]. A sample might contain no valid data in which case this field is [`None`].
    pub data: Option<Foo>,
    /// Information of the sample received by the [`DataReader`].
    pub sample_info: SampleInfo,
}

impl<Foo> Sample<Foo>
where
    Foo: TypeSupport,
{
    pub(crate) fn from_bytes(
        topic_name: &str,
        type_support: DynamicType<'static>,
        data: Arc<[u8]>,
        sample_info: SampleInfo,
    ) -> Self {
        let data = if sample_info.valid_data {
            crate::dcps::dcps_domain_participant::reader_methods::deserialize_topic_type(
                topic_name,
                type_support,
                data.as_ref(),
            )
            .as_mut()
            .and_then(Foo::create_sample)
        } else {
            None
        };
        Self { data, sample_info }
    }
}

/// Enumeration of the possible sample states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampleStateKind {
    /// This value indicates that the sample has already been access by means of a read operation.
    Read,
    /// This value indicates that the sample has not been accessed before.
    NotRead,
}

/// Special constant containing a list of all the sample states.
pub const ANY_SAMPLE_STATE: &[SampleStateKind] = &[SampleStateKind::Read, SampleStateKind::NotRead];

/// Enumeration of the possible sample view states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewStateKind {
    /// This value indicates that either this is the first time that the reader has ever accessed samples of that instance, or else
    /// that the reader has accessed previous samples of the instance, but the instance has since been reborn (i.e., become
    /// not-alive and then alive again). These two cases are distinguished by examining the [`SampleInfo::disposed_generation_count`] and
    /// [`SampleInfo::no_writers_generation_count`].
    New,
    /// This value indicates that the reader has already accessed samples of the same instance and that the instance has not been reborn since.
    NotNew,
}

/// Special constant containing a list of all the view states.
pub const ANY_VIEW_STATE: &[ViewStateKind] = &[ViewStateKind::New, ViewStateKind::NotNew];

/// Enumeration of the possible instance states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InstanceStateKind {
    /// This value indicates that  (a) samples have been received for the instance, (b) there are live [`DataWriter`](crate::publication::data_writer::DataWriter)
    /// entities writing the instance, and (c) the instance has not been explicitly disposed (or else more samples have been received after it was disposed).
    Alive,
    /// This value indicates the instance was explicitly disposed by means of the [`DataWriter::dispose`](crate::publication::data_writer::DataWriter) operation.
    NotAliveDisposed,
    /// This value indicates the instance has been declared as not-alive by the DataReader because it
    /// detected that there are no live [`DataWriter`](crate::publication::data_writer::DataWriter) entities writing that instance.
    NotAliveNoWriters,
}

/// Special constant containing a list of all the instance states.
pub const ANY_INSTANCE_STATE: &[InstanceStateKind] = &[
    InstanceStateKind::Alive,
    InstanceStateKind::NotAliveDisposed,
    InstanceStateKind::NotAliveNoWriters,
];

/// Special constant containing a list of all the *not alive* instance states.
pub const NOT_ALIVE_INSTANCE_STATE: &[InstanceStateKind] = &[
    InstanceStateKind::NotAliveDisposed,
    InstanceStateKind::NotAliveNoWriters,
];

/// The [`SampleInfo`] contains the information associated with each received data value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SampleInfo {
    /// This field indicates whether or not the corresponding data sample has already been read.
    pub sample_state: SampleStateKind,
    /// This field indicates whether the [`DataReader`](crate::subscription::data_reader::DataReader) has already seen
    /// samples for the most current generation of the related instance.
    pub view_state: ViewStateKind,
    /// The field indicates whether the instance is currently in existence or, if it has been disposed, the reason why it was disposed..
    pub instance_state: InstanceStateKind,
    /// This field indicates the number of times the instance had become alive after it was disposed
    /// explicitly by a [`DataWriter`](crate::publication::data_writer::DataWriter), at the time the sample was received.
    pub disposed_generation_count: i32,
    /// This field indicates the number of times the instance had become alive after it was disposed because there
    /// were no writers, at the time the sample was received.
    pub no_writers_generation_count: i32,
    /// This field indicates the number of samples related to the same instance that follow in the returned collection.
    pub sample_rank: i32,
    /// This field indicates the generation difference (number of times the instance was disposed and become
    /// alive again) between the time the sample was received, and the time the most recent sample in the collection related to
    /// the same instance was received.
    pub generation_rank: i32,
    /// This field indicates the generation difference (number of times the instance was disposed and
    /// become alive again) between the time the sample was received, and the time the most recent sample (which may not be
    /// in the returned collection) related to the same instance was received.
    pub absolute_generation_rank: i32,
    /// This field indicates the time provided by the [`DataWriter`](crate::publication::data_writer::DataWriter) when the sample was written.
    pub source_timestamp: Option<Time>,
    /// This field indicated the [`InstanceHandle`] that identifies locally the corresponding instance.
    pub instance_handle: InstanceHandle,
    /// This field indicated the local identification of the [`DataWriter`](crate::publication::data_writer::DataWriter) that modified the instance.
    /// This handle is the same [`InstanceHandle`] that is returned by the operation [`DataReader::get_matched_publications`](crate::subscription::data_reader::DataReader)
    /// and can also be used as a parameter to the [`DataReader::get_matched_publication_data`](crate::subscription::data_reader::DataReader) operation.
    pub publication_handle: InstanceHandle,
    /// This field indicates whether the sample contains data or if it is only used to communicate of a change in the [`SampleInfo::instance_state`] of the instance.
    pub valid_data: bool,
}

/// A mask representation of [`SampleStateKind`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampleStateMask(u8);

impl SampleStateMask {
    /// Read sample state.
    pub const READ: Self = Self(1 << 0);
    /// NotRead sample state.
    pub const NOT_READ: Self = Self(1 << 1);
    /// Any sample state (Read or NotRead).
    pub const ANY: Self = Self((1 << 0) | (1 << 1));

    /// Checks if the sample state kind is enabled in this mask.
    pub const fn contains(&self, state: SampleStateKind) -> bool {
        let bit = match state {
            SampleStateKind::Read => 1 << 0,
            SampleStateKind::NotRead => 1 << 1,
        };
        (self.0 & bit) != 0
    }
}

impl From<&[SampleStateKind]> for SampleStateMask {
    fn from(states: &[SampleStateKind]) -> Self {
        let mut mask = 0;
        for &state in states {
            mask |= match state {
                SampleStateKind::Read => 1 << 0,
                SampleStateKind::NotRead => 1 << 1,
            };
        }
        Self(mask)
    }
}

impl From<SampleStateKind> for SampleStateMask {
    fn from(state: SampleStateKind) -> Self {
        Self(match state {
            SampleStateKind::Read => 1 << 0,
            SampleStateKind::NotRead => 1 << 1,
        })
    }
}

/// A mask representation of [`ViewStateKind`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewStateMask(u8);

impl ViewStateMask {
    /// New view state.
    pub const NEW: Self = Self(1 << 0);
    /// NotNew view state.
    pub const NOT_NEW: Self = Self(1 << 1);
    /// Any view state (New or NotNew).
    pub const ANY: Self = Self((1 << 0) | (1 << 1));

    /// Checks if the view state kind is enabled in this mask.
    pub const fn contains(&self, state: ViewStateKind) -> bool {
        let bit = match state {
            ViewStateKind::New => 1 << 0,
            ViewStateKind::NotNew => 1 << 1,
        };
        (self.0 & bit) != 0
    }
}

impl From<&[ViewStateKind]> for ViewStateMask {
    fn from(states: &[ViewStateKind]) -> Self {
        let mut mask = 0;
        for &state in states {
            mask |= match state {
                ViewStateKind::New => 1 << 0,
                ViewStateKind::NotNew => 1 << 1,
            };
        }
        Self(mask)
    }
}

impl From<ViewStateKind> for ViewStateMask {
    fn from(state: ViewStateKind) -> Self {
        Self(match state {
            ViewStateKind::New => 1 << 0,
            ViewStateKind::NotNew => 1 << 1,
        })
    }
}

/// A mask representation of [`InstanceStateKind`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InstanceStateMask(u8);

impl InstanceStateMask {
    /// Alive instance state.
    pub const ALIVE: Self = Self(1 << 0);
    /// NotAliveDisposed instance state.
    pub const NOT_ALIVE_DISPOSED: Self = Self(1 << 1);
    /// NotAliveNoWriters instance state.
    pub const NOT_ALIVE_NO_WRITERS: Self = Self(1 << 2);
    /// Any instance state.
    pub const ANY: Self = Self((1 << 0) | (1 << 1) | (1 << 2));
    /// Any not alive instance state (NotAliveDisposed or NotAliveNoWriters).
    pub const NOT_ALIVE: Self = Self((1 << 1) | (1 << 2));

    /// Checks if the instance state kind is enabled in this mask.
    pub const fn contains(&self, state: InstanceStateKind) -> bool {
        let bit = match state {
            InstanceStateKind::Alive => 1 << 0,
            InstanceStateKind::NotAliveDisposed => 1 << 1,
            InstanceStateKind::NotAliveNoWriters => 1 << 2,
        };
        (self.0 & bit) != 0
    }
}

impl From<&[InstanceStateKind]> for InstanceStateMask {
    fn from(states: &[InstanceStateKind]) -> Self {
        let mut mask = 0;
        for &state in states {
            mask |= match state {
                InstanceStateKind::Alive => 1 << 0,
                InstanceStateKind::NotAliveDisposed => 1 << 1,
                InstanceStateKind::NotAliveNoWriters => 1 << 2,
            };
        }
        Self(mask)
    }
}

impl From<InstanceStateKind> for InstanceStateMask {
    fn from(state: InstanceStateKind) -> Self {
        Self(match state {
            InstanceStateKind::Alive => 1 << 0,
            InstanceStateKind::NotAliveDisposed => 1 << 1,
            InstanceStateKind::NotAliveNoWriters => 1 << 2,
        })
    }
}
