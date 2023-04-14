use std::{env, io::Write};

use schemars::schema_for;

mod configuration;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        let msg = format!("Usage: {} <output_filepath> ", args[0]);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, msg));
    }
    let file_path = &args[1];

    let root_schema = schema_for!(configuration::DustDdsConfiguration);
    let json_schema_str_pretty = serde_json::to_string_pretty(&root_schema)?;

    let mut file = std::fs::File::create(file_path)?;
    file.write_all(json_schema_str_pretty.as_bytes())?;

    println!("Schema written to: {}", file_path);
    Ok(())
}
