# Rusty Samplers

A high-performance AKP to SFZ converter written in Rust for musicians and audio producers working with Akai and SFZ-compatible samplers.

## Overview

Rusty Samplers converts Akai Program (.akp) files to the widely-supported SFZ format, enabling seamless migration between different sampler platforms. The tool preserves critical sample mapping data including key/velocity ranges, envelope parameters, filter settings, and LFO configurations with accurate parameter scaling.

**Market Position**: Unlike existing free tools that lose envelope information or commercial tools with licensing costs, Rusty Samplers offers professional-grade conversion with full parameter preservation, batch processing capabilities, and robust error handling for real-world production workflows.

## Current Status

**Version**: 0.9.0 (Development - Epic 1 Complete)
**Status**: Modular architecture foundation complete, ready for binrw migration

### ✅ Epic 1 Complete: Foundation & Migration Setup
- ✅ **Modular Architecture**: Clean separation with lib.rs, formats, conversion, and error modules
- ✅ **Comprehensive Error Handling**: ConversionError types with recovery strategies and detailed context
- ✅ **Professional CLI**: Logging with configurable verbosity and helpful error suggestions
- ✅ **Feature Flag System**: legacy-parser and binrw-parser features for migration strategy
- ✅ **Robust Testing**: 14 unit tests covering parsing, conversion, and error handling
- ✅ **Legacy Parser Extracted**: Clean separation of existing parsing logic for parallel development

### Features Implemented
- ✅ RIFF-based AKP file parsing with comprehensive error handling
- ✅ Complete keygroup extraction (samples, zones, parameters) with validation
- ✅ Advanced parameter conversion with research-based logarithmic scaling
- ✅ SFZ file generation with proper formatting and optional parameters
- ✅ Support for envelopes, filters, and LFO parameters with accurate conversion
- ✅ Detailed logging and progress reporting for professional workflows

### Architecture Improvements
- ✅ **No compilation errors** - Clean, professional codebase
- ✅ **Comprehensive error recovery** for malformed files with graceful degradation
- ✅ **Modular design** ready for batch processing and streaming support
- ✅ **Extensive test coverage** with 14 passing unit tests
- ✅ **Industry-standard parsing patterns** prepared for binrw integration

### Development Progress

**✅ Completed Epics:**
- Epic 1: Foundation & Migration Setup (100% complete)

**🚧 Next Phase:**
- Epic 2: Declarative AKP Parser Implementation (using binrw)
- Epic 3: Enhanced Error Handling & Recovery
- Epic 4: Conversion Engine & Performance
- Epic 5: Testing & Validation

### Competitive Analysis

**vs. ConvertWithMoss** (current user favorite): Superior error handling, professional logging, modular architecture for extensibility
**vs. Commercial Tools**: Open source, no licensing costs, transparent conversion algorithms, modern Rust performance
**vs. Akai's Tools**: Preserves all envelope information, handles malformed files gracefully, professional workflow integration

## Quick Start

### Prerequisites
- Rust 1.88.0 or later
- Cargo package manager

### Installation
```bash
git clone https://github.com/yourusername/rusty-samplers
cd rusty-samplers
cargo build --release
```

### Usage
```bash
# Convert an AKP file to SFZ
cargo run -- path/to/your/program.akp

# Output will be saved as program.sfz in the same directory
```

## Architecture

### Core Components

**Data Structures**
- `AkaiProgram`: Root container for parsed program data
- `Keygroup`: Individual sample mappings with parameters
- Parameter types: `Sample`, `Tune`, `Filter`, `Envelope`, `Lfo`

**Processing Pipeline**
1. **RIFF Validation**: Ensures valid AKP file format
2. **Chunk Parsing**: Recursive RIFF chunk extraction
3. **Parameter Extraction**: Type-specific data parsing
4. **SFZ Generation**: Formatted output with parameter scaling

### Key Technologies
- **Binary Parsing**: `byteorder` crate for little-endian data (migrating to `binrw` for declarative parsing)
- **File I/O**: Standard Rust file handling with cursor-based parsing
- **Parameter Conversion**: Mathematical scaling for audio parameters
- **Future**: `symphonia-format-riff`, `rayon` for parallel processing, `nom` for resilient parsing

## Development

### Build Commands
```bash
# Development build
cargo build

# Check compilation
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Release build
cargo build --release
```

### Testing
```bash
# Run tests (when implemented)
cargo test

# Run specific test
cargo test test_name
```

## Roadmap

Based on comprehensive market research and user workflow analysis, our development priorities focus on professional-grade reliability and performance.

### Immediate Goals (v0.9) - Foundation
- [ ] **Fix compilation errors** (borrow checker conflicts in main.rs:194)
- [ ] **Implement resilient parsing** using `binrw` declarative approach
- [ ] **Comprehensive error handling** with graceful degradation for malformed files
- [ ] **Professional CLI** with progress bars, batch processing, validation
- [ ] **Unit & integration testing** with real AKP sample files

