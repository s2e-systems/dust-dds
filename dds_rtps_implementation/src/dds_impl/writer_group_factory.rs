use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::PublisherQos,
    publication::publisher_listener::PublisherListener, return_type::DDSResult,
};

use crate::rtps_impl::rtps_writer_group_impl::RTPSWriterGroupImpl;

const ENTITYKIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;

pub struct WriterGroupFactory<'a, PSM: rust_rtps_pim::PIM> {
    guid_prefix: PSM::GuidPrefix,
    publisher_counter: u8,
    default_publisher_qos: PublisherQos<'a>,
}

impl<'a, PSM: rust_rtps_pim::PIM> WriterGroupFactory<'a, PSM> {
    pub fn new(guid_prefix: PSM::GuidPrefix) -> Self {
        Self {
            guid_prefix,
            publisher_counter: 0,
            default_publisher_qos: PublisherQos::default(),
        }
    }

    pub fn create_writer_group(
        &mut self,
        _qos: Option<PublisherQos<'a>>,
        _a_listener: Option<&'a (dyn PublisherListener + 'a)>,
        _mask: StatusMask,
    ) -> DDSResult<RTPSWriterGroupImpl<'a, PSM>> {
        todo!()
        // let guid_prefix = self.guid_prefix.clone();

        // self.publisher_counter += 1;
        // let entity_id = [
        //     self.publisher_counter,
        //     0,
        //     0,
        //     ENTITYKIND_USER_DEFINED_WRITER_GROUP,
        // ]
        // .into();
        // let guid = GUID::new(guid_prefix, entity_id);
        // let group = Arc::new(Mutex::new(RTPSWriterGroupImpl::new(
        //     guid,
        //     qos,
        //     listener,
        //     status_mask,
        // )));
        // self.rtps_writer_groups.push(group.clone());
        // group
    }

    pub fn set_default_publisher_qos(&mut self, _default_publisher_qos: PublisherQos<'a>) {
        todo!()
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos<'a> {
        todo!()
    }
}
