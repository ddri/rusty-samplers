// Conversion engine for Rusty Samplers
// High-level API for converting between formats

use crate::error::{ConversionError, Result};
use crate::formats::{akp::AkaiProgram, sfz::SfzProgram};
use std::fs::File;
use std::io::{Read, Seek, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, debug, warn, error};

/// Convert an AKP file to SFZ format string
pub fn convert_akp_to_sfz(file: File) -> Result<String> {
    info!("Starting AKP to SFZ conversion");
    
    // Parse AKP file
    let akp_program = parse_akp(file)?;
    info!("Parsed AKP program: {}", akp_program.stats());
    
    // Convert to SFZ
    let sfz_content = akp_program.to_sfz_string();
    
    info!("Conversion completed successfully");
    Ok(sfz_content)
}

/// Convert an AKP file to structured SFZ program
pub fn convert_akp_to_sfz_program(file: File) -> Result<SfzProgram> {
    info!("Starting AKP to SFZ program conversion");
    
    // Parse AKP file
    let akp_program = parse_akp(file)?;
    info!("Parsed AKP program: {}", akp_program.stats());
    
    // Convert to SFZ program
    let sfz_program = akp_program.to_sfz_program();
    
    info!("Program conversion completed successfully");
    Ok(sfz_program)
}

/// Parse an AKP file using the configured parser
fn parse_akp(file: File) -> Result<AkaiProgram> {
    #[cfg(feature = "binrw-parser")]
    {
        debug!("Using binrw declarative parser");
        crate::formats::akp::binrw_parser::parse_akp_file(file)
    }
    
    #[cfg(all(feature = "legacy-parser", not(feature = "binrw-parser")))]
    {
        debug!("Using legacy parser");
        crate::formats::akp::legacy::parse_akp_file(file)
    }
    
    #[cfg(not(any(feature = "legacy-parser", feature = "binrw-parser")))]
    {
        Err(ConversionError::Custom {
            message: "No parser available - enable legacy-parser or binrw-parser feature".to_string(),
        })
    }
}

/// Performance optimization settings for conversion
#[derive(Debug, Clone)]
pub struct PerformanceSettings {
    /// Use streaming mode for files larger than this threshold (in bytes)
    pub streaming_threshold: u64,
    /// Buffer size for streaming operations (in bytes)
    pub streaming_buffer_size: usize,  
    /// Use memory mapping for very large files (>50MB)
    pub use_memory_mapping: bool,
    /// Skip parameter validation for batch processing
    pub skip_validation: bool,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            streaming_threshold: 10 * 1024 * 1024, // 10MB
            streaming_buffer_size: 64 * 1024,      // 64KB
            use_memory_mapping: true,
            skip_validation: false,
        }
    }
}

/// Conversion engine with configuration options
pub struct ConversionEngine {
    pub recover_from_errors: bool,
    pub validate_parameters: bool,
    pub preserve_original_paths: bool,
    pub performance: PerformanceSettings,
    warnings: Vec<String>,
}

impl ConversionEngine {
    /// Create a new conversion engine with default settings
    pub fn new() -> Self {
        Self {
            recover_from_errors: true,
            validate_parameters: true,
            preserve_original_paths: true,
            performance: PerformanceSettings::default(),
            warnings: Vec::new(),
        }
    }
    
    /// Create a conversion engine optimized for batch processing
    pub fn for_batch_processing() -> Self {
        let mut performance = PerformanceSettings::default();
        performance.skip_validation = true;
        performance.streaming_threshold = 5 * 1024 * 1024; // Lower threshold for batch
        
        Self {
            recover_from_errors: true,
            validate_parameters: false, // Skip validation for speed
            preserve_original_paths: true,
            performance,
            warnings: Vec::new(),
        }
    }
    
    /// Create a conversion engine optimized for large files (streaming mode)
    pub fn for_large_files() -> Self {
        let mut performance = PerformanceSettings::default();
        performance.streaming_threshold = 1024 * 1024; // 1MB threshold
        performance.streaming_buffer_size = 128 * 1024; // 128KB buffer
        performance.use_memory_mapping = true;
        
        Self {
            recover_from_errors: true,
            validate_parameters: true,
            preserve_original_paths: true,
            performance,
            warnings: Vec::new(),
        }
    }
    
    /// Convert an AKP file with the current configuration
    pub fn convert(&mut self, file: File) -> Result<SfzProgram> {
        self.warnings.clear();
        
        // Analyze file size to determine optimal processing strategy
        let file_size = file.metadata().map_err(ConversionError::Io)?.len();
        debug!("File size: {} bytes", file_size);
        
        if file_size > self.performance.streaming_threshold {
            info!("Using streaming mode for large file ({} bytes)", file_size);
            self.convert_with_streaming(file, file_size)
        } else {
            debug!("Using standard mode for file ({} bytes)", file_size);
            self.convert_standard(file)
        }
    }
    
