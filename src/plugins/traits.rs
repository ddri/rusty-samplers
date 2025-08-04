// Core plugin traits for multi-format support
// Based on industry analysis of format conversion tools

use crate::error::{ConversionError, Result};
use std::collections::HashMap;

/// Capabilities that a format plugin can support
#[derive(Debug, Clone)]
pub struct FormatCapabilities {
    /// Can read this format
    pub can_read: bool,
    /// Can write this format  
    pub can_write: bool,
    /// Supports multi-sample programs
    pub supports_multisamples: bool,
    /// Supports velocity layers
    pub supports_velocity_layers: bool,
    /// Supports filter parameters
    pub supports_filters: bool,
    /// Supports envelope parameters
    pub supports_envelopes: bool,
    /// Supports LFO parameters
    pub supports_lfos: bool,
    /// Maximum number of samples per program (0 = unlimited)
    pub max_samples: usize,
    /// Supported sample rates
    pub supported_sample_rates: Vec<u32>,
    /// Plugin quality rating (1-5, where 5 is production ready)
    pub quality_rating: u8,
}

impl Default for FormatCapabilities {
    fn default() -> Self {
        Self {
            can_read: true,
            can_write: false,
            supports_multisamples: true,
            supports_velocity_layers: true,
            supports_filters: true,
            supports_envelopes: true,
            supports_lfos: false,
            max_samples: 0, // unlimited
            supported_sample_rates: vec![44100, 48000, 96000],
            quality_rating: 3, // default medium quality
        }
    }
}

/// Internal format abstraction to avoid N×M conversion complexity
/// All formats convert through this intermediate representation
#[derive(Debug, Clone)]
pub struct InternalProgram {
    /// Program metadata
    pub metadata: ProgramMetadata,
    /// Audio regions/keygroups
    pub regions: Vec<InternalRegion>,
    /// Global program settings
    pub global_settings: HashMap<String, f64>,
}

/// Program metadata independent of format
#[derive(Debug, Clone, Default)]
pub struct ProgramMetadata {
    pub name: String,
    pub category: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub created_date: Option<String>,
    pub format_version: Option<String>,
}

/// Audio region in internal format
#[derive(Debug, Clone)]
pub struct InternalRegion {
    /// Sample file reference
    pub sample_path: String,
    /// Key mapping
    pub key_range: KeyRange,
    /// Velocity mapping  
    pub velocity_range: VelocityRange,
    /// Audio parameters
    pub audio_params: AudioParameters,
    /// Modulation parameters
    pub modulation: ModulationParameters,
}

/// MIDI key range
#[derive(Debug, Clone)]
pub struct KeyRange {
    pub low: u8,
    pub high: u8,
    pub root: Option<u8>, // Root/unity key
}

/// MIDI velocity range
#[derive(Debug, Clone)]
pub struct VelocityRange {
    pub low: u8,
    pub high: u8,
}

/// Audio processing parameters
#[derive(Debug, Clone, Default)]
pub struct AudioParameters {
    pub volume: Option<f32>,        // dB
    pub pan: Option<f32>,           // -1.0 to 1.0
    pub tune: Option<i16>,          // semitones
    pub fine_tune: Option<i16>,     // cents
    pub cutoff: Option<f32>,        // Hz
    pub resonance: Option<f32>,     // Q factor
    pub filter_type: Option<String>,
    pub drive: Option<f32>,         // distortion/saturation
}

/// Modulation parameters (envelopes, LFOs)
#[derive(Debug, Clone, Default)]
pub struct ModulationParameters {
    // Amplitude envelope
    pub amp_attack: Option<f32>,    // seconds
    pub amp_decay: Option<f32>,     // seconds
    pub amp_sustain: Option<f32>,   // level 0-1
    pub amp_release: Option<f32>,   // seconds
    
    // Filter envelope
    pub filter_attack: Option<f32>,
    pub filter_decay: Option<f32>,
    pub filter_sustain: Option<f32>,
    pub filter_release: Option<f32>,
    pub filter_env_amount: Option<f32>,
    
    // LFOs
    pub lfo1_rate: Option<f32>,     // Hz
    pub lfo1_amount: Option<f32>,   // depth
    pub lfo1_target: Option<String>, // "cutoff", "volume", etc.
}

/// Main plugin trait that all format plugins must implement
pub trait FormatPlugin: Send + Sync {
    /// Plugin identification
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    /// Supported file extensions (e.g., ["akp", "pgm"])
    fn file_extensions(&self) -> &[&str];
    
    /// Magic bytes for format detection (if available)
    fn magic_bytes(&self) -> Option<&[u8]>;
    
    /// Plugin capabilities
    fn capabilities(&self) -> FormatCapabilities;
    
    /// Check if this plugin can handle the given data
    fn can_handle(&self, data: &[u8]) -> bool {
        // Default implementation checks magic bytes
        if let Some(magic) = self.magic_bytes() {
            data.starts_with(magic)
        } else {
            false // Must be overridden if no magic bytes
        }
    }
    
