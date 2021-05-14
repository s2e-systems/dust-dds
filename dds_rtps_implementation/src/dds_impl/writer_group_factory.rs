use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::PublisherQos,
    publication::publisher_listener::PublisherListener, return_type::DDSResult,
};
use rust_rtps_pim::structure::types::GUID;

use crate::rtps_impl::rtps_writer_group_impl::RTPSWriterGroupImpl;

const ENTITYKIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;

pub struct WriterGroupFactory<PSM: rust_rtps_pim::PIM> {
    guid_prefix: PSM::GuidPrefix,
    publisher_counter: u8,
}

impl<PSM: rust_rtps_pim::PIM> WriterGroupFactory<PSM> {
    pub fn new(guid_prefix: PSM::GuidPrefix) -> Self {
        Self {
            guid_prefix,
            publisher_counter: 0,
        }
    }

    pub fn create_writer_group<'a>(
        &mut self,
        qos: PublisherQos<'a>,
        a_listener: Option<&'a (dyn PublisherListener + 'a)>,
        mask: StatusMask,
    ) -> DDSResult<RTPSWriterGroupImpl<'a, PSM>> {
        let guid_prefix = self.guid_prefix.clone();

        self.publisher_counter += 1;
        let entity_id = [
            self.publisher_counter,
            0,
            0,
            ENTITYKIND_USER_DEFINED_WRITER_GROUP,
        ]
        .into();
        let guid = GUID::new(guid_prefix, entity_id);
        Ok(RTPSWriterGroupImpl::new(guid, qos, a_listener, mask))
    }
}
