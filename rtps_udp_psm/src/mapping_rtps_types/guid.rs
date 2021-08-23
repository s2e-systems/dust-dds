use std::io::{Read, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::structure::types::GuidPrefix;

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};