    /// Get format reader if available
    fn reader(&self) -> Option<Box<dyn FormatReader>> {
        None
    }
    
    /// Get format writer if available  
    fn writer(&self) -> Option<Box<dyn FormatWriter>> {
        None
    }
}

/// Format reading capabilities
pub trait FormatReader: Send + Sync {
    /// Read format data and convert to internal representation
    fn read(&self, data: &[u8]) -> Result<InternalProgram>;
    
    /// Validate format data without full parsing
    fn validate(&self, data: &[u8]) -> Result<()>;
    
    /// Get format-specific metadata
    fn metadata(&self, data: &[u8]) -> Result<ProgramMetadata>;
}

/// Format writing capabilities
pub trait FormatWriter: Send + Sync {
    /// Write internal representation to format
    fn write(&self, program: &InternalProgram) -> Result<Vec<u8>>;
    
    /// Validate internal program can be written to this format
    fn can_write(&self, program: &InternalProgram) -> Result<()>;
    
    /// Get conversion warnings/information
    fn conversion_info(&self, program: &InternalProgram) -> Vec<String>;
}

/// Conversion context for tracking conversion state
#[derive(Debug, Default)]
pub struct ConversionContext {
    pub source_format: String,
    pub target_format: String,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl ConversionContext {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source_format: source.into(),
            target_format: target.into(),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
    
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

impl Default for InternalRegion {
    fn default() -> Self {
        Self {
            sample_path: String::new(),
            key_range: KeyRange { low: 0, high: 127, root: None },
            velocity_range: VelocityRange { low: 0, high: 127 },
            audio_params: AudioParameters::default(),
            modulation: ModulationParameters::default(),
        }
    }
}

impl Default for InternalProgram {
    fn default() -> Self {
        Self {
            metadata: ProgramMetadata::default(),
            regions: Vec::new(),
            global_settings: HashMap::new(),
        }
    }
}

impl InternalProgram {
    /// Create a new empty program
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a region to this program
    pub fn add_region(&mut self, region: InternalRegion) {
        self.regions.push(region);
    }
    
    /// Get program statistics
    pub fn stats(&self) -> String {
        format!(
            "Program '{}': {} regions, {} global settings",
            self.metadata.name,
            self.regions.len(),
            self.global_settings.len()
        )
    }
    
    /// Validate program integrity
    pub fn validate(&self) -> Result<()> {
        if self.regions.is_empty() {
            return Err(ConversionError::Custom {
                message: "Program contains no regions".to_string(),
            });
        }
        
        for (i, region) in self.regions.iter().enumerate() {
            if region.sample_path.is_empty() {
                return Err(ConversionError::Custom {
                    message: format!("Region {} missing sample path", i),
                });
            }
            
            if region.key_range.low > region.key_range.high {
                return Err(ConversionError::Custom {
                    message: format!("Region {} has invalid key range", i),
                });
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_capabilities_default() {
        let caps = FormatCapabilities::default();
        assert!(caps.can_read);
        assert!(!caps.can_write);
        assert!(caps.supports_multisamples);
        assert_eq!(caps.quality_rating, 3);
    }

    #[test]
    fn test_internal_program_creation() {
        let mut program = InternalProgram::new();
        assert_eq!(program.regions.len(), 0);
        assert_eq!(program.global_settings.len(), 0);
        
        let region = InternalRegion {
            sample_path: "test.wav".to_string(),
            ..Default::default()
        };
        program.add_region(region);
        
        assert_eq!(program.regions.len(), 1);
        assert!(program.stats().contains("1 regions"));
    }

    #[test]
    fn test_internal_program_validation() {
        let empty_program = InternalProgram::new();
        assert!(empty_program.validate().is_err());
        
        let mut valid_program = InternalProgram::new();
        valid_program.add_region(InternalRegion {
            sample_path: "test.wav".to_string(),
            key_range: KeyRange { low: 60, high: 72, root: Some(66) },
            ..Default::default()
        });
        
        assert!(valid_program.validate().is_ok());
    }

    #[test]
    fn test_conversion_context() {
        let mut context = ConversionContext::new("akp", "sfz");
        assert_eq!(context.source_format, "akp");
        assert_eq!(context.target_format, "sfz");
        
        context.add_warning("Test warning");
        context.set_metadata("version", "1.0");
        
        assert_eq!(context.warnings.len(), 1);
        assert_eq!(context.metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_key_and_velocity_ranges() {
        let key_range = KeyRange { low: 60, high: 72, root: Some(66) };
        assert_eq!(key_range.low, 60);
        assert_eq!(key_range.high, 72);
        assert_eq!(key_range.root, Some(66));
        
        let vel_range = VelocityRange { low: 1, high: 127 };
        assert_eq!(vel_range.low, 1);
        assert_eq!(vel_range.high, 127);
    }
}