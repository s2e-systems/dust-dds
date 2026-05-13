use dust_dds::xtypes::dynamic_type::{
    DynamicDataFactory, DynamicTypeBuilderFactory, ExtensibilityKind, TypeKind,
};

const TYPES_XML_PRIMITIVES: &str = r#"<dds>
    <types>
        <module name="Test">
            <struct name="struct_primitives_final"   extensibility="final">
                <member name="x1"   type="uint8"   />
                <member name="x2"   type="uint16"  />
                <member name="x3"   type="uint32"  />
                <member name="x4"   type="uint64"  />
                <member name="x5"   type="int8"    />
                <member name="x6"   type="int16"   />
                <member name="x7"   type="int32"   />
                <member name="x8"   type="int64"   />
                <member name="x9"   type="boolean" />
                <member name="x10"  type="float32" />
                <member name="x11"  type="float64" />
                <member name="x12"  type="float128"/>
                <member name="x13"  type="byte"    />
                <member name="x14"  type="char8"   />
            </struct>

            <struct name="struct_primitives_appendable"   extensibility="appendable">
                <member name="x1"   type="uint8"   />
                <member name="x2"   type="uint16"  />
                <member name="x3"   type="uint32"  />
                <member name="x4"   type="uint64"  />
                <member name="x5"   type="int8"   />
                <member name="x6"   type="int16"   />
                <member name="x7"   type="int32"   />
                <member name="x8"   type="int64"   />
                <member name="x9"   type="boolean" />
                <member name="x10"  type="float32" />
                <member name="x11"  type="float64" />
                <member name="x12"  type="float128"/>
                <member name="x13"  type="byte"    />
                <member name="x14"  type="char8"   />
            </struct>

            <struct name="struct_primitives_mutable"   extensibility="mutable">
                <member name="x1"   type="uint8"   />
                <member name="x2"   type="uint16"  />
                <member name="x3"   type="uint32"  />
                <member name="x4"   type="uint64"  />
                <member name="x5"   type="int8"   />
                <member name="x6"   type="int16"   />
                <member name="x7"   type="int32"   />
                <member name="x8"   type="int64"   />
                <member name="x9"   type="boolean" />
                <member name="x10"  type="float32" />
                <member name="x11"  type="float64" />
                <member name="x12"  type="float128"/>
                <member name="x13"  type="byte"    />
                <member name="x14"  type="char8"   />
            </struct>

            <union name="union_primitives">
                <discriminator type="uint8"/>
                <case><caseDiscriminator value="0x01"/><member name="x1"  type="uint8"   /></case>
                <case><caseDiscriminator value="0x02"/><member name="x2"  type="uint16"  /></case>
                <case><caseDiscriminator value="0x03"/><member name="x3"  type="uint32"  /></case>
                <case><caseDiscriminator value="0x04"/><member name="x4"  type="uint64"  /></case>
                <case><caseDiscriminator value="0x05"/><member name="x5"  type="int8"    /></case>
                <case><caseDiscriminator value="0x06"/><member name="x6"  type="int16"   /></case>
                <case><caseDiscriminator value="0x07"/><member name="x7"  type="int32"   /></case>
                <case><caseDiscriminator value="0x08"/><member name="x8"  type="int64"   /></case>
                <case><caseDiscriminator value="0x09"/><member name="x9"  type="boolean" /></case>
                <case><caseDiscriminator value="0x0a"/><member name="x10" type="float32" /></case>
                <case><caseDiscriminator value="0x0b"/><member name="x11" type="float64" /></case>
                <case><caseDiscriminator value="0x0c"/><member name="x12" type="float64" /></case>
                <case><caseDiscriminator value="0x0d"/><member name="x13" type="byte"    /></case>
                <case><caseDiscriminator value="0x0e"/><member name="x14" type="char8"   /></case>
            </union>


            <struct name="struct_primitive_uint8"   extensibility="final">
                <member name="x1" type="uint8"/>
            </struct>

            <struct name="struct_primitive_uint16"   extensibility="final">
                <member name="x1"  type="uint16"/>
            </struct>

            <struct name="struct_primitive_uint32"   extensibility="final">
                <member name="x1"  type="uint32"/>
            </struct>

            <struct name="struct_primitive_uint64"   extensibility="final">
                <member name="x1"  type="uint64"/>
            </struct>

            <struct name="struct_primitive_int8"   extensibility="final">
                <member name="x1"  type="int8"/>
            </struct>

            <struct name="struct_primitive_int16"   extensibility="final">
                <member name="x1" type="int16"/>
            </struct>

            <struct name="struct_primitive_int32"   extensibility="final">
                <member name="x1"  type="int32"/>
            </struct>

            <struct name="struct_primitive_int64"   extensibility="final">
                <member name="x1" type="int64"/>
            </struct>

            <struct name="struct_primitive_boolean"   extensibility="final">
                <member name="x1" type="boolean"/>
            </struct>

            <struct name="struct_primitive_float32"   extensibility="final">
                <member name="x1" type="float32"/>
            </struct>

            <struct name="struct_primitive_float64"   extensibility="final">
                <member name="x1" type="float64"/>
            </struct>

            <struct name="struct_primitive_float128"   extensibility="final">
                <member name="x1"  type="float128"/>
            </struct>

            <struct name="struct_primitive_byte"   extensibility="final">
                <member name="x1"  type="byte"/>
            </struct>

            <struct name="struct_primitive_char8"   extensibility="final">
                <member name="x1"  type="char8"/>
            </struct>
        </module>
    </types>
