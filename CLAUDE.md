# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rusty Samplers is an AKP to SFZ converter that parses Akai Program (.akp) files and converts them to SFZ format. The tool processes binary RIFF-based files containing audio sample program data and outputs text-based SFZ files compatible with various samplers.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the application (requires an .akp file as argument)
cargo run -- path/to/file.akp

# Check for compilation errors
cargo check

# Build for release
cargo build --release

# Format code
cargo fmt

# Run clippy for lints
cargo clippy
```

## Architecture Overview

The codebase follows a data-driven parsing approach:

### Core Data Structures
- `AkaiProgram`: Root container holding program header and keygroups
- `Keygroup`: Represents a sample mapping with key/velocity ranges and parameters
- Parameter structs: `Sample`, `Tune`, `Filter`, `Envelope`, `Lfo` for different aspects of sound synthesis

### Processing Flow
1. **RIFF Validation**: Validates file format and APRG signature in `validate_riff_header()`
2. **Chunk Parsing**: Recursively parses RIFF chunks using `parse_top_level_chunks()` and `parse_keygroup()`
3. **Data Extraction**: Specialized parsers for each chunk type (zone, smpl, tune, filt, env, lfo)
4. **SFZ Generation**: `to_sfz_string()` method converts parsed data to SFZ format with parameter scaling

### Key Implementation Details
- Uses `byteorder` crate for little-endian binary parsing
- RIFF chunk-based parsing with `RiffChunkHeader` structure
- Parameter conversion includes logarithmic scaling for frequency values and dB conversions
- Binary cursor-based parsing for efficient chunk data processing

## Current Status (v0.9.0)

✅ **Foundation Complete**: Modular architecture with comprehensive error handling
- All compilation errors resolved
- Clean modular structure with separate format, conversion, and error modules
- Professional logging with configurable verbosity
- Feature flags for parser selection (legacy-parser, binrw-parser)

## Migration Progress

**Epic 1 ✅ COMPLETED**: Foundation & Migration Setup
- ✅ Modular project structure
- ✅ binrw dependency with feature flags
- ✅ Comprehensive error handling with ConversionError types
- ✅ Professional CLI with logging
- ✅ Clean separation of AKP legacy parser

**Next Phase**: Epic 2 - Declarative binrw Parser Implementation