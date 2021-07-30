use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    behavior::types::Duration,
    structure::types::{EntityId, EntityKind, GUID},
};

use crate::submessage_elements::u8_into_entity_kind;

impl crate::serialize::Serialize for EntityKind {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        crate::submessage_elements::entity_kind_into_u8(*self).serialize::<_, B>(&mut writer)
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
        self.entity_key.serialize::<_, B>(&mut writer)?;
        self.entity_kind.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for EntityId {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let entity_key = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let entity_kind = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self {
            entity_key,
            entity_kind,
        })
    }
}

impl crate::serialize::Serialize for GUID {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.prefix().serialize::<_, B>(&mut writer)?;
        self.entity_id().serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for GUID {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let prefix = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let entity_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self { prefix, entity_id })
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
