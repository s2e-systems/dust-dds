use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        dynamic_type::DynamicData,
        error::XTypesError,
        serializer::{BigEndian, LittleEndian, Write, XTypesSerializer},
        xcdr_serializer::{Xcdr1Serializer, Xcdr2Serializer},
    },
};
use alloc::vec::Vec;

struct Md5 {
    key: [u8; 16],
    context: md5::Context,
    length: usize,
}

impl Md5 {
    fn into_key(mut self) -> [u8; 16] {
        const ZEROS: [u8; 16] = [0; 16];
        if self.length < ZEROS.len() {
            self.context.consume(&ZEROS[self.length..]);
        }
        if self.length <= 16 {
            self.key
        } else {
            self.context.compute().into()
        }
    }
}

impl Write for Md5 {
    fn write(&mut self, buf: &[u8]) {
        let total_new_length = self.length + buf.len();
        if total_new_length <= self.key.len() {
            self.key[self.length..total_new_length].copy_from_slice(buf);
        }
        self.context.consume(buf);
        self.length += buf.len();
    }
}

pub fn get_instance_handle_from_dynamic_data(
    mut dynamic_data: DynamicData,
) -> Result<InstanceHandle, XTypesError> {
    let md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    dynamic_data.clear_nonkey_values()?;
    dynamic_data.make_descriptor_extensibility_kind_final();

    let serializer = dynamic_data.serialize(Xcdr2Serializer::new(md5_collection, BigEndian))?;
    Ok(InstanceHandle::new(serializer.into_inner().into_key()))
}

pub fn get_serialized_key_from_dynamic_data(
    mut dynamic_data: DynamicData,
) -> Result<Vec<u8>, XTypesError> {
    type RepresentationIdentifier = [u8; 2];
    const CDR_LE: RepresentationIdentifier = [0x00, 0x01];

    dynamic_data.make_descriptor_extensibility_kind_final();
    dynamic_data.clear_nonkey_values()?;
    let mut serialized_key = Vec::new();
    serialized_key.extend_from_slice(&CDR_LE);
    serialized_key.extend_from_slice(&[0, 0]);
    let serializer = Xcdr1Serializer::new(Vec::new(), LittleEndian);
    let serializer = dynamic_data.serialize(serializer)?;

    let mut serialized_key = Vec::new();
    serialized_key.extend_from_slice(&CDR_LE);
    serialized_key.extend_from_slice(&[0, 0]);
    serialized_key.extend_from_slice(&serializer.into_inner());
    let padding_len = serialized_key.len().div_ceil(4) * 4 - serialized_key.len();
    const ZEROS: [u8; 4] = [0; 4];
    serialized_key.extend_from_slice(&ZEROS[..padding_len]);
    serialized_key[3] |= padding_len as u8;

    Ok(serialized_key)
}
