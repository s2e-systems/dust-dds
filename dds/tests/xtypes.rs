use dust_dds::xtypes::{dynamic_type::DynamicTypeBuilderFactory, type_object::TypeKind};

#[test]
fn build_valid_primitive_types() {
    let kind = TypeKind::INT64;
    let dynamic_type = DynamicTypeBuilderFactory::get_primitive_type(kind);
    assert_eq!(dynamic_type.get_kind(), kind);
    assert_eq!(dynamic_type.get_name(), "");
    assert_eq!(dynamic_type.get_member_count(), 0);
}

#[test]
fn build_invalid_primitive_types() {
    let dynamic_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::STRUCTURE);
    assert_eq!(dynamic_type.get_kind(), TypeKind::NONE);
    assert_eq!(dynamic_type.get_name(), "");
    assert_eq!(dynamic_type.get_member_count(), 0);
}
