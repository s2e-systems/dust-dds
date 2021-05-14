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

    pub fn create_writer_group(
        &mut self,
        qos: PublisherQos,
        a_listener: Option<&'static dyn PublisherListener>,
        mask: StatusMask,
    ) -> DDSResult<RTPSWriterGroupImpl<PSM>> {
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

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::RTPSEntity;
    use rust_rtps_udp_psm::RtpsUdpPsm;

    use super::*;

    #[test]
    fn basic_create_writer_group() {
        let guid_prefix = [1; 12];
        let mut writer_group_factory: WriterGroupFactory<RtpsUdpPsm> =
            WriterGroupFactory::new(guid_prefix);

        writer_group_factory
            .create_writer_group(PublisherQos::default(), None, 0)
            .unwrap();
        assert_eq!(writer_group_factory.publisher_counter, 1);
    }

    #[test]
    fn create_multiple_writer_groups() {
        let guid_prefix = [1; 12];
        let mut writer_group_factory: WriterGroupFactory<RtpsUdpPsm> =
            WriterGroupFactory::new(guid_prefix);

        let writer_group1 = writer_group_factory
            .create_writer_group(PublisherQos::default(), None, 0)
            .unwrap();
        let writer_group2 = writer_group_factory
            .create_writer_group(PublisherQos::default(), None, 0)
            .unwrap();

        assert!(writer_group1.guid() != writer_group2.guid());
    }
}
