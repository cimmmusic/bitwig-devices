"""Local structural helpers for current-format document graphs."""

from __future__ import annotations

import hashlib
from pathlib import Path
import subprocess
from typing import Callable, Iterable


def digest_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_hash(path: Path, expected: str, label: str) -> None:
    if not path.is_file() or digest_file(path) != expected:
        raise ValueError(f"{label} is missing or has the wrong SHA-256: {path}")


def run(command: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {result.stderr.strip() or result.stdout.strip()}")
    return result


def iter_owned_objects(root: dict) -> Iterable[dict]:
    yield root
    for entry in root.get("fields", {}).values():
        kind, value = entry.get("type"), entry.get("value")
        if kind == "Object" and isinstance(value, dict):
            yield from iter_owned_objects(value)
        elif kind == "ObjectList" and isinstance(value, list):
            for item in value:
                if isinstance(item.get("Object"), dict):
                    yield from iter_owned_objects(item["Object"])


def direct_refs(obj: dict) -> Iterable[int]:
    for entry in obj.get("fields", {}).values():
        if entry.get("type") == "ObjectRef":
            yield int(entry["value"])
        elif entry.get("type") == "ObjectList":
            for item in entry.get("value", []):
                if "ObjectRef" in item:
                    yield int(item["ObjectRef"])


def update_direct_refs(obj: dict, resolve: Callable[[int], int]) -> None:
    for entry in obj.get("fields", {}).values():
        if entry.get("type") == "ObjectRef":
            entry["value"] = resolve(int(entry["value"]))
        elif entry.get("type") == "ObjectList":
            for item in entry.get("value", []):
                if "ObjectRef" in item:
                    item["ObjectRef"] = resolve(int(item["ObjectRef"]))
