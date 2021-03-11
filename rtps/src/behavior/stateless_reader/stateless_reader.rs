use core::ops::{Deref, DerefMut};

use crate::behavior::RTPSReader;

pub trait StatelessReader<T: RTPSReader>: Deref<Target = T> + DerefMut {}
