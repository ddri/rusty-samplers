# AKP Format Reference

Research notes on the Akai AKP (APRG) sampler program file format, covering model compatibility, format specifications, and reference implementations.

## Model Compatibility

### AKP-compatible models (RIFF/APRG format)

| Model | Era | Notes |
|-------|-----|-------|
| S5000 | 1999 | Introduced the AKP format |
| S6000 | 1999 | Same format as S5000; primary spec was reverse-engineered from S6000 OS v1.11 |
| Z4 | 2002 | Same base structure, adds extra filter chunks (filter 2, filter 3) |
| Z8 | 2002 | Same as Z4; supports three filters per program |
| MPC4000 | 2002 | Same base structure, adds pad assignment chunk before `prg ` |

**Current project scope: S5000/S6000 only.** Z4/Z8 and MPC4000 are future work.

### NOT AKP (different formats entirely)

| Model | Format |
|-------|--------|
| S900/S950 | Proprietary binary |
| S1000/S1100/S2000 | Proprietary disk format, 150-byte program + 150-byte keygroup blocks |
| S3000/S3000XL/S3200 | Extended S1000 format, 256-byte blocks |
| MPC5000/Live/Force | Can *read* AKP loosely, but native formats differ |

## Format Specification Sources

### Primary: burnit.co.uk AKP Spec

- **URL**: https://burnit.co.uk/AKPspec/
- **Mirror**: http://mda.smartelectronix.com/akai/AKPspec.html
- Reverse-engineered from S6000 running OS v1.11
- Complete byte-level specification for all chunk types
- Includes default values, valid ranges, and enum definitions
- Notes: "an S5000 or an older OS may not support everything in this format, but the structure should be identical"

### Akai Disk & File Format Overview

- **URL**: http://mda.smartelectronix.com/akai/akaiinfo.htm
- Broader overview covering S1000/S3000 disk formats alongside S5000/AKP info

### S1000/S3000 SysEx-Based Documentation (out of scope but useful for context)

- S2800/S3000/S3200: https://lakai.sourceforge.net/docs/s2800_sysex.html
- S2000/S3000XL/S3200XL: https://lakai.sourceforge.net/docs/s2000_sysex.html
- S1000: https://lakai.sourceforge.net/docs/s1000_sysex.html
- S3000 disk format: https://lsnl.jp/~ohsaki/software/akaitools/S3000-format.html

## Reference Implementations

### ConvertWithMoss (Java, LGPLv3) — Most mature

- **URL**: https://github.com/git-moss/ConvertWithMoss
- Full AKP parser at `src/main/java/de/mossgrabers/convertwithmoss/format/akai/akp/`
- Handles all standard chunks: `prg `, `out `, `mods`, `lfo `, `tune`, `kgrp`, `kloc`, `env `, `filt`, `zone`
- Also handles AKM (multi) files
- Detects S5000-series vs Z-series via RIFF header size field (0 = S5000)
- Also has separate S1000/S3000 format support

### akai-akp-python (Python)

- **URL**: https://github.com/matchalunatic/akai-akp-python
- Modules: `akaiakp` (core parser), `akairaw`, `akaixpm`, `akptoxpm`
- Less mature than ConvertWithMoss

### akp-maker (JavaScript, MIT)

- **URL**: https://github.com/cornelius-k/akp-maker
- Creates AKP files for S5000/S6000 (write direction, not read)
- Minimal project

### aksy (Python)

- **URL**: https://github.com/watzo/aksy
- Python USB API for controlling Akai samplers (Z4, Z8, MPC4000)
- Contains extensive model-specific parameter definitions

### akai-fs (Rust)

- **URL**: https://github.com/dialtr/akai-fs
- Rust access to Akai S900/S1000/S3000 filesystems (not AKP, but Rust ecosystem)

## Test File Sources

### Currently in repo

- `test_akp_files/` — 4 factory S6000 piano AKP files (CS_PF_PAD, CS_PIANO, HONKY_PF, ST_GRAND_PF)
- All use the same PNO93L multisamples with different envelope/filter/effects settings

### Internet Archive — Tested Against

All six S6000 CD-ROM volumes have been tested (2,648 AKP files, 99.96% pass rate):

