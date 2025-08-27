# 🎵 Rusty Samplers

A powerful multi-format sampler converter that transforms Akai AKP files into modern sampler formats including SFZ and Decent Sampler XML.

## ✨ Features

- **Multi-Format Output**: Convert to SFZ and Decent Sampler formats
- **Advanced Parameter Mapping**: Precise envelope, filter, and LFO conversion
- **Modern GUI Interface**: User-friendly graphical interface with drag & drop
- **Command Line Interface**: Powerful CLI for batch processing and scripting  
- **Comprehensive Conversion**: Handles samples, envelopes, filters, LFOs, and modulation
- **Batch Processing**: Convert multiple files or entire directories
- **Error Handling**: Robust error reporting and validation
- **Progress Tracking**: Real-time conversion progress indicators

## 🚀 Quick Start

### GUI Application

Run the graphical interface for easy file conversion:

```bash
cd gui
cargo run
```

**GUI Features:**
- Drag & drop AKP files
- Format selection (SFZ/Decent Sampler)
- Batch conversion mode
- Real-time progress tracking
- Results summary with error reporting

### Command Line Interface

For advanced users and scripting:

```bash
# Convert single file to SFZ (default)
cargo run --bin rusty-samplers-cli -- sample.akp

# Convert to Decent Sampler format
cargo run --bin rusty-samplers-cli -- --format ds sample.akp

# Batch convert directory
cargo run --bin rusty-samplers-cli -- --batch samples/

# Batch convert to Decent Sampler
cargo run --bin rusty-samplers-cli -- --batch --format ds samples/
```

## 📦 Installation

### Prerequisites

- [Rust](https://rustlang.org/tools/install) (1.70 or later)
- Git

### Build from Source

```bash
git clone https://github.com/yourusername/rusty-samplers.git
cd rusty-samplers

# Build CLI version
cargo build --release --bin rusty-samplers-cli

# Build GUI version
cd gui
cargo build --release
```

## 🎼 Supported Formats

### Input Format

- **AKP (Akai Program)**: RIFF-based Akai sampler program files

### Output Formats

#### SFZ Format
- Standard SFZ sampler format
- Compatible with most modern samplers
- Advanced parameter mapping including:
  - Sample key/velocity mapping
  - ADSR envelopes with exponential scaling
  - Filter cutoff/resonance (20Hz-20kHz range)
  - LFO modulation with waveform selection
  - Modulation routing (18 sources, 17 destinations)
  - Velocity tracking and dynamics

#### Decent Sampler XML
- Native Decent Sampler preset format
- Complete UI integration with labeled controls
- Advanced features including:
  - Interactive UI controls (Attack, Decay, Sustain, Release, Filter, Resonance)
  - Effects chain (Lowpass filter + Reverb)
  - MIDI CC bindings (CC1, CC2, CC7)
  - LFO modulators with waveform selection
  - Comprehensive metadata and tagging

## 🛠️ Parameter Conversion

### Envelope Conversion
- **Attack/Decay/Release**: Exponential scaling for musical response
- **Sustain**: Direct level mapping
- **Velocity Tracking**: Automatic velocity sensitivity

### Filter Conversion  
- **Cutoff**: Logarithmic mapping (20Hz to 20kHz)
- **Resonance**: Scaled to appropriate dB ranges
- **Filter Types**: Lowpass, Bandpass, Highpass support

### LFO Conversion
- **Rate**: Logarithmic frequency conversion (0.1Hz to 30Hz)
- **Waveforms**: Triangle, Sine, Square, Saw, Ramp, Random
- **Delay/Fade**: Time-based parameter conversion

### Modulation Mapping
Comprehensive modulation routing with support for:
- **Sources**: LFO1/2, Mod Wheel, Aftertouch, Velocity, Key, Pitch Bend
- **Destinations**: Pitch, Filter, Volume, Pan, Envelope times, LFO parameters

## 📊 Technical Details

### Architecture
- **Rust-based**: Memory-safe, high-performance conversion engine
- **RIFF Parser**: Comprehensive AKP chunk parsing
- **Multi-threaded**: Parallel processing for batch conversions
- **Error Recovery**: Graceful handling of malformed files

### File Structure
```
rusty-samplers/
├── src/
│   ├── main.rs           # CLI application
│   ├── lib.rs            # Core library
│   └── bin/
│       └── simple_gui.rs # Standalone GUI
├── gui/                  # GUI application
│   ├── Cargo.toml
│   └── src/main.rs
├── tests/                # Integration tests
├── CLAUDE.md            # Development documentation
└── README.md           # This file
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run specific test
cargo test test_dspreset_generation
```

Test coverage includes:
- AKP chunk parsing validation
- Parameter conversion accuracy
- SFZ output format compliance
- Decent Sampler XML validation
- Error handling scenarios

## 🐛 Troubleshooting

### Common Issues

**"Invalid RIFF header"**
- Ensure file is a valid AKP file
- Check file isn't corrupted or truncated

**"Missing required keygroup chunk"**
- AKP file may be empty or malformed
- Try with a different AKP file

**GUI won't start**
- Ensure all dependencies are installed
- Try running from the `gui/` directory

**Batch conversion fails**
- Check directory permissions
- Ensure AKP files have correct extensions

## 🤝 Contributing

We welcome contributions! Please see our development guidelines:

1. **Code Style**: Follow Rust conventions and run `cargo fmt`
2. **Testing**: Add tests for new features
3. **Documentation**: Update docs for API changes
4. **Error Handling**: Use proper error types and messages

### Development Setup

```bash
# Clone and setup
git clone https://github.com/yourusername/rusty-samplers.git
cd rusty-samplers

# Install development dependencies
cargo build

# Run tests
cargo test

# Run GUI in development
cd gui && cargo run
```

## 📋 TODO / Roadmap

- [ ] Additional sampler format support (Kontakt, EXS24)
- [ ] Sample auto-detection and path resolution
- [ ] Advanced modulation matrix conversion
- [ ] Preset organization and management
- [ ] Cloud storage integration
- [ ] Plugin version for DAW integration

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Akai for the AKP format specifications
- SFZ format community for documentation
- Decent Sampler team for XML format details
- Rust community for excellent tooling and libraries

---

**Made with ❤️ in Rust**