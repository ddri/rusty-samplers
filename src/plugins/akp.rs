// AKP format plugin implementation
// Wraps existing AKP parsing functionality in the plugin system

use super::traits::{
    FormatPlugin, FormatReader, FormatWriter, FormatCapabilities,
    InternalProgram, InternalRegion, ProgramMetadata, KeyRange, VelocityRange,
    AudioParameters, ModulationParameters
};
use crate::error::{ConversionError, Result};
use crate::formats::akp::AkaiProgram;
use crate::formats::common::Keygroup;
use std::io::{Read, Seek};
use log::{debug, info, warn};

/// AKP format plugin for Akai S5000/S6000 programs
pub struct AkpPlugin {
    capabilities: FormatCapabilities,
}

impl AkpPlugin {
    /// Create new AKP plugin instance
    pub fn new() -> Result<Self> {
        let capabilities = FormatCapabilities {
            can_read: true,
            can_write: false, // SFZ writing handled separately for now
            supports_multisamples: true,
            supports_velocity_layers: true,
            supports_filters: true,
            supports_envelopes: true,
            supports_lfos: false, // Limited LFO support in AKP
            max_samples: 128,     // Akai S5000/S6000 limit
            supported_sample_rates: vec![44100, 48000],
            quality_rating: 5,    // Production ready
        };

        Ok(Self { capabilities })
    }
}

impl FormatPlugin for AkpPlugin {
    fn name(&self) -> &str {
        "akp"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Akai S5000/S6000 program format support"
    }

    fn file_extensions(&self) -> &[&str] {
        &["akp"]
    }

    fn magic_bytes(&self) -> Option<&[u8]> {
        // AKP files start with RIFF header
        Some(b"RIFF")
    }

    fn capabilities(&self) -> FormatCapabilities {
        self.capabilities.clone()
    }

    fn can_handle(&self, data: &[u8]) -> bool {
        // Check for RIFF header and APRG chunk
        if data.len() < 12 {
            return false;
        }

        data.starts_with(b"RIFF") && 
        data.get(8..12).map(|chunk| chunk == b"APRG").unwrap_or(false)
    }

    fn reader(&self) -> Option<Box<dyn FormatReader>> {
        Some(Box::new(AkpReader::new()))
    }

    fn writer(&self) -> Option<Box<dyn FormatWriter>> {
        // AKP writing not implemented yet
        None
    }
}

/// AKP format reader implementation
pub struct AkpReader;

impl AkpReader {
    pub fn new() -> Self {
        Self
    }

    /// Convert AKP Keygroup to internal region
    fn convert_keygroup(&self, keygroup: &Keygroup, sample_index: usize) -> InternalRegion {
        debug!("Converting keygroup: {}", keygroup.description());

        // Get sample path
        let sample_path = if let Some(ref sample) = keygroup.sample {
            sample.filename.clone()
        } else {
            format!("missing_sample_{}.wav", sample_index)
        };

        // Convert key mapping
        let key_range = KeyRange {
            low: keygroup.zone.low_key,
            high: keygroup.zone.high_key,
            root: None, // Could be calculated from keygroup data if available
        };

        // Convert velocity mapping
        let velocity_range = VelocityRange {
            low: keygroup.zone.low_vel,
            high: keygroup.zone.high_vel,
        };

        // Convert audio parameters
        let mut audio_params = AudioParameters::default();
        
        // Apply tuning if available
        if let Some(ref tune) = keygroup.tune {
            audio_params.tune = Some(tune.semitone as i16);
            audio_params.fine_tune = Some(tune.fine_tune as i16);
            // Convert AKP level to volume (approximate)
            audio_params.volume = Some(crate::formats::common::conversion::akp_level_to_db(tune.level));
        }

        // Apply filter if available
        if let Some(ref filter) = keygroup.filter {
            audio_params.cutoff = Some(crate::formats::common::conversion::akp_cutoff_to_hz(filter.cutoff));
            audio_params.resonance = Some(crate::formats::common::conversion::akp_resonance_to_db(filter.resonance));
        }

        // Convert modulation parameters
        let mut modulation = ModulationParameters::default();
        
        // Apply amplitude envelope if available
        if let Some(ref env) = keygroup.amp_env {
            modulation.amp_attack = Some(crate::formats::common::conversion::akp_envelope_to_seconds(env.attack, 1.0));
            modulation.amp_decay = Some(crate::formats::common::conversion::akp_envelope_to_seconds(env.decay, 1.0));
            modulation.amp_sustain = Some(env.sustain as f32 / 255.0); // Convert 0-255 to 0-1
            modulation.amp_release = Some(crate::formats::common::conversion::akp_envelope_to_seconds(env.release, 1.0));
        }

        // Apply filter envelope if available
        if let Some(ref env) = keygroup.filter_env {
            modulation.filter_attack = Some(crate::formats::common::conversion::akp_envelope_to_seconds(env.attack, 1.0));
            modulation.filter_decay = Some(crate::formats::common::conversion::akp_envelope_to_seconds(env.decay, 1.0));
            modulation.filter_sustain = Some(env.sustain as f32 / 255.0);
            modulation.filter_release = Some(crate::formats::common::conversion::akp_envelope_to_seconds(env.release, 1.0));
        }

        // Apply LFO if available
        if let Some(ref lfo) = keygroup.lfo1 {
            modulation.lfo1_rate = Some(crate::formats::common::conversion::akp_lfo_rate_to_hz(lfo.rate));
            modulation.lfo1_amount = Some(lfo.depth as f32 / 255.0); // Convert to 0-1 range
            modulation.lfo1_target = Some("cutoff".to_string()); // Common LFO target
        }

        InternalRegion {
            sample_path,
            key_range,
            velocity_range,
            audio_params,
            modulation,
        }
    }
}

