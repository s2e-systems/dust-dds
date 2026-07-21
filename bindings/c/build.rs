extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cbindgen.toml");
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut config =
        cbindgen::Config::from_file(format!("{}/cbindgen.toml", crate_dir)).unwrap();
    config.language = cbindgen::Language::C;

    let include_dir = PathBuf::from(&crate_dir).join("include");
    std::fs::create_dir_all(&include_dir).ok();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(include_dir.join("dust_dds.h"));
}
