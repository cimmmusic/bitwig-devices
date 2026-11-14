#!/usr/bin/env python3
"""Build the repository-owned document codec and Nitro source bridge.

Generated tools and exact-version compiler inputs stay ignored. No Bitwig
installation file is modified and no application is launched.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

import compiler_profiles


ROOT = Path(__file__).resolve().parents[1]
CODEC = ROOT / "tools/document_codec"
BRIDGE_SOURCE = ROOT / "tools/host_compiler/NitroBridge.java"


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(block)
    return hasher.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile-id", required=True, choices=compiler_profiles.PROFILES)
    parser.add_argument("--compiler-inputs", type=Path,
                        help="read jar, libs, and java paths from an existing private input file")
    parser.add_argument("--jar", type=Path)
    parser.add_argument("--libs", type=Path)
    parser.add_argument("--java", type=Path)
    parser.add_argument("--javac", default="javac")
    args = parser.parse_args()
    existing = json.loads(args.compiler_inputs.read_text(encoding="utf-8")) if args.compiler_inputs else {}
    profile = compiler_profiles.PROFILES[args.profile_id]
    paths = {}
    for name in ("jar", "libs", "java"):
        supplied = getattr(args, name) or existing.get(name)
        if not supplied:
            parser.error(f"{name} path is required")
        paths[name] = Path(supplied).expanduser().resolve()
        if not paths[name].is_file():
            parser.error(f"{name} path is missing")
    for name in ("jar", "libs"):
        if digest(paths[name]) != profile[f"{name}Sha256"]:
            parser.error(f"{name} does not match the exact target profile")
    output = ROOT / "local/host-tools" / args.profile_id
    bridge_dir = output / "bridge"
    bridge_dir.mkdir(parents=True, exist_ok=True)
    (output / "compiler-inputs.json").unlink(missing_ok=True)
    subprocess.run(["cargo", "build", "--release", "--locked", "--manifest-path", str(CODEC / "Cargo.toml")],
                   cwd=ROOT, check=True)
    binary = CODEC / "target/release" / ("bwmodule-parser.exe" if os.name == "nt" else "bwmodule-parser")
    subprocess.run([args.javac, "-cp", os.pathsep.join((str(paths["jar"]), str(paths["libs"]))),
                    "-d", str(bridge_dir), str(BRIDGE_SOURCE)], check=True)
    bridge = bridge_dir / "NitroBridge.class"
    bridge_classpath = os.pathsep.join((str(bridge_dir), str(paths["jar"]), str(paths["libs"])))
    probe = subprocess.run([str(paths["java"]), "-cp", bridge_classpath, "NitroBridge"],
                           capture_output=True, text=True, encoding="utf-8", errors="replace")
    if probe.returncode != 2 or "Usage: NitroBridge" not in probe.stderr:
        raise RuntimeError("Selected Java runtime cannot initialize the exact-version Nitro bridge: "
                           + probe.stderr.strip())
    subprocess.run([str(binary), "json", "version"], check=True, capture_output=True, text=True)
    inputs = {
        "schemaVersion": 1,
        "profileId": args.profile_id,
        "jar": str(paths["jar"]),
        "libs": str(paths["libs"]),
        "java": str(paths["java"]),
        "javac": args.javac,
        "bridge": str(bridge),
        "bridgeSha256": digest(bridge),
        "parser": str(binary),
        "parserSha256": digest(binary),
    }
    config = output / "compiler-inputs.json"
    config.write_text(json.dumps(inputs, indent=2) + "\n", encoding="utf-8")
    try:
        compiler_profiles.validate_inputs(config)
    except Exception:
        config.unlink(missing_ok=True)
        raise
    print(f"Validated local tools and compiler inputs: {config}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
