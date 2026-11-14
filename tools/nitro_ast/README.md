# Nitro AST developer subset

Status: MIT-licensed source subset used by device builders; no Bitwig archive,
keys, decrypted modules, or installation files are included.
Purpose: Make original Nitro AST builds and offline roundtrips reproducible
without a separate Nitro Tools checkout.

The source files `nitrobin_decompiler.py`, `nitrobin_writer.py`,
`nitro_builder.py`, `nitro_pretty.py`, and `paths.py`, and the two structural
grammar tables under `src/bitwig_nitro/data/`, are byte-identical copies from
Blake Bratcher's `bitwig-nitro-tools` checkout at Git commit
`8d9432b994f68f634013bb91ef2a7aba26791804`. That checkout documents the
grammar tables as the narrow structure-only exception to its rule against
committing installation-derived material. The copied MIT notice is in
[`LICENSE`](LICENSE); the package `__init__.py` and packaging manifest are
local adapters. This is a bounded AST library, not the full upstream toolchain.

From the `bitwig.devices` root, install the local package into a development
environment:

```sh
python -m pip install -e tools/nitro_ast
```

The public Python API used here is `decompile_nitrobin`,
`serialize_nitrobin`, `pretty_print`, and `nitro_builder`. These support
synthetic AST construction and byte-level offline checks. They do not compile
Nitro text through Bitwig's parser or prove loader, JIT, audio, GUI, or recall
behavior. Commands for decrypting installed Nitro archives and generating
catalogs remain outside this subset and require separately installed upstream
tools and locally supplied licensed inputs.
