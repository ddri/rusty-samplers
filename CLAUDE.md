# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust application that converts Akai AKP (APRG) sampler program files to multiple modern sampler formats. The converter parses RIFF-based AKP files and extracts keygroups, samples, envelopes, filters, LFOs, and modulation data to generate equivalent SFZ files and Decent Sampler XML presets. It includes both a command-line interface and a modern GUI application.

## Architecture

The application is structured around these core components:

- **Data Structures** (`src/main.rs:31-108`): Rust structs representing AKP file components
  - `AkaiProgram`: Top-level container for the entire program
  - `Keygroup`: Individual keygroup with sample, tuning, filter, envelopes, LFOs, and modulation data
  - Component structs: `Sample`, `Tune`, `Filter`, `Envelope`, `Lfo`, `Modulation`

- **RIFF Parser** (`src/main.rs:252-408`): Handles binary file parsing
  - Validates RIFF/APRG headers
  - Parses nested chunk structures (prg, kgrp, zone, smpl, tune, filt, env, lfo, mods)
  - Uses `byteorder` crate for little-endian binary data

- **Multi-Format Generator**: Converts parsed AKP data to multiple output formats
  - **SFZ Generator** (`src/main.rs:110-212`): Converts to SFZ format with refined parameter scaling
  - **Decent Sampler Generator**: Converts to Decent Sampler XML with UI controls and effects
  - Implements exponential envelope curves, logarithmic filter/LFO scaling
  - Advanced modulation routing with comprehensive source/destination mapping

- **Error Handling** (`src/main.rs`): Custom AkpError enum for robust error reporting
  - InvalidRiffHeader, InvalidAprgSignature, UnknownChunkType
  - InvalidKeyRange, InvalidVelocityRange, MissingRequiredChunk
  - Detailed error messages for debugging malformed AKP files

- **GUI Application** (`gui/src/main.rs`): Modern egui-based interface
  - Drag & drop file selection with multi-file support
  - Format selection (SFZ/Decent Sampler) with format descriptions  
  - Batch processing mode with progress tracking
  - Real-time conversion results with success/failure reporting

## Development Commands

### CLI Application (Note: Currently library-only, no standalone CLI binary)
The CLI functionality is implemented but main.rs serves as a library. CLI commands from the original design:
- Convert single AKP file to SFZ format
- Convert to Decent Sampler format with `--format ds`
- Batch convert entire directories with `--batch`

### GUI Application (Fully Functional)
- `cd gui && cargo run`: Launch the GUI interface with full AKP conversion
- `cargo build`: Build the GUI binary (rusty-samplers-gui)
- GUI includes: drag & drop, format selection, batch processing, progress tracking

### Library Integration
- `src/lib.rs`: Provides `convert_file()` function for GUI integration
- `src/main.rs`: Contains core AKP parsing and conversion logic (now public)
- GUI uses actual conversion logic via library interface

### Development and Testing
- `cargo build`: Build library and GUI application
- `cargo check`: Check for compilation errors
- `cargo test`: Run tests (basic test suite implemented)

## Key Implementation Details

- **Envelope Scaling**: Uses exponential curves for attack/decay/release timing (`src/main.rs:146,147,149`)
- **Filter Cutoff**: Logarithmic scaling from AKP 0-100 to Hz range (`src/main.rs:162`)
- **LFO Rate**: Logarithmic conversion to Hz frequency (`src/main.rs:173,177`)
- **Modulation**: Basic linear scaling with placeholder source/destination mappings (`src/main.rs:182-205`)

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

## Project Structure

```
rusty-samplers/
├── src/
│   ├── main.rs              # Core conversion logic (public library functions)
│   ├── lib.rs               # Library interface for GUI integration
│   └── bin/
│       └── simple_gui.rs    # Alternative GUI launcher (unused)
├── gui/                     # Primary GUI application (fully functional)
│   ├── Cargo.toml          # GUI-specific dependencies + rusty-samplers lib
│   └── src/main.rs         # Complete GUI with real AKP conversion
├── tests/                   # Integration tests
├── CLAUDE.md               # This documentation file  
├── README.md               # User-facing documentation
└── Cargo.toml              # Main project configuration
```

## Current Status (Latest Update)

**✅ Fully Functional**: The project now has a complete working GUI that performs real AKP file conversion.

**Key Achievements**:
- GUI integrated with actual conversion logic (not simulation)
- Error handling propagates from core library to GUI interface  
- File writing system works with real converted content
- Multi-format support (SFZ + Decent Sampler) fully operational
- Drag & drop, batch processing, and progress tracking all working

**Architecture Notes**:
- `src/main.rs` serves as core library with public functions
- `src/lib.rs` provides clean interface for GUI consumption  
- `gui/` contains standalone application using the core library
- Both compilation and runtime integration verified working