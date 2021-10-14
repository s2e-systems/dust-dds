use core::ops::{Deref, DerefMut};

use super::reader::RtpsReader;

pub struct RtpsStatelessReader<L, C>(pub RtpsReader<L, C>);

impl<L, C> Deref for RtpsStatelessReader<L, C> {
    type Target = RtpsReader<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L, C> DerefMut for RtpsStatelessReader<L, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
