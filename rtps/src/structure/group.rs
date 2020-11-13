use std::sync::{Arc, Mutex};

use crate::types::GUID;
use crate::structure::{RtpsEndpoint, RtpsEntity, };

pub struct RtpsGroup {
    guid: GUID,
    endpoint_counter: usize,
    endpoints: Vec<Arc<dyn RtpsEndpoint>>,
}

impl RtpsGroup {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
            endpoint_counter: 0,
            endpoints: Vec::new(),
        }
    }

    pub fn mut_endpoints(&mut self) -> &mut Vec<Arc<dyn RtpsEndpoint>> {
        &mut self.endpoints
    }

    pub fn endpoints(&self) -> &[Arc<dyn RtpsEndpoint>] {
        self.endpoints.as_slice()
    }
}

impl<'a> IntoIterator for &'a RtpsGroup {
    type Item = &'a Arc<dyn RtpsEndpoint>;
    type IntoIter = std::slice::Iter<'a, Arc<dyn RtpsEndpoint>>;
    fn into_iter(self) -> Self::IntoIter {
        self.endpoints.iter()
    }
}

impl RtpsEntity for RtpsGroup {
    fn guid(&self) -> GUID {
        self.guid
    }
}