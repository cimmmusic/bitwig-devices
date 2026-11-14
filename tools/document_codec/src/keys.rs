//! Resolve a locally supplied document key without embedding key material.

use std::env;
use std::fs;
use std::path::PathBuf;

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    let value = value.trim();
    if value.len() % 2 != 0 {
        return Err("document key has an invalid hex length".into());
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for index in (0..value.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&value[index..index + 2], 16)
            .map_err(|_| "document key is not valid hex")?);
    }
    if bytes.len() != 128 {
        return Err("document key has the wrong byte length".into());
    }
    Ok(bytes)
}

fn candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = env::var_os("BITWIG_NITRO_KEYS") {
        paths.push(PathBuf::from(path));
    }
    paths.push(PathBuf::from("keys.json"));
    if let Some(appdata) = env::var_os("APPDATA") {
        paths.push(PathBuf::from(appdata).join("bitwig-nitro/keys.json"));
    }
    if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(config).join("bitwig-nitro/keys.json"));
    } else if let Some(home) = env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".config/bitwig-nitro/keys.json"));
    }
    paths
}

pub(crate) fn dag_key() -> Result<Vec<u8>, String> {
    if let Ok(value) = env::var("BITWIG_NITRO_DAG_KEY") {
        return decode_hex(&value);
    }
    for path in candidates() {
        if !path.is_file() {
            continue;
        }
        let document: serde_json::Value = serde_json::from_slice(
            &fs::read(&path).map_err(|_| "cannot read document key file")?,
        ).map_err(|_| "document key file is invalid JSON")?;
        if let Some(value) = document.get("dag_key").and_then(serde_json::Value::as_str) {
            return decode_hex(value);
        }
    }
    Err("document key is unavailable; supply BITWIG_NITRO_KEYS or BITWIG_NITRO_DAG_KEY from your own licensed installation".into())
}
