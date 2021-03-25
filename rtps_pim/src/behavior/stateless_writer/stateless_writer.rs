use core::ops::{Deref, DerefMut};

use crate::behavior::RTPSWriter;
use crate::types;

pub trait RTPSStatelessWriter<T: RTPSWriter>: Deref<Target = T> + DerefMut {
    fn new(writer: T) -> Self;
    fn reader_locator_add(&mut self, a_locator: impl types::Locator);
    fn reader_locator_remove(&mut self, a_locator: &impl types::Locator);
    fn unsent_changes_reset(&mut self);
}
