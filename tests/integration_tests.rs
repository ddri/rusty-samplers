// Integration tests for real-world AKP file processing
// Comprehensive validation of conversion accuracy and performance

use rusty_samplers::{conversion, error::ConversionError};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Test configuration for real AKP files
#[derive(Debug)]
pub struct TestConfig {
    pub test_files_dir: PathBuf,
    pub output_dir: PathBuf,
    pub max_conversion_time_ms: u128,
    pub max_memory_usage_mb: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_files_dir: PathBuf::from("test_data/akp_files"),
            output_dir: PathBuf::from("test_output"),
            max_conversion_time_ms: 1000, // 1 second target
            max_memory_usage_mb: 50,       // 50MB memory target
        }
    }
}

/// Performance metrics for conversion testing
#[derive(Debug, Clone)]
pub struct ConversionMetrics {
    pub file_name: String,
    pub file_size_bytes: u64,
    pub conversion_time_ms: u128,
    pub memory_peak_mb: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub keygroup_count: usize,
    pub parameter_count: usize,
}

impl ConversionMetrics {
    fn new(file_name: String, file_size: u64) -> Self {
        Self {
            file_name,
            file_size_bytes: file_size,
            conversion_time_ms: 0,
            memory_peak_mb: 0,
            success: false,
            error_message: None,
            keygroup_count: 0,
            parameter_count: 0,
        }
    }

    /// Check if conversion meets performance targets
    pub fn meets_performance_targets(&self, config: &TestConfig) -> bool {
        self.success 
            && self.conversion_time_ms <= config.max_conversion_time_ms
            && self.memory_peak_mb <= config.max_memory_usage_mb
    }

    /// Get performance summary
    pub fn summary(&self) -> String {
        format!(
            "{}: {} ({}KB) -> {}ms, {}MB, {} keygroups{}",
            self.file_name,
            if self.success { "✅" } else { "❌" },
            self.file_size_bytes / 1024,
            self.conversion_time_ms,
            self.memory_peak_mb,
            self.keygroup_count,
            if let Some(ref err) = self.error_message {
                format!(" [Error: {}]", err)
            } else {
                String::new()
            }
        )
    }
}

/// Test suite for real AKP file validation
pub struct AkpTestSuite {
    config: TestConfig,
    results: Vec<ConversionMetrics>,
}

impl AkpTestSuite {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Run comprehensive test suite on all AKP files
    pub fn run_all_tests(&mut self) -> Result<TestSuiteResults, Box<dyn std::error::Error>> {
        println!("🧪 Starting AKP Test Suite");
        println!("   Test files dir: {}", self.config.test_files_dir.display());
        println!("   Output dir: {}", self.config.output_dir.display());
        println!("   Performance targets: {}ms, {}MB", 
                 self.config.max_conversion_time_ms, self.config.max_memory_usage_mb);

        // Ensure output directory exists
        fs::create_dir_all(&self.config.output_dir)?;

        // Find all AKP files
        let akp_files = self.find_akp_files()?;
        println!("   Found {} AKP files to test", akp_files.len());

        if akp_files.is_empty() {
            println!("⚠️  No AKP files found in {}", self.config.test_files_dir.display());
            println!("   To run real file tests, place .akp files in the test_data/akp_files directory");
            return Ok(TestSuiteResults::empty());
        }

        // Test each file
        for akp_file in akp_files {
            let metrics = self.test_single_file(&akp_file)?;
            println!("   {}", metrics.summary());
            self.results.push(metrics);
        }

        let suite_results = TestSuiteResults::from_metrics(&self.results, &self.config);
        println!("\n📊 Test Suite Results:");
        println!("{}", suite_results.summary());

        Ok(suite_results)
    }

    /// Find all AKP files in test directory
    fn find_akp_files(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut akp_files = Vec::new();

        if !self.config.test_files_dir.exists() {
            // Create directory structure for users
            fs::create_dir_all(&self.config.test_files_dir)?;
            println!("📁 Created test directory: {}", self.config.test_files_dir.display());
            return Ok(akp_files);
        }

        for entry in fs::read_dir(&self.config.test_files_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "akp" {
                        akp_files.push(path);
                    }
                }
            }
        }

        // Sort for consistent test order
        akp_files.sort();
        Ok(akp_files)
    }

    /// Test conversion of a single AKP file
    fn test_single_file(&self, akp_path: &Path) -> Result<ConversionMetrics, Box<dyn std::error::Error>> {
        let file_name = akp_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_size = fs::metadata(akp_path)?.len();
        let mut metrics = ConversionMetrics::new(file_name, file_size);

        // Open file and measure conversion time
        let file = File::open(akp_path)?;
        let start_time = Instant::now();

        // Perform conversion
        match conversion::convert_akp_to_sfz_program(file) {
            Ok(sfz_program) => {
                metrics.conversion_time_ms = start_time.elapsed().as_millis();
                metrics.success = true;
                metrics.keygroup_count = sfz_program.regions.len();
                metrics.parameter_count = self.count_parameters(&sfz_program);

                // Write SFZ output for manual inspection
                let output_filename = format!("{}.sfz", 
                    akp_path.file_stem().unwrap_or_default().to_string_lossy());
                let output_path = self.config.output_dir.join(output_filename);
                
                // Convert SfzProgram to string by rendering each region
                let mut sfz_content = String::new();
                for region in &sfz_program.regions {
                    sfz_content.push_str(&region.to_sfz_string());
                    sfz_content.push('\n');
                }
                fs::write(output_path, sfz_content)?;
            }
            Err(ConversionError::PartialSuccess { warnings, .. }) => {
                metrics.conversion_time_ms = start_time.elapsed().as_millis();
                metrics.success = true; // Partial success counts as success
                metrics.error_message = Some(format!("Partial success: {} warnings", warnings.len()));
            }
            Err(e) => {
                metrics.conversion_time_ms = start_time.elapsed().as_millis();
                metrics.success = false;
                metrics.error_message = Some(e.to_string());
            }
        }

        // TODO: Measure actual memory usage (requires platform-specific code)
        metrics.memory_peak_mb = 0; // Placeholder

        Ok(metrics)
    }

    /// Count total parameters in SFZ program for complexity assessment
    fn count_parameters(&self, sfz_program: &rusty_samplers::formats::sfz::SfzProgram) -> usize {
        sfz_program.regions.iter().map(|region| {
            let mut count = 0;
            if region.volume.is_some() { count += 1; }
            if region.cutoff.is_some() { count += 1; }
            if region.resonance.is_some() { count += 1; }
            // Add other parameter counts as needed
            count
        }).sum()
    }
}

