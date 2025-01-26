mod fast_dds_idl {
    use std::{env, fs};

    #[test]
    fn AllocTestType() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/AllocTestType.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Benchmark() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Benchmark.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Benchmark_big() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Benchmark_big.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Benchmark_medium() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Benchmark_medium.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn extensibility_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/extensibility_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn FixedSized() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/FixedSized.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn FlowControl() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/FlowControl.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn HelloWorld() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/HelloWorld.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn key_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/key_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn map_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/map_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn monitorservice_types() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/monitorservice_types.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn KeyedData1mb() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/KeyedData1mb.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn primitives_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/primitives_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Benchmark_small() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Benchmark_small.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn KeyedHelloWorld() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/KeyedHelloWorld.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn alias_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/alias_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn rpc_types() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/rpc_types.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Calculator() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Calculator.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn ShapeType() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/ShapeType.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn array_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/array_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn sequence_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/sequence_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn ComprehensiveType() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/ComprehensiveType.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn StringTest() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/StringTest.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn bitmask_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/bitmask_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn string_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/string_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Configuration() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Configuration.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn TestIncludeRegression3361() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/TestIncludeRegression3361.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn bitset_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/bitset_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn struct_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/struct_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn ContentFilterTestType() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/ContentFilterTestType.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn TestRegression3361() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/TestRegression3361.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn core_types() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/core_types.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn types() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/types.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn DDSReturnCode() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/DDSReturnCode.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn TypeLookupTypes() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/TypeLookupTypes.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn dds_xtypes_typeobject() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/dds-xtypes_typeobject.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn union_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/union_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Data1mb() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Data1mb.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn UnboundedHelloWorld() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/UnboundedHelloWorld.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn dynamic_language_binding() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/dynamic_language_binding.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn Data64kb() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/Data64kb.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn enum_struct() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/enum_struct.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn DeliveryMechanism() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/DeliveryMechanism.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn XtypesTestsType1() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/XtypesTestsType1.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    fn XtypesTestsType2() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/XtypesTestsType2.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn XtypesTestsType3() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/XtypesTestsType3.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn XtypesTestsTypeBig() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/XtypesTestsTypeBig.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn XtypesTestsTypeDep() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/XtypesTestsTypeDep.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn XtypesTestsTypeNoTypeObject() {
        let dds_idl = match fs::read_to_string("test_data/fast_dds/XtypesTestsTypeNoTypeObject.idl")
        {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                panic!("Unable to read file: {}\n", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}\n", e),
        };

        assert!(!result.is_empty());
    }
}
