// Conversion engine for Rusty Samplers
// High-level API for converting between formats

use crate::error::{ConversionError, Result};
use crate::formats::{akp::AkaiProgram, sfz::SfzProgram};
use std::fs::File;
// use std::io::{Read, Seek}; // For future generic reader support
use log::{info, debug, warn};

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
    #[cfg(feature = "legacy-parser")]
    {
        debug!("Using legacy parser");
        crate::formats::akp::legacy::parse_akp_file(file)
    }
    
    #[cfg(not(feature = "legacy-parser"))]
    {
        Err(ConversionError::Custom {
            message: "No parser available - enable legacy-parser or binrw-parser feature".to_string(),
        })
    }
}

/// Conversion engine with configuration options
pub struct ConversionEngine {
    pub recover_from_errors: bool,
    pub validate_parameters: bool,
    pub preserve_original_paths: bool,
    warnings: Vec<String>,
}

impl ConversionEngine {
    /// Create a new conversion engine with default settings
    pub fn new() -> Self {
        Self {
            recover_from_errors: true,
            validate_parameters: true,
            preserve_original_paths: true,
            warnings: Vec::new(),
        }
    }
    
    /// Create a conversion engine optimized for batch processing
    pub fn for_batch_processing() -> Self {
        Self {
            recover_from_errors: true,
            validate_parameters: false, // Skip validation for speed
            preserve_original_paths: true,
            warnings: Vec::new(),
        }
    }
    
    /// Convert an AKP file with the current configuration
    pub fn convert(&mut self, file: File) -> Result<SfzProgram> {
        self.warnings.clear();
        
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
        
        if self.validate_parameters {
            self.validate_sfz_program(&sfz_program)?;
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
    }
    
    #[test]
    fn test_batch_processing_engine() {
        let engine = ConversionEngine::for_batch_processing();
        assert!(engine.recover_from_errors);
        assert!(!engine.validate_parameters); // Disabled for speed
    }
}