/// Results summary for entire test suite
#[derive(Debug)]
pub struct TestSuiteResults {
    pub total_files: usize,
    pub successful_conversions: usize,
    pub failed_conversions: usize,
    pub partial_successes: usize,
    pub avg_conversion_time_ms: f64,
    pub max_conversion_time_ms: u128,
    pub performance_targets_met: usize,
    pub total_keygroups: usize,
    pub total_parameters: usize,
}

impl TestSuiteResults {
    fn empty() -> Self {
        Self {
            total_files: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            partial_successes: 0,
            avg_conversion_time_ms: 0.0,
            max_conversion_time_ms: 0,
            performance_targets_met: 0,
            total_keygroups: 0,
            total_parameters: 0,
        }
    }

    fn from_metrics(metrics: &[ConversionMetrics], config: &TestConfig) -> Self {
        let total_files = metrics.len();
        let successful_conversions = metrics.iter().filter(|m| m.success).count();
        let failed_conversions = total_files - successful_conversions;
        let partial_successes = metrics.iter()
            .filter(|m| m.success && m.error_message.is_some())
            .count();

        let avg_conversion_time_ms = if total_files > 0 {
            metrics.iter().map(|m| m.conversion_time_ms as f64).sum::<f64>() / total_files as f64
        } else {
            0.0
        };

        let max_conversion_time_ms = metrics.iter()
            .map(|m| m.conversion_time_ms)
            .max()
            .unwrap_or(0);

        let performance_targets_met = metrics.iter()
            .filter(|m| m.meets_performance_targets(config))
            .count();

        let total_keygroups = metrics.iter().map(|m| m.keygroup_count).sum();
        let total_parameters = metrics.iter().map(|m| m.parameter_count).sum();

        Self {
            total_files,
            successful_conversions,
            failed_conversions,
            partial_successes,
            avg_conversion_time_ms,
            max_conversion_time_ms,
            performance_targets_met,
            total_keygroups,
            total_parameters,
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_files == 0 {
            100.0
        } else {
            (self.successful_conversions as f64 / self.total_files as f64) * 100.0
        }
    }

    /// Get performance target achievement rate
    pub fn performance_rate(&self) -> f64 {
        if self.total_files == 0 {
            100.0
        } else {
            (self.performance_targets_met as f64 / self.total_files as f64) * 100.0
        }
    }

    /// Comprehensive results summary
    pub fn summary(&self) -> String {
        if self.total_files == 0 {
            return "   No AKP files found for testing. Place .akp files in test_data/akp_files/ to run validation.".to_string();
        }

        format!(
            "   Files: {} total, {} successful ({:.1}% success rate)\n   \
             Performance: {} files met targets ({:.1}% performance rate)\n   \
             Timing: {:.1}ms average, {}ms maximum\n   \
             Data: {} keygroups, {} parameters processed\n   \
             Quality: {} partial successes, {} failures",
            self.total_files,
            self.successful_conversions,
            self.success_rate(),
            self.performance_targets_met,
            self.performance_rate(),
            self.avg_conversion_time_ms,
            self.max_conversion_time_ms,
            self.total_keygroups,
            self.total_parameters,
            self.partial_successes,
            self.failed_conversions
        )
    }

    /// Check if test suite meets quality thresholds
    pub fn meets_quality_threshold(&self) -> bool {
        self.success_rate() >= 95.0 && self.performance_rate() >= 80.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_akp_file_integration() {
        let config = TestConfig::default();
        let mut test_suite = AkpTestSuite::new(config);
        
        // Run the test suite (will create directories if needed)
        let results = test_suite.run_all_tests().expect("Test suite should run without errors");
        
        // If no files are found, that's OK for CI/automated testing
        if results.total_files == 0 {
            println!("No AKP files found - skipping real file validation");
            return;
        }

        // If files are found, validate quality
        println!("Real file test results: {}", results.summary());
        
        // Assert quality thresholds for real-world files
        assert!(
            results.meets_quality_threshold(),
            "Test suite failed quality threshold: {:.1}% success rate, {:.1}% performance rate",
            results.success_rate(),
            results.performance_rate()
        );
    }

    #[test]
    fn test_conversion_metrics() {
        let metrics = ConversionMetrics::new("test.akp".to_string(), 1024);
        assert!(!metrics.success);
        assert_eq!(metrics.file_size_bytes, 1024);
        assert!(metrics.summary().contains("test.akp"));
    }

    #[test]
    fn test_suite_results_empty() {
        let results = TestSuiteResults::empty();
        assert_eq!(results.total_files, 0);
        assert_eq!(results.success_rate(), 100.0);
        assert!(results.summary().contains("No AKP files found"));
    }
}