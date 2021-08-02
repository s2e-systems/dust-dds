use std::io::{Read, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::{
    behavior::types::Duration,
    structure::types::{EntityId, EntityKind, Guid},
};

use crate::{serialize::NumberofBytes, submessage_elements::{EntityIdUdp, GuidPrefixUdp, u8_into_entity_kind}};

impl crate::serialize::Serialize for bool {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        if *self {
            1_u8.serialize::<_, B>(&mut writer)?;
        }
        else {
            0_u8.serialize::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for bool {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let v: u8 = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        if v == 0 {
            Ok(false)
        }
        else {
            Ok(true)
        }
    }
}
impl NumberofBytes for bool {
    fn number_of_bytes(&self) -> usize {
        1
    }
}

#[derive(Debug, PartialEq)]
pub struct GuidUdp {
    prefix: GuidPrefixUdp,
    entity_id: EntityIdUdp,
}

impl GuidUdp {
    pub fn prefix(&self) -> &GuidPrefixUdp {
        &self.prefix
    }
}
impl From<GuidUdp> for Guid {
    fn from(v: GuidUdp) -> Self {
        Guid::new(v.prefix.0, v.entity_id.into())
    }
}
impl From<Guid> for GuidUdp {
    fn from(v: Guid) -> Self {
        Self{ prefix: (*v.prefix()).into(), entity_id: (*v.entity_id()).into() }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DurationUdp {
    pub seconds: i32,
    pub fraction: u32,
}
impl From<DurationUdp> for Duration {
    fn from(v: DurationUdp) -> Self {
        Duration::new(v.seconds, v.fraction)
    }
}
impl From<Duration> for DurationUdp {
    fn from(v: Duration) -> Self {
        Self{ seconds: *v.seconds(), fraction: *v.fraction() }
    }
}
impl crate::serialize::Serialize for DurationUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.seconds.serialize::<_, B>(&mut writer)?;
        self.fraction.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for DurationUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let seconds = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let fraction = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self{seconds, fraction})
    }
}
impl NumberofBytes for DurationUdp {
    fn number_of_bytes(&self) -> usize {
        8
    }
}


impl crate::serialize::Serialize for EntityKind {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        crate::submessage_elements::entity_kind_into_u8(self).serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for EntityKind {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let entity_kind: u8 = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(u8_into_entity_kind(entity_kind))
    }
}

impl crate::serialize::Serialize for EntityId {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.entity_key().serialize::<_, B>(&mut writer)?;
        self.entity_kind().serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for EntityId {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let entity_key = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let entity_kind = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self::new(
            entity_key,
            entity_kind,
        ))
    }
}

impl crate::serialize::Serialize for GuidUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.prefix.serialize::<_, B>(&mut writer)?;
        self.entity_id.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for GuidUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let prefix = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let entity_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self { prefix, entity_id })
    }
}
impl NumberofBytes for GuidUdp {
    fn number_of_bytes(&self) -> usize {
        16
    }
}

impl crate::serialize::Serialize for Duration {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.seconds.serialize::<_, B>(&mut writer)?;
        self.fraction.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for Duration {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let seconds = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let fraction = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self { seconds, fraction })
    }
}


impl crate::serialize::Serialize for String {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        let length = self.len() as u32 + 1;
        length.serialize::<_, B>(&mut writer)?;
        self.as_bytes().serialize::<_, B>(&mut writer)?;
        0u8.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for String {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let length: u32 = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let mut string_buf = vec![0_u8; length as usize];
        buf.read_exact(&mut string_buf[..])?;
        string_buf.pop();
        String::from_utf8(string_buf).map_err(|_err| std::io::ErrorKind::Other.into())
    }
}
impl NumberofBytes for String {
    fn number_of_bytes(&self) -> usize {
        4 + self.len() + 1
    }
}