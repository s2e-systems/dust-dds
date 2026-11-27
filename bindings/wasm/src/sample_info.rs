use wasm_bindgen::prelude::*;

use crate::infrastructure::instance::InstanceHandle;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleStateKind {
    Read,
    NotRead,
}

impl From<dust_dds::infrastructure::sample_info::SampleStateKind> for SampleStateKind {
    fn from(value: dust_dds::infrastructure::sample_info::SampleStateKind) -> Self {
        match value {
            dust_dds::infrastructure::sample_info::SampleStateKind::Read => SampleStateKind::Read,
            dust_dds::infrastructure::sample_info::SampleStateKind::NotRead => {
                SampleStateKind::NotRead
            }
        }
    }
}

impl From<SampleStateKind> for dust_dds::infrastructure::sample_info::SampleStateKind {
    fn from(value: SampleStateKind) -> Self {
        match value {
            SampleStateKind::Read => dust_dds::infrastructure::sample_info::SampleStateKind::Read,
            SampleStateKind::NotRead => {
                dust_dds::infrastructure::sample_info::SampleStateKind::NotRead
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewStateKind {
    New,
    NotNew,
}

impl From<dust_dds::infrastructure::sample_info::ViewStateKind> for ViewStateKind {
    fn from(value: dust_dds::infrastructure::sample_info::ViewStateKind) -> Self {
        match value {
            dust_dds::infrastructure::sample_info::ViewStateKind::New => ViewStateKind::New,
            dust_dds::infrastructure::sample_info::ViewStateKind::NotNew => ViewStateKind::NotNew,
        }
    }
}

impl From<ViewStateKind> for dust_dds::infrastructure::sample_info::ViewStateKind {
    fn from(value: ViewStateKind) -> Self {
        match value {
            ViewStateKind::New => dust_dds::infrastructure::sample_info::ViewStateKind::New,
            ViewStateKind::NotNew => dust_dds::infrastructure::sample_info::ViewStateKind::NotNew,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceStateKind {
    Alive,
    NotAliveDisposed,
    NotAliveNoWriters,
}

impl From<dust_dds::infrastructure::sample_info::InstanceStateKind> for InstanceStateKind {
    fn from(value: dust_dds::infrastructure::sample_info::InstanceStateKind) -> Self {
        match value {
            dust_dds::infrastructure::sample_info::InstanceStateKind::Alive => {
                InstanceStateKind::Alive
            }
            dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed => {
                InstanceStateKind::NotAliveDisposed
            }
            dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters => {
                InstanceStateKind::NotAliveNoWriters
            }
        }
    }
}

impl From<InstanceStateKind> for dust_dds::infrastructure::sample_info::InstanceStateKind {
    fn from(value: InstanceStateKind) -> Self {
        match value {
            InstanceStateKind::Alive => {
                dust_dds::infrastructure::sample_info::InstanceStateKind::Alive
            }
            InstanceStateKind::NotAliveDisposed => {
                dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed
            }
            InstanceStateKind::NotAliveNoWriters => {
                dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters
            }
        }
    }
}

pub const ANY_SAMPLE_STATE: &[dust_dds::infrastructure::sample_info::SampleStateKind] = &[
    dust_dds::infrastructure::sample_info::SampleStateKind::Read,
    dust_dds::infrastructure::sample_info::SampleStateKind::NotRead,
];

pub const ANY_VIEW_STATE: &[dust_dds::infrastructure::sample_info::ViewStateKind] = &[
    dust_dds::infrastructure::sample_info::ViewStateKind::New,
    dust_dds::infrastructure::sample_info::ViewStateKind::NotNew,
];

pub const ANY_INSTANCE_STATE: &[dust_dds::infrastructure::sample_info::InstanceStateKind] = &[
    dust_dds::infrastructure::sample_info::InstanceStateKind::Alive,
    dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed,
    dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters,
];

#[wasm_bindgen]
#[derive(Clone)]
pub struct SampleInfo(pub(crate) dust_dds::infrastructure::sample_info::SampleInfo);

impl From<dust_dds::infrastructure::sample_info::SampleInfo> for SampleInfo {
    fn from(value: dust_dds::infrastructure::sample_info::SampleInfo) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl SampleInfo {
    #[wasm_bindgen(js_name = getSampleState)]
    pub fn get_sample_state(&self) -> SampleStateKind {
        self.0.sample_state.into()
    }

    #[wasm_bindgen(js_name = getViewState)]
    pub fn get_view_state(&self) -> ViewStateKind {
        self.0.view_state.into()
    }

    #[wasm_bindgen(js_name = getInstanceState)]
    pub fn get_instance_state(&self) -> InstanceStateKind {
        self.0.instance_state.into()
    }

    #[wasm_bindgen(js_name = getDisposedGenerationCount)]
    pub fn get_disposed_generation_count(&self) -> i32 {
        self.0.disposed_generation_count
    }

    #[wasm_bindgen(js_name = getNoWritersGenerationCount)]
    pub fn get_no_writers_generation_count(&self) -> i32 {
        self.0.no_writers_generation_count
    }

    #[wasm_bindgen(js_name = getSampleRank)]
    pub fn get_sample_rank(&self) -> i32 {
        self.0.sample_rank
    }

    #[wasm_bindgen(js_name = getGenerationRank)]
    pub fn get_generation_rank(&self) -> i32 {
        self.0.generation_rank
    }

    #[wasm_bindgen(js_name = getAbsoluteGenerationRank)]
    pub fn get_absolute_generation_rank(&self) -> i32 {
        self.0.absolute_generation_rank
    }

    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.0.instance_handle.into()
    }

    #[wasm_bindgen(js_name = getPublicationHandle)]
    pub fn get_publication_handle(&self) -> InstanceHandle {
        self.0.publication_handle.into()
    }

    #[wasm_bindgen(js_name = getValidData)]
    pub fn get_valid_data(&self) -> bool {
        self.0.valid_data
    }
}
