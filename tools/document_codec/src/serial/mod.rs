pub mod reader;
pub mod writer;

use std::io;
use std::str;

use crate::cipher::Dag;
use crate::keys::dag_key;
use crate::types::*;
use reader::BinaryReader;
use writer::BinaryWriter;

/// Parse BtWg 42-byte hex-ASCII header.
pub fn parse_header(data: &[u8]) -> BtWgHeader {
    let s = str::from_utf8(&data[..42]).expect("header not ASCII");
    assert_eq!(&s[..4], "BtWg", "bad magic");
    BtWgHeader {
        format_version: u32::from_str_radix(&s[4..8], 16).unwrap(),
        serialization_format: u32::from_str_radix(&s[8..12], 16).unwrap(),
        document_version: u32::from_str_radix(&s[12..16], 16).unwrap(),
        object_offset: u64::from_str_radix(&s[16..24], 16).unwrap(),
        resources_offset: u64::from_str_radix(&s[24..40], 16).unwrap(),
        flags: u16::from_str_radix(&s[40..42], 16).unwrap_or(0),
    }
}

/// Encode BtWg header to 42-byte hex-ASCII.
pub fn encode_header(header: &BtWgHeader) -> Vec<u8> {
    format!(
        "BtWg{:04x}{:04x}{:04x}{:08x}{:016x}{:02x}",
        header.format_version,
        header.serialization_format,
        header.document_version,
        header.object_offset,
        header.resources_offset,
        header.flags,
    ).into_bytes()
}

/// Decrypt an encrypted section (1-byte version + 16-byte nonce + ciphertext).
pub fn decrypt_section(data: &[u8]) -> Vec<u8> {
    let _yik_version = data[0];
    let nonce = &data[1..17];
    let encrypted = &data[17..];
    let key = dag_key().expect("locally supplied document key is required");
    let mut dag = Dag::new(&key, nonce);
    dag.apply(encrypted)
}

/// Encrypt a plaintext section with the given nonce.
pub fn encrypt_section(plaintext: &[u8], nonce: &[u8; 16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + 16 + plaintext.len());
    out.push(0); // yik version byte
    out.extend_from_slice(nonce);
    let key = dag_key().expect("locally supplied document key is required");
    let mut dag = Dag::new(&key, nonce);
    out.extend(dag.apply(plaintext)); // XOR cipher: encrypt == decrypt
    out
}

/// The padding between metadata and body is always 5000 × 0x20 + 0x0A = 5001 bytes.
const PADDING_LEN: usize = 5001;

/// Whether the serialization format uses encryption.
/// Format 4: encrypted (1-byte version + 16-byte nonce + ciphertext per section).
/// Format 2: plain (raw Ramona bytes, no encryption prefix).
fn is_encrypted(header: &BtWgHeader) -> bool {
    header.serialization_format == 4
}

/// Read a section's Ramona bytes, handling encryption transparently.
/// Encrypted (format 4): strips version+nonce prefix, decrypts ciphertext.
/// Plain (format 2): returns the section bytes directly.
fn read_section(data: &[u8], encrypted: bool) -> Vec<u8> {
    if encrypted {
        decrypt_section(data)
    } else {
        data.to_vec()
    }
}

/// Extract nonces from an encrypted section (format 4 only).
/// Returns zeroed nonces for plain format.
fn extract_nonce(data: &[u8], encrypted: bool) -> [u8; 16] {
    if encrypted {
        let mut nonce = [0u8; 16];
        nonce.copy_from_slice(&data[1..17]);
        nonce
    } else {
        [0u8; 16]
    }
}

