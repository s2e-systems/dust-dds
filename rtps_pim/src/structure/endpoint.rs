use crate::types::Locator;

use super::RTPSEntity;

pub trait RTPSEndpoint: RTPSEntity {
    type Locator: Locator;
}
