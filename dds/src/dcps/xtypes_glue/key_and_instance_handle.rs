use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        dynamic_type::{DynamicData, TypeKind},
        error::XTypesError,
        read_write::Write,
        serializer::Cdr2BeSerializer,
    },
};

pub fn get_instance_handle_from_dynamic_data(
    mut dynamic_data: DynamicData,
) -> Result<InstanceHandle, XTypesError> {
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

    let md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    let key = if dynamic_data.type_ref().get_kind() == TypeKind::STRUCTURE {
        dynamic_data.clear_nonkey_values()?;
        dynamic_data.make_descriptor_extensibility_kind_final();
        Cdr2BeSerializer::serialize_without_header(md5_collection, &dynamic_data)?.into_key()
    } else {
        [0; 16]
    };
    Ok(InstanceHandle::new(key))
}
