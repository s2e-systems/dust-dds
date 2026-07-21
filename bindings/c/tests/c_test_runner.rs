use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_c_create_participant() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // Workspace target directory
    let workspace_target = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug");

    // Ensure dust_dds_c library is built
    let build_status = Command::new("cargo")
        .args(["build", "-p", "dust_dds_c"])
        .status()
        .expect("Failed to run cargo build");
    assert!(build_status.success(), "Failed to build dust_dds_c");

    let out_dir = env::var("OUT_DIR").unwrap();
    let bin_path = PathBuf::from(&out_dir).join("test_create_participant");

    let status = Command::new("cc")
        .arg(manifest_dir.join("tests/test_create_participant.c"))
        .arg("-I")
        .arg(manifest_dir.join("include"))
        .arg("-L")
        .arg(&workspace_target)
        .arg("-ldust_dds_c")
        .arg("-lpthread")
        .arg("-ldl")
        .arg("-lm")
        .arg("-o")
        .arg(&bin_path)
        .status()
        .expect("Failed to compile C test");

    assert!(status.success(), "Failed to compile C test binary");

    let run_status = Command::new(&bin_path)
        .env("LD_LIBRARY_PATH", &workspace_target)
        .status()
        .expect("Failed to run C test binary");

    assert!(run_status.success(), "C test binary failed execution");
}
