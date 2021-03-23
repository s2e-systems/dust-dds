use crate::types::{Locator, ProtocolVersion, VendorId};

use super::RTPSEntity;

pub trait RTPSParticipant: RTPSEntity {
    type Locator: Locator;
    type ProtocolVersion: ProtocolVersion;
    type VendorId: VendorId;
}
