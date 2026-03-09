# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust application that converts Akai AKP (APRG) sampler program files to multiple modern sampler formats. The converter parses RIFF-based AKP files and extracts keygroups, samples, envelopes, filters, LFOs, and modulation data to generate equivalent SFZ files and Decent Sampler XML presets. It includes both a command-line interface and a modern GUI application.

## Architecture

The library is split into focused modules under `src/`:

- **Error types** (`src/error.rs`): Custom `AkpError` enum with Display impl, `From<io::Error>`, and `Result<T>` type alias. Variants: `InvalidRiffHeader`, `InvalidAprgSignature`, `UnknownChunkType`, `InvalidChunkSize`, `CorruptedChunk`, `InvalidKeyRange`, `InvalidVelocityRange`, `MissingRequiredChunk`, `InvalidParameterValue`.

- **Data structures** (`src/types.rs`): All pub structs representing AKP file components — `AkaiProgram`, `Keygroup`, `Sample`, `Tune`, `Filter`, `Envelope`, `Lfo`, `Modulation`, `RiffChunkHeader`, `OutputFormat` enum.

- **RIFF parser** (`src/parser.rs`): Binary file parsing with `validate_riff_header()` and `parse_top_level_chunks()` as the public entry points. Parses nested chunk structures (prg, kgrp, zone, smpl, tune, filt, env, lfo, mods). Uses `byteorder` crate for little-endian binary data.

- **SFZ generator** (`src/sfz.rs`): `impl AkaiProgram { to_sfz_string() }` — converts parsed data to SFZ format with exponential envelope curves, logarithmic filter/LFO scaling, and comprehensive modulation routing.

- **Decent Sampler generator** (`src/dspreset.rs`): `impl AkaiProgram { to_dspreset_string() }` — converts to Decent Sampler XML with UI controls (ADSR, filter, resonance knobs), effects chain, LFO modulators, and MIDI CC routing.

- **Library root** (`src/lib.rs`): Module declarations, re-exports (`AkpError`, `Result`, `AkaiProgram`, `OutputFormat`, `validate_riff_header`, `parse_top_level_chunks`), and `convert_file()` convenience function for GUI integration.

- **CLI binary** (`src/bin/cli.rs`): Command-line interface using `clap` derive for argument parsing. Supports single-file conversion, `--format` option, `--batch` directory mode, and progress bars via `indicatif`.

- **GUI application** (`gui/src/main.rs`): Separate crate using `eframe`/`egui`. Clickable drop zone with hover feedback, format selection using `rusty_samplers::OutputFormat` directly, custom output directory option, real-time progress tracking in bottom bar, threaded conversion.

## Project Structure

```
rusty-samplers/
├── src/
│   ├── lib.rs               # Library root: module declarations, re-exports, convert_file()
│   ├── error.rs             # AkpError enum, Display, From, Result alias
│   ├── types.rs             # All data structs + OutputFormat enum
│   ├── parser.rs            # RIFF/APRG parsing functions + parser unit tests
│   ├── sfz.rs               # SFZ generation + SFZ unit tests
│   ├── dspreset.rs          # Decent Sampler XML generation
│   └── bin/
│       └── cli.rs           # CLI binary (rusty-samplers-cli)
├── gui/                     # GUI application (separate crate)
│   ├── Cargo.toml           # Depends on eframe, egui, rfd, rusty-samplers
│   └── src/main.rs          # egui GUI with drag & drop, batch processing
├── tests/
│   └── integration_tests.rs # CLI integration tests (5 tests)
├── examples/
│   └── test_runner.rs       # Manual test utility for AKP conversion
├── create_test_akp.py       # Python script to generate test AKP files
├── CLAUDE.md
├── README.md
└── Cargo.toml               # Library + CLI binary, deps: byteorder, clap, indicatif
```

## Development Commands

### Build and Check
- `cargo check`: Check library + CLI for compilation errors (should produce zero warnings)
- `cargo build`: Build library and CLI binary
- `cd gui && cargo check`: Check GUI compilation
- `cd gui && cargo build`: Build GUI binary

### Run
- `cargo run --bin rusty-samplers-cli -- <file.akp>`: Convert single AKP to SFZ
- `cargo run --bin rusty-samplers-cli -- --format ds <file.akp>`: Convert to Decent Sampler
- `cargo run --bin rusty-samplers-cli -- --batch <directory>`: Batch convert directory
- `cd gui && cargo run`: Launch the GUI

### Test
- `cargo test`: Run all tests (25 unit + 5 integration = 30 total)
- `cargo test --lib`: Unit tests only
- `cargo test --test integration_tests`: Integration tests only

## Key Implementation Details

- **Envelope Scaling**: Exponential curves for attack/decay/release timing (`src/sfz.rs`)
- **Filter Cutoff**: Logarithmic scaling from AKP 0-100 to 20Hz–20kHz (`src/sfz.rs`)
- **LFO Rate**: Logarithmic conversion to 0.1Hz–30Hz (`src/sfz.rs`)
- **Modulation**: Bipolar normalization with per-destination scale factors, 13 sources × 18 destinations (`src/sfz.rs`)
- **Volume**: Linear mapping from AKP 0-100 to -60dB–+6dB range

## File Format Notes

### Input Format
- RIFF-based AKP files with APRG signature
- Nested chunk structure: prg > kgrp > zone > smpl/tune/filt/env/lfo/mods
- Little-endian binary data with specific parameter encoding

### Output Formats

#### SFZ Format (.sfz)
- Text-based sampler format with opcodes
- Forward-slash sample paths (converted from AKP backslashes)
- Advanced parameter mapping for envelopes, filters, and LFOs
- Modulation routing with source/destination assignments

#### Decent Sampler XML (.dspreset)
- Complete XML preset with `<DecentSampler>` root element
- UI controls: Attack, Decay, Sustain, Release, Filter, Resonance knobs
- Effects chain: Lowpass filter + Reverb with parameter bindings
- LFO modulators with waveform selection and MIDI CC routing
- Comprehensive sample mapping with velocity/key ranges
