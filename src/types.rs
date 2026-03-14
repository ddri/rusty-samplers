// ---- Output format enum (unchanged) ----

#[derive(Clone, Copy, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Sfz,
    DecentSampler,
}

// ---- RIFF chunk header (unchanged) ----

#[derive(Debug)]
pub struct RiffChunkHeader {
    pub id: String,
    pub size: u32,
}

// ---- Top-level program ----

#[derive(Debug, Default)]
pub struct AkaiProgram {
    pub header: Option<ProgramHeader>,
    pub output: Option<ProgramOutput>,
    pub tuning: Option<ProgramTuning>,
    pub lfo1: Option<Lfo>,
    pub lfo2: Option<Lfo>,
    pub modulation: Option<ProgramModulation>,
    pub keygroups: Vec<Keygroup>,
}

#[derive(Debug, Default)]
pub struct ProgramHeader {
    pub midi_program_number: u8,
    pub number_of_keygroups: u8,
}

// ---- ProgramOutput (out chunk, 8 bytes) ----

#[derive(Debug)]
pub struct ProgramOutput {
    pub loudness: u8,
    pub amp_mod_1: u8,
    pub amp_mod_2: u8,
    pub pan_mod_1: u8,
    pub pan_mod_2: u8,
    pub pan_mod_3: u8,
    pub velocity_sensitivity: i8,
}

impl Default for ProgramOutput {
    fn default() -> Self {
        Self {
            loudness: 85,
            amp_mod_1: 0,
            amp_mod_2: 0,
            pan_mod_1: 0,
            pan_mod_2: 0,
            pan_mod_3: 0,
            velocity_sensitivity: 25,
        }
    }
}

// ---- ProgramTuning (tune chunk, 22 bytes) ----

#[derive(Debug)]
pub struct ProgramTuning {
    pub semitone: i8,
    pub fine: i8,
    pub detune: [i8; 12],
    pub pitchbend_up: u8,
    pub pitchbend_down: u8,
    pub bend_mode: u8,
    pub aftertouch: i8,
}

impl Default for ProgramTuning {
    fn default() -> Self {
        Self {
            semitone: 0,
            fine: 0,
            detune: [0; 12],
            pitchbend_up: 2,
            pitchbend_down: 2,
            bend_mode: 0,
            aftertouch: 0,
        }
    }
}

// ---- Lfo (lfo chunk, 12 bytes) ----

#[derive(Debug)]
#[derive(Default)]
pub struct Lfo {
    pub waveform: u8,
    pub rate: u8,
    pub delay: u8,
    pub depth: u8,
    pub sync: u8,
    pub retrigger: u8,
    pub modwheel: u8,
    pub aftertouch: u8,
    pub rate_mod: i8,
    pub delay_mod: i8,
    pub depth_mod: i8,
}


// ---- ProgramModulation (mods chunk, 38 bytes) ----

#[derive(Debug)]
pub struct ProgramModulation {
    pub amp_mod_1_source: u8,
    pub amp_mod_2_source: u8,
    pub pan_mod_1_source: u8,
    pub pan_mod_2_source: u8,
    pub pan_mod_3_source: u8,
    pub lfo1_rate_mod_source: u8,
    pub lfo1_delay_mod_source: u8,
    pub lfo1_depth_mod_source: u8,
    pub lfo2_rate_mod_source: u8,
    pub lfo2_delay_mod_source: u8,
    pub lfo2_depth_mod_source: u8,
    pub pitch_mod_1_source: u8,
    pub pitch_mod_2_source: u8,
    pub amp_mod_source: u8,
    pub filter_mod_1_source: u8,
    pub filter_mod_2_source: u8,
    pub filter_mod_3_source: u8,
}

impl Default for ProgramModulation {
    fn default() -> Self {
        Self {
            amp_mod_1_source: 6,   // KEYBOARD
            amp_mod_2_source: 0,
            pan_mod_1_source: 0,
            pan_mod_2_source: 0,
            pan_mod_3_source: 0,
            lfo1_rate_mod_source: 0,
            lfo1_delay_mod_source: 0,
            lfo1_depth_mod_source: 0,
            lfo2_rate_mod_source: 0,
            lfo2_delay_mod_source: 0,
            lfo2_depth_mod_source: 0,
            pitch_mod_1_source: 7, // LFO1
            pitch_mod_2_source: 0,
            amp_mod_source: 5,     // VELOCITY
            filter_mod_1_source: 0,
            filter_mod_2_source: 0,
            filter_mod_3_source: 0,
        }
    }
}

