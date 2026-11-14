[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$python = Join-Path $projectRoot '.venv\Scripts\python.exe'
if (-not (Test-Path -LiteralPath $python)) {
    throw "Project Python is missing. Run .\scripts\setup.ps1 first."
}

& $python -m pip install --disable-pip-version-check 'skia-python==144.0.post2' 'llvmlite==0.50.0'
if ($LASTEXITCODE -ne 0) {
    throw "Could not install the pinned SkSL and LLVM preflight dependencies."
}
Write-Output 'Preflight dependencies installed.'
