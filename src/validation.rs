// Parameter validation and accuracy testing for converted programs
// Ensures conversion fidelity and parameter range compliance

use crate::formats::sfz::{SfzProgram, SfzRegion};
use log::{debug, info};
use std::collections::HashMap;

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,    // Informational - doesn't affect quality
    Warning, // Minor issue - conversion still valid
    Error,   // Major issue - conversion quality compromised
    Critical, // Critical issue - conversion likely unusable
}

/// Validation issue with detailed context
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub category: String,
    pub message: String,
    pub parameter: Option<String>,
    pub expected_range: Option<(f64, f64)>,
    pub actual_value: Option<f64>,
    pub suggested_fix: Option<String>,
}

impl ValidationIssue {
    pub fn new(severity: ValidationSeverity, category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            category: category.into(),
            message: message.into(),
            parameter: None,
            expected_range: None,
            actual_value: None,
            suggested_fix: None,
        }
    }

    pub fn with_parameter(mut self, parameter: impl Into<String>, value: f64) -> Self {
        self.parameter = Some(parameter.into());
        self.actual_value = Some(value);
        self
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.expected_range = Some((min, max));
        self
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.suggested_fix = Some(fix.into());
        self
    }

    pub fn summary(&self) -> String {
        let severity_icon = match self.severity {
            ValidationSeverity::Info => "ℹ️",
            ValidationSeverity::Warning => "⚠️",
            ValidationSeverity::Error => "❌",
            ValidationSeverity::Critical => "🔥",
        };

        let mut summary = format!("{} [{}] {}", severity_icon, self.category, self.message);

        if let Some(ref param) = self.parameter {
            if let Some(value) = self.actual_value {
                summary.push_str(&format!(" ({}={})", param, value));
                
                if let Some((min, max)) = self.expected_range {
                    summary.push_str(&format!(" [expected: {}-{}]", min, max));
                }
            }
        }

        if let Some(ref fix) = self.suggested_fix {
            summary.push_str(&format!(" → Fix: {}", fix));
        }

        summary
    }
}

/// Comprehensive validation results
#[derive(Debug)]
pub struct ValidationResults {
    pub issues: Vec<ValidationIssue>,
    pub parameter_stats: ParameterStatistics,
    pub quality_score: f64,
}

impl ValidationResults {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            parameter_stats: ParameterStatistics::new(),
            quality_score: 0.0,
        }
    }

    /// Add a validation issue
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        debug!("Validation issue: {}", issue.summary());
        self.issues.push(issue);
    }

    /// Get issues by severity level
    pub fn issues_by_severity(&self, severity: ValidationSeverity) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.severity == severity).collect()
    }

    /// Calculate overall quality score (0-100)
    pub fn calculate_quality_score(&mut self) {
        let total_issues = self.issues.len() as f64;
        if total_issues == 0.0 {
            self.quality_score = 100.0;
            return;
        }

        // Weight issues by severity
        let severity_weights = [
            (ValidationSeverity::Info, 0.1),
            (ValidationSeverity::Warning, 1.0),
            (ValidationSeverity::Error, 5.0),
            (ValidationSeverity::Critical, 20.0),
        ];

        let weighted_score = severity_weights.iter().map(|(severity, weight)| {
            let count = self.issues_by_severity(*severity).len() as f64;
            count * weight
        }).sum::<f64>();

        // Convert to 0-100 scale (lower weighted score = higher quality)
        self.quality_score = (100.0 - (weighted_score * 2.0)).max(0.0);
    }

    /// Generate comprehensive validation report
    pub fn report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("📊 Validation Report (Quality Score: {:.1}/100)\n", self.quality_score));
        report.push_str(&format!("   Parameters: {} processed\n", self.parameter_stats.total_parameters));
        
        // Summary by severity
        for severity in [ValidationSeverity::Critical, ValidationSeverity::Error, 
                        ValidationSeverity::Warning, ValidationSeverity::Info] {
            let issues = self.issues_by_severity(severity);
            if !issues.is_empty() {
                report.push_str(&format!("   {:?}: {} issues\n", severity, issues.len()));
            }
        }

        // Detailed issues (limit to most severe)
        let critical_and_errors: Vec<_> = self.issues.iter()
            .filter(|i| matches!(i.severity, ValidationSeverity::Critical | ValidationSeverity::Error))
            .collect();

        if !critical_and_errors.is_empty() {
            report.push_str("\n🔍 Critical Issues:\n");
            for issue in critical_and_errors.iter().take(10) {
                report.push_str(&format!("   {}\n", issue.summary()));
            }
        }

        // Parameter statistics
        report.push_str(&format!("\n📈 Parameter Statistics:\n{}", self.parameter_stats.report()));

        report
    }

    /// Check if validation passes quality threshold
    pub fn passes_quality_threshold(&self, threshold: f64) -> bool {
        self.quality_score >= threshold
    }
}

