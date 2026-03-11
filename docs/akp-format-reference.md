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

### Internet Archive

- **AKAI S6000 CD-ROM Volume 1**: https://archive.org/details/akai-s6000-cd-rom-volume-1
  - 1.3 GB, 23,375 files. Factory library for S6000. Should contain AKP files.
- **Akai CD-ROM Sound Library (8 CDs)**: https://archive.org/details/akai-cd-rom-sound-library-volume-1
  - 4.0 GB, 8 ISO images. S1000/S3000 format (NOT AKP).
- **Retro Sample CDs**: https://archive.org/details/retro-sample-cds
  - 25.5 GB mixed collection. Includes some AKP files.

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
    mods (38 bytes)  — Global modulation routing (15 mod source assignments)
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

| Value | Source |
|-------|--------|
| 0 | NO SOURCE |
| 1 | MODWHEEL |
| 2 | BEND |
| 3 | AFTERTOUCH |
| 4 | EXTERNAL |
| 5 | VELOCITY |
| 6 | KEYBOARD |
| 7 | LFO1 |
| 8 | LFO2 |
| 9 | AMP ENV |
| 10 | FILT ENV |
| 11 | AUX ENV |
| 12 | dMODWHEEL |
| 13 | dBEND |
| 14 | dEXTERNAL |

## ConvertWithMoss Cross-Check Notes

Cross-referenced against the Java source at `git-moss/ConvertWithMoss` (commit as of March 2026):

- **kgrp parsing approach**: ConvertWithMoss treats kgrp as a flat byte array with absolute offsets. Our recursive RIFF subchunk approach also works since the subchunk headers are present in real files. Either approach is valid.
- **tune/lfo/mods/out are program-level (top-level) chunks** — they appear BEFORE keygroups, not inside kgrp. ConvertWithMoss handles them at the top level. Our parser incorrectly looks for tune/lfo/mods inside kgrp.
- **Zone sizes**: ConvertWithMoss comments "Found 0x2E and 0x30" (46 and 48 bytes). The spec says 46, but real files can have 48. Our test files have 48-byte zones.
- **Zone detection**: ConvertWithMoss checks if the first byte of the zone chunk header is 0 to skip empty zone slots.
- **Envelope time mapping**: ConvertWithMoss uses linear `value / 100.0 * 6.0` (0-6 seconds). Comments say "No real idea, assume 6 seconds max." Neither our exponential mapping nor their linear mapping is spec-confirmed.
- **Resonance conversion**: ConvertWithMoss uses `Math.pow(Math.clamp(resonance / 12.0, 0, 1.0), 1.0 / 3.0) / 4.0` — cube root scaling, not linear.
- **ConvertWithMoss bug**: `getPanMod2()` and `getPanMod3()` both return `panMod1` (copy-paste bug in their code).

## Known Discrepancies in Current Parser

Documented here so we can track what needs fixing:

1. **Zone sample name field**: Parser reads 14 bytes, spec says 20 bytes
2. **Zone velocity offsets**: Parser uses 34-35 (empirically derived), spec may differ — needs verification against spec
3. **LFO waveform mapping**: Parser has 0=triangle, 1=sine; spec has 0=sine, 1=triangle
4. **Envelope layout**: Parser reads sequentially at offsets 2-5; spec shows non-sequential layout
5. **Filter type 0**: Parser treats as "off/bypass"; spec says it's "2-Pole LP" (an active filter)
6. **Top-level chunks**: tune (24 bytes), lfo, mods at program level are skipped/misparsed
7. **`smpl` chunk**: Not in the spec — likely from third-party tools, kept as fallback