/// Reject documents this reader cannot handle, as a recoverable error.
///
/// Only BtWg format 3 is supported (serialization 2 = plain .bwpreset,
/// serialization 4 = encrypted .bwmodule/.bwdevice/.bwmodulator). Both entry
/// points take arbitrary bytes and already return `io::Result`, so anything
/// else — a truncated file, a document from a different Bitwig generation, or
/// something that is not a Bitwig document at all — is better reported than
/// asserted on.
///
/// This used to be `assert_eq!`. In the GUI that is worse than it sounds: the
/// module list is parsed by several worker threads, so one unreadable file
/// killed its worker and every file still queued behind it silently vanished
/// from the list.
fn check_format_version(header: &BtWgHeader) -> io::Result<()> {
    if header.format_version != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported BtWg format version {} (only 3 is supported)",
                header.format_version
            ),
        ));
    }
    Ok(())
}

/// Decode only the metadata section, skipping the body.
/// Much faster for listing modules since the body can be large.
/// Supports both encrypted (format 4) and plain (format 2) files.
pub fn decode_module_metadata_only(data: &[u8]) -> io::Result<(BtWgHeader, RamonaObject)> {
    let header = parse_header(data);
    check_format_version(&header)?;
    let encrypted = is_encrypted(&header);

    let obj_offset = header.object_offset as usize;
    let meta_end = obj_offset - PADDING_LEN;
    let meta_section = &data[42..meta_end];

    let meta_bytes = read_section(meta_section, encrypted);
    let mut reader = BinaryReader::new(meta_bytes);
    let metadata = reader.read_object(None)?;

    Ok((header, metadata))
}

/// Decode a BtWg file from raw bytes into a Module.
/// Supports both encrypted (format 4: .bwmodule/.bwdevice/.bwmodulator)
/// and plain (format 2: .bwpreset) serialization formats.
pub fn decode_module(data: &[u8]) -> io::Result<Module> {
    let header = parse_header(data);
    check_format_version(&header)?;
    let encrypted = is_encrypted(&header);

    let obj_offset = header.object_offset as usize;
    let meta_end = obj_offset - PADDING_LEN;
    let meta_section = &data[42..meta_end];
    let body_section = &data[obj_offset..];

    // Extract nonces (zeroed for plain format)
    let meta_nonce = extract_nonce(meta_section, encrypted);
    let body_nonce = extract_nonce(body_section, encrypted);

    // Parse metadata
    let meta_bytes = read_section(meta_section, encrypted);
    let mut reader = BinaryReader::new(meta_bytes);
    let metadata = reader.read_object(None)?;

    // Parse body
    let body_bytes = read_section(body_section, encrypted);
    let mut reader = BinaryReader::new(body_bytes);
    let body = reader.read_object(None)?;

    let mut module = Module { header, metadata, body, meta_nonce, body_nonce };
    crate::refs::resolve_refs(&mut module);
    Ok(module)
}

/// Encode a Module back to raw bytes.
///
/// Internally clones the module and bakes stable ObjectRef IDs back to
/// sequential wire-format IDs before writing.
/// Preserves the original serialization format: encrypted files stay encrypted,
/// plain files stay plain.
pub fn encode_module(module: &Module) -> Vec<u8> {
    let mut baked = module.clone();
    crate::refs::bake_refs(&mut baked);

    let mut meta_writer = BinaryWriter::new();
    meta_writer.write_object(&baked.metadata);
    let meta_plain = meta_writer.data;

    let mut body_writer = BinaryWriter::new();
    body_writer.write_object(&baked.body);
    let body_plain = body_writer.data;

    let encrypted = is_encrypted(&module.header);

    let (meta_out, body_out) = if encrypted {
        (encrypt_section(&meta_plain, &module.meta_nonce),
         encrypt_section(&body_plain, &module.body_nonce))
    } else {
        (meta_plain, body_plain)
    };

    let mut header = module.header.clone();
    header.object_offset = (42 + meta_out.len() + PADDING_LEN) as u64;
    let header_bytes = encode_header(&header);

    let mut out = Vec::with_capacity(
        header_bytes.len() + meta_out.len() + PADDING_LEN + body_out.len()
    );
    out.extend_from_slice(&header_bytes);
    out.extend_from_slice(&meta_out);
    out.extend(std::iter::repeat_n(0x20u8, 5000));
    out.push(0x0a);
    out.extend_from_slice(&body_out);
    out
}