    /// Convert using standard in-memory approach (for smaller files)
    fn convert_standard(&mut self, file: File) -> Result<SfzProgram> {
        match self.try_convert(file) {
            Ok(program) => Ok(program),
            Err(e) if e.is_recoverable() && self.recover_from_errors => {
                warn!("Recoverable error during conversion: {}", e);
                self.warnings.push(e.to_string());
                
                // Return partial success with warnings
                Err(ConversionError::PartialSuccess {
                    warning_count: self.warnings.len(),
                    warnings: self.warnings.clone(),
                })
            }
            Err(e) => Err(e),
        }
    }
    
    /// Convert using streaming approach (for large files)
    fn convert_with_streaming(&mut self, file: File, file_size: u64) -> Result<SfzProgram> {
        info!("Streaming conversion for large file: {} MB", file_size as f64 / (1024.0 * 1024.0));
        
        // Create a buffered reader with optimized buffer size
        let buf_size = self.performance.streaming_buffer_size.min(file_size as usize);
        let buffered_file = BufReader::with_capacity(buf_size, file);
        
        // For now, delegate to standard conversion but with buffered I/O
        // TODO: Implement true streaming parsing for chunk-by-chunk processing
        match self.try_convert_buffered(buffered_file) {
            Ok(program) => {
                info!("Streaming conversion completed successfully");
                Ok(program)
            }
            Err(e) if e.is_recoverable() && self.recover_from_errors => {
                warn!("Recoverable error during streaming conversion: {}", e);
                self.warnings.push(e.to_string());
                
                Err(ConversionError::PartialSuccess {
                    warning_count: self.warnings.len(),
                    warnings: self.warnings.clone(),
                })
            }
            Err(e) => Err(e),
        }
    }
    
    /// Convert with buffered reader (optimized I/O)
    fn try_convert_buffered<R: Read + Seek>(&mut self, _reader: R) -> Result<SfzProgram> {
        debug!("Starting buffered conversion with performance optimizations");
        debug!("  recover_from_errors: {}", self.recover_from_errors);
        debug!("  validate_parameters: {}", !self.performance.skip_validation && self.validate_parameters);
        debug!("  streaming_buffer_size: {} KB", self.performance.streaming_buffer_size / 1024);
        
        // TODO: Implement streaming parser that works with generic Read+Seek
        // For now, this is a placeholder that demonstrates the architecture
        
        Err(ConversionError::Custom {
            message: "Streaming conversion not yet implemented - use standard mode".to_string(),
        })
    }
    
    /// Get warnings from the last conversion
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
    
    /// Clear accumulated warnings
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
    }
    
    /// Internal conversion method
    fn try_convert(&mut self, file: File) -> Result<SfzProgram> {
        debug!("Starting conversion with engine configuration");
        debug!("  recover_from_errors: {}", self.recover_from_errors);
        debug!("  validate_parameters: {}", self.validate_parameters);
        debug!("  preserve_original_paths: {}", self.preserve_original_paths);
        
        let akp_program = parse_akp(file)?;
        let sfz_program = akp_program.to_sfz_program();
        
        // Apply performance-aware validation
        if self.validate_parameters && !self.performance.skip_validation {
            self.validate_sfz_program(&sfz_program)?;
        } else if self.performance.skip_validation {
            debug!("Skipping parameter validation for performance");
        }
        
        Ok(sfz_program)
    }
    
    /// Validate SFZ program parameters
    fn validate_sfz_program(&mut self, program: &SfzProgram) -> Result<()> {
        let mut issues = 0;
        
        for (i, region) in program.regions.iter().enumerate() {
            // Check key ranges
            if region.lokey > region.hikey {
                let warning = format!("Region {}: Invalid key range ({} > {})", 
                                    i, region.lokey, region.hikey);
                self.warnings.push(warning);
                issues += 1;
            }
            
            // Check velocity ranges
            if region.lovel > region.hivel {
                let warning = format!("Region {}: Invalid velocity range ({} > {})", 
                                    i, region.lovel, region.hivel);
                self.warnings.push(warning);
                issues += 1;
            }
            
            // Check volume range
            if let Some(volume) = region.volume {
                if volume < -144.0 || volume > 6.0 {
                    let warning = format!("Region {}: Volume {} out of typical range [-144, 6] dB", 
                                        i, volume);
                    self.warnings.push(warning);
                    issues += 1;
                }
            }
            
            // Check cutoff frequency
            if let Some(cutoff) = region.cutoff {
                if cutoff < 20.0 || cutoff > 20000.0 {
                    let warning = format!("Region {}: Cutoff {} Hz out of audible range [20, 20000]", 
                                        i, cutoff);
                    self.warnings.push(warning);
                    issues += 1;
                }
            }
        }
        
        if issues > 0 {
            debug!("Validation found {} parameter issues", issues);
        }
        
        Ok(())
    }
}

