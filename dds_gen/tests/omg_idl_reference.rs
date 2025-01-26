//! The purpose of this test is to ensure that the IDL parser is able to handle
//! the default `IDL` files provided by OMG.
//!
//!

mod omg_idl {
    use std::{env, fs};

    /*************  ✨ Codeium Command ⭐  *************/
    /// This test function reads the content of the `robot.idl` file located in `tests/reference_idl`.
    /// It ensures that the file can be successfully read without any errors.

    /******  99784f6b-3e59-4be0-91be-2204a78a858d  *******/
    #[test]
    pub fn test_rpc_idl_robot() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/rpc-idl/robot.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_rpc_annotations() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/rpc-idl/rpc_annotations.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_rpc_types() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/rpc-idl/rpc_types.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_dds_dcps() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/dds_dcps.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn dds_dlrl() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/dds_dlrl.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_dds_security_plugin_spis() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/dds_security_plugin_spis.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_dds_xrce_types() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/dds_xrce_types.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_dds_xtypes_discovery_buildin_topics() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/dds_xtypes_discovery_buildin_topics.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
    #[test]
    pub fn test_dds_xtypes_typeobject() {
        let dds_idl = match fs::read_to_string("test_data/omg_dds/dds_xtypes_typeobject.idl") {
            Ok(dds_idl) => dds_idl,
            Err(e) => {
                dbg!("Reference directory: ", env::current_dir());
                panic!("Unable to read file: {}", e)
            }
        };

        let result = match dust_dds_gen::compile_idl(&dds_idl) {
            Ok(parsed_idl) => parsed_idl,
            Err(e) => panic!("Error parsing IDL string: {}", e),
        };

        assert!(!result.is_empty());
    }
}
