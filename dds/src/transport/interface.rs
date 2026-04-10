use crate::{
    dcps::dcps_mail::{DcpsMail, MessageServiceMail},
    dds_async::domain_participant_factory::DcpsSender,
    infrastructure::instance::InstanceHandle,
    transport::types::Locator,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};

pub trait WriteMessage {
    fn write_message(&self, buf: &[u8], locators: &[Locator]);
}

#[derive(Clone)]
pub struct TransportDataReceiver {
    participant_handle: InstanceHandle,
    dcps_sender: DcpsSender,
}
impl TransportDataReceiver {
    pub(crate) fn new(participant_handle: InstanceHandle, dcps_sender: DcpsSender) -> Self {
        Self {
            participant_handle,
            dcps_sender,
        }
    }

    pub async fn receive_message(&self, data_message: Arc<[u8]>) {
        self.dcps_sender
            .send(DcpsMail::Message(MessageServiceMail::HandleData {
                participant_handle: self.participant_handle,
                data_message,
            }))
            .await;
    }
}

pub struct RtpsTransportParticipant {
    pub message_writer: Box<dyn WriteMessage + Send + Sync>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub fragment_size: usize,
}
pub trait TransportParticipantFactory: Send + 'static {
    fn create_participant(
        &self,
        domain_id: i32,
        data_receiver: TransportDataReceiver,
    ) -> RtpsTransportParticipant;
}