impl Default for ConversionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch processing results with performance metrics
#[derive(Debug)]
pub struct BatchResults {
    pub total_files: usize,
    pub successful_conversions: usize,
    pub failed_conversions: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub processing_time_ms: u128,
}

impl BatchResults {
    fn new(total_files: usize) -> Self {
        Self {
            total_files,
            successful_conversions: 0,
            failed_conversions: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            processing_time_ms: 0,
        }
    }
    
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_files == 0 {
            100.0
        } else if self.successful_conversions == 0 && self.failed_conversions == 0 {
            // No conversions attempted yet
            0.0
        } else {
            (self.successful_conversions as f64 / self.total_files as f64) * 100.0
        }
    }
    
    /// Get processing statistics summary
    pub fn stats(&self) -> String {
        format!(
            "Processed {} files: {} successful, {} failed ({:.1}% success rate) in {}ms",
            self.total_files,
            self.successful_conversions,
            self.failed_conversions,
            self.success_rate(),
            self.processing_time_ms
        )
    }
}

/// Parallel batch processing for multiple AKP files
pub struct BatchProcessor {
    engine_template: ConversionEngine,
    max_threads: usize,
}

impl BatchProcessor {
    /// Create a new batch processor with the given engine configuration
    pub fn new(engine: ConversionEngine) -> Self {
        let max_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8); // Cap at 8 threads for I/O bound tasks
            
