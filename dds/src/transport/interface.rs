use crate::{
    dcps::dcps_mail::WireMail,
    dds_async::domain_participant_factory::WireSender,
    infrastructure::instance::InstanceHandle,
    transport::types::{Guid, Locator},
};

pub trait Write {
    fn write(&mut self, buf: &[u8]);
    fn flush(&mut self);
}

pub trait RtpsParticipant {
    fn default_unicast_locator_list(&self, guid: Guid) -> &[Locator];
    fn metatraffic_unicast_locator_list(&self, guid: Guid) -> &[Locator];
    fn metatraffic_multicast_locator_list(&self, guid: Guid) -> &[Locator];
    fn default_multicast_locator_list(&self, guid: Guid) -> &[Locator];
    fn fragment_size(&self, guid: Guid) -> usize;
}
pub trait Transport: Send + 'static {
    fn create_participant(&mut self, domain_id: i32) -> Guid;

    fn delete_participant(&mut self, guid: Guid);

    fn writer(&mut self, guid: Guid, locator: &[Locator]) -> impl Write + '_;

    fn read(&mut self) -> impl Future<Output = &[u8]>;
}
