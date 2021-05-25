pub mod node;
pub mod shared_object;
// pub mod endpoint_traits;
pub mod mask_listener;
// pub mod message_sender;

use rust_dds_api::dcps_psm::InstanceHandle;
use rust_rtps_pim::structure::types::GUID;

pub fn instance_handle_from_guid<PSM: crate::rtps_impl::PIM>(guid: &GUID<PSM>) -> InstanceHandle {
    let entity_id_bytes = guid.entity_id().clone().into();
    InstanceHandle::from_le_bytes(entity_id_bytes)
}
