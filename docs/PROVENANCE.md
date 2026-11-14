# Source provenance

Blueprint Gain, its Nitro source, its panel and its document constructor are
original examples. The builder serializes a plain document from typed
structures and compiles the demo's Nitro against the user's local,
hash-pinned Bitwig compiler JARs. No factory document or factory DSP is a
build input.

The preflight, document lints, compiler setup and small graph helpers come
from the `bitwig.devices` source snapshot
`0afc203f486d06539461a39dfe572a179bc573de`. This copy includes a
display-only control contract for the demo. Retained profile checks for other
device kinds are implementation references. The only example and supported
workflow here are the Windows 6.1.3 hybrid Audio FX.

- [Nitro AST subset](../tools/nitro_ast/README.md) and [MIT notice](../tools/nitro_ast/LICENSE).
- [Document codec](../tools/document_codec/README.md) and [GPL-3.0-only license](../tools/document_codec/LICENSE).
- [Nitro bridge](../tools/host_compiler/README.md) and [GPL-3.0-only license](../tools/host_compiler/LICENSE).