// ---- Keygroup ----

#[derive(Debug)]
pub struct Keygroup {
    // From kloc (16 bytes)
    pub low_key: u8,
    pub high_key: u8,
    pub semitone_tune: i8,
    pub fine_tune: i8,
    pub override_fx: u8,
    pub fx_send_level: u8,
    pub pitch_mod_1: i8,
    pub pitch_mod_2: i8,
    pub amp_mod: i8,
    pub zone_crossfade: u8,
    pub mute_group: u8,
    // Zones (up to 4)
    pub zones: Vec<Zone>,
    // Envelopes
    pub amp_env: Option<Envelope>,
    pub filter_env: Option<FilterEnvelope>,
    pub aux_env: Option<AuxEnvelope>,
    // Filter
    pub filter: Option<Filter>,
}

impl Default for Keygroup {
    fn default() -> Self {
        Self {
            low_key: 21,
            high_key: 127,
            semitone_tune: 0,
            fine_tune: 0,
            override_fx: 0,
            fx_send_level: 0,
            pitch_mod_1: 0,
            pitch_mod_2: 0,
            amp_mod: 0,
            zone_crossfade: 0,
            mute_group: 0,
            zones: Vec::new(),
            amp_env: None,
            filter_env: None,
            aux_env: None,
            filter: None,
        }
    }
}

// ---- Zone (zone chunk, 46-48 bytes) ----

#[derive(Debug)]
pub struct Zone {
    pub sample_name: String,
    pub low_vel: u8,
    pub high_vel: u8,
    pub fine_tune: i8,
    pub semitone_tune: i8,
    pub filter: i8,
    pub pan: i8,
    pub playback: u8,
    pub output: u8,
    pub level: i8,
    pub keyboard_track: u8,
    pub vel_to_start: i16,
}

impl Default for Zone {
    fn default() -> Self {
        Self {
            sample_name: String::new(),
            low_vel: 0,
            high_vel: 127,
            fine_tune: 0,
            semitone_tune: 0,
            filter: 0,
            pan: 0,
            playback: 4, // AS SAMPLE
            output: 0,
            level: 0,
            keyboard_track: 1, // ON
            vel_to_start: 0,
        }
    }
}

// ---- Envelope (amp env, 18 bytes) ----

#[derive(Debug, Default)]
pub struct Envelope {
    pub attack: u8,
    pub decay: u8,
    pub release: u8,
    pub sustain: u8,
    pub velocity_attack: i8,
    pub keyscale: i8,
    pub on_vel_release: i8,
    pub off_vel_release: i8,
}

// ---- FilterEnvelope (filter env, 18 bytes) ----

#[derive(Debug, Default)]
pub struct FilterEnvelope {
    pub attack: u8,
    pub decay: u8,
    pub release: u8,
    pub sustain: u8,
    pub depth: i8,
    pub velocity_attack: i8,
    pub keyscale: i8,
    pub on_vel_release: i8,
    pub off_vel_release: i8,
}

// ---- AuxEnvelope (aux env, 18 bytes) ----

#[derive(Debug, Default)]
pub struct AuxEnvelope {
    pub rate_1: u8,
    pub rate_2: u8,
    pub rate_3: u8,
    pub rate_4: u8,
    pub level_1: u8,
    pub level_2: u8,
    pub level_3: u8,
    pub level_4: u8,
    pub vel_rate_1: i8,
    pub key_rate_2_4: i8,
    pub vel_rate_4: i8,
    pub off_vel_rate_4: i8,
    pub vel_output_level: i8,
}

// ---- Filter (filt chunk, 10 bytes) ----

#[derive(Debug)]
pub struct Filter {
    pub filter_type: u8,
    pub cutoff: u8,
    pub resonance: u8,
    pub keyboard_track: i8,
    pub mod_input_1: i8,
    pub mod_input_2: i8,
    pub mod_input_3: i8,
    pub headroom: u8,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            filter_type: 0,
            cutoff: 100,
            resonance: 0,
            keyboard_track: 0,
            mod_input_1: 0,
            mod_input_2: 0,
            mod_input_3: 0,
            headroom: 0,
        }
    }
}

