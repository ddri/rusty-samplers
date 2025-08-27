# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust application that converts Akai AKP (APRG) sampler program files to SFZ format. The converter parses RIFF-based AKP files and extracts keygroups, samples, envelopes, filters, LFOs, and modulation data to generate equivalent SFZ files.

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

- **SFZ Generator** (`src/main.rs:110-212`): Converts parsed AKP data to SFZ format
  - Implements refined parameter scaling formulas for envelopes, filters, and LFOs
  - Maps Akai-specific parameters to SFZ opcodes
  - Handles modulation routing with basic source/destination mapping

## Development Commands

- `cargo run -- <file.akp>`: Run the converter on an AKP file
- `cargo build`: Build the application
- `cargo check`: Check for compilation errors without building
- `cargo test`: Run tests (none currently implemented)

## Key Implementation Details

- **Envelope Scaling**: Uses exponential curves for attack/decay/release timing (`src/main.rs:146,147,149`)
- **Filter Cutoff**: Logarithmic scaling from AKP 0-100 to Hz range (`src/main.rs:162`)
- **LFO Rate**: Logarithmic conversion to Hz frequency (`src/main.rs:173,177`)
- **Modulation**: Basic linear scaling with placeholder source/destination mappings (`src/main.rs:182-205`)

## File Format Notes

- Input: RIFF-based AKP files with APRG signature
- Output: Text-based SFZ files with forward-slash sample paths
- Sample references in AKP use backslashes, converted to forward slashes for SFZ compatibility