use core::ops::{Deref, DerefMut};

use crate::behavior::RTPSReader;

pub trait RTPSStatelessReader<T: RTPSReader>: Deref<Target = T> + DerefMut {}
