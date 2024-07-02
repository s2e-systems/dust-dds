use pyo3::prelude::*;

use crate::infrastructure::{instance::InstanceHandle, time::Time};

#[pyclass]
#[derive(Clone)]
pub enum SampleStateKind {
    Read,
    NotRead,
}

impl From<dust_dds::subscription::sample_info::SampleStateKind> for SampleStateKind {
    fn from(value: dust_dds::subscription::sample_info::SampleStateKind) -> Self {
        match value {
            dust_dds::subscription::sample_info::SampleStateKind::Read => SampleStateKind::Read,
            dust_dds::subscription::sample_info::SampleStateKind::NotRead => {
                SampleStateKind::NotRead
            }
        }
    }
}

impl From<SampleStateKind> for dust_dds::subscription::sample_info::SampleStateKind {
    fn from(value: SampleStateKind) -> Self {
        match value {
            SampleStateKind::Read => dust_dds::subscription::sample_info::SampleStateKind::Read,
            SampleStateKind::NotRead => {
                dust_dds::subscription::sample_info::SampleStateKind::NotRead
            }
        }
    }
}

pub const ANY_SAMPLE_STATE: &[SampleStateKind] = &[SampleStateKind::Read, SampleStateKind::NotRead];

#[pyclass]
#[derive(Clone)]
pub enum ViewStateKind {
    New,
    NotNew,
}

impl From<dust_dds::subscription::sample_info::ViewStateKind> for ViewStateKind {
    fn from(value: dust_dds::subscription::sample_info::ViewStateKind) -> Self {
        match value {
            dust_dds::subscription::sample_info::ViewStateKind::New => ViewStateKind::New,
            dust_dds::subscription::sample_info::ViewStateKind::NotNew => ViewStateKind::NotNew,
        }
    }
}

impl From<ViewStateKind> for dust_dds::subscription::sample_info::ViewStateKind {
    fn from(value: ViewStateKind) -> Self {
        match value {
            ViewStateKind::New => dust_dds::subscription::sample_info::ViewStateKind::New,
            ViewStateKind::NotNew => dust_dds::subscription::sample_info::ViewStateKind::NotNew,
        }
    }
}

pub const ANY_VIEW_STATE: &[ViewStateKind] = &[ViewStateKind::New, ViewStateKind::NotNew];

#[pyclass]
#[derive(Clone)]
pub enum InstanceStateKind {
    Alive,
    NotAliveDisposed,
    NotAliveNoWriters,
}

impl From<dust_dds::subscription::sample_info::InstanceStateKind> for InstanceStateKind {
    fn from(value: dust_dds::subscription::sample_info::InstanceStateKind) -> Self {
        match value {
            dust_dds::subscription::sample_info::InstanceStateKind::Alive => {
                InstanceStateKind::Alive
            }
            dust_dds::subscription::sample_info::InstanceStateKind::NotAliveDisposed => {
                InstanceStateKind::NotAliveDisposed
            }
            dust_dds::subscription::sample_info::InstanceStateKind::NotAliveNoWriters => {
                InstanceStateKind::NotAliveNoWriters
            }
        }
    }
}

impl From<InstanceStateKind> for dust_dds::subscription::sample_info::InstanceStateKind {
    fn from(value: InstanceStateKind) -> Self {
        match value {
            InstanceStateKind::Alive => {
                dust_dds::subscription::sample_info::InstanceStateKind::Alive
            }
            InstanceStateKind::NotAliveDisposed => {
                dust_dds::subscription::sample_info::InstanceStateKind::NotAliveDisposed
            }
            InstanceStateKind::NotAliveNoWriters => {
                dust_dds::subscription::sample_info::InstanceStateKind::NotAliveNoWriters
            }
        }
    }
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

#[pyclass]
#[derive(Clone)]
pub struct SampleInfo(dust_dds::subscription::sample_info::SampleInfo);

impl From<dust_dds::subscription::sample_info::SampleInfo> for SampleInfo {
    fn from(value: dust_dds::subscription::sample_info::SampleInfo) -> Self {
        Self(value)
    }
}

#[pymethods]
impl SampleInfo {
    fn get_sample_state(&self) -> SampleStateKind {
        self.0.sample_state.into()
    }

    fn get_view_state(&self) -> ViewStateKind {
        self.0.view_state.into()
    }

    fn get_instance_state(&self) -> InstanceStateKind {
        self.0.instance_state.into()
    }

    fn get_disposed_generation_count(&self) -> i32 {
        self.0.disposed_generation_count
    }

    fn get_no_writers_generation_count(&self) -> i32 {
        self.0.no_writers_generation_count
    }

    fn get_sample_rank(&self) -> i32 {
        self.0.sample_rank
    }

    fn get_generation_rank(&self) -> i32 {
        self.0.generation_rank
    }

    fn get_absolute_generation_rank(&self) -> i32 {
        self.0.absolute_generation_rank
    }

    fn get_source_timestamp(&self) -> Option<Time> {
        self.0.source_timestamp.map(Time::from)
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.0.instance_handle.into()
    }

    fn get_publication_handle(&self) -> InstanceHandle {
        self.0.publication_handle.into()
    }

    fn get_valid_data(&self) -> bool {
        self.0.valid_data
    }
}
