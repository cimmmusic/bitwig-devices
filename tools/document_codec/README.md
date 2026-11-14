# Current Bitwig document codec

This standalone GPL-3.0-only Rust crate provides the three document operations
used by maintained device builds: `to-json`, `from-json`, and `roundtrip`.
It is built locally with `cargo build --release --locked` by
`scripts/build-local-host-tools.py`; binaries remain ignored.

The bounded serializer, object model, references, name table, and cipher code
were extracted from the local `bwmodule-parser` Research source snapshot at
commit `cac5df0b0d5155c918839652de0138b93fa6dd57`, with local patch SHA-256
`2fe9160d15a437e273eb2d3fcded807600ae31392f26999a1c559c0ae2504e92`.
The CLI and key resolver in this subtree were written for this repository.
Embedded Research key constants were removed from `cipher.rs`; encrypted
documents require an ignored local key source. The original Normalizer output
uses the plain document format and requires no key. See [LICENSE](LICENSE).

The codec does not establish Bitwig loader acceptance. Keep each target
version's roundtrip, graph, compiler, and host evidence separate.