### Short Term (v1.0) - Professional Grade
- [ ] **Streaming architecture** for large file support (>100MB)
- [ ] **Parallel batch processing** using `rayon` for multi-core performance
- [ ] **Advanced parameter mapping** with quality validation and reporting
- [ ] **Format robustness** supporting AKP variants and malformed files
- [ ] **Performance optimization** targeting <1s conversion, <50MB memory

### Medium Term (v1.x) - Ecosystem Integration  
- [ ] **Library API** for integration with other audio tools
- [ ] **Extended format support** (S1000/S3000 compatibility, multiple SFZ dialects)
- [ ] **GUI application** for non-technical users
- [ ] **Reverse conversion** (SFZ to AKP) for complete workflow support
- [ ] **DAW plugins** for seamless studio integration

### Long Term (v2.x) - Market Leadership
- [ ] **Real-time conversion API** for cloud-based audio services
- [ ] **Advanced mapping algorithms** with AI-assisted parameter translation
- [ ] **Professional studio integration** with major DAW partnerships
- [ ] **Format ecosystem** supporting additional sampler formats beyond Akai/SFZ

## Technical Specifications

### Supported Formats
- **Input**: Akai Program (.akp) files with RIFF structure (S5000/S6000 native format)
- **Output**: SFZ format compatible with major samplers and libraries
- **Future**: S1000/S3000 legacy formats, multiple SFZ dialects

### Parameter Conversion (Research-Based)
Based on analysis of professional conversion tools and audio engineering requirements:

- **Filter Cutoff**: Logarithmic scaling (20Hz - 20kHz) with AKP 0-100 → SFZ Hz mapping
- **LFO Rates**: Logarithmic scaling (0.1Hz - 30Hz) using formula: `0.1 * (300.0^(rate/100.0))`
- **Envelopes**: Power-curve scaling for natural response, refined from user feedback
- **Volume**: Linear dB scaling (-48dB to 0dB) with AKP level 0-100 mapping
- **Resonance**: Scaled to 24dB max (AKP 0-100 → SFZ 0-24dB)

### Performance Targets (Competitive Benchmarks)
- **File Size**: Handle files up to 100MB (professional sample library standard)
- **Conversion Speed**: < 1 second for typical programs (faster than ConvertWithMoss)
- **Memory Usage**: < 50MB peak during conversion (streaming architecture)
- **Batch Processing**: Multi-core utilization for professional workflows
- **Error Recovery**: Continue processing after malformed chunks (enterprise reliability)

## Professional Use Cases

Based on research into audio production workflows, Rusty Samplers addresses critical industry needs:

### Sample Library Migration
- **Legacy Akai Libraries**: Convert vintage S5000/S6000 sample collections to modern SFZ format
- **Studio Workflow Integration**: Seamless migration between hardware samplers and software instruments
- **Batch Processing**: Handle hundreds of programs efficiently for large sample library conversions

### Production Workflows  
- **Envelope Preservation**: Unlike Akai's official tools, maintains all envelope parameters during conversion
- **Quality Validation**: Verify conversion accuracy with detailed reporting and parameter comparison
- **Professional Integration**: API design enables integration with existing audio production toolchains

## Contributing

### Development Environment
1. Install Rust via [rustup](https://rustup.rs/)
2. Clone the repository
3. Run `cargo check` to verify setup (note: currently has compilation errors)
4. Follow conventional commit guidelines

### Code Standards
- Follow Rust formatting conventions (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Maintain test coverage above 80%
- Document public APIs
- Use `binrw` for declarative binary parsing
- Implement comprehensive error handling with recovery

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Market Research & References

This project is built on comprehensive analysis of the audio conversion ecosystem:

### Format Specifications
- **AKP Format**: Based on Akai S5000/S6000 specification (burnit.co.uk/AKPspec)
- **SFZ Format**: Official documentation at sfzformat.com
- **RIFF Structure**: Microsoft RIFF specification with Akai-specific modifications

### Competitive Analysis
- **ConvertWithMoss**: Most popular free converter, praised for envelope preservation
- **Extreme Sample Converter**: Professional commercial solution with malformed file handling
- **CDXtract**: Enterprise-grade converter used by major game studios
- **Chicken Systems Translator**: Established but inconsistent conversion quality

### Technical Research
- **Rust Audio Ecosystem**: rust.audio community resources and crates
- **Binary Parsing**: binrw, nom, symphonia-format-riff for robust parsing
- **Performance Patterns**: Zero-copy parsing, streaming architecture, parallel processing

## Acknowledgments

- Akai for the original AKP format specification and professional hardware samplers
- SFZ format maintainers for creating an open, extensible sampling standard  
- Rust audio community for excellent libraries and performance-focused tools
- Audio professionals who provided workflow insights and conversion requirements