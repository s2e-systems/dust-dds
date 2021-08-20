use rust_rtps_pim::messages::RtpsMessage;

use crate::{
    deserialize::{Deserialize, Result},
    serialize::Serialize,
};

impl<M> Serialize for RtpsMessage<M>
where
    for<'a> &'a M: IntoIterator,
{
    fn serialize<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        self.header.serialize::<_, B>(&mut writer)?;
        for submessage in &self.submessages {
            todo!()
        }
        Ok(())
    }
}

impl<'de, M> Deserialize<'de> for RtpsMessage<M> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: byteorder::ByteOrder,
    {
        todo!()
        // let header = crate::deserialize::Deserialize::deserialize(&mut buf)?;
        // let submessages = ();
        // Self {
        //     header,
        //     submessages,
        // };
    }
}
