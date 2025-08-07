# Test Data Directory

This directory contains test files for validating Rusty Samplers conversion accuracy and performance.

## Directory Structure

```
test_data/
├── akp_files/          # Real-world AKP files for integration testing
├── malformed/          # Corrupted/malformed files for error recovery testing  
├── reference/          # Reference conversion outputs for accuracy validation
└── synthetic/          # Generated test files for automated testing
```

## Adding Test Files

### Real AKP Files
Place your `.akp` files in the `akp_files/` directory to run comprehensive validation tests:

```bash
# Example: Copy your AKP files
cp /path/to/your/samples/*.akp test_data/akp_files/
```

### Expected Test Files
- **Small files** (1-10KB): Basic program validation
- **Medium files** (10-100KB): Multi-keygroup programs  
- **Large files** (100KB-1MB): Complex programs with many samples
- **Various sources**: S5000, S6000, MPC4000+ AKP files

### Privacy and Copyright
**Important**: Do not commit copyrighted AKP files to version control. This directory is excluded via `.gitignore`. Only use:
- Your own created AKP files
- Public domain sample libraries
- Files with explicit permission for testing

## Running Tests

### Integration Tests (requires real AKP files)
```bash
# Run integration tests with your AKP files
cargo test integration_tests

# Run with logging to see detailed results
RUST_LOG=debug cargo test integration_tests
```

### Performance Benchmarks
```bash  
# Run performance benchmarks
cargo bench

# Run with specific benchmark
cargo bench conversion_benchmarks
```

### Validation Testing
```bash
# Test parameter validation accuracy
cargo test validation::tests
```

## Test Results

Test outputs are written to `test_output/` directory:
- **Converted SFZ files** - Manual inspection of conversion quality
- **Validation reports** - Parameter accuracy and quality scores
- **Benchmark results** - Performance metrics and comparisons

## Quality Targets

Our testing aims to meet these quality standards:
- **95%+ success rate** on real-world AKP files
- **<1 second** conversion time for typical files
- **<50MB** peak memory usage
- **90+ quality score** on parameter validation
- **Cross-platform compatibility** (Windows, macOS, Linux)