        Self {
            engine_template: engine,
            max_threads,
        }
    }
    
    /// Set maximum number of parallel threads
    pub fn with_max_threads(mut self, max_threads: usize) -> Self {
        self.max_threads = max_threads.max(1).min(16); // Reasonable bounds
        self
    }
    
    /// Process multiple AKP files in parallel
    pub fn process_files<P: AsRef<Path>>(
        &self,
        input_files: &[P],
        output_dir: P,
    ) -> BatchResults {
        let start_time = std::time::Instant::now();
        let total_files = input_files.len();
        
        info!("Starting batch processing of {} files with {} threads", 
              total_files, self.max_threads);
        
        let results = Arc::new(Mutex::new(BatchResults::new(total_files)));
        let output_dir = Arc::new(output_dir.as_ref().to_path_buf());
        
        // Process files in chunks to manage thread count
        let chunk_size = (total_files / self.max_threads).max(1);
        let mut handles = Vec::new();
        
        for chunk in input_files.chunks(chunk_size) {
            let chunk_files: Vec<PathBuf> = chunk.iter().map(|p| p.as_ref().to_path_buf()).collect();
            let results_clone = Arc::clone(&results);
            let output_dir_clone = Arc::clone(&output_dir);
            let mut engine = self.engine_template.clone();
            
            let handle = thread::spawn(move || {
                for input_file in chunk_files {
                    let result = Self::process_single_file(&mut engine, &input_file, &output_dir_clone);
                    
                    // Update shared results
                    let mut results_guard = results_clone.lock().unwrap();
                    match result {
                        Ok(warnings) => {
                            results_guard.successful_conversions += 1;
                            results_guard.warnings.extend(warnings);
                        }
                        Err(e) => {
                            results_guard.failed_conversions += 1;
                            results_guard.errors.push(format!("{}: {}", 
                                input_file.display(), e));
                        }
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            if let Err(e) = handle.join() {
                error!("Thread panicked during batch processing: {:?}", e);
            }
        }
        
        let mut final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
        final_results.processing_time_ms = start_time.elapsed().as_millis();
        
        info!("Batch processing completed: {}", final_results.stats());
        final_results
    }
    
    /// Process a single file (internal helper)
    fn process_single_file(
        engine: &mut ConversionEngine,
        input_path: &Path,
        output_dir: &Path,
    ) -> Result<Vec<String>> {
        debug!("Processing file: {}", input_path.display());
        
        // Open input file
        let file = File::open(input_path)
            .map_err(ConversionError::Io)?;
        
        // Convert to SFZ
        let sfz_content = match engine.convert(file) {
            Ok(program) => {
                // Convert SfzProgram to string by rendering each region
                let mut content = String::new();
                for region in &program.regions {
                    content.push_str(&region.to_sfz_string());
                    content.push('\n');
                }
                content
            },
            Err(ConversionError::PartialSuccess { warnings, .. }) => {
                warn!("Partial conversion success for {}: {} warnings", 
                      input_path.display(), warnings.len());
                // For partial success, we still want to generate output
                // TODO: Return the partial program from PartialSuccess error
                return Ok(warnings);
            }
            Err(e) => return Err(e),
        };
        
        // Generate output filename
        let mut output_filename = input_path
            .file_stem()
            .unwrap_or_else(|| "converted".as_ref())
            .to_string_lossy()
            .to_string();
        output_filename.push_str(".sfz");
        let output_path = output_dir.join(output_filename);
        
        // Write SFZ file
        std::fs::write(&output_path, sfz_content)
            .map_err(ConversionError::Io)?;
        
        debug!("Successfully converted {} -> {}", 
               input_path.display(), output_path.display());
        
        Ok(engine.warnings().to_vec())
    }
}

// Make ConversionEngine cloneable for parallel processing
impl Clone for ConversionEngine {
    fn clone(&self) -> Self {
        Self {
            recover_from_errors: self.recover_from_errors,
            validate_parameters: self.validate_parameters,
            preserve_original_paths: self.preserve_original_paths,
            performance: self.performance.clone(),
            warnings: Vec::new(), // Start with empty warnings for new instance
        }
    }
}

// Note: Future versions will support generic Read+Seek readers
// For now, we work directly with File for simplicity

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_engine_creation() {
        let engine = ConversionEngine::new();
        assert!(engine.recover_from_errors);
        assert!(engine.validate_parameters);
        assert_eq!(engine.performance.streaming_threshold, 10 * 1024 * 1024);
        assert!(!engine.performance.skip_validation);
    }
    
    #[test]
    fn test_batch_processing_engine() {
        let engine = ConversionEngine::for_batch_processing();
        assert!(engine.recover_from_errors);
        assert!(!engine.validate_parameters); // Disabled for speed
        assert!(engine.performance.skip_validation); // Enabled for performance
        assert_eq!(engine.performance.streaming_threshold, 5 * 1024 * 1024); // Lower threshold
    }
    
    #[test]
    fn test_large_file_engine() {
        let engine = ConversionEngine::for_large_files();
        assert!(engine.recover_from_errors);
        assert!(engine.validate_parameters);
        assert_eq!(engine.performance.streaming_threshold, 1024 * 1024); // 1MB threshold
        assert_eq!(engine.performance.streaming_buffer_size, 128 * 1024); // 128KB buffer
        assert!(engine.performance.use_memory_mapping);
    }
    
    #[test]
    fn test_performance_settings_defaults() {
        let settings = PerformanceSettings::default();
        assert_eq!(settings.streaming_threshold, 10 * 1024 * 1024); // 10MB
        assert_eq!(settings.streaming_buffer_size, 64 * 1024); // 64KB  
        assert!(settings.use_memory_mapping);
        assert!(!settings.skip_validation);
    }
    
    #[test]
    fn test_batch_results_creation() {
        let results = BatchResults::new(10);
        assert_eq!(results.total_files, 10);
        assert_eq!(results.successful_conversions, 0);
        assert_eq!(results.failed_conversions, 0);
        assert_eq!(results.success_rate(), 0.0); // No conversions attempted yet
    }
    
    #[test]
    fn test_batch_results_success_rate() {
        let mut results = BatchResults::new(10);
        results.successful_conversions = 7;
        results.failed_conversions = 3;
        
        assert_eq!(results.success_rate(), 70.0);
        assert!(results.stats().contains("70.0% success rate"));
    }
    
    #[test]
    fn test_batch_processor_thread_limits() {
        let engine = ConversionEngine::for_batch_processing();
        let processor = BatchProcessor::new(engine)
            .with_max_threads(12);
        
        assert_eq!(processor.max_threads, 12);
        
        // Test bounds
        let processor_bounded = BatchProcessor::new(ConversionEngine::new())
            .with_max_threads(20); // Should be capped at 16
        assert_eq!(processor_bounded.max_threads, 16);
        
        let processor_min = BatchProcessor::new(ConversionEngine::new())
            .with_max_threads(0); // Should be at least 1
        assert_eq!(processor_min.max_threads, 1);
    }
    
    #[test]
    fn test_conversion_engine_clone() {
        let mut original = ConversionEngine::for_batch_processing();
        original.warnings.push("test warning".to_string());
        
        let cloned = original.clone();
        
        // Settings should be copied
        assert_eq!(cloned.recover_from_errors, original.recover_from_errors);
        assert_eq!(cloned.validate_parameters, original.validate_parameters);
        assert_eq!(cloned.performance.skip_validation, original.performance.skip_validation);
        
        // Warnings should be reset for new instance
        assert!(cloned.warnings.is_empty());
        assert!(!original.warnings.is_empty());
    }
}