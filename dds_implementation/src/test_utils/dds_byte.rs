use crate::dds_type::{DdsDeserialize, DdsSerialize, DdsType};
use dds_api::return_type::DdsError;

pub struct DdsByte(pub u8);

impl DdsType for DdsByte {
    fn type_name() -> &'static str {
        "DdsByte"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerialize for DdsByte {
    fn serialize<W: std::io::Write, E: crate::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> dds_api::return_type::DdsResult<()> {
        writer
            .write(&[self.0])
            .map(|_| ())
            .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
    }
}

impl<'de> DdsDeserialize<'de> for DdsByte {
    fn deserialize(buf: &mut &'de [u8]) -> dds_api::return_type::DdsResult<Self> {
        Ok(DdsByte(buf[0]))
    }
}
