#[derive(Debug, Default)]
pub struct AkaiProgram {
    pub header: Option<ProgramHeader>,
    pub keygroups: Vec<Keygroup>,
}

#[derive(Debug, Default)]
pub struct ProgramHeader {
    pub midi_program_number: u8,
    pub number_of_keygroups: u8,
}

#[derive(Debug, Default)]
pub struct Keygroup {
    pub low_key: u8,
    pub high_key: u8,
    pub low_vel: u8,
    pub high_vel: u8,
    pub sample: Option<Sample>,
    pub tune: Option<Tune>,
    pub filter: Option<Filter>,
    pub amp_env: Option<Envelope>,
    pub filter_env: Option<Envelope>,
    pub aux_env: Option<Envelope>,
    pub lfo1: Option<Lfo>,
    pub lfo2: Option<Lfo>,
    pub mods: Vec<Modulation>,
}

#[derive(Debug, Default)]
pub struct Sample {
    pub filename: String,
}

#[derive(Debug, Default)]
pub struct Tune {
    pub level: u8,
    pub semitone: i8,
    pub fine_tune: i8,
}

#[derive(Debug, Default)]
pub struct Filter {
    pub cutoff: u8,
    pub resonance: u8,
    pub filter_type: u8,
}

#[derive(Debug, Default)]
pub struct Envelope {
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

#[derive(Debug, Default)]
pub struct Lfo {
    pub waveform: u8,
    pub rate: u8,
    pub delay: u8,
    pub depth: u8,
}

#[derive(Debug, Default)]
pub struct Modulation {
    pub source: u8,
    pub destination: u8,
    pub amount: u8,
}

#[derive(Debug)]
pub struct RiffChunkHeader {
    pub id: String,
    pub size: u32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Sfz,
    DecentSampler,
}

// --- Shared conversion helpers ---

impl Lfo {
    /// Convert AKP waveform byte to waveform name string.
    pub fn waveform_name(&self) -> &'static str {
        match self.waveform {
            0 => "triangle",
            1 => "sine",
            2 => "square",
            3 => "saw",
            4 => "ramp",
            5 => "random",
            _ => "triangle",
        }
    }

    /// Convert AKP LFO rate (0-100) to Hz (0.1–30 Hz, logarithmic).
    pub fn rate_hz(&self) -> f32 {
        0.1 * (300.0f32).powf(self.rate as f32 / 100.0)
    }

    /// Convert AKP LFO depth (0-100) to normalized 0.0–1.0.
    pub fn depth_normalized(&self) -> f32 {
        self.depth as f32 / 100.0
    }
}

impl Envelope {
    /// Convert AKP envelope attack (0-100) to seconds (exponential curve).
    pub fn attack_time(&self) -> f32 {
        if self.attack == 0 { 0.0 } else { (self.attack as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    /// Convert AKP envelope decay (0-100) to seconds (exponential curve).
    pub fn decay_time(&self) -> f32 {
        if self.decay == 0 { 0.0 } else { (self.decay as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    /// Convert AKP envelope release (0-100) to seconds (exponential curve).
    /// Uses a minimum of 0.001s to avoid clicks.
    pub fn release_time(&self) -> f32 {
        if self.release == 0 { 0.001 } else { (self.release as f32 / 100.0 * 5.0).exp() * 0.001 }
    }

    /// Convert AKP envelope sustain (0-100) to normalized 0.0–1.0 for formats that need it.
    pub fn sustain_normalized(&self) -> f32 {
        self.sustain as f32 / 100.0
    }
}

impl Tune {
    /// Convert AKP level (0-100) to dB (-60 to +6 dB range).
    pub fn volume_db(&self) -> f32 {
        (self.level as f32 / 100.0) * 66.0 - 60.0
    }
}
