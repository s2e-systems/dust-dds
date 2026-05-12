#[cfg(feature = "xtypes-xml")]
#[test]
fn create_struct_primitive_int8_from_xml() {
    use dust_dds::xtypes::dynamic_type::{DynamicTypeBuilderFactory, TypeKind};

    let xml = r#"<dds>
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

    let builder = DynamicTypeBuilderFactory::create_type_w_document(
        xml,
        "struct_primitive_int8",
        vec![],
    )
    .unwrap();

    let ty = builder.build();

    assert_eq!(ty.get_member_count(), 1);
    let member = ty.get_member_by_name("x1").unwrap();
    assert_eq!(member.get_id(), 0);
    assert_eq!(member.descriptor.r#type.get_kind(), TypeKind::UINT8);
}
