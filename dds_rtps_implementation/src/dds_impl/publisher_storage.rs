use rust_dds_api::infrastructure::qos::PublisherQos;
use rust_rtps_pim::{
    messages::{
        submessages::{DataSubmessage, RtpsSubmessagePIM, RtpsSubmessageType},
        RTPSMessage,
    },
    structure::{RTPSEntity, RTPSParticipant},
};

use crate::utils::{
    message_sender::send_data, shared_object::RtpsShared, transport::TransportWrite,
};

use super::data_writer_storage::{self, DataWriterStorage};

pub struct PublisherStorage {
    qos: PublisherQos,
    data_writer_storage_list: Vec<RtpsShared<DataWriterStorage>>,
}

impl PublisherStorage {
    pub fn new(
        qos: PublisherQos,
        data_writer_storage_list: Vec<RtpsShared<DataWriterStorage>>,
    ) -> Self {
        Self {
            qos,
            data_writer_storage_list,
        }
    }

    /// Get a reference to the publisher storage's data writer storage list.
    pub fn data_writer_storage_list(&self) -> &[RtpsShared<DataWriterStorage>] {
        self.data_writer_storage_list.as_slice()
    }
}
