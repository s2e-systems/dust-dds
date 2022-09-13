use crate::{dcps_psm::StatusKind, infrastructure::entity::StatusCondition};

use super::data_writer_impl::AnyDataWriterListener;

pub struct DataWriterCommunicationStatus {
    listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
    status_condition: StatusCondition,
}

impl DataWriterCommunicationStatus {
    pub fn new(listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>) -> Self {
        Self {
            listener,
            status_condition: StatusCondition::default(),
        }
    }

    pub fn get_statuscondition(&self) -> StatusCondition {
        self.status_condition.clone()
    }

    pub fn set_listener(
        &mut self,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
    ) {
        self.listener = a_listener;
    }

    pub fn trigger_communication_status(&mut self, state: StatusKind) {
        self.status_condition.add_communication_state(state);
        // if state == PUBLICATION_MATCHED
        //  call listener methods
        // notify wait set

        // let mut listener_lock = self.listener.write_lock();
        //         if let Some(l) = listener_lock.as_mut() {
        //             let publication_matched_status = self.get_publication_matched_status().unwrap();
        //             l.trigger_on_publication_matched(self, publication_matched_status)
        //         }

        // let mut listener_lock = self.listener.write_lock();
        //         if let Some(l) = listener_lock.as_mut() {
        //             let offered_incompatible_qos_status =
        //                 self.get_offered_incompatible_qos_status().unwrap();
        //             l.trigger_on_offered_incompatible_qos(self, offered_incompatible_qos_status)
        //         }


    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger() {
        let mut communication_status = DataWriterCommunicationStatus::new(None);
        communication_status.trigger_communication_status(1);
    }
}