impl FormatReader for AkpReader {
    fn read(&self, data: &[u8]) -> Result<InternalProgram> {
        info!("Reading AKP file via plugin system");
        
        // Create a cursor from the data
        let mut cursor = std::io::Cursor::new(data);
        
        // Use existing AKP parsing functionality
        let akp_program = crate::formats::akp::legacy::parse_akp_reader(&mut cursor)?;
        
        debug!("Parsed AKP program: {}", akp_program.stats());
        
        // Convert to internal format
        let mut internal_program = InternalProgram::new();
        
        // Set metadata
        internal_program.metadata = ProgramMetadata {
            name: format!("AKP Program ({})", akp_program.keygroups.len()),
            category: Some("Akai Program".to_string()),
            format_version: Some("AKP".to_string()),
            description: Some(format!("Converted from AKP: {} keygroups", 
                                    akp_program.keygroups.len())),
            ..Default::default()
        };

        // Convert keygroups to regions
        for (i, keygroup) in akp_program.keygroups.iter().enumerate() {
            if keygroup.sample.is_none() {
                warn!("Keygroup {} has no sample, creating placeholder", i);
            }
            
            let region = self.convert_keygroup(keygroup, i);
            internal_program.add_region(region);
        }

        // Add global settings
        internal_program.global_settings.insert(
            "original_keygroup_count".to_string(), 
            akp_program.keygroups.len() as f64
        );

        info!("Converted AKP to internal format: {}", internal_program.stats());
        
        Ok(internal_program)
    }

    fn validate(&self, data: &[u8]) -> Result<()> {
        debug!("Validating AKP file format");
        
        // Check minimum length
        if data.len() < 12 {
            return Err(ConversionError::Custom {
                message: "File too short for AKP format".to_string(),
            });
        }
        
        let header = &data[0..12];
        
        if &header[0..4] != b"RIFF" {
            return Err(ConversionError::Custom {
                message: "Invalid RIFF header".to_string(),
            });
        }
        
        if &header[8..12] != b"APRG" {
            return Err(ConversionError::Custom {
                message: "Not an AKP program file (missing APRG chunk)".to_string(),
            });
        }
        
        debug!("AKP format validation passed");
        Ok(())
    }

    fn metadata(&self, data: &[u8]) -> Result<ProgramMetadata> {
        // Quick metadata extraction without full parsing
        self.validate(data)?;
        
        // For now, return basic metadata
        Ok(ProgramMetadata {
            name: "AKP Program".to_string(),
            category: Some("Akai Program".to_string()),
            format_version: Some("AKP".to_string()),
            description: Some("Akai S5000/S6000 program file".to_string()),
            ..Default::default()
        })
    }
}

// Helper function for the existing conversion system to work with plugins
pub fn parse_akp_reader<R: Read + Seek>(reader: &mut R) -> Result<AkaiProgram> {
    // This bridges the gap between the plugin system and existing code
    // until we fully migrate to the plugin architecture
    
    // Use the existing legacy parser for now
    crate::formats::akp::legacy::parse_akp_reader(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_akp_plugin_creation() {
        let plugin = AkpPlugin::new().unwrap();
        assert_eq!(plugin.name(), "akp");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.capabilities().can_read);
        assert!(!plugin.capabilities().can_write);
        assert_eq!(plugin.capabilities().quality_rating, 5);
    }

    #[test]
    fn test_akp_plugin_magic_bytes() {
        let plugin = AkpPlugin::new().unwrap();
        assert_eq!(plugin.magic_bytes(), Some(b"RIFF".as_slice()));
        assert_eq!(plugin.file_extensions(), &["akp"]);
    }

    #[test]
    fn test_akp_format_detection() {
        let plugin = AkpPlugin::new().unwrap();
        
        // Valid AKP header
        let valid_data = b"RIFF\x00\x00\x00\x00APRG";
        assert!(plugin.can_handle(valid_data));
        
        // Invalid data
        let invalid_data = b"INVALID";
        assert!(!plugin.can_handle(invalid_data));
        
        // Too short
        let short_data = b"RIFF";
        assert!(!plugin.can_handle(short_data));
    }

    #[test]
    fn test_akp_reader_creation() {
        let plugin = AkpPlugin::new().unwrap();
        let reader = plugin.reader();
        assert!(reader.is_some());
        
        let writer = plugin.writer();
        assert!(writer.is_none()); // Not implemented yet
    }

    #[test]
    fn test_akp_reader_validation() {
        let reader = AkpReader::new();
        
        // Valid AKP header
        let valid_data = b"RIFF\x00\x00\x00\x00APRG";
        assert!(reader.validate(valid_data).is_ok());
        
        // Invalid header
        let invalid_data = b"INVALID_DATA";
        assert!(reader.validate(invalid_data).is_err());
    }

    #[test]
    fn test_metadata_extraction() {
        let reader = AkpReader::new();
        let valid_data = b"RIFF\x00\x00\x00\x00APRG";
        
        let metadata = reader.metadata(valid_data).unwrap();
        assert_eq!(metadata.name, "AKP Program");
        assert_eq!(metadata.category, Some("Akai Program".to_string()));
        assert_eq!(metadata.format_version, Some("AKP".to_string()));
    }
}