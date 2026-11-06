use crate::result::{Result, into_js_error};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeKind {
    None = 0x00,
    Boolean = 0x01,
    Byte = 0x02,
    Int16 = 0x03,
    Int32 = 0x04,
    Int64 = 0x05,
    Uint16 = 0x06,
    Uint32 = 0x07,
    Uint64 = 0x08,
    Float32 = 0x09,
    Float64 = 0x0A,
    Float128 = 0x0B,
    Int8 = 0x0C,
    Uint8 = 0x0D,
    Char8 = 0x10,
    Char16 = 0x11,
    String8 = 0x20,
    String16 = 0x21,
    Alias = 0x30,
    Enum = 0x40,
    Bitmask = 0x41,
    Annotation = 0x50,
    Structure = 0x51,
    Union = 0x52,
    Bitset = 0x53,
    Sequence = 0x60,
    Array = 0x61,
    Map = 0x62,
}

impl From<TypeKind> for dust_dds::xtypes::dynamic_type::TypeKind {
    fn from(kind: TypeKind) -> Self {
        match kind {
            TypeKind::None => dust_dds::xtypes::dynamic_type::TypeKind::NONE,
            TypeKind::Boolean => dust_dds::xtypes::dynamic_type::TypeKind::BOOLEAN,
            TypeKind::Byte => dust_dds::xtypes::dynamic_type::TypeKind::BYTE,
            TypeKind::Int16 => dust_dds::xtypes::dynamic_type::TypeKind::INT16,
            TypeKind::Int32 => dust_dds::xtypes::dynamic_type::TypeKind::INT32,
            TypeKind::Int64 => dust_dds::xtypes::dynamic_type::TypeKind::INT64,
            TypeKind::Uint16 => dust_dds::xtypes::dynamic_type::TypeKind::UINT16,
            TypeKind::Uint32 => dust_dds::xtypes::dynamic_type::TypeKind::UINT32,
            TypeKind::Uint64 => dust_dds::xtypes::dynamic_type::TypeKind::UINT64,
            TypeKind::Float32 => dust_dds::xtypes::dynamic_type::TypeKind::FLOAT32,
            TypeKind::Float64 => dust_dds::xtypes::dynamic_type::TypeKind::FLOAT64,
            TypeKind::Float128 => dust_dds::xtypes::dynamic_type::TypeKind::FLOAT128,
            TypeKind::Int8 => dust_dds::xtypes::dynamic_type::TypeKind::INT8,
            TypeKind::Uint8 => dust_dds::xtypes::dynamic_type::TypeKind::UINT8,
            TypeKind::Char8 => dust_dds::xtypes::dynamic_type::TypeKind::CHAR8,
            TypeKind::Char16 => dust_dds::xtypes::dynamic_type::TypeKind::CHAR16,
            TypeKind::String8 => dust_dds::xtypes::dynamic_type::TypeKind::STRING8,
            TypeKind::String16 => dust_dds::xtypes::dynamic_type::TypeKind::STRING16,
            TypeKind::Alias => dust_dds::xtypes::dynamic_type::TypeKind::ALIAS,
            TypeKind::Enum => dust_dds::xtypes::dynamic_type::TypeKind::ENUM,
            TypeKind::Bitmask => dust_dds::xtypes::dynamic_type::TypeKind::BITMASK,
            TypeKind::Annotation => dust_dds::xtypes::dynamic_type::TypeKind::ANNOTATION,
            TypeKind::Structure => dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
            TypeKind::Union => dust_dds::xtypes::dynamic_type::TypeKind::UNION,
            TypeKind::Bitset => dust_dds::xtypes::dynamic_type::TypeKind::BITSET,
            TypeKind::Sequence => dust_dds::xtypes::dynamic_type::TypeKind::SEQUENCE,
            TypeKind::Array => dust_dds::xtypes::dynamic_type::TypeKind::ARRAY,
            TypeKind::Map => dust_dds::xtypes::dynamic_type::TypeKind::MAP,
        }
    }
}

