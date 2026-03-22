# User Guide

Rusty Samplers converts Akai S5000/S6000 AKP sampler program files to SFZ and Decent Sampler formats. It preserves key/velocity mapping, envelopes, filters, LFOs, and modulation routing. Tested against 2,632 factory files from all six Internet Archive S6000 volumes with a 99.96% success rate.

## Installation

No pre-built releases yet. Build from source with Rust installed:

```sh
# Build both CLI and library
cargo build --release

# Build the GUI (separate crate)
cd gui && cargo build --release
```

Binaries:
- CLI: `target/release/rusty-samplers-cli`
- GUI: `gui/target/release/rusty-samplers-gui`

## CLI Usage

### Single File

```sh
rusty-samplers-cli my_program.akp
```

Converts to SFZ by default. Output is written next to the input file with the new extension (e.g., `my_program.sfz`).

### Format Selection

```sh
rusty-samplers-cli --format ds my_program.akp
rusty-samplers-cli -f ds my_program.akp
```

Format aliases (case-insensitive):
- **SFZ**: `sfz`
- **Decent Sampler**: `ds`, `dspreset`, `decent`, `decentsampler`

### Batch Mode

```sh
rusty-samplers-cli --batch ./samples/
rusty-samplers-cli -b ./samples/
```

Recursively finds all `.akp` files (case-insensitive, matches both `.akp` and `.AKP`) and converts each one. A progress bar shows conversion status.

### CLI Reference

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `<input>` | — | Required | AKP file path, or directory when using `--batch` |
| `--format` | `-f` | `sfz` | Output format |
| `--batch` | `-b` | off | Batch convert all AKP files in a directory |

### Examples

```sh
# Convert a single file to SFZ
$ rusty-samplers-cli "Warm Strings.akp"
✔ [00:00:00] [########################################] 100/100 Created Warm Strings.sfz

# Convert to Decent Sampler format
$ rusty-samplers-cli -f ds "Warm Strings.akp"
✔ [00:00:00] [########################################] 100/100 Created Warm Strings.dspreset

# Batch convert an entire library
$ rusty-samplers-cli -b ./S6000_Factory/
Starting batch conversion of 142 files...

OK: Warm Strings.akp
OK: Brass Section.akp
...
[########################################] 142/142 files (100%) Batch conversion complete!

BATCH SUMMARY:
   Successful: 142
   Failed:     0
   Total:      142
```

## GUI Usage

Launch the GUI:

```sh
cd gui && cargo run --release
```

### Workflow

1. **Select format** — Choose SFZ or Decent Sampler using the radio buttons at the top.
2. **Add files** — Drag and drop `.akp` files onto the drop zone, or click it to open a file browser. The drop zone highlights when hovering with files.
3. **Set output directory** (optional) — Check "Custom output directory" and browse to a folder. When unchecked, output files are written next to the input files.
4. **Convert** — Click the Convert button. A progress bar in the bottom panel shows the current file and count.
5. **Review results** — After conversion, a summary shows how many succeeded and failed, with per-file details.

Files can be removed individually (× button) or all at once (Clear button) before converting.

## What Gets Converted — SFZ

| AKP Feature | SFZ Output | Notes |
|---|---|---|
| Key ranges | `lokey`, `hikey` | Exact mapping |
| Velocity ranges | `lovel`, `hivel` | Exact mapping |
| Amp envelope (ADSR) | `ampeg_attack`, `ampeg_decay`, `ampeg_sustain`, `ampeg_release` | Exponential timing curves |
| Filter envelope (ADSR + depth) | `fileg_attack`, `fileg_decay`, `fileg_sustain`, `fileg_release`, `fileg_depth` | Depth converted to cents |
| Aux envelope → pitch | `pitcheg_attack`, `pitcheg_decay`, `pitcheg_sustain`, `pitcheg_release` | 4-stage breakpoint approximated as ADSR (lossy) |
| 26 filter types | 7 SFZ types: `lpf_2p`, `bpf_2p`, `hpf_2p`, `hpf_1p`, `brf_2p`, `pkf_2p` | Morphing, phaser, voweliser fall back to `lpf_2p` |
| Filter cutoff | `cutoff` (Hz) | Logarithmic scaling, 20 Hz–20 kHz |
| Filter resonance | `resonance` (dB) | Direct mapping |
| 9 LFO waveforms | 5 SFZ waveforms: sine, triangle, square, saw, random | Phase variants collapsed (see Limitations) |
| LFO rate | `lfoN_freq` (Hz) | Logarithmic conversion, 0.1–30 Hz |
| LFO depth | `lfoN_pitch`, `fillfo_depth`, `amplfo_depth` | Per-destination scaling |
| Flexible mod routes | Native opcodes where available | ~10 routes map directly; others emitted as comments |
| Hardwired mod routes | `pitchlfo_depthcc1`, `pitch_chanaft`, `amplfo_depthcc1`, etc. | Modwheel, aftertouch, velocity |
| Volume (loudness 0–100) | `amplitude` | Logarithmic: `20 × log10(loudness / 100)`, 0 floors to −60 dB |
| Velocity sensitivity | `amp_veltrack` | Direct 1:1 mapping |
| Tuning (semitone + fine) | `transpose`, `tune` | Keygroup + zone tuning additive |
| Pitchbend range | `bend_up`, `bend_down` | Converted to cents |
| Playback mode | `loop_mode` | `no_loop`, `one_shot`, `loop_continuous`, `loop_sustain` |
| Pan | `pan` | Converted to −100..100 range |

### Modulation Sources

The converter maps these modulation sources to SFZ opcodes:

| Source | SFZ Suffix | Example |
|--------|-----------|---------|
| Modwheel (CC1) | `_oncc1` | `pitchlfo_depthcc1` |
| Pitch Bend | `_bend` | `pitch_bend` |
| Aftertouch | `_chanaft` | `pitch_chanaft` |
| External (CC16) | `_oncc16` | `cutoff_oncc16` |
| Velocity | Direct opcodes | `amp_veltrack`, `fil_veltrack` |
| LFO1, LFO2 | Dedicated opcodes | `amplfo_depth`, `fillfo_depth` |
| Amp/Filter/Aux Envelope | Dedicated opcodes | `fileg_depth`, `pitcheg_depth` |

Routes that have no SFZ1 equivalent (LFO cross-modulation, delta sources) are written as comments in the output file, preserving the information for manual editing.

## What Gets Converted — Decent Sampler

| AKP Feature | DS Output | Notes |
|---|---|---|
| Key ranges | `loNote`, `hiNote` | Exact mapping |
| Velocity ranges | `loVel`, `hiVel` | Exact mapping |
| Amp envelope | Group `attack`, `decay`, `sustain`, `release` | Sustain normalized 0–1 |
| Filter | Lowpass effect with UI knobs | All AKP filter types become lowpass |
| Filter envelope | Envelope modulator targeting `FX_FILTER_FREQUENCY` | With frequency translation table |
| LFO1 → filter | LFO modulator targeting filter cutoff | Only filter target supported in DS |
| Velocity → filter | Velocity modulator | When velocity is routed to filter |
| Modwheel → pan | CC1 modulator targeting PAN | When modwheel is routed to pan |
| Volume | Group `volume` attribute (dB) | Same logarithmic formula as SFZ |
| Velocity sensitivity | `ampVelTrack` (0–1) | Negative values clamped to 0 |
| Pan | Group `pan` attribute | Converted to −1..1 range |
| Tuning | Group `tuning` attribute | Semitone + fine cents |

### UI Controls

Every preset includes interactive knobs:

| Knob | Parameter | Range |
|------|-----------|-------|
| Attack | Amp envelope attack time | 0–10 s |
| Decay | Amp envelope decay time | 0–10 s |
| Sustain | Amp envelope sustain level | 0–1 |
| Release | Amp envelope release time | 0–10 s |
| Filter | Lowpass cutoff frequency | 20–22000 Hz |
| Resonance | Filter resonance | 0–5 |

When a filter envelope is present, four additional knobs appear: Filt Att, Filt Dec, Filt Sus, Filt Rel.

### MIDI CC Bindings

| CC | Parameter | Description |
|----|-----------|-------------|
| CC1 | Filter Cutoff | Modwheel controls filter |
| CC2 | Filter Resonance | Breath controls resonance |
| CC7 | Main Volume | Standard volume control |

### Effects Chain

Every DS preset includes:
- **Lowpass filter** — bound to the Filter and Resonance knobs
- **Reverb** — subtle room ambience

Unsupported modulation routes are written as XML comments, preserving them for manual editing.

## Known Limitations

### Filters

- 26 AKP filter types are reduced to 7 SFZ types or 1 DS type (lowpass only in Decent Sampler).
- Morphing filters (crossfading between filter modes), phasers, and the voweliser all fall back to lowpass.
- 4-pole AKP filters are approximated as 2-pole — SFZ and DS don't support 4-pole variants natively.

### LFO Waveforms

9 AKP waveforms collapse to 5 in SFZ:

| AKP | SFZ | What's Lost |
|-----|-----|-------------|
| SQ, SQ+, SQ− | square | Positive/negative phase offset |
| SAW_BI, SAW_UP, SAW_DN | saw | Direction and bipolar character |

### Aux Envelope

The AKP aux envelope is a 4-stage breakpoint envelope (rate1–4, level1–4) that doesn't map cleanly to ADSR. In SFZ, it's approximated as an ADSR pitch envelope. In Decent Sampler, it's not supported.

### Decent Sampler Modulation

DS has limited modulation compared to SFZ:

- Only 4 of the 17 flexible mod routes are supported natively (velocity→filter, modwheel→pan, LFO1→filter, filter envelope→filter).
- Pitch modulation, amplitude modulation, LFO cross-modulation, and most pan routes are emitted as XML comments only.
- LFO2 is not supported.
- Negative velocity sensitivity values are clamped to 0.

### Sample Paths

- The converter appends `.WAV` (uppercase) to sample names, matching most Akai factory files.
- Some libraries (e.g., the S6000 World Sounds volume) use lowercase `.wav` — on case-sensitive filesystems (Linux), you may need to rename files.
- Backslashes in AKP paths are converted to forward slashes.
- Drive letters and path traversal (`..`) are stripped for security.
- Subdirectory structure is preserved (e.g., `Strings/Violin_C3.WAV`).

### Not Mapped in Either Format

These AKP features have no equivalent in SFZ or Decent Sampler and are silently dropped:

- Mute groups
- Zone crossfade
- Output assignments
- FX send level
- Zone keyboard track
- Velocity-to-sample-start

## Troubleshooting

### "Invalid file format: Expected RIFF header but found different signature"

The file isn't an AKP program file, or it's corrupted. AKP files must start with a RIFF header followed by an APRG signature.

### "Invalid file format: Expected APRG signature but found different signature (not an Akai program file)"

The file has a valid RIFF header but isn't an Akai program. It may be a different RIFF-based format (WAV, AVI, etc.).

### "No .akp files found in directory: ..."

Batch mode searches recursively and matches `.akp`/`.AKP` case-insensitively. Check that your files actually have the `.akp` extension — some Akai software uses different extensions for multi-samples or effects.

### Silence in sampler after loading converted file

The most common cause is missing sample files. The converted SFZ/dspreset references WAV files by the names stored in the AKP, but you need the actual WAV files in the expected locations. Use the validation tool (see below) to check which samples are missing.

### Wrong volume levels

If converted presets are too quiet or too loud, ensure you're using the latest version. An early version used incorrect linear-dB conversion for the loudness parameter.

## Validation Tools

These are developer/power-user tools for verifying conversion quality.

### Sample Path Validator

Checks that all sample references in converted files resolve to actual files on disk:

```sh
cargo run --example validate_samples -- ./converted_output/
```

Reports three categories:
- **Found** — sample file exists at the expected path
- **Case mismatch** — file exists but with different casing (works on macOS/Windows, fails on Linux)
- **Missing** — sample file not found

Defaults to `test_akp_files/` if no directory is given.

### SFZ Diff Tool

Compares converter output against reference SFZ files (e.g., from ConvertWithMoss) to verify accuracy:

```sh
# Single file comparison
cargo run --example diff_sfz -- our_output.sfz reference.sfz

# Batch comparison against a reference directory
cargo run --example diff_sfz -- --dir tools/reference_sfz/ our_output_dir/
```

Comparison tolerances:
- Envelope times: within 20%
- Filter cutoff: within 10%
- Volume: within 1 dB
- Key/velocity ranges: exact
- Tuning: exact

Regions are matched by sample path (case-insensitive) and differences are reported per-opcode.
