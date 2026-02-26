#[derive(Debug, Default)]
pub struct AkaiProgram {
    pub header: Option<ProgramHeader>,
    pub keygroups: Vec<Keygroup>,
}

#[derive(Debug, Default)]
pub struct ProgramHeader {
    pub _midi_program_number: u8,
    pub _number_of_keygroups: u8,
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
    pub _depth: u8,
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

#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Sfz,
    DecentSampler,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Sfz
    }
}