| Archive Item | AKP Files | WAV Files | Notes |
|---|---|---|---|
| [akai-s6000-cd-rom-volume-1](https://archive.org/details/akai-s6000-cd-rom-volume-1) | 1,877 | ~6,000 | Main factory library. 1 corrupted file. |
| [akai-s6000-cd-rom-volume-2](https://archive.org/details/akai-s6000-cd-rom-volume-2) | 10 | ~40 | Expansion disc |
| [akai-s6000-cd-rom-volume-4](https://archive.org/details/akai-s6000-cd-rom-volume-4) | 56 | ~200 | Expansion disc |
| [akai-s6000-cd-rom-volume-6](https://archive.org/details/akai-s6000-cd-rom-volume-6) | 139 | ~400 | Expansion disc |
| [akai-s6000-cd-rom-ultimate](https://archive.org/details/akai-s6000-cd-rom-ultimate) | 61 | ~250 | Compilation disc |
| [akai-s6000-cd-rom-worldsounds](https://archive.org/details/akai-s6000-cd-rom-worldsounds) | 505 | ~600 | Uses lowercase `.akp` |

Download AKP files with the Internet Archive CLI: `ia download <item-id> --glob="*.AKP" --glob="*.akp"`

### NOT S6000 AKP

- **[akai-cd-rom-sound-library-volume-1](https://archive.org/details/akai-cd-rom-sound-library-volume-1)** — S1000/S3000 format (ISO images, NOT AKP)
- **Retro Sample CDs** — Mixed collection, may contain some AKP files

### Community Archives (Z4/Z8, MPC4000 — future scope)

- **Akai Z4/Z8 Archive**: http://zine.r-massive.com/akai-z4-z8-archive/
  - Factory sound libraries in 24-bit format. Hosted on Mega.nz.
- **Akai MPC4000 Archive**: http://zine.r-massive.com/akai-mpc4000-archive/
  - Factory demo library. Hosted on Mega.nz.

## AKP Chunk Hierarchy (S5000/S6000)

From the burnit.co.uk spec:

```
RIFF (size field may be 0x00000000 in S5000-series)
  APRG (form type)
    prg  (6 bytes)   — Program metadata
    out  (8 bytes)   — Output: loudness, amp mod, pan mod, velocity sensitivity
    tune (22 bytes)  — Tuning: semitone, fine, per-note detuning, pitchbend, aftertouch
    lfo  (12 bytes)  — LFO 1
    lfo  (12 bytes)  — LFO 2
    mods (38 bytes)  — Global modulation routing (17 source assignments for flexible routes)
    kgrp (variable, repeats 1-99 times)
      kloc (16 bytes)  — Key location: low/high note, tune, FX, zone crossfade, mute group
      env  (18 bytes)  — Amp envelope
      env  (18 bytes)  — Filter envelope
      env  (18 bytes)  — Aux envelope
      filt (10 bytes)  — Filter: mode (0-25), cutoff, resonance, key track, mod inputs
      zone (46 bytes)  — Zone 1: sample name (20 chars), velocity, tune, filter, pan, playback
      zone (46 bytes)  — Zone 2
      zone (46 bytes)  — Zone 3
      zone (46 bytes)  — Zone 4
```

## Spec Corrections and Discoveries

Issues discovered through testing against 2,632 factory S6000 files from all six Akai S6000 CD-ROM volumes. These are not documented in the burnit.co.uk spec and were found empirically.

### Loudness is Linear Gain (not linear dB)

The `out` chunk byte 1 is "loudness" (0-100, default 85). The spec doesn't say what the unit is. Testing in Decent Sampler confirmed it's a **linear gain percentage**, not a perceptual or dB value.

Correct dB conversion: `20.0 * log10(loudness / 100.0)`

| Loudness | dB |
|----------|-----|
| 100 | 0.0 |
| 85 | -1.4 |
| 68 | -3.4 |
| 50 | -6.0 |
| 25 | -12.0 |
| 0 | -60.0 (floor) |

An incorrect linear mapping like `(loudness/100) * 66 - 60` produces values 10-12 dB too quiet across the board.

### Sample Names Have No File Extension

The 20-character sample name field in zone chunks stores just the name (e.g., `BRASS 02-C.1`) with **no `.wav` extension**. Converters must append the extension.

Complication: some sample names contain dots as part of note notation (e.g., `C.1` meaning octave 1). Naive checks like `name.contains('.')` will fail. Instead, check if the string after the last dot matches a known audio extension (`wav`, `aif`, `aiff`, `flac`, `ogg`, `mp3`). If it doesn't, append `.wav`.

### Zone Chunk Size Varies

The spec says zone chunks are 46 bytes. Real files also have 48-byte zones. Both sizes are valid and must be handled. ConvertWithMoss comments confirm this: "Found 0x2E and 0x30" (46 and 48 bytes).

### Modulation Source IDs 12-14

The spec lists these as "delta" sources (dMODWHEEL, dBEND, dEXTERNAL). In practice, they appear to function as MIDI Note, MIDI Velocity, and MIDI Random respectively. ConvertWithMoss does not map these to delta controllers either.

### The `smpl` Chunk

Not in the official spec. Appears in files created by third-party tools (not from the S6000 itself). Contains a sample name in a different format. Can be safely used as a fallback if no zone chunks are found, but standard files always have zone chunks.

### ConvertWithMoss Copy-Paste Bug

ConvertWithMoss's `getPanMod2()` and `getPanMod3()` both return `panMod1` — this is a copy-paste bug in their code. Their pan modulation routing for sources 2 and 3 reads the wrong field.

## Known Enumerations

### Filter Types (26 values)

| Value | Type | SFZ Mapping |
|-------|------|-------------|
| 0 | 2-Pole LP | lpf_2p |
| 1 | 4-Pole LP | lpf_2p (approx) |
| 2 | 2-Pole LP+ | lpf_2p |
| 3 | 2-Pole BP | bpf_2p |
| 4 | 4-Pole BP | bpf_2p (approx) |
| 5 | 2-Pole BP+ | bpf_2p |
| 6 | 1-Pole HP | hpf_1p |
| 7 | 2-Pole HP | hpf_2p |
| 8 | 1-Pole HP+ | hpf_1p |
| 9 | LO<>HI | lpf_2p (fallback) |
| 10 | LO<>BAND | lpf_2p (fallback) |
| 11 | BAND<>HI | hpf_2p (fallback) |
| 12 | NOTCH 1 | brf_2p |
| 13 | NOTCH 2 | brf_2p |
| 14 | NOTCH 3 | brf_2p |
| 15 | WIDE NOTCH | brf_2p |
| 16 | BI-NOTCH | brf_2p |
| 17 | PEAK 1 | pkf_2p |
| 18 | PEAK 2 | pkf_2p |
| 19 | PEAK 3 | pkf_2p |
| 20 | WIDE PEAK | pkf_2p |
| 21 | BI-PEAK | pkf_2p |
| 22 | PHASER 1 | lpf_2p (fallback) |
| 23 | PHASER 2 | lpf_2p (fallback) |
| 24 | BI-PHASE | lpf_2p (fallback) |
| 25 | VOWELISER | lpf_2p (fallback) |

### LFO Waveforms (9 values)

| Value | Waveform | SFZ Equivalent |
|-------|----------|----------------|
| 0 | SINE | — |
| 1 | TRIANGLE | — |
| 2 | SQUARE | — |
| 3 | SQUARE+ | — |
| 4 | SQUARE- | — |
| 5 | SAW BI | — |
| 6 | SAW UP | — |
| 7 | SAW DOWN | — |
| 8 | RANDOM | — |

### Modulation Sources (15 values)

| Value | Spec Name | Actual Behavior | SFZ Mapping |
|-------|-----------|-----------------|-------------|
| 0 | NO SOURCE | None | — |
| 1 | MODWHEEL | CC1 | `_oncc1` |
| 2 | BEND | Pitch bend | `_bend` |
| 3 | AFTERTOUCH | Channel aftertouch | `_chanaft` |
| 4 | EXTERNAL | CC16 (general purpose) | `_oncc16` |
| 5 | VELOCITY | Note velocity | Hardwired |
| 6 | KEYBOARD | Key number | Hardwired |
| 7 | LFO1 | LFO 1 output | Hardwired |
| 8 | LFO2 | LFO 2 output | Hardwired |
| 9 | AMP ENV | Amp envelope output | Hardwired |
| 10 | FILT ENV | Filter envelope output | Hardwired |
| 11 | AUX ENV | Aux envelope output | Hardwired |
| 12 | dMODWHEEL | MIDI Note (see corrections above) | — |
| 13 | dBEND | MIDI Velocity (see corrections above) | — |
| 14 | dEXTERNAL | MIDI Random (see corrections above) | — |

### Modulation Destinations (17 flexible routes)

These are the 17 assignable modulation connections in the `mods` chunk. Each stores a source ID (0-14) from the table above. The modulation amount is stored in the relevant chunk (kloc, out, filt, or lfo).

| Route | Amount Location | Description |
|-------|----------------|-------------|
| pitch_mod_1 | kloc byte 8 | Pitch modulation route 1 |
| pitch_mod_2 | kloc byte 9 | Pitch modulation route 2 |
| filter_mod_1 | filt byte 7 | Filter cutoff mod route 1 |
| filter_mod_2 | filt byte 8 | Filter cutoff mod route 2 |
| filter_mod_3 | filt byte 9 | Filter cutoff mod route 3 |
| amp_mod | kloc byte 10 | Amplitude modulation |
| amp_mod_1 | out byte 2 | Amplitude mod route 1 |
| amp_mod_2 | out byte 3 | Amplitude mod route 2 |
| pan_mod_1 | out byte 4 | Pan modulation route 1 |
| pan_mod_2 | out byte 5 | Pan modulation route 2 |
| pan_mod_3 | out byte 6 | Pan modulation route 3 |
| lfo1_rate_mod | lfo1 byte 7 | LFO1 rate modulation |
| lfo1_delay_mod | lfo1 byte 8 | LFO1 delay modulation |
| lfo1_depth_mod | lfo1 byte 9 | LFO1 depth modulation |
| lfo2_rate_mod | lfo2 byte 7 | LFO2 rate modulation |
| lfo2_delay_mod | lfo2 byte 8 | LFO2 delay modulation |
| lfo2_depth_mod | lfo2 byte 9 | LFO2 depth modulation |

Additionally, there are **17 hardwired routes** where the source is fixed (not assignable):
- Aftertouch → pitch (tune chunk)
- LFO modwheel/aftertouch shortcuts (lfo chunk)
- Velocity → attack time, on/off velocity → release (amp env)
- Keyboard → envelope time scaling (amp env)
- Filter key tracking (filt chunk)

## ConvertWithMoss Cross-Check Notes

Cross-referenced against the Java source at `git-moss/ConvertWithMoss` (commit as of March 2026):

- **kgrp parsing**: ConvertWithMoss treats kgrp as a flat byte array with absolute offsets. Our recursive RIFF subchunk approach also works since the subchunk headers are present in real files. Either approach is valid.
- **Zone sizes**: ConvertWithMoss comments "Found 0x2E and 0x30" (46 and 48 bytes). The spec says 46, but real files can have 48. Both must be handled.
- **Zone detection**: ConvertWithMoss checks if the first byte of the zone chunk header is 0 to skip empty zone slots.
- **Envelope time mapping**: ConvertWithMoss uses linear `value / 100.0 * 6.0` (0-6 seconds). Comments say "No real idea, assume 6 seconds max." We use exponential curves. Neither mapping is spec-confirmed — the S6000 OS source code is not available.
- **Resonance conversion**: ConvertWithMoss uses `Math.pow(Math.clamp(resonance / 12.0, 0, 1.0), 1.0 / 3.0) / 4.0` — cube root scaling. We use linear scaling. Neither is spec-confirmed.
- **ConvertWithMoss bug**: `getPanMod2()` and `getPanMod3()` both return `panMod1` (copy-paste bug in their code). Our parser reads the correct offsets for all three pan mod sources.

## Parser Status

All discrepancies from the initial implementation have been resolved. The parser is now spec-aligned and tested against 2,632 factory S6000 files across all six Akai S6000 CD-ROM volumes:

| Volume | Files | Pass |
|--------|-------|------|
| Volume 1 | 1,877 | 1,876 (1 corrupted source) |
| Volume 2 | 10 | 10 |
| Volume 4 | 56 | 56 |
| Volume 6 | 139 | 139 |
| Ultimate | 61 | 61 |
| World Sounds | 505 | 505 |
| **Total** | **2,648** | **2,647 (99.96%)** |

The single failure (`TRANS FL-LED.AKP` from Volume 1) is a corrupted source file filled entirely with `0x22` bytes.

Resolved issues from earlier development:
- Zone sample name: reads 20 bytes (spec-correct)
- LFO waveform: 0=sine, 1=triangle (spec-correct)
- Envelope layout: reads all 18 bytes with spec offsets
- Filter type 0: treated as 2-Pole LP (spec-correct)
- Top-level chunks: tune, lfo, mods, out all parsed at program level
- `smpl` chunk: kept as fallback for third-party files