</dds>"#;

const DATA_XML_STRUCT_PRIMITIVES: &str = r#"<struct_primitives>
  <x1>0x01</x1>
  <x2>2</x2>
  <x3>3</x3>
  <x4>4</x4>
  <x5>0x05</x5>
  <x6>6</x6>
  <x7>7</x7>
  <x8>8</x8>
  <x9>true</x9>
  <x10>10.100000</x10>
  <x11>11.200000</x11>
  <x12>12.300000</x12>
  <x13>13</x13>
  <x14>0x0e</x14>
</struct_primitives>"#;

#[cfg(feature = "xtypes-xml")]
#[test]
fn create_struct_primitive_int8_from_xml() {
    let builder = DynamicTypeBuilderFactory::create_type_w_document(
        TYPES_XML_PRIMITIVES,
        "struct_primitive_int8",
        vec![],
    )
    .unwrap();

    let ty = builder.build();

    assert_eq!(ty.get_member_count(), 1);
    let member = ty.get_member_by_name("x1").unwrap();
    assert_eq!(member.descriptor.r#type.get_kind(), TypeKind::INT8);
}

#[cfg(feature = "xtypes-xml")]
#[test]
fn create_struct_primitives_final_from_xml() {
    let builder = DynamicTypeBuilderFactory::create_type_w_document(
        TYPES_XML_PRIMITIVES,
        "struct_primitives_final",
        vec![],
    )
    .unwrap();

    let ty = builder.build();

    assert_eq!(
        ty.get_descriptor().extensibility_kind,
        ExtensibilityKind::Final
    );
    assert_eq!(ty.get_member_count(), 14);

    let m1 = ty.get_member_by_name("x1").unwrap();
    assert_eq!(m1.descriptor.r#type.get_kind(), TypeKind::UINT8);

    let m2 = ty.get_member_by_name("x2").unwrap();
    assert_eq!(m2.descriptor.r#type.get_kind(), TypeKind::UINT16);

    let m3 = ty.get_member_by_name("x3").unwrap();
    assert_eq!(m3.descriptor.r#type.get_kind(), TypeKind::UINT32);

    let m4 = ty.get_member_by_name("x4").unwrap();
    assert_eq!(m4.descriptor.r#type.get_kind(), TypeKind::UINT64);

    let m5 = ty.get_member_by_name("x5").unwrap();
    assert_eq!(m5.descriptor.r#type.get_kind(), TypeKind::INT8);

    let m6 = ty.get_member_by_name("x6").unwrap();
    assert_eq!(m6.descriptor.r#type.get_kind(), TypeKind::INT16);

    let m7 = ty.get_member_by_name("x7").unwrap();
    assert_eq!(m7.descriptor.r#type.get_kind(), TypeKind::INT32);

    let m8 = ty.get_member_by_name("x8").unwrap();
    assert_eq!(m8.descriptor.r#type.get_kind(), TypeKind::INT64);

    let m9 = ty.get_member_by_name("x9").unwrap();
    assert_eq!(m9.descriptor.r#type.get_kind(), TypeKind::BOOLEAN);

    let m10 = ty.get_member_by_name("x10").unwrap();
    assert_eq!(m10.descriptor.r#type.get_kind(), TypeKind::FLOAT32);

    let m11 = ty.get_member_by_name("x11").unwrap();
    assert_eq!(m11.descriptor.r#type.get_kind(), TypeKind::FLOAT64);

    let m12 = ty.get_member_by_name("x12").unwrap();
    assert_eq!(m12.descriptor.r#type.get_kind(), TypeKind::FLOAT128);

    let m13 = ty.get_member_by_name("x13").unwrap();
    assert_eq!(m13.descriptor.r#type.get_kind(), TypeKind::BYTE);

    let m14 = ty.get_member_by_name("x14").unwrap();
    assert_eq!(m14.descriptor.r#type.get_kind(), TypeKind::CHAR8);
}

#[cfg(feature = "xtypes-xml")]
#[test]
fn struct_primitives_final_data_from_xml() {
    let builder = DynamicTypeBuilderFactory::create_type_w_document(
        TYPES_XML_PRIMITIVES,
        "struct_primitives_final",
        vec![],
    )
    .unwrap();

    let ty = builder.build();

    let mut d = DynamicDataFactory::create_data(ty);
    d.from_xml(DATA_XML_STRUCT_PRIMITIVES).unwrap();

    assert_eq!(d.get_uint8_value(0).unwrap(), &1);
}

#[cfg(feature = "xtypes-xml")]
#[test]
fn create_struct_primitives_appendable_from_xml() {
    let builder = DynamicTypeBuilderFactory::create_type_w_document(
        TYPES_XML_PRIMITIVES,
        "struct_primitives_appendable",
        vec![],
    )
    .unwrap();

    let ty = builder.build();

    assert_eq!(
        ty.get_descriptor().extensibility_kind,
        ExtensibilityKind::Appendable
    );
    assert_eq!(ty.get_member_count(), 14);

    let m1 = ty.get_member_by_name("x1").unwrap();
    assert_eq!(m1.descriptor.r#type.get_kind(), TypeKind::UINT8);

    let m2 = ty.get_member_by_name("x2").unwrap();
    assert_eq!(m2.descriptor.r#type.get_kind(), TypeKind::UINT16);

    let m3 = ty.get_member_by_name("x3").unwrap();
    assert_eq!(m3.descriptor.r#type.get_kind(), TypeKind::UINT32);

    let m4 = ty.get_member_by_name("x4").unwrap();
    assert_eq!(m4.descriptor.r#type.get_kind(), TypeKind::UINT64);

    let m5 = ty.get_member_by_name("x5").unwrap();
    assert_eq!(m5.descriptor.r#type.get_kind(), TypeKind::INT8);

    let m6 = ty.get_member_by_name("x6").unwrap();
    assert_eq!(m6.descriptor.r#type.get_kind(), TypeKind::INT16);

    let m7 = ty.get_member_by_name("x7").unwrap();
    assert_eq!(m7.descriptor.r#type.get_kind(), TypeKind::INT32);

    let m8 = ty.get_member_by_name("x8").unwrap();
    assert_eq!(m8.descriptor.r#type.get_kind(), TypeKind::INT64);

    let m9 = ty.get_member_by_name("x9").unwrap();
    assert_eq!(m9.descriptor.r#type.get_kind(), TypeKind::BOOLEAN);

    let m10 = ty.get_member_by_name("x10").unwrap();
    assert_eq!(m10.descriptor.r#type.get_kind(), TypeKind::FLOAT32);

    let m11 = ty.get_member_by_name("x11").unwrap();
    assert_eq!(m11.descriptor.r#type.get_kind(), TypeKind::FLOAT64);

    let m12 = ty.get_member_by_name("x12").unwrap();
    assert_eq!(m12.descriptor.r#type.get_kind(), TypeKind::FLOAT128);

    let m13 = ty.get_member_by_name("x13").unwrap();
    assert_eq!(m13.descriptor.r#type.get_kind(), TypeKind::BYTE);

    let m14 = ty.get_member_by_name("x14").unwrap();
    assert_eq!(m14.descriptor.r#type.get_kind(), TypeKind::CHAR8);
}

#[cfg(feature = "xtypes-xml")]
#[test]
fn create_union_primitives_from_xml() {
    let builder = DynamicTypeBuilderFactory::create_type_w_document(
        TYPES_XML_PRIMITIVES,
        "union_primitives",
        vec![],
    )
    .unwrap();

    let ty = builder.build();

    assert_eq!(ty.get_kind(), TypeKind::UNION);
    assert_eq!(
        ty.get_descriptor().discriminator_type.unwrap().get_kind(),
        TypeKind::UINT8
    );

    assert_eq!(ty.get_member_count(), 14);

    let m1 = ty.get_member_by_name("x1").unwrap();
    assert_eq!(m1.descriptor.r#type.get_kind(), TypeKind::UINT8);
    assert_eq!(m1.descriptor.label, Some(1));

    let m2 = ty.get_member_by_name("x2").unwrap();
    assert_eq!(m2.descriptor.r#type.get_kind(), TypeKind::UINT16);
    assert_eq!(m2.descriptor.label, Some(2));

    let m3 = ty.get_member_by_name("x3").unwrap();
    assert_eq!(m3.descriptor.r#type.get_kind(), TypeKind::UINT32);
    assert_eq!(m3.descriptor.label, Some(3));

    let m4 = ty.get_member_by_name("x4").unwrap();
    assert_eq!(m4.descriptor.r#type.get_kind(), TypeKind::UINT64);
    assert_eq!(m4.descriptor.label, Some(4));

    let m5 = ty.get_member_by_name("x5").unwrap();
    assert_eq!(m5.descriptor.r#type.get_kind(), TypeKind::INT8);
    assert_eq!(m5.descriptor.label, Some(5));

    let m6 = ty.get_member_by_name("x6").unwrap();
    assert_eq!(m6.descriptor.r#type.get_kind(), TypeKind::INT16);
    assert_eq!(m6.descriptor.label, Some(6));

    let m7 = ty.get_member_by_name("x7").unwrap();
    assert_eq!(m7.descriptor.r#type.get_kind(), TypeKind::INT32);
    assert_eq!(m7.descriptor.label, Some(7));

    let m8 = ty.get_member_by_name("x8").unwrap();
    assert_eq!(m8.descriptor.r#type.get_kind(), TypeKind::INT64);
    assert_eq!(m8.descriptor.label, Some(8));

    let m9 = ty.get_member_by_name("x9").unwrap();
    assert_eq!(m9.descriptor.r#type.get_kind(), TypeKind::BOOLEAN);
    assert_eq!(m9.descriptor.label, Some(9));

    let m10 = ty.get_member_by_name("x10").unwrap();
    assert_eq!(m10.descriptor.r#type.get_kind(), TypeKind::FLOAT32);
    assert_eq!(m10.descriptor.label, Some(10));

    let m11 = ty.get_member_by_name("x11").unwrap();
    assert_eq!(m11.descriptor.r#type.get_kind(), TypeKind::FLOAT64);
    assert_eq!(m11.descriptor.label, Some(11));

    let m12 = ty.get_member_by_name("x12").unwrap();
    assert_eq!(m12.descriptor.r#type.get_kind(), TypeKind::FLOAT64);
    assert_eq!(m12.descriptor.label, Some(12));

    let m13 = ty.get_member_by_name("x13").unwrap();
    assert_eq!(m13.descriptor.r#type.get_kind(), TypeKind::BYTE);
    assert_eq!(m13.descriptor.label, Some(13));

    let m14 = ty.get_member_by_name("x14").unwrap();
    assert_eq!(m14.descriptor.r#type.get_kind(), TypeKind::CHAR8);
    assert_eq!(m14.descriptor.label, Some(14));
}
