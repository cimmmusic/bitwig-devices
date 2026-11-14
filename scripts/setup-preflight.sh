#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd -- "${script_dir}/.." && pwd)"
python="${project_root}/.venv/bin/python"

if [[ ! -x "${python}" ]]; then
    echo "Project Python is missing. Create .venv first (see docs/BUILD.md)." >&2
    exit 1
fi

"${python}" -m pip install --disable-pip-version-check 'skia-python==144.0.post2' 'llvmlite==0.50.0'
echo "Preflight dependencies installed."
