# VU Meter

A native stereo needle VU meter for Bitwig Studio, in the spirit of Klanghelm VUMT.
Audio passes through unchanged (apart from Trim).

![VU Meter](previews/vu-meter.png)

## Device panel

- **Two needle meters**: left and right, or mid and side (set in the Inspector).
- **Readouts**: the held peak of each meter in dB, top right of each meter.
- **Clip LED**: next to each meter's letter. Amber when the signal passes the Clip LED
  threshold, red at 0 dBFS.
- **Ref**: the 0 VU reference, the sine peak level in dBFS that reads 0 VU.
  -30 to 0 dB, default -18 dB.
- **Trim**: gain applied to the audio and the meter. -24 to +24 dB, default 0 dB.

## Inspector settings

Select the device and open Bitwig's Inspector for the set-and-forget settings.

| Section | Setting | What it does | Default |
| --- | --- | --- | --- |
| Meter | Type | VU, PPM (DIN) or RMS ballistics | VU |
| Meter | Channels | Meter L / R or M / S | L / R |
| Ballistics | Overshoot | VU needle overshoot: 0 % is critically damped, 1.5 % is classic (0 to 10 %) | 1.5 % |
| Peak Hold | Hold | Show the peak-hold marker | On |
| Peak Hold | Values | Show the numeric readouts | On |
| Peak Hold | Time | How long the peak hold stays (0 to 10 s) | 2 s |
| Clip LED | Threshold | Sample-peak level that lights the LED amber; red is always 0 dBFS (-24 to 0 dB) | -3 dB |
| Output | Force Mono | Sum the output to mono | Off |

Every setting can be automated and modulated.

## Mini view in the mixer

With the [Mixer: Device Mini Panels](https://github.com/cimmmusic/bitwig-mods/tree/perf/mixer-device-mini-panels)
mod installed, VU Meter gets a mini view under its name in the mixer's Devices section,
so every channel shows a live meter without opening the device:

- One needle meter across the channel strip, following the louder of the device's two
  meters (L / R or M / S).
- The device's own look: grey scale with ticks, orange 0 to +6 zone, cream needle.
- An orange peak-hold marker when the held peak is above the needle.
- The needle is smoothed at 60 fps, so it moves like a needle rather than in steps.

Turn on the mixer's Devices section (the toggles at the bottom of the mixer) to see it.

## Build

Built against Bitwig Studio 6.1.7 on macOS (Apple silicon). Build and load it the
same way as the blueprint device: see [BUILD.md](../../docs/BUILD.md) and
[BITWIG-TEST.md](../../docs/BITWIG-TEST.md).

Opening a project with VU Meter in a Bitwig that doesn't have it installed can crash
Bitwig. Use [`scripts/share_safe_copy.py`](../../scripts/share_safe_copy.py) to make
a copy of a project without it.
