# Build and preflight

The reference procedure targets Windows x64 and the hash-pinned Bitwig Studio
6.1.3 compiler. Use uv with Python 3.12 x64, a Rust toolchain with Cargo and a working
MSVC build environment, and a JDK with `javac` on PATH. The setup downloads
Python and Rust dependencies on first use. Once those are installed, builds
and preflight use local files and run with Bitwig closed.

## One-time setup

From the repository root in PowerShell, adjust `$bitwig` to your own 6.1.3
installation path.

```powershell
uv venv --seed --python 3.12 .venv
if ($LASTEXITCODE -ne 0) { throw 'Could not create Python environment' }
$python = '.\.venv\Scripts\python.exe'
& $python -m pip install --no-deps -e tools/nitro_ast
if ($LASTEXITCODE -ne 0) { throw 'Nitro AST installation failed' }
.\scripts\setup-preflight.ps1
$bitwig = 'C:\Program Files\Bitwig Studio'
& $python scripts/build-local-host-tools.py `
  --profile-id windows-x86_64-bitwig-6.1.3 `
  --jar (Join-Path $bitwig 'bin\bitwig.jar') `
  --libs (Join-Path $bitwig 'bin\libs.jar') `
  --java (Join-Path $bitwig 'jre\bin\java.exe') `
  --javac (Get-Command javac -CommandType Application).Source
if ($LASTEXITCODE -ne 0) { throw 'Compiler tool setup failed' }
```

If a JAR hash differs, verify that the paths point at the exact target build.
For another Bitwig version or platform, treat the document, toolchain and host
test as a separate target. No key or donor device is needed.

## Run the gate and build

```powershell
$python = '.\.venv\Scripts\python.exe'
$inputs = 'local/host-tools/windows-x86_64-bitwig-6.1.3/compiler-inputs.json'
$demo = 'devices/blueprint-gain'
& $python scripts/device_preflight.py $demo --compiler-inputs $inputs
if ($LASTEXITCODE -ne 0) { throw 'Preflight failed' }
& $python "$demo/build.py" --compiler-inputs $inputs
if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
$report = Get-Content build/preflight/polarity.blueprint-gain/report.json -Raw | ConvertFrom-Json
$artifact = "$demo/build/windows-6.1.3/Blueprint Gain.bwdevice"
if ($report.status -ne 'pass' -or (Get-FileHash -Algorithm SHA256 -LiteralPath $artifact).Hash -ne $report.artifactSha256) {
    throw 'Built file does not match the passing preflight'
}
```

Start with the first failed check or rule ID in the JSON report. Profiles
check the exact compiler inputs, Nitro syntax, semantic analysis and LLVM IR,
two clean builds, byte-stable document roundtrips, graph and panel contracts,
and embedded SkSL compilation in local Skia.

`PASS (ready-for-controlled-host-test)` means the artifact is ready for the
[Bitwig test](BITWIG-TEST.md). Native JIT, browser insertion, real audio,
shader paint, mouse interaction and saved-project recall are live checks.

## Try a deliberate preflight failure

In a disposable checkout, make one change at a time and run the preflight
command above. Restore each edit before trying the next one.

| Edit | Expected result |
| --- | --- |
| In `gain.nitro`, replace `fmin(1.0, fmax(0.0, gain))` with `clamp(gain, 0.0, 1.0)`. | `NITRO020` blocks the source before compilation. This builtin is a known Nitro trap on the target. |
| In `demo_document.py`, change the drawing connection from `wire(dsp, 1)` to `wire(dsp, 0)`. | `ABI035` flags the stereo audio output wired to a drawing input that requires four lanes. The document can build and still fail this check. |
| In `bitwig-project.json`, change `defaultNormalized` from `1.0` to `0.5`. | `PANEL014` flags the mismatch with the serialized knob default. |

Each error ends with `FAIL (blocked-before-host-test)` and exit code 1.
Use the rule ID, source path or artifact finding to locate the mismatch.
Fix the source and rerun the gate before copying a file into Patchbay.
