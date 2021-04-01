use core::ops::{Deref, DerefMut};

use crate::{
    behavior::{self,RTPSWriter},
    structure::{self, RTPSHistoryCache},
};

pub trait RTPSStatelessWriter:
    Deref<Target = RTPSWriter<Self::PSM, Self::HistoryCache>> + DerefMut
{
    type PSM: structure::Types + behavior::Types;
    type HistoryCache: RTPSHistoryCache;

    fn reader_locator_add(&mut self, a_locator: <Self::PSM as structure::Types>::Locator);
    fn reader_locator_remove(&mut self, a_locator: <Self::PSM as structure::Types>::Locator);
    fn unsent_changes_reset(&mut self);
}