impl From<dust_dds::xtypes::dynamic_type::TypeKind> for TypeKind {
    fn from(kind: dust_dds::xtypes::dynamic_type::TypeKind) -> Self {
        match kind {
            dust_dds::xtypes::dynamic_type::TypeKind::NONE => TypeKind::None,
            dust_dds::xtypes::dynamic_type::TypeKind::BOOLEAN => TypeKind::Boolean,
            dust_dds::xtypes::dynamic_type::TypeKind::BYTE => TypeKind::Byte,
            dust_dds::xtypes::dynamic_type::TypeKind::INT16 => TypeKind::Int16,
            dust_dds::xtypes::dynamic_type::TypeKind::INT32 => TypeKind::Int32,
            dust_dds::xtypes::dynamic_type::TypeKind::INT64 => TypeKind::Int64,
            dust_dds::xtypes::dynamic_type::TypeKind::UINT16 => TypeKind::Uint16,
            dust_dds::xtypes::dynamic_type::TypeKind::UINT32 => TypeKind::Uint32,
            dust_dds::xtypes::dynamic_type::TypeKind::UINT64 => TypeKind::Uint64,
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT32 => TypeKind::Float32,
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT64 => TypeKind::Float64,
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT128 => TypeKind::Float128,
            dust_dds::xtypes::dynamic_type::TypeKind::INT8 => TypeKind::Int8,
            dust_dds::xtypes::dynamic_type::TypeKind::UINT8 => TypeKind::Uint8,
            dust_dds::xtypes::dynamic_type::TypeKind::CHAR8 => TypeKind::Char8,
            dust_dds::xtypes::dynamic_type::TypeKind::CHAR16 => TypeKind::Char16,
            dust_dds::xtypes::dynamic_type::TypeKind::STRING8 => TypeKind::String8,
            dust_dds::xtypes::dynamic_type::TypeKind::STRING16 => TypeKind::String16,
            dust_dds::xtypes::dynamic_type::TypeKind::ALIAS => TypeKind::Alias,
            dust_dds::xtypes::dynamic_type::TypeKind::ENUM => TypeKind::Enum,
            dust_dds::xtypes::dynamic_type::TypeKind::BITMASK => TypeKind::Bitmask,
            dust_dds::xtypes::dynamic_type::TypeKind::ANNOTATION => TypeKind::Annotation,
            dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE => TypeKind::Structure,
            dust_dds::xtypes::dynamic_type::TypeKind::UNION => TypeKind::Union,
            dust_dds::xtypes::dynamic_type::TypeKind::BITSET => TypeKind::Bitset,
            dust_dds::xtypes::dynamic_type::TypeKind::SEQUENCE => TypeKind::Sequence,
            dust_dds::xtypes::dynamic_type::TypeKind::ARRAY => TypeKind::Array,
            dust_dds::xtypes::dynamic_type::TypeKind::MAP => TypeKind::Map,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct DynamicType {
    pub(crate) inner: dust_dds::xtypes::dynamic_type::DynamicType<'static>,
}

impl DynamicType {
    pub(crate) fn new(inner: dust_dds::xtypes::dynamic_type::DynamicType<'static>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl DynamicType {
    #[wasm_bindgen(js_name = getName)]
    pub fn get_name(&self) -> String {
        self.inner.get_name().to_string()
    }

    #[wasm_bindgen(js_name = getKind)]
    pub fn get_kind(&self) -> TypeKind {
        self.inner.get_kind().into()
    }

    #[wasm_bindgen(js_name = getMemberCount)]
    pub fn get_member_count(&self) -> u32 {
        self.inner.get_member_count()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct MemberDescriptor {
    pub(crate) name: String,
    pub(crate) r#type: DynamicType,
    pub(crate) id: Option<u32>,
    pub(crate) is_key: bool,
    pub(crate) is_optional: bool,
}

#[wasm_bindgen]
impl MemberDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new(
        name: &str,
        r#type: &DynamicType,
        id: Option<u32>,
        is_key: Option<bool>,
        is_optional: Option<bool>,
    ) -> Self {
        Self {
            name: name.to_string(),
            r#type: *r#type,
            id,
            is_key: is_key.unwrap_or(false),
            is_optional: is_optional.unwrap_or(false),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    #[wasm_bindgen(getter)]
    pub fn r#type(&self) -> DynamicType {
        self.r#type
    }

    #[wasm_bindgen(setter)]
    pub fn set_type(&mut self, r#type: &DynamicType) {
        self.r#type = *r#type;
    }

    #[wasm_bindgen(getter, js_name = isKey)]
    pub fn is_key(&self) -> bool {
        self.is_key
    }

    #[wasm_bindgen(setter, js_name = setIsKey)]
    pub fn set_is_key(&mut self, is_key: bool) {
        self.is_key = is_key;
    }

    #[wasm_bindgen(getter, js_name = isOptional)]
    pub fn is_optional(&self) -> bool {
        self.is_optional
    }

    #[wasm_bindgen(setter, js_name = setIsOptional)]
    pub fn set_is_optional(&mut self, is_optional: bool) {
        self.is_optional = is_optional;
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Option<u32> {
        self.id
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: Option<u32>) {
        self.id = id;
    }
}

#[wasm_bindgen]
pub struct DynamicTypeBuilder {
    pub(crate) inner: Option<dust_dds::xtypes::dynamic_type::DynamicTypeBuilder>,
}

#[wasm_bindgen]
impl DynamicTypeBuilder {
    #[wasm_bindgen(js_name = addMember)]
    pub fn add_member(&mut self, descriptor: &MemberDescriptor) -> Result<()> {
        let builder = self
            .inner
            .as_mut()
            .ok_or_else(|| js_sys::Error::new("DynamicTypeBuilder has already been consumed"))?;

        let current_member_count = builder.get_all_members().map_err(into_js_error)?.len() as u32;
        let id = descriptor.id.unwrap_or(current_member_count);
        let leaked_name: &'static str = Box::leak(descriptor.name.clone().into_boxed_str());

        let member_descriptor = dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name: leaked_name,
            id,
            r#type: descriptor.r#type.inner,
            default_value: None,
            index: current_member_count,
            label: &[],
            try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
            is_key: descriptor.is_key,
            is_optional: descriptor.is_optional,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
            is_external: false,
        };

        builder
            .add_member(member_descriptor)
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = build)]
    pub fn build(&mut self) -> Result<DynamicType> {
        let builder = self
            .inner
            .take()
            .ok_or_else(|| js_sys::Error::new("DynamicTypeBuilder has already been consumed"))?;
        let dynamic_type = builder.build();
        Ok(DynamicType::new(dynamic_type))
    }
}

#[wasm_bindgen]
pub struct DynamicTypeBuilderFactory;

#[wasm_bindgen]
impl DynamicTypeBuilderFactory {
    #[wasm_bindgen(js_name = getPrimitiveType)]
    pub fn get_primitive_type(kind: TypeKind) -> DynamicType {
        let rust_kind: dust_dds::xtypes::dynamic_type::TypeKind = kind.into();
        let dt = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(
            rust_kind,
        );
        DynamicType::new(dt)
    }

    #[wasm_bindgen(js_name = createStringType)]
    pub fn create_string_type(bound: Option<u32>) -> DynamicType {
        let b = bound.unwrap_or(0);
        let builder =
            dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_string_type(b);
        DynamicType::new(builder.build())
    }

    #[wasm_bindgen(js_name = createWstringType)]
    pub fn create_wstring_type(bound: Option<u32>) -> DynamicType {
        let b = bound.unwrap_or(0);
        let builder =
            dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_wstring_type(b);
        DynamicType::new(builder.build())
    }

    #[wasm_bindgen(js_name = createSequenceType)]
    pub fn create_sequence_type(element_type: &DynamicType, bound: Option<u32>) -> DynamicType {
        let b = bound.unwrap_or(0);
        let builder =
            dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_sequence_type(
                element_type.inner,
                b,
            );
        DynamicType::new(builder.build())
    }

    #[wasm_bindgen(js_name = createArrayType)]
    pub fn create_array_type(element_type: &DynamicType, bound: Vec<u32>) -> DynamicType {
        let leaked_bound: &'static [u32] = bound.leak();
        let builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_array_type(
            element_type.inner,
            leaked_bound,
        );
        DynamicType::new(builder.build())
    }

    #[wasm_bindgen(js_name = createStructureType)]
    pub fn create_structure_type(name: &str) -> DynamicTypeBuilder {
        let leaked_name: &'static str = Box::leak(name.to_string().into_boxed_str());
        let descriptor = dust_dds::xtypes::dynamic_type::TypeDescriptor {
            kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
            name: leaked_name,
            base_type: None,
            discriminator_type: None,
            bound: &[],
            element_type: None,
            key_element_type: None,
            extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
            is_nested: false,
            is_autoid_hash: false,
        };
        let builder =
            dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(descriptor);
        DynamicTypeBuilder {
            inner: Some(builder),
        }
    }
}
