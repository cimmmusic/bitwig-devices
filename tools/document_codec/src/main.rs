use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use bitwig_document_codec::{decode_module, encode_module, Module};

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [operation, input] if operation == "to-json" => {
            let bytes = fs::read(input).map_err(|error| error.to_string())?;
            let document = decode_module(&bytes).map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?);
        }
        [operation, input, output] if operation == "from-json" => {
            let raw = fs::read(input).map_err(|error| error.to_string())?;
            let document: Module = serde_json::from_slice(&raw).map_err(|error| error.to_string())?;
            let bytes = encode_module(&document);
            fs::write(output, bytes).map_err(|error| error.to_string())?;
        }
        [operation, input] if operation == "roundtrip" => {
            let bytes = fs::read(input).map_err(|error| error.to_string())?;
            let document = decode_module(&bytes).map_err(|error| error.to_string())?;
            if encode_module(&document) != bytes {
                return Err(format!("document roundtrip changed bytes: {}", Path::new(input).display()));
            }
            eprintln!("Document roundtrip OK");
        }
        [first, second] if first == "json" && second == "version" => {
            println!("{{\"schema_version\":1,\"tool\":\"bitwig-document-codec\"}}");
        }
        _ => return Err("usage: bwmodule-parser to-json FILE | from-json JSON FILE | roundtrip FILE | json version".into()),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
