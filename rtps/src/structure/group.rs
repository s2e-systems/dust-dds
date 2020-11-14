use std::sync::Arc;

use crate::types::GUID;
use crate::structure::{RtpsEndpoint, RtpsEntity, };

pub struct RtpsGroup {
    guid: GUID,
    endpoints: Vec<Arc<dyn RtpsEndpoint>>,
}

impl RtpsGroup {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
            endpoints: Vec::new(),
        }
    }
}

impl RtpsEntity for RtpsGroup {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl std::ops::Deref for RtpsGroup {
    type Target = Vec<Arc<dyn RtpsEndpoint>>;

    fn deref(&self) -> &Self::Target {
        &self.endpoints
    }
}

impl std::ops::DerefMut for RtpsGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.endpoints
    }
}