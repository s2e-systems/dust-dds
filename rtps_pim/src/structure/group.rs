use crate::structure;

use super::RTPSEntity;

pub trait RTPSGroup<PSM: structure::Types>: RTPSEntity<PSM> {}