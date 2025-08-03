// SFZ format type definitions

/// Complete SFZ program data
#[derive(Debug, Default)]
pub struct SfzProgram {
    pub regions: Vec<SfzRegion>,
    pub global_settings: Option<SfzGlobal>,
}

/// SFZ region definition
#[derive(Debug, Clone)]
pub struct SfzRegion {
    // Sample information
    pub sample: String,
    
    // Key/velocity mapping
    pub lokey: u8,
    pub hikey: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub pitch_keycenter: Option<u8>,
    
    // Amplitude
    pub volume: Option<f32>,
    pub tune: Option<i8>,
    pub fine_tune: Option<i8>,
    
    // Amplitude envelope
    pub ampeg_attack: Option<f32>,
    pub ampeg_decay: Option<f32>,
    pub ampeg_sustain: Option<u8>,
    pub ampeg_release: Option<f32>,
    
    // Filter
    pub fil_type: Option<String>,
    pub cutoff: Option<f32>,
    pub resonance: Option<f32>,
    
    // Filter envelope
    pub fileg_attack: Option<f32>,
    pub fileg_decay: Option<f32>,
    pub fileg_sustain: Option<u8>,
    pub fileg_release: Option<f32>,
    
    // LFOs
    pub lfo1_freq: Option<f32>,
    pub lfo2_freq: Option<f32>,
}

/// SFZ global settings
#[derive(Debug, Clone, Default)]
pub struct SfzGlobal {
    pub volume: Option<f32>,
    pub tune: Option<i8>,
}

impl SfzProgram {
    /// Create a new empty SFZ program
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a region to this program
    pub fn add_region(&mut self, region: SfzRegion) {
        self.regions.push(region);
    }
    
    /// Get the number of regions
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

impl SfzRegion {
    /// Create a new SFZ region with basic parameters
    pub fn new(
        sample: String,
        lokey: u8,
        hikey: u8,
        lovel: u8,
        hivel: u8,
    ) -> Self {
        Self {
            sample,
            lokey,
            hikey,
            lovel,
            hivel,
            pitch_keycenter: None,
            volume: None,
            tune: None,
            fine_tune: None,
            ampeg_attack: None,
            ampeg_decay: None,
            ampeg_sustain: None,
            ampeg_release: None,
            fil_type: None,
            cutoff: None,
            resonance: None,
            fileg_attack: None,
            fileg_decay: None,
            fileg_sustain: None,
            fileg_release: None,
            lfo1_freq: None,
            lfo2_freq: None,
        }
    }
    
    /// Convert this region to SFZ format string
    pub fn to_sfz_string(&self) -> String {
        let mut lines = Vec::new();
        
        lines.push("<region>".to_string());
        
        // Sample
        lines.push(format!("sample={}", self.sample.replace("\\", "/")));
        
        // Key/velocity ranges
        lines.push(format!("lokey={}", self.lokey));
        lines.push(format!("hikey={}", self.hikey));
        lines.push(format!("lovel={}", self.lovel));
        lines.push(format!("hivel={}", self.hivel));
        
        // Optional parameters
        if let Some(center) = self.pitch_keycenter {
            lines.push(format!("pitch_keycenter={}", center));
        }
        
        if let Some(vol) = self.volume {
            lines.push(format!("volume={:.2}", vol));
        }
        
        if let Some(tune) = self.tune {
            lines.push(format!("tune={}", tune));
        }
        
        if let Some(fine) = self.fine_tune {
            lines.push(format!("fine_tune={}", fine));
        }
        
        // Amplitude envelope
        if let Some(attack) = self.ampeg_attack {
            lines.push(format!("ampeg_attack={:.3}", attack));
        }
        if let Some(decay) = self.ampeg_decay {
            lines.push(format!("ampeg_decay={:.3}", decay));
        }
        if let Some(sustain) = self.ampeg_sustain {
            lines.push(format!("ampeg_sustain={}", sustain));
        }
        if let Some(release) = self.ampeg_release {
            lines.push(format!("ampeg_release={:.3}", release));
        }
        
        // Filter
        if let Some(ref fil_type) = self.fil_type {
            lines.push(format!("fil_type={}", fil_type));
        }
        if let Some(cutoff) = self.cutoff {
            lines.push(format!("cutoff={:.1}", cutoff));
        }
        if let Some(resonance) = self.resonance {
            lines.push(format!("resonance={:.1}", resonance));
        }
        
        // Filter envelope
        if let Some(attack) = self.fileg_attack {
            lines.push(format!("fileg_attack={:.3}", attack));
        }
        if let Some(decay) = self.fileg_decay {
            lines.push(format!("fileg_decay={:.3}", decay));
        }
        if let Some(sustain) = self.fileg_sustain {
            lines.push(format!("fileg_sustain={}", sustain));
        }
        if let Some(release) = self.fileg_release {
            lines.push(format!("fileg_release={:.3}", release));
        }
        
        // LFOs
        if let Some(freq) = self.lfo1_freq {
            lines.push(format!("lfo1_freq={:.2}", freq));
        }
        if let Some(freq) = self.lfo2_freq {
            lines.push(format!("lfo2_freq={:.2}", freq));
        }
        
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sfz_region_creation() {
        let region = SfzRegion::new(
            "test.wav".to_string(),
            60, 64, 1, 127
        );
        
        assert_eq!(region.sample, "test.wav");
        assert_eq!(region.lokey, 60);
        assert_eq!(region.hikey, 64);
    }
    
    #[test]
    fn test_sfz_string_generation() {
        let mut region = SfzRegion::new(
            "piano.wav".to_string(),
            60, 64, 1, 127
        );
        region.volume = Some(-6.0);
        region.tune = Some(12);
        
        let sfz_string = region.to_sfz_string();
        
        assert!(sfz_string.contains("<region>"));
        assert!(sfz_string.contains("sample=piano.wav"));
        assert!(sfz_string.contains("lokey=60"));
        assert!(sfz_string.contains("hikey=64"));
        assert!(sfz_string.contains("volume=-6.00"));
        assert!(sfz_string.contains("tune=12"));
    }
}