/// Statistics about parameter conversion accuracy
#[derive(Debug)]
pub struct ParameterStatistics {
    pub total_parameters: usize,
    pub valid_parameters: usize,
    pub out_of_range_parameters: usize,
    pub missing_parameters: usize,
    pub parameter_ranges: HashMap<String, (f64, f64)>, // min, max per parameter type
    pub parameter_counts: HashMap<String, usize>,
}

impl ParameterStatistics {
    pub fn new() -> Self {
        Self {
            total_parameters: 0,
            valid_parameters: 0,
            out_of_range_parameters: 0,
            missing_parameters: 0,
            parameter_ranges: HashMap::new(),
            parameter_counts: HashMap::new(),
        }
    }

    pub fn record_parameter(&mut self, name: &str, value: f64, is_valid: bool) {
        self.total_parameters += 1;
        
        if is_valid {
            self.valid_parameters += 1;
        } else {
            self.out_of_range_parameters += 1;
        }

        // Update parameter ranges
        let entry = self.parameter_ranges.entry(name.to_string()).or_insert((value, value));
        entry.0 = entry.0.min(value);
        entry.1 = entry.1.max(value);

        // Update parameter counts
        *self.parameter_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn record_missing_parameter(&mut self, _name: &str) {
        self.missing_parameters += 1;
    }

    pub fn accuracy_rate(&self) -> f64 {
        if self.total_parameters == 0 {
            100.0
        } else {
            (self.valid_parameters as f64 / self.total_parameters as f64) * 100.0
        }
    }

    pub fn report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("   Accuracy: {:.1}% ({}/{} valid)\n", 
                                self.accuracy_rate(), self.valid_parameters, self.total_parameters));
        
        if self.out_of_range_parameters > 0 {
            report.push_str(&format!("   Out of range: {} parameters\n", self.out_of_range_parameters));
        }
        
        if self.missing_parameters > 0 {
            report.push_str(&format!("   Missing: {} parameters\n", self.missing_parameters));
        }

        // Top parameter types
        let mut param_counts: Vec<_> = self.parameter_counts.iter().collect();
        param_counts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        
        if !param_counts.is_empty() {
            report.push_str("   Top parameters: ");
            for (i, (name, count)) in param_counts.iter().take(5).enumerate() {
                if i > 0 { report.push_str(", "); }
                report.push_str(&format!("{}({})", name, count));
            }
            report.push('\n');
        }

        report
    }
}

/// Comprehensive parameter validator for converted programs  
pub struct ParameterValidator {
    strict_ranges: bool,
    audio_range_tolerance: f64,
}

impl Default for ParameterValidator {
    fn default() -> Self {
        Self {
            strict_ranges: false,
            audio_range_tolerance: 0.1, // 10% tolerance for audio parameters
        }
    }
}

