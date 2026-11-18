#!/usr/bin/env python3
"""Write a copy of a Bitwig project with the custom (BML-registered) native devices taken out.

Plain Bitwig (without BML and the device registrar) crashes opening a project that holds a native
device it doesn't know, e.g. VU Meter. The copy opens anywhere. The original is never changed.

usage: share_safe_copy.py PROJECT.bwproject [--out FILE] [--uuid UUID ...] [--allow-trim]

- Devices removed: every native device whose device UUID belongs to a device in devices/*/bitwig-project.json
  (or the --uuid values).
- A VU Meter with Trim not at 0 dB changes the level, so removing it changes the mix: the script stops
  and lists them, unless --allow-trim is given.
- Object references in the project are numbered by position, so after removing devices they are renumbered.
  The script stops if anything outside a removed device points into it (e.g. modulation or automation).
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CODEC = ROOT / "tools/document_codec/target/release/bwmodule-parser"


def codec(*args: str) -> str:
    done = subprocess.run([str(CODEC), *args], capture_output=True, text=True)
    if done.returncode:
        sys.exit(f"codec {args[0]} failed: {done.stderr.strip()}")
    return done.stdout


def workspace_devices() -> dict[str, str]:
    found = {}
    for manifest in ROOT.glob("devices/*/bitwig-project.json"):
        data = json.loads(manifest.read_text(encoding="utf-8"))
        uuid = data.get("identity", {}).get("uuid")
        if uuid:
            found[uuid.lower()] = data.get("name", manifest.parent.name)
    return found


# --- object numbering, in the same depth-first order as the codec (tools/document_codec/src/refs.rs) ---

def number(root: dict) -> list[dict]:
    """Objects of one section in wire order; ObjectRef n points at order[n - 1]."""
    order: list[dict] = []

    def obj(o: dict) -> None:
        order.append(o)
        for value in o.get("fields", {}).values():
            val(value)

    def relative(data: dict) -> None:
        target = data.get("target", {})
        if target.get("kind") == "Object":
            obj(target)

    def val(v: dict) -> None:
        kind = v.get("type")
        if kind == "Object":
            obj(v["value"])
        elif kind == "RelativeRef":
            relative(v["value"])
        elif kind == "ObjectList":
            for item in v["value"]:
                if isinstance(item, dict) and "Object" in item:
                    obj(item["Object"])
                elif isinstance(item, dict) and "RelativeRef" in item:
                    relative(item["RelativeRef"])
        elif kind == "StringToObjectMap":
            for _key, inner in v["value"]:
                val(inner)

    obj(root)
    return order


def refs(root: dict):
    """Yield (holder, key) for every ObjectRef slot: holder[key] is the referenced number."""
    def obj(o: dict):
        for value in o.get("fields", {}).values():
            yield from val(value)

    def relative(data: dict):
        target = data.get("target", {})
        if target.get("kind") == "ObjectRef":
            yield target, "id"
        elif target.get("kind") == "Object":
            yield from obj(target)

    def val(v: dict):
        kind = v.get("type")
        if kind == "ObjectRef":
            yield v, "value"
        elif kind == "Object":
            yield from obj(v["value"])
        elif kind == "RelativeRef":
            yield from relative(v["value"])
        elif kind == "ObjectList":
            for item in v["value"]:
                if not isinstance(item, dict):
                    continue
                if "ObjectRef" in item:
                    yield item, "ObjectRef"
                elif "Object" in item:
                    yield from obj(item["Object"])
                elif "RelativeRef" in item:
                    yield from relative(item["RelativeRef"])
        elif kind == "StringToObjectMap":
            for _key, inner in v["value"]:
                yield from val(inner)

    yield from obj(root)


# --- finding the devices ---

def device_uuid(device: dict) -> str | None:
    preset = device.get("fields", {}).get("preset_to_serialize#407", {})
    if preset.get("type") != "Object":
        return None
    value = preset["value"].get("fields", {}).get("#153", {})
    return value.get("value", "").lower() if value.get("type") == "Uuid" else None


def trim_db(device: dict) -> float:
    """VU Meter's TRIM parameter value (dB); 0 when absent."""
    result = 0.0

    def walk(o):
        nonlocal result
        if isinstance(o, dict):
            fields = o.get("fields", {})
            if fields.get("identifier#697", {}).get("value") == "TRIM" and "value#310" in fields:
                result = float(fields["value#310"]["value"])
            for v in o.values():
                walk(v)
        elif isinstance(o, list):
            for v in o:
                walk(v)

    walk(device)
    return result


