use dds_implementation::dds_type::DdsType;
use dds_derive::DdsType;

#[derive(DdsType)]
struct TypeNoKey {}

#[test]
fn test_dds_type_derive_no_key() {
    assert_eq!(TypeNoKey::type_name(), "TypeNoKey");
    assert_eq!(TypeNoKey::has_key(), false);
}

#[derive(DdsType)]
#[dds_type_with_key]
struct TypeWithKey {}

#[test]
fn test_dds_type_derive_with_key() {
    assert_eq!(TypeWithKey::type_name(), "TypeWithKey");
    assert_eq!(TypeWithKey::has_key(), true);
}