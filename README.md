# Rusty Samplers

A multi-format sampler converter that transforms Akai AKP files into modern sampler formats including SFZ and Decent Sampler XML.

## Features

- **Multi-Format Output**: Convert to SFZ and Decent Sampler formats
- **Advanced Parameter Mapping**: Precise envelope, filter, and LFO conversion
- **GUI Interface**: Graphical interface with drag & drop and batch processing
- **CLI**: Command-line interface for scripting and automation
- **Comprehensive Conversion**: Handles samples, envelopes, filters, LFOs, and modulation
- **Batch Processing**: Convert multiple files or entire directories

## Quick Start

### GUI Application

```bash
cd gui
cargo run
```

- Drag & drop AKP files or use the file picker
- Select output format (SFZ or Decent Sampler)
- Batch conversion with progress tracking
- Hover feedback on drag & drop area

### Command Line Interface

```bash
# Single file conversion (default: SFZ)
cargo run --bin rusty-samplers-cli -- my_sample.akp

# Convert to Decent Sampler format
cargo run --bin rusty-samplers-cli -- --format ds my_sample.akp

# Batch convert a directory
cargo run --bin rusty-samplers-cli -- --batch ./samples/

# Batch convert to Decent Sampler
cargo run --bin rusty-samplers-cli -- --batch --format ds ./samples/

# Show help
cargo run --bin rusty-samplers-cli -- --help
```

### Library Usage

```rust
use rusty_samplers::{convert_file, OutputFormat};
use std::path::Path;

let result = convert_file(Path::new("input.akp"), OutputFormat::Sfz);
match result {
    Ok(content) => println!("{}", content),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Installation

### Prerequisites

- [Rust](https://rustlang.org/tools/install) (1.70 or later)

### Build from Source

```bash
git clone https://github.com/ddri/rusty-samplers.git
cd rusty-samplers

# Build library + CLI
cargo build --release

# Build GUI
cd gui && cargo build --release
```

## Supported Formats

### Input

- **AKP (Akai Program)**: RIFF-based Akai sampler program files

### Output

#### SFZ Format (.sfz)
- Standard text-based sampler format
- Compatible with most modern samplers
- Sample key/velocity mapping
- ADSR envelopes with exponential scaling
- Filter cutoff/resonance (20Hz-20kHz logarithmic)
- LFO modulation with waveform selection
- Modulation routing (13 sources, 18 destinations)
- Velocity tracking and dynamics

#### Decent Sampler XML (.dspreset)
- Native Decent Sampler preset format
- Interactive UI controls (Attack, Decay, Sustain, Release, Filter, Resonance)
- Effects chain (Lowpass filter + Reverb)
- MIDI CC bindings (CC1, CC2, CC7)
- LFO modulators with waveform selection

## Parameter Conversion

| Parameter | Scaling | Range |
|---|---|---|
| Envelope Attack/Decay/Release | Exponential | Musical response curve |
| Envelope Sustain | Linear | 0-100% |
| Filter Cutoff | Logarithmic | 20Hz - 20kHz |
| Filter Resonance | Linear | 0 - 40dB |
| LFO Rate | Logarithmic | 0.1Hz - 30Hz |
| Volume | Linear | -60dB - +6dB |
| Modulation | Bipolar normalized | Per-destination scaling |

### Modulation Sources
LFO1/2, Mod Wheel, Aftertouch, Key, Key Gate, Velocity, Pitch Bend, Channel Pressure, Polyphonic Pressure, Breath, Foot, Expression

### Modulation Destinations
Pitch, Filter Cutoff, Resonance, Volume, Pan, LFO Freq, Envelope Times, Envelope Sustain, LFO Depths

## Project Structure

```
rusty-samplers/
├── src/
│   ├── lib.rs            # Library root: module declarations, re-exports, convert_file()
│   ├── error.rs          # AkpError enum and Result type alias
│   ├── types.rs          # Data structures and OutputFormat enum
│   ├── parser.rs         # RIFF/APRG binary file parsing
│   ├── sfz.rs            # SFZ format generation
│   ├── dspreset.rs       # Decent Sampler XML generation
│   └── bin/
│       └── cli.rs        # CLI binary (rusty-samplers-cli)
├── gui/                  # GUI application (separate crate)
│   ├── Cargo.toml        # Depends on eframe, egui, rfd, rusty-samplers
│   └── src/main.rs       # egui GUI with drag & drop, batch processing
├── tests/
│   └── integration_tests.rs
├── examples/
│   └── test_runner.rs    # Manual test utility
└── create_test_akp.py    # Generate test AKP files
```

## Testing

```bash
# Run all tests (25 unit + 5 integration)
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests
```

## Troubleshooting

**"Invalid RIFF header"** — Ensure the file is a valid AKP file, not corrupted or truncated.

**"Missing required keygroup chunk"** — The AKP file may be empty or malformed.

**GUI won't start** — Run from the `gui/` directory: `cd gui && cargo run`. Ensure dependencies compile with `cargo build`.

**Build fails** — Update Rust (`rustup update`), then clean and rebuild (`cargo clean && cargo build`).

## License

This project is licensed under the MIT License - see the LICENSE file for details.