// ---- Conversion helpers ----

/// Shared envelope timing conversions for amp and filter envelopes.
/// Both use identical exponential curves: attack/decay use exp(x*4)*0.001,
/// release uses exp(x*5)*0.001 with 0.001s minimum to avoid clicks.
pub trait EnvelopeTiming {
    fn attack_raw(&self) -> u8;
    fn decay_raw(&self) -> u8;
    fn release_raw(&self) -> u8;
    fn sustain_raw(&self) -> u8;

    /// Convert AKP attack (0-100) to seconds (exponential curve).
    fn attack_time(&self) -> f32 {
        let v = self.attack_raw();
        if v == 0 { 0.0 } else { (v as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    /// Convert AKP decay (0-100) to seconds (exponential curve).
    fn decay_time(&self) -> f32 {
        let v = self.decay_raw();
        if v == 0 { 0.0 } else { (v as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    /// Convert AKP release (0-100) to seconds. Minimum 0.001s to avoid clicks.
    fn release_time(&self) -> f32 {
        let v = self.release_raw();
        if v == 0 { 0.001 } else { (v as f32 / 100.0 * 5.0).exp() * 0.001 }
    }

    /// Convert AKP sustain (0-100) to normalized 0.0-1.0.
    fn sustain_normalized(&self) -> f32 {
        self.sustain_raw() as f32 / 100.0
    }
}

impl EnvelopeTiming for Envelope {
    fn attack_raw(&self) -> u8 { self.attack }
    fn decay_raw(&self) -> u8 { self.decay }
    fn release_raw(&self) -> u8 { self.release }
    fn sustain_raw(&self) -> u8 { self.sustain }
}

impl EnvelopeTiming for FilterEnvelope {
    fn attack_raw(&self) -> u8 { self.attack }
    fn decay_raw(&self) -> u8 { self.decay }
    fn release_raw(&self) -> u8 { self.release }
    fn sustain_raw(&self) -> u8 { self.sustain }
}

impl Lfo {
    /// Convert AKP waveform byte to name. Spec: 0=SINE,1=TRI,2=SQ,3=SQ+,4=SQ-,5=SAW_BI,6=SAW_UP,7=SAW_DN,8=RANDOM
    pub fn waveform_name(&self) -> &'static str {
        match self.waveform {
            0 => "sine",
            1 => "triangle",
            2 => "square",
            3 => "square",    // SQ+ (positive phase)
            4 => "square",    // SQ- (negative phase)
            5 => "saw",       // SAW BI (bipolar)
            6 => "saw",       // SAW UP
            7 => "saw",       // SAW DOWN (ramp)
            8 => "random",
            _ => "sine",
        }
    }

    /// Convert AKP LFO rate (0-100) to Hz (0.1-30 Hz, logarithmic).
    pub fn rate_hz(&self) -> f32 {
        0.1 * (300.0f32).powf(self.rate as f32 / 100.0)
    }

    /// Convert AKP LFO depth (0-100) to normalized 0.0-1.0.
    /// Used for pitch modulation: depth * 100 gives cents (0-100 cent range).
    /// depth=1 → 1 cent (subtle vibrato), depth=99 → 99 cents (dramatic wobble).
    pub fn depth_normalized(&self) -> f32 {
        self.depth as f32 / 100.0
    }
}

impl ProgramOutput {
    /// Convert AKP loudness (0-100) to dB. Loudness is a linear gain percentage,
    /// so dB = 20 * log10(loudness / 100). Loudness=100 → 0dB, 85 → -1.4dB, 50 → -6dB.
    pub fn volume_db(&self) -> f32 {
        if self.loudness == 0 {
            return -60.0;
        }
        20.0 * (self.loudness as f32 / 100.0).log10()
    }
}

/// Map AKP modulation source (0-14) to SFZ opcode suffix.
/// Returns the full suffix including connector: "_oncc1", "_chanaft", etc.
/// This avoids invalid opcodes like "pitch_onbend" — bend and aftertouch
/// use dedicated suffixes without the "on" prefix.
/// Returns None for sources that don't map to a CC/controller.
pub fn mod_source_sfz_suffix(source: u8) -> Option<&'static str> {
    match source {
        1 => Some("_oncc1"),    // MODWHEEL
        2 => Some("_bend"),     // BEND (not "_onbend")
        3 => Some("_chanaft"),  // AFTERTOUCH (not "_onchanaft")
        4 => Some("_oncc16"),   // EXTERNAL (typically general purpose controller)
        _ => None,              // NO_SOURCE, VELOCITY, KEYBOARD, LFOs, envelopes, deltas
    }
}

/// Classify AKP modulation source (0-14) into a type category.
pub fn mod_source_type(source: u8) -> &'static str {
    match source {
        0 => "none",
        1..=4 => "cc",
        5 => "velocity",
        6 => "keyboard",
        7 => "lfo1",
        8 => "lfo2",
        9 => "amp_env",
        10 => "filt_env",
        11 => "aux_env",
        12..=14 => "delta",
        _ => "unknown",
    }
}

/// Human-readable name for an AKP modulation source.
pub fn mod_source_name(source: u8) -> &'static str {
    match source {
        0 => "NO_SOURCE",
        1 => "MODWHEEL",
        2 => "BEND",
        3 => "AFTERTOUCH",
        4 => "EXTERNAL",
        5 => "VELOCITY",
        6 => "KEYBOARD",
        7 => "LFO1",
        8 => "LFO2",
        9 => "AMP_ENV",
        10 => "FILT_ENV",
        11 => "AUX_ENV",
        12 => "MIDI_NOTE",
        13 => "MIDI_VELOCITY",
        14 => "MIDI_RANDOM",
        _ => "UNKNOWN",
    }
}

impl Filter {
    /// Convert AKP cutoff (0-100) to Hz (20-20000, logarithmic).
    pub fn cutoff_hz(&self) -> f32 {
        20.0 * (1000.0f32).powf(self.cutoff as f32 / 100.0)
    }

    /// Convert AKP resonance (0-12) to dB (0-40 range, linear).
    pub fn resonance_db(&self) -> f32 {
        (self.resonance as f32 / 12.0) * 40.0
    }

    /// Map Akai filter type (0-25) to SFZ fil_type opcode.
    pub fn sfz_filter_type(&self) -> &'static str {
        match self.filter_type {
            0..=2 => "lpf_2p",           // 2-pole LP, 4-pole LP, 2-pole LP+
            3..=5 => "bpf_2p",           // 2-pole BP, 4-pole BP, 2-pole BP+
            6 | 8 => "hpf_1p",               // 1-pole HP, 1-pole HP+
            7 => "hpf_2p",                    // 2-pole HP
            12..=16 => "brf_2p", // Notch variants
            17..=21 => "pkf_2p", // Peak variants
            _ => "lpf_2p",                    // Morphing, phaser, voweliser -> fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_depth_normalized_boundaries() {
        assert_eq!(Lfo { depth: 0, ..Default::default() }.depth_normalized(), 0.0);
        assert!((Lfo { depth: 50, ..Default::default() }.depth_normalized() - 0.5).abs() < f32::EPSILON);
        assert!((Lfo { depth: 100, ..Default::default() }.depth_normalized() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_envelope_timing_trait_shared_behavior() {
        let amp = Envelope { attack: 50, decay: 50, sustain: 80, release: 50, ..Default::default() };
        let filt = FilterEnvelope { attack: 50, decay: 50, sustain: 80, release: 50, ..Default::default() };
        // Same raw values produce identical timing
        assert_eq!(amp.attack_time(), filt.attack_time());
        assert_eq!(amp.decay_time(), filt.decay_time());
        assert_eq!(amp.release_time(), filt.release_time());
        assert_eq!(amp.sustain_normalized(), filt.sustain_normalized());
    }

    #[test]
    fn test_envelope_timing_zero_values() {
        let env = Envelope { attack: 0, decay: 0, sustain: 0, release: 0, ..Default::default() };
        assert_eq!(env.attack_time(), 0.0);
        assert_eq!(env.decay_time(), 0.0);
        assert_eq!(env.sustain_normalized(), 0.0);
        assert_eq!(env.release_time(), 0.001); // minimum to avoid clicks
    }
}
