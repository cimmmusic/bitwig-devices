# Cimm's Bitwig devices

Native Bitwig Studio devices by Cimm, made with polarity's
[Bitwig device blueprint](https://github.com/polarity/bitwig-devices-blueprint).

## Devices

- **[VU Meter](devices/cimm-vu/)**: a stereo needle VU meter with VU, PPM and RMS
  ballistics, L/R or M/S, peak hold and a clip LED. Audio passes through.

[![VU Meter](devices/cimm-vu/previews/vu-0vu.png)](devices/cimm-vu/)

Mods for Bitwig's interface live in [bitwig-mods](https://github.com/cimmmusic/bitwig-mods).

---

## About the blueprint

A small, source-built Bitwig Studio Audio FX for people and coding agents.
**Blueprint Gain** combines a native gain knob, original stereo DSP in Nitro,
and a SkSL output display. The reference target is **Bitwig Studio 6.1.3 on
Windows x64**.

1. [Set up and run the offline preflight](docs/BUILD.md).
2. [Adapt the DSP, graph or shader panel](docs/AUTHORING.md).
3. [Load the verified artifact through Patchbay and test it](docs/BITWIG-TEST.md).

Point a coding agent at [AGENTS.md](AGENTS.md). Builds use the compiler JARs
from your own Bitwig installation.
