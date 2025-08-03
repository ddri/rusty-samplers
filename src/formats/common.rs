// Common data structures and utilities shared between formats

/// Sample information common to both AKP and SFZ formats
#[derive(Debug, Clone, Default)]
pub struct Sample {
    pub filename: String,
}

/// Tuning information for sample playback
#[derive(Debug, Clone, Default)]
pub struct Tune {
    pub level: u8,
    pub semitone: i8,
    pub fine_tune: i8,
}

/// Filter parameters for sample processing
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub cutoff: u8,
    pub resonance: u8,
    pub filter_type: u8,
}

/// ADSR envelope parameters
#[derive(Debug, Clone, Default)]
pub struct Envelope {
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

/// Low-frequency oscillator parameters
#[derive(Debug, Clone, Default)]
pub struct Lfo {
    pub waveform: u8,
    pub rate: u8,
    pub delay: u8,
    pub depth: u8,
}

/// Key and velocity mapping information
#[derive(Debug, Clone, Default)]
pub struct Zone {
    pub low_key: u8,
    pub high_key: u8,
    pub low_vel: u8,
    pub high_vel: u8,
}

/// Complete keygroup/region information
#[derive(Debug, Clone, Default)]
pub struct Keygroup {
    pub zone: Zone,
    pub sample: Option<Sample>,
    pub tune: Option<Tune>,
    pub filter: Option<Filter>,
    pub amp_env: Option<Envelope>,
    pub filter_env: Option<Envelope>,
    pub aux_env: Option<Envelope>,
    pub lfo1: Option<Lfo>,
    pub lfo2: Option<Lfo>,
}

/// Program header information
#[derive(Debug, Clone, Default)]
pub struct ProgramHeader {
    pub midi_program_number: u8,
    pub number_of_keygroups: u8,
}

impl Keygroup {
    /// Create a new keygroup with basic zone information
    pub fn new(low_key: u8, high_key: u8, low_vel: u8, high_vel: u8) -> Self {
        Self {
            zone: Zone { low_key, high_key, low_vel, high_vel },
            ..Default::default()
        }
    }
    
    /// Check if this keygroup has all required components for conversion
    pub fn is_valid(&self) -> bool {
        self.sample.is_some()
    }
    
    /// Get a human-readable description of this keygroup
    pub fn description(&self) -> String {
        let sample_name = self.sample
            .as_ref()
            .map(|s| s.filename.as_str())
            .unwrap_or("no sample");
        
        format!(
            "Keygroup: keys {}-{}, vel {}-{}, sample: {}",
            self.zone.low_key,
            self.zone.high_key,
            self.zone.low_vel,
            self.zone.high_vel,
            sample_name
        )
    }
}

/// Parameter conversion utilities
pub mod conversion {
    /// Convert AKP filter cutoff (0-100) to SFZ frequency (Hz)
    pub fn akp_cutoff_to_hz(akp_value: u8) -> f32 {
        20.0 * (1000.0f32).powf(akp_value as f32 / 100.0)
    }
    
    /// Convert AKP resonance (0-100) to SFZ dB (0-24dB)
    pub fn akp_resonance_to_db(akp_value: u8) -> f32 {
        akp_value as f32 * 0.24
    }
    
    /// Convert AKP LFO rate (0-100) to SFZ frequency (Hz)
    pub fn akp_lfo_rate_to_hz(akp_value: u8) -> f32 {
        0.1 * (300.0f32).powf(akp_value as f32 / 100.0)
    }
    
    /// Convert AKP level (0-100) to SFZ volume (dB)
    pub fn akp_level_to_db(akp_value: u8) -> f32 {
        (akp_value as f32 / 100.0) * 48.0 - 48.0
    }
    
    /// Convert AKP envelope time to SFZ seconds
    pub fn akp_envelope_to_seconds(akp_value: u8, scale_factor: f32) -> f32 {
        (akp_value as f32).powf(2.0) / 10000.0 * scale_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygroup_validity() {
        let mut kg = Keygroup::new(60, 64, 1, 127);
        assert!(!kg.is_valid()); // No sample
        
        kg.sample = Some(Sample { filename: "test.wav".to_string() });
        assert!(kg.is_valid()); // Now has sample
    }
    
    #[test]
    fn test_parameter_conversion() {
        use conversion::*;
        
        // Test filter cutoff conversion
        let cutoff_hz = akp_cutoff_to_hz(50);
        assert!((cutoff_hz - 632.4).abs() < 1.0);
        
        // Test resonance conversion
        let resonance_db = akp_resonance_to_db(25);
        assert_eq!(resonance_db, 6.0);
        
        // Test LFO rate conversion
        let lfo_hz = akp_lfo_rate_to_hz(50);
        assert!((lfo_hz - 1.73).abs() < 0.1);
        
        // Test level conversion
        let volume_db = akp_level_to_db(75);
        assert_eq!(volume_db, -12.0);
    }
}