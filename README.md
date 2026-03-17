# Rusty Samplers

A Rust converter that transforms Akai S5000/S6000 AKP sampler programs into modern formats (SFZ and Decent Sampler). Parses the full RIFF/APRG binary format including keygroups, envelopes, filters, LFOs, and the complete modulation matrix.

Tested against **2,632 factory AKP files** from all six Akai S6000 CD-ROM volumes with a 99.96% success rate (the single failure is a corrupted source file).

## Quick Start

### CLI

```bash
# Build
cargo build --release

# Convert to SFZ (default)
./target/release/rusty-samplers-cli my_sample.akp

# Convert to Decent Sampler
./target/release/rusty-samplers-cli --format ds my_sample.akp

# Batch convert a directory
./target/release/rusty-samplers-cli --batch ./samples/

# Batch convert to Decent Sampler
./target/release/rusty-samplers-cli --batch --format ds ./samples/
```

### GUI

```bash
cd gui && cargo run --release
```

Drag and drop AKP files, select output format, convert with progress tracking.

### Library

```rust
use rusty_samplers::{convert_file, OutputFormat};
use std::path::Path;

let result = convert_file(Path::new("input.akp"), OutputFormat::Sfz);
match result {
    Ok(content) => println!("{}", content),
    Err(e) => eprintln!("Error: {}", e),
}
```

## What Gets Converted

| AKP Feature | SFZ | Decent Sampler |
|---|---|---|
| Sample mapping (key/velocity zones, up to 4 per keygroup) | Yes | Yes |
| Amp envelope (ADSR + velocity/keyboard scaling) | Yes | Yes (with UI knobs) |
| Filter envelope (ADSR + depth) | Yes | Yes (with UI knobs) |
| Filter (26 types, cutoff, resonance, key tracking) | Yes | Yes (lowpass + resonance) |
| LFO 1 & 2 (9 waveforms, rate, delay, depth) | Yes | Yes |
| Volume (logarithmic dB conversion) | Yes | Yes |
| Velocity sensitivity | Yes | Yes |
| Pitch bend range | Yes | - |
| Full modulation matrix (17 flexible + 17 hardwired routes) | Yes | Partial |
| MIDI CC routing | - | Yes (CC1, CC2, CC7) |

### Modulation Matrix

The Akai S5000/S6000 has a powerful modulation system with 34 total routes:

**17 flexible routes** (source assignable from 15 options):
- Pitch mod 1 & 2, filter mod 1/2/3, amp mod + amp mod 1/2, pan mod 1/2/3, LFO1 rate/delay/depth mod, LFO2 rate/delay/depth mod

**17 hardwired routes** (fixed source, variable amount):
- Aftertouch → pitch, LFO modwheel/aftertouch shortcuts, velocity → attack/release, keyboard → envelope scaling, filter key tracking

**15 modulation sources**: No Source, Mod Wheel, Pitch Bend, Aftertouch, External, Velocity, Keyboard, LFO1, LFO2, Amp Env, Filter Env, Aux Env, MIDI Note, MIDI Velocity, MIDI Random

## Parameter Scaling

| Parameter | Method | Range |
|---|---|---|
| Envelope times (A/D/R) | Exponential curve | 0.001s - 30s |
| Envelope sustain | Linear | 0 - 100% |
| Filter cutoff | Logarithmic: `20 * 1000^(x/100)` | 20 Hz - 20 kHz |
| Filter resonance | Linear | 0 - 40 dB |
| LFO rate | Logarithmic | 0.1 Hz - 30 Hz |
| Volume | Logarithmic: `20 * log10(loudness/100)` | -60 dB - 0 dB |
| Modulation amounts | Bipolar normalized | Per-destination scaling |

## AKP Format Notes

The AKP format is poorly documented. The primary spec (reverse-engineered from S6000 OS v1.11) is at [burnit.co.uk/AKPspec](https://burnit.co.uk/AKPspec/). We found several issues during development that aren't covered by the spec:

- **Loudness is linear gain, not linear dB.** The spec lists loudness as a byte (0-100) but doesn't specify the unit. The correct conversion to dB is `20 * log10(loudness / 100)`, not a linear mapping.
- **Sample names have no file extension.** The 20-character sample name field in zone chunks does not include `.wav` — the extension must be appended. Some sample names contain dots as part of note names (e.g., `BRASS 02-C.1`), so extension detection must check for known audio suffixes, not just any dot.
- **Zone chunk sizes vary.** The spec says 46 bytes, but real files also use 48 bytes. Both must be handled.
- **Modulation source IDs 12-14** are listed as dMODWHEEL/dBEND/dEXTERNAL in the spec but appear to function as MIDI Note/MIDI Velocity/MIDI Random in practice.
- **The `smpl` chunk** is not in the official spec — it appears in files created by third-party tools and can be safely ignored.

See [docs/akp-format-reference.md](docs/akp-format-reference.md) for the full byte-level specification, chunk hierarchy, and cross-references with other implementations.

## Project Structure

```
rusty-samplers/
├── src/
│   ├── lib.rs            # Library root, re-exports, convert_file()
│   ├── error.rs          # AkpError enum and Result alias
│   ├── types.rs          # Data structures, parameter scaling, mod source helpers
│   ├── parser.rs         # RIFF/APRG binary parser
│   ├── sfz.rs            # SFZ output generation
│   ├── dspreset.rs       # Decent Sampler XML output generation
│   └── bin/
│       └── cli.rs        # CLI binary (clap)
├── gui/                  # GUI application (eframe/egui, separate crate)
├── tests/
│   └── integration_tests.rs
├── docs/
│   └── akp-format-reference.md
└── create_test_akp.py    # Test AKP file generator
```

## Testing

```bash
# All tests (54 unit + 5 integration = 59 total)
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Check both crates compile cleanly
cargo clippy --all-targets -- -D warnings
cd gui && cargo clippy -- -D warnings
```

## Building

Requires [Rust](https://rustlang.org/tools/install) 1.70 or later.

```bash
git clone https://github.com/ddri/rusty-samplers.git
cd rusty-samplers

# Library + CLI
cargo build --release

# GUI
cd gui && cargo build --release
```

## License

MIT
