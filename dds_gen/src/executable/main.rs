use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <idl_file> [output_file]", args[0]);
        std::process::exit(1);
    }
    let idl_path = Path::new(&args[1]);
    match dust_dds_gen::compile_idl_c(idl_path) {
        Ok(c_code) => {
            if args.len() >= 3 {
                let out_path = Path::new(&args[2]);
                if let Err(e) = fs::write(out_path, c_code) {
                    eprintln!("Error writing output file {}: {}", out_path.display(), e);
                    std::process::exit(1);
                }
            } else {
                print!("{}", c_code);
            }
        }
        Err(e) => {
            eprintln!("Error compiling IDL to C: {}", e);
            std::process::exit(1);
        }
    }
}
