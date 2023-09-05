use crate::{
    infrastructure::{instance::InstanceHandle, time::Time},
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
};

use super::{
    messages::submessage_elements::{Data, ParameterList},
    types::{ChangeKind, Guid},
};

pub struct RtpsReaderCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub data: Data,
    pub inline_qos: ParameterList,
    pub source_timestamp: Option<Time>,
    pub sample_state: SampleStateKind,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub reception_timestamp: Time,
}

pub struct Instance {
    pub view_state: ViewStateKind,
    pub instance_state: InstanceStateKind,
    pub most_recent_disposed_generation_count: i32,
    pub most_recent_no_writers_generation_count: i32,
}

impl Instance {
    pub fn new() -> Self {
        Self {
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    pub fn update_state(&mut self, change_kind: ChangeKind) {
        match self.instance_state {
            InstanceStateKind::Alive => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveDisposedUnregistered
                {
                    self.instance_state = InstanceStateKind::NotAliveDisposed;
                } else if change_kind == ChangeKind::NotAliveUnregistered {
                    self.instance_state = InstanceStateKind::NotAliveNoWriters;
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_disposed_generation_count += 1;
                }
            }
            InstanceStateKind::NotAliveNoWriters => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_no_writers_generation_count += 1;
                }
            }
        }

        match self.view_state {
            ViewStateKind::New => (),
            ViewStateKind::NotNew => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveUnregistered
                {
                    self.view_state = ViewStateKind::New;
                }
            }
        }
    }

    pub fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }
}
