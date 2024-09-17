#[cfg(test)]
mod tests {
    use crate::{
        topic_definition::type_support::TypeSupport,
        xtypes::{
            deserializer::XTypesDeserializer,
            dynamic_type::DynamicType,
            error::XTypesError,
            serialize::{Write, XTypesSerialize},
            type_object,
            xcdr_deserializer::Xcdr1LeDeserializer,
            xcdr_serializer::{Xcdr1LeSerializer, Xcdr2BeSerializer},
        },
    };
    use dust_dds_derive::TypeSupport;

    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL) @nested
    struct Nested {
        #[xtypes(key)]
        x: u8,
        #[xtypes(key)]
        y: u8,
    }
    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL)
    struct Complex {
        field1: i64,
        #[xtypes(key)]
        key_field1: u16,
        field2: u32,
        #[xtypes(key)]
        key_field2: Nested,
    }

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

    fn push_to_key(
        dynamic_type: &dyn DynamicType,
        serializer: &mut Xcdr2BeSerializer<'_, Md5>,
        de: &mut Xcdr1LeDeserializer<'_>,
    ) -> Result<(), XTypesError> {
        for id in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(id)?;
            let is_key_field = member.get_descriptor()?.is_key;
            match member.get_descriptor()?.type_.get_kind() {
                type_object::TK_INT64 => {
                    let v = de.deserialize_int64()?;
                    if is_key_field {
                        v.serialize(&mut *serializer)?;
                    }
                }
                type_object::TK_UINT16 => {
                    let v = de.deserialize_uint16()?;
                    if is_key_field {
                        v.serialize(&mut *serializer)?;
                    }
                }
                type_object::TK_UINT32 => {
                    let v = de.deserialize_uint32()?;
                    if is_key_field {
                        v.serialize(&mut *serializer)?;
                    }
                }
                type_object::TK_UINT8 => {
                    let v = de.deserialize_uint8()?;
                    if is_key_field {
                        v.serialize(&mut *serializer)?;
                    }
                }
                type_object::TK_STRUCTURE => {
                    push_to_key(member.get_descriptor()?.type_, serializer, de)?;
                }
                _ => panic!(),
            }
        }
        Ok(())
    }

    fn get_key_from_serialized_type<T: TypeSupport>(data: &[u8]) -> Result<[u8; 16], XTypesError> {
        let dynamic_type = T::get_type();

        let mut md5_collection = Md5 {
            key: [0; 16],
            context: md5::Context::new(),
            length: 0,
        };
        let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
        let mut de: Xcdr1LeDeserializer<'_> = Xcdr1LeDeserializer::new(data);

        push_to_key(&dynamic_type, &mut serializer, &mut de)?;
        Ok(md5_collection.into_key())
    }

    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL)
    struct Simple {
        #[xtypes(key)]
        key_field1: i64,
        #[xtypes(key)]
        key_field2: i64,
    }

    #[test]
    fn simple_key() {
        let v = Simple {
            key_field1: 1,
            key_field2: 2,
        };
        let data = [
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        let mut collection = Vec::new();
        v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
            .unwrap();
        assert_eq!(&data, collection.as_slice());
        let key_data = [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2];
        assert_eq!(
            get_key_from_serialized_type::<Simple>(&data).unwrap(),
            key_data.as_slice()
        );
    }

    #[test]
    fn key() {
        let v = Complex {
            field1: 2,
            key_field1: 3,
            field2: 4,
            key_field2: Nested { x: 5, y: 6 },
        };
        let data = [
            2, 0, 0, 0, 0, 0, 0, 0, //field1 (i64)
            3, 0, 0, 0, //key_field1 (u16) | padding (2B)
            4, 0, 0, 0, //field2 (u32)
            5, 6, //key_field2 (u8, u8)
        ];
        let mut collection = Vec::new();
        v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
            .unwrap();
        assert_eq!(&data, collection.as_slice());
        let key_data = [0, 3, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            get_key_from_serialized_type::<Complex>(&data).unwrap(),
            key_data.as_slice()
        );
    }

    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL)
    struct Large {
        #[xtypes(key)]
        key_field1: i64,
        #[xtypes(key)]
        key_field2: i64,
        #[xtypes(key)]
        key_field3: i64,
    }

    #[test]
    fn large_key() {
        let v = Large {
            key_field1: 1,
            key_field2: 2,
            key_field3: 3,
        };
        let data = [
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            3, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        let mut collection = Vec::new();
        v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
            .unwrap();
        assert_eq!(&data, collection.as_slice());
        let key_data = [
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3,
        ];
        assert_eq!(
            get_key_from_serialized_type::<Large>(&data).unwrap(),
            md5::compute(key_data).as_slice()
        );
    }
}