impl ParameterValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_strict_ranges(mut self) -> Self {
        self.strict_ranges = true;
        self
    }

    /// Validate converted SFZ program comprehensively
    pub fn validate_sfz_program(&self, program: &SfzProgram) -> ValidationResults {
        let mut results = ValidationResults::new();
        
        info!("Starting comprehensive parameter validation for {} regions", program.regions.len());

        // Validate each region
        for (region_index, region) in program.regions.iter().enumerate() {
            self.validate_region(region, region_index, &mut results);
        }

        // Global program validation
        self.validate_program_structure(program, &mut results);

        // Calculate quality score
        results.calculate_quality_score();

        info!("Validation complete: {:.1} quality score, {} issues", 
              results.quality_score, results.issues.len());

        results
    }

    /// Validate individual region parameters
    fn validate_region(&self, region: &SfzRegion, region_index: usize, results: &mut ValidationResults) {
        let region_context = format!("Region {}", region_index);

        // Key mapping validation
        self.validate_key_mapping(region, &region_context, results);

        // Velocity mapping validation  
        self.validate_velocity_mapping(region, &region_context, results);

        // Audio parameters validation
        self.validate_audio_parameters(region, &region_context, results);

        // Sample reference validation
        self.validate_sample_reference(region, &region_context, results);
    }

    /// Validate key mapping ranges and logic
    fn validate_key_mapping(&self, region: &SfzRegion, context: &str, results: &mut ValidationResults) {
        // Key range validation
        if region.lokey > region.hikey {
            results.add_issue(
                ValidationIssue::new(
                    ValidationSeverity::Error,
                    "Key Mapping",
                    format!("{}: Invalid key range", context)
                )
                .with_parameter("lokey-hikey", region.lokey as f64)
                .with_range(0.0, 127.0)
                .with_fix("Ensure lokey <= hikey")
            );
        }

        // MIDI key range validation (0-127)
        for (param_name, key_value) in [("lokey", region.lokey), ("hikey", region.hikey)] {
            if key_value > 127 {
                results.add_issue(
                    ValidationIssue::new(
                        ValidationSeverity::Error,
                        "Key Mapping", 
                        format!("{}: MIDI key out of range", context)
                    )
                    .with_parameter(param_name, key_value as f64)
                    .with_range(0.0, 127.0)
                    .with_fix("Use MIDI key range 0-127")
                );
                results.parameter_stats.record_parameter(param_name, key_value as f64, false);
            } else {
                results.parameter_stats.record_parameter(param_name, key_value as f64, true);
            }
        }
    }

    /// Validate velocity mapping ranges
    fn validate_velocity_mapping(&self, region: &SfzRegion, context: &str, results: &mut ValidationResults) {
        // Velocity range validation
        if region.lovel > region.hivel {
            results.add_issue(
                ValidationIssue::new(
                    ValidationSeverity::Error,
                    "Velocity Mapping",
                    format!("{}: Invalid velocity range", context)
                )
                .with_parameter("lovel-hivel", region.lovel as f64)
                .with_range(0.0, 127.0)
                .with_fix("Ensure lovel <= hivel")
            );
        }

        // MIDI velocity range validation (0-127)
        for (param_name, vel_value) in [("lovel", region.lovel), ("hivel", region.hivel)] {
            if vel_value > 127 {
                results.add_issue(
                    ValidationIssue::new(
                        ValidationSeverity::Error,
                        "Velocity Mapping",
                        format!("{}: MIDI velocity out of range", context)
                    )
                    .with_parameter(param_name, vel_value as f64)
                    .with_range(0.0, 127.0)
                    .with_fix("Use MIDI velocity range 0-127")
                );
                results.parameter_stats.record_parameter(param_name, vel_value as f64, false);
            } else {
                results.parameter_stats.record_parameter(param_name, vel_value as f64, true);
            }
        }
    }

    /// Validate audio processing parameters
    fn validate_audio_parameters(&self, region: &SfzRegion, context: &str, results: &mut ValidationResults) {
        // Volume validation (typical range: -144 to +6 dB)
        if let Some(volume) = region.volume {
            let is_valid = volume >= -144.0 && volume <= 6.0;
            if !is_valid && self.strict_ranges {
                results.add_issue(
                    ValidationIssue::new(
                        ValidationSeverity::Warning,
                        "Audio Parameter",
                        format!("{}: Volume outside typical range", context)
                    )
                    .with_parameter("volume", volume as f64)
                    .with_range(-144.0, 6.0)
                    .with_fix("Consider volume range -144 to +6 dB")
                );
            }
            results.parameter_stats.record_parameter("volume", volume as f64, is_valid);
        }

        // Cutoff frequency validation (20Hz - 20kHz)
        if let Some(cutoff) = region.cutoff {
            let is_valid = cutoff >= 20.0 && cutoff <= 20000.0;
            if !is_valid {
                let severity = if cutoff < 1.0 || cutoff > 48000.0 {
                    ValidationSeverity::Error
                } else {
                    ValidationSeverity::Warning
                };

                results.add_issue(
                    ValidationIssue::new(
                        severity,
                        "Audio Parameter",
                        format!("{}: Cutoff frequency out of audible range", context)
                    )
                    .with_parameter("cutoff", cutoff as f64)
                    .with_range(20.0, 20000.0)
                    .with_fix("Use audible frequency range 20Hz-20kHz")
                );
            }
            results.parameter_stats.record_parameter("cutoff", cutoff as f64, is_valid);
        }

        // Resonance validation (0-40 dB typical)
        if let Some(resonance) = region.resonance {
            let is_valid = resonance >= 0.0 && resonance <= 40.0;
            if !is_valid && self.strict_ranges {
                results.add_issue(
                    ValidationIssue::new(
                        ValidationSeverity::Warning,
                        "Audio Parameter",
                        format!("{}: Resonance outside typical range", context)
                    )
                    .with_parameter("resonance", resonance as f64)
                    .with_range(0.0, 40.0)
                    .with_fix("Consider resonance range 0-40 dB")
                );
            }
            results.parameter_stats.record_parameter("resonance", resonance as f64, is_valid);
        }
    }

    /// Validate sample file references
    fn validate_sample_reference(&self, region: &SfzRegion, context: &str, results: &mut ValidationResults) {
        if region.sample.is_empty() {
            results.add_issue(
                ValidationIssue::new(
                    ValidationSeverity::Critical,
                    "Sample Reference",
                    format!("{}: Missing sample file reference", context)
                )
                .with_fix("Ensure each region has a valid sample file")
            );
            results.parameter_stats.record_missing_parameter("sample");
        } else {
            // Check for placeholder samples (from error recovery)
            if region.sample.contains("missing_sample") || region.sample.contains("placeholder") {
                results.add_issue(
                    ValidationIssue::new(
                        ValidationSeverity::Warning,
                        "Sample Reference",
                        format!("{}: Placeholder sample detected", context)
                    )
                    .with_parameter("sample", 0.0)
                    .with_fix("Replace with actual sample file reference")
                );
            }
            results.parameter_stats.record_parameter("sample", 1.0, true);
        }
    }

    /// Validate overall program structure
    fn validate_program_structure(&self, program: &SfzProgram, results: &mut ValidationResults) {
        // Check if program has any regions
        if program.regions.is_empty() {
            results.add_issue(
                ValidationIssue::new(
                    ValidationSeverity::Critical,
                    "Program Structure",
                    "Program contains no regions"
                )
                .with_fix("Ensure source file contains valid keygroup data")
            );
            return;
        }

        // Check for key/velocity coverage gaps
        self.validate_coverage_gaps(program, results);
    }

    /// Validate key and velocity coverage for gaps
    fn validate_coverage_gaps(&self, program: &SfzProgram, results: &mut ValidationResults) {
        // Build coverage map
        let mut key_coverage = vec![false; 128]; // MIDI keys 0-127
        
        for region in &program.regions {
            for key in region.lokey..=region.hikey.min(127) {
                key_coverage[key as usize] = true;
            }
        }

        // Find significant gaps (more than 12 consecutive keys)
        let mut gap_start = None;
        for (key, &covered) in key_coverage.iter().enumerate() {
            if !covered {
                if gap_start.is_none() {
                    gap_start = Some(key);
                }
            } else if let Some(start) = gap_start {
                let gap_size = key - start;
                if gap_size >= 12 { // One octave or more
                    results.add_issue(
                        ValidationIssue::new(
                            ValidationSeverity::Info,
                            "Key Coverage",
                            format!("Key coverage gap: {} keys ({}-{})", gap_size, start, key - 1)
                        )
                        .with_fix("Consider if this gap is intentional")
                    );
                }
                gap_start = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_issue() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Warning,
            "Test Category",
            "Test message"
        )
        .with_parameter("volume", -50.0)
        .with_range(-144.0, 6.0)
        .with_fix("Adjust volume");

        let summary = issue.summary();
        assert!(summary.contains("⚠️"));
        assert!(summary.contains("Test Category"));
        assert!(summary.contains("volume=-50"));
        assert!(summary.contains("Fix: Adjust volume"));
    }

    #[test]
    fn test_parameter_statistics() {
        let mut stats = ParameterStatistics::new();
        
        stats.record_parameter("volume", -12.0, true);
        stats.record_parameter("volume", -6.0, true);
        stats.record_parameter("cutoff", 1000.0, true);
        stats.record_missing_parameter("filter");

        assert_eq!(stats.total_parameters, 3);
        assert_eq!(stats.valid_parameters, 3);
        assert_eq!(stats.missing_parameters, 1);
        assert_eq!(stats.accuracy_rate(), 100.0);
        
        let report = stats.report();
        assert!(report.contains("100.0%"));
        assert!(report.contains("volume(2)"));
    }

    #[test]
    fn test_parameter_validator_key_ranges() {
        let validator = ParameterValidator::new();
        
        let mut program = SfzProgram::new();
        let mut region = SfzRegion::new("test.wav".to_string(), 60, 50, 0, 127); // Invalid: lokey > hikey
        region.volume = Some(-12.0);
        program.add_region(region);

        let results = validator.validate_sfz_program(&program);
        
        // Should have at least one error for invalid key range
        let errors = results.issues_by_severity(ValidationSeverity::Error);
        assert!(!errors.is_empty());
        
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        assert!(error_messages.iter().any(|msg| msg.contains("Invalid key range")));
    }

    #[test]
    fn test_parameter_validator_audio_parameters() {
        let validator = ParameterValidator::new().with_strict_ranges();
        
        let mut program = SfzProgram::new();
        let mut region = SfzRegion::new("test.wav".to_string(), 60, 72, 0, 127);
        region.volume = Some(-200.0); // Outside typical range
        region.cutoff = Some(50000.0); // Outside audible range
        region.resonance = Some(100.0); // Outside typical range
        program.add_region(region);

        let results = validator.validate_sfz_program(&program);
        
        // Should have warnings for out-of-range parameters
        let warnings = results.issues_by_severity(ValidationSeverity::Warning);
        assert!(warnings.len() >= 2); // volume and resonance warnings
        
        let errors = results.issues_by_severity(ValidationSeverity::Error);
        assert!(!errors.is_empty()); // cutoff error
    }

    #[test]
    fn test_validation_quality_score() {
        let mut results = ValidationResults::new();
        
        // Add various severity issues
        results.add_issue(ValidationIssue::new(ValidationSeverity::Info, "Test", "Info issue"));
        results.add_issue(ValidationIssue::new(ValidationSeverity::Warning, "Test", "Warning issue"));
        results.add_issue(ValidationIssue::new(ValidationSeverity::Error, "Test", "Error issue"));
        
        results.calculate_quality_score();
        
        // Quality score should be reduced based on weighted issues
        assert!(results.quality_score < 100.0);
        assert!(results.quality_score > 0.0);
        
        let report = results.report();
        assert!(report.contains("Quality Score"));
        assert!(report.contains("Warning: 1 issues"));
        assert!(report.contains("Error: 1 issues"));
    }
}