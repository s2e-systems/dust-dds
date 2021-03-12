use core::ops::{Deref, DerefMut};

use crate::behavior::RTPSReader;

pub trait RTPSStatelessReader<'a, T: RTPSReader<'a>>: Deref<Target = T> + DerefMut {}