def find_devices(root: dict, wanted: set[str]):
    """Yield (list, index, device, label) for each wanted native device held in an object list.

    label = the nearest enclosing name found on the way down (best effort: usually the track).
    """
    def own_name(o: dict) -> str:
        fields = o.get("fields")
        if isinstance(fields, dict):
            for fk, fv in fields.items():
                if (fk == "#347" or fk.startswith("name#")) and isinstance(fv, dict) \
                        and isinstance(fv.get("value"), str) and fv["value"]:
                    return fv["value"]
        return ""

    def walk(o, label):
        if isinstance(o, dict):
            label = own_name(o) or label
            for v in o.values():
                if isinstance(v, dict) and v.get("type") == "ObjectList":
                    items = v["value"]
                    for i, item in enumerate(items):
                        if isinstance(item, dict) and isinstance(item.get("Object"), dict):
                            dev = item["Object"]
                            if dev.get("class_name") == "native_device" and device_uuid(dev) in wanted:
                                yield items, i, dev, label
                                continue
                            yield from walk(dev, label)
                        else:
                            yield from walk(item, label)
                else:
                    yield from walk(v, label)
        elif isinstance(o, list):
            for v in o:
                yield from walk(v, label)

    yield from walk(root, "")


def subtree_ids(device: dict) -> set[int]:
    return {id(o) for o in number(device)}


def main() -> int:
    cli = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    cli.add_argument("project", type=Path)
    cli.add_argument("--out", type=Path, help="default: '<name> (share-safe).bwproject' next to the original")
    cli.add_argument("--uuid", action="append", default=[], help="extra device UUID to remove")
    cli.add_argument("--allow-trim", action="store_true", help="remove VU Meters even when their Trim is not 0 dB")
    args = cli.parse_args()

    source = args.project.expanduser().resolve()
    out = (args.out or source.with_name(f"{source.stem} (share-safe).bwproject")).expanduser().resolve()
    if out == source:
        sys.exit("refusing to overwrite the original project")
    names = workspace_devices()
    for extra in args.uuid:
        names.setdefault(extra.lower(), extra)
    if not names:
        sys.exit("no device UUIDs to remove")

    codec("roundtrip", str(source))  # the codec reproduces this file byte for byte
    document = json.loads(codec("to-json", str(source)))
    sections = [s for s in ("body", "metadata") if isinstance(document.get(s), dict)]

    # The numbering must agree with every reference already in the file.
    for s in sections:
        count = len(number(document[s]))
        bad = [h[k] for h, k in refs(document[s]) if not 1 <= h[k] <= count]
        if bad:
            sys.exit(f"{s}: object numbering doesn't match the file (refs {bad[:5]}); nothing written")

    found = []
    for s in sections:
        for items, index, device, path in find_devices(document[s], set(names)):
            found.append((s, items, index, device, path))
    if not found:
        print(f"No custom devices in {source.name}; nothing to do.")
        return 0

    labels = [f"{names[device_uuid(f[3])]} from '{f[4] or '?'}'" for f in found]
    trimmed = [(f, trim_db(f[3])) for f in found if abs(trim_db(f[3])) > 1e-9]
    if trimmed and not args.allow_trim:
        for f, t in trimmed:
            print(f"  {names[device_uuid(f[3])]} on '{f[4] or '?'}': Trim {t:+.1f} dB")
        sys.exit("These change the level, so removing them changes the mix. Set Trim to 0 dB (or add a Tool "
                 "with that gain) and run again, or pass --allow-trim. Nothing written.")

    # Nothing outside a removed device may point into it.
    for s in sections:
        order = number(document[s])
        removed = set()
        for f in found:
            if f[0] == s:
                removed |= subtree_ids(f[3])
        inside_refs = set()
        for f in found:
            if f[0] == s:
                inside_refs |= {id(h) for h, _ in refs(f[3])}
        for holder, key in refs(document[s]):
            if id(holder) in inside_refs:
                continue
            if id(order[holder[key] - 1]) in removed:
                sys.exit(f"{s}: something outside a custom device refers into it (automation or modulation?). "
                         "Remove that link in Bitwig first. Nothing written.")

    # Remove, then renumber every reference.
    for s in sections:
        before = number(document[s])
        slots = list(refs(document[s]))
        targets = [(h, k, before[h[k] - 1]) for h, k in slots]
        for f in sorted((f for f in found if f[0] == s), key=lambda f: -f[2]):
            del f[1][f[2]]
        after = {id(o): n for n, o in enumerate(number(document[s]), 1)}
        for h, k, target in targets:
            if id(target) in after:
                h[k] = after[id(target)]

    with tempfile.TemporaryDirectory() as tmp:
        edited = Path(tmp) / "edited.json"
        edited.write_text(json.dumps(document), encoding="utf-8")
        codec("from-json", str(edited), str(out))
    codec("roundtrip", str(out))
    check = json.loads(codec("to-json", str(out)))
    left = sum(1 for s in sections for _ in find_devices(check[s], set(names)))
    if left:
        out.unlink(missing_ok=True)
        sys.exit("verification failed: custom devices still present; output removed")

    for label in labels:
        print(f"  removed {label}")
    print(f"Wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
