use crate::{PIM};

use super::RTPSEntity;

pub trait RTPSGroup<PSM: PIM>: RTPSEntity<PSM> {}
