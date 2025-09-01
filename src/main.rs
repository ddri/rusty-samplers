// Rusty Samplers: AKP to SFZ Converter - v1.0
//
// This version significantly refines the parameter conversion for envelopes, filter, and LFO.
//
// Key Changes:
// 1. Improved filter cutoff and resonance scaling for more accurate sound.
// 2. Added LFO rate conversion to SFZ `lfoN_freq` opcode.
// 3. Added parsing and initial SFZ generation for the `mods` chunk.
// 4. Refined envelope (attack, decay, release) scaling to use an exponential curve for better sonic accuracy.
// 5. Updated version to 1.0.
//
// To compile and run this:
// 1. Make sure you have Rust installed: https://www.rust-lang.org/tools/install
// 2. Create a new project: `cargo new rusty_samplers`
// 3. `cd rusty_samplers`
// 4. Add the `byteorder` crate to your `Cargo.toml` file:
//    [dependencies]
//    byteorder = "1.4"
// 5. Replace the contents of `src/main.rs` with this code.
// 6. Place an .akp file (e.g., "test.akp") in the root of the project directory.
// 7. Run the program: `cargo run -- test.akp`
// 8. A new "test.sfz" file should be created!

use byteorder::{LittleEndian, ReadBytesExt};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Cursor};
use std::path::Path;
use std::str;
use std::fmt;
use std::error::Error as StdError;
use indicatif::{ProgressBar, ProgressStyle};

// --- Error Types ---

#[derive(Debug)]
pub enum AkpError {
    Io(io::Error),
    InvalidRiffHeader,
    InvalidAprgSignature,
    UnknownChunkType(String),
    InvalidChunkSize(String, u32),
    CorruptedChunk(String, String),
    InvalidKeyRange(u8, u8),
    InvalidVelocityRange(u8, u8),
    MissingRequiredChunk(String),
    InvalidParameterValue(String, u8),
}

impl fmt::Display for AkpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AkpError::Io(err) => write!(f, "I/O error: {}", err),
            AkpError::InvalidRiffHeader => write!(f, "Invalid file format: Expected RIFF header but found different signature"),
            AkpError::InvalidAprgSignature => write!(f, "Invalid file format: Expected APRG signature but found different signature (not an Akai program file)"),
            AkpError::UnknownChunkType(chunk) => write!(f, "Unknown chunk type '{}' encountered", chunk),
            AkpError::InvalidChunkSize(chunk, size) => write!(f, "Invalid size {} for chunk '{}'", size, chunk),
            AkpError::CorruptedChunk(chunk, reason) => write!(f, "Corrupted '{}' chunk: {}", chunk, reason),
            AkpError::InvalidKeyRange(low, high) => write!(f, "Invalid key range: low_key ({}) must be <= high_key ({})", low, high),
            AkpError::InvalidVelocityRange(low, high) => write!(f, "Invalid velocity range: low_vel ({}) must be <= high_vel ({})", low, high),
            AkpError::MissingRequiredChunk(chunk) => write!(f, "Missing required '{}' chunk", chunk),
            AkpError::InvalidParameterValue(param, value) => write!(f, "Invalid value {} for parameter '{}'", value, param),
        }
    }
}

impl StdError for AkpError {}

impl From<io::Error> for AkpError {
    fn from(err: io::Error) -> Self {
        AkpError::Io(err)
    }
}

type Result<T> = std::result::Result<T, AkpError>;

// --- Data Structures ---

#[derive(Debug, Default)]
pub struct AkaiProgram {
    header: Option<ProgramHeader>,
    keygroups: Vec<Keygroup>,
}

#[derive(Debug, Default)]
struct ProgramHeader {
    midi_program_number: u8,
    number_of_keygroups: u8,
}

#[derive(Debug, Default)]
struct Keygroup {
    low_key: u8,
    high_key: u8,
    low_vel: u8,
    high_vel: u8,
    sample: Option<Sample>,
    tune: Option<Tune>,
    filter: Option<Filter>,
    amp_env: Option<Envelope>,
    filter_env: Option<Envelope>,
    aux_env: Option<Envelope>,
    lfo1: Option<Lfo>,
    lfo2: Option<Lfo>,
    mods: Vec<Modulation>,
}

#[derive(Debug, Default)]
struct Sample {
    filename: String,
}

#[derive(Debug, Default)]
struct Tune {
    level: u8,
    semitone: i8,
    fine_tune: i8,
}

#[derive(Debug, Default)]
struct Filter {
    cutoff: u8,
    resonance: u8,
    filter_type: u8,
}

#[derive(Debug, Default)]
struct Envelope {
    attack: u8,
    decay: u8,
    sustain: u8,
    release: u8,
}

#[derive(Debug, Default)]
struct Lfo {
    waveform: u8,
    rate: u8,
    delay: u8,
    depth: u8,
}

#[derive(Debug, Default)]
struct Modulation {
    source: u8,
    destination: u8,
    amount: u8,
}

#[derive(Debug)]
struct RiffChunkHeader {
    id: String,
    size: u32,
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

// --- Conversion Trait ---

trait SamplerFormat {
    fn convert(&self, program: &AkaiProgram) -> String;
    fn file_extension(&self) -> &str;
}

struct SfzConverter;
struct DecentSamplerConverter;

impl SamplerFormat for SfzConverter {
    fn convert(&self, program: &AkaiProgram) -> String {
        program.to_sfz_string()
    }
    
    fn file_extension(&self) -> &str {
        "sfz"
    }
}

impl SamplerFormat for DecentSamplerConverter {
    fn convert(&self, program: &AkaiProgram) -> String {
        program.to_dspreset_string()
    }
    
    fn file_extension(&self) -> &str {
        "dspreset"
    }
}

// --- SFZ Generation Logic ---

impl AkaiProgram {
    /// Converts the parsed AkaiProgram into an SFZ formatted string.
    pub fn to_sfz_string(&self) -> String {
        let mut sfz_content = String::new();
        sfz_content.push_str("// Generated by Rusty Samplers\n\n");

        for keygroup in &self.keygroups {
            sfz_content.push_str("<region>\n");

            // Sample
            if let Some(sample) = &keygroup.sample {
                // SFZ expects forward slashes, so we replace them.
                sfz_content.push_str(&format!("sample={}\n", sample.filename.replace("\\", "/")));
            }

            // Key/Velocity Range
            sfz_content.push_str(&format!("lokey={}\n", keygroup.low_key));
            sfz_content.push_str(&format!("hikey={}\n", keygroup.high_key));
            sfz_content.push_str(&format!("lovel={}\n", keygroup.low_vel));
            sfz_content.push_str(&format!("hivel={}\n", keygroup.high_vel));

            // Tune/Level - Enhanced
            if let Some(tune) = &keygroup.tune {
                // Improved volume mapping: 0-100 -> -60dB to +6dB (more usable range)
                let volume_db = (tune.level as f32 / 100.0) * 66.0 - 60.0;
                sfz_content.push_str(&format!("volume={volume_db:.2}\n"));
                sfz_content.push_str(&format!("tune={}\n", tune.semitone));
                sfz_content.push_str(&format!("fine_tune={}\n", tune.fine_tune));
                
                // Add amp_veltrack for velocity sensitivity (common in Akai samplers)
                sfz_content.push_str("amp_veltrack=100\n");
            }

            // Amp Envelope - Enhanced
            if let Some(env) = &keygroup.amp_env {
                // Improved exponential scaling for more musical envelope times
                let attack_time = if env.attack == 0 { 0.0 } else { (env.attack as f32 / 100.0 * 4.0).exp() * 0.001 };
                let decay_time = if env.decay == 0 { 0.0 } else { (env.decay as f32 / 100.0 * 4.0).exp() * 0.001 };
                let release_time = if env.release == 0 { 0.001 } else { (env.release as f32 / 100.0 * 5.0).exp() * 0.001 };
                
                sfz_content.push_str(&format!("ampeg_attack={attack_time:.3}\n"));
                sfz_content.push_str(&format!("ampeg_decay={decay_time:.3}\n"));
                sfz_content.push_str(&format!("ampeg_sustain={}\n", env.sustain));
                sfz_content.push_str(&format!("ampeg_release={release_time:.3}\n"));
                
                // Add velocity tracking for envelope times (common feature)
                if env.attack > 10 {
                    sfz_content.push_str("ampeg_vel2attack=-20\n"); // Faster attack at high velocity
                }
                if env.decay > 10 {
                    sfz_content.push_str("ampeg_vel2decay=-10\n"); // Faster decay at high velocity
                }
            }
            
            // Filter - Improved mapping
            if let Some(filter) = &keygroup.filter {
                if filter.filter_type > 0 {
                    match filter.filter_type {
                        1 => sfz_content.push_str("fil_type=lpf_2p\n"),
                        2 => sfz_content.push_str("fil_type=bpf_2p\n"),
                        3 => sfz_content.push_str("fil_type=hpf_2p\n"),
                        _ => sfz_content.push_str("fil_type=lpf_2p\n") // Default to lowpass
                    }
                    
                    // Improved cutoff mapping: 20Hz to 20kHz logarithmic
                    let cutoff_hz = 20.0 * (1000.0f32).powf(filter.cutoff as f32 / 100.0);
                    sfz_content.push_str(&format!("cutoff={cutoff_hz:.1}\n"));
                    
                    // Improved resonance mapping: 0 to 40dB
                    let resonance_db = (filter.resonance as f32 / 100.0) * 40.0;
                    sfz_content.push_str(&format!("resonance={resonance_db:.1}\n"));
                    
                    // Add filter envelope depth if we have filter envelope
                    if keygroup.filter_env.is_some() {
                        sfz_content.push_str("fileg_depth=2400\n"); // 2 octaves modulation
                    }
                }
            }
            
            // Filter Envelope
            if let Some(env) = &keygroup.filter_env {
                let attack_time = (env.attack as f32 / 100.0 * 5.0).exp() * 0.001;
                let decay_time = (env.decay as f32 / 100.0 * 5.0).exp() * 0.001;
                let release_time = (env.release as f32 / 100.0 * 6.0).exp() * 0.001;
                
                sfz_content.push_str(&format!("fileg_attack={attack_time:.3}\n"));
                sfz_content.push_str(&format!("fileg_decay={decay_time:.3}\n"));
                sfz_content.push_str(&format!("fileg_sustain={}\n", env.sustain));
                sfz_content.push_str(&format!("fileg_release={release_time:.3}\n"));
            }

            // LFOs - Enhanced with all parameters
            if let Some(lfo) = &keygroup.lfo1 {
                // Frequency: 0.1Hz to 30Hz logarithmic
                let lfo_freq_hz = 0.1 * (300.0f32).powf(lfo.rate as f32 / 100.0);
                sfz_content.push_str(&format!("lfo1_freq={lfo_freq_hz:.2}\n"));
                
                // Waveform mapping
                let waveform = match lfo.waveform {
                    0 => "triangle",
                    1 => "sine",
                    2 => "square", 
                    3 => "saw",
                    4 => "ramp",
                    5 => "random",
                    _ => "triangle" // Default
                };
                sfz_content.push_str(&format!("lfo1_wave={}\n", waveform));
                
                // Delay in seconds (0-10 seconds)
                if lfo.delay > 0 {
                    let delay_time = (lfo.delay as f32 / 100.0) * 10.0;
                    sfz_content.push_str(&format!("lfo1_delay={delay_time:.2}\n"));
                }
                
                // Fade-in time (same as delay for simplicity)
                if lfo.delay > 0 {
                    let fade_time = (lfo.delay as f32 / 100.0) * 5.0;
                    sfz_content.push_str(&format!("lfo1_fade={fade_time:.2}\n"));
                }
            }
            
            if let Some(lfo) = &keygroup.lfo2 {
                let lfo_freq_hz = 0.1 * (300.0f32).powf(lfo.rate as f32 / 100.0);
                sfz_content.push_str(&format!("lfo2_freq={lfo_freq_hz:.2}\n"));
                
                let waveform = match lfo.waveform {
                    0 => "triangle",
                    1 => "sine",
                    2 => "square",
                    3 => "saw", 
                    4 => "ramp",
                    5 => "random",
                    _ => "triangle"
                };
                sfz_content.push_str(&format!("lfo2_wave={}\n", waveform));
                
                if lfo.delay > 0 {
                    let delay_time = (lfo.delay as f32 / 100.0) * 10.0;
                    sfz_content.push_str(&format!("lfo2_delay={delay_time:.2}\n"));
                    
                    let fade_time = (lfo.delay as f32 / 100.0) * 5.0;
                    sfz_content.push_str(&format!("lfo2_fade={fade_time:.2}\n"));
                }
            }

            // Modulations - Expanded mapping
            for modulation in &keygroup.mods {
                let sfz_source = match modulation.source {
                    0 => "lfo1",
                    1 => "modwheel", // CC1
                    2 => "aftertouch",
                    3 => "key",
                    4 => "keygate", // Note on/off
                    5 => "vel",
                    6 => "lfo2",
                    7 => "pitchbend",
                    8 => "chanpress", // Channel pressure
                    9 => "polypress", // Polyphonic pressure  
                    10 => "breath", // CC2
                    11 => "foot", // CC4
                    12 => "expression", // CC11
                    _ => continue // Skip unknown modulation sources
                };

                let (sfz_destination, scale_factor) = match modulation.destination {
                    0 => ("pitch", 12.0), // Pitch in semitones
                    1 => ("cutoff", 9600.0), // Filter cutoff in cents
                    2 => ("resonance", 40.0), // Resonance in dB
                    3 => ("volume", 60.0), // Volume in dB
                    4 => ("pan", 100.0), // Pan -100 to 100
                    5 => ("lfo1_freq", 20.0), // LFO freq in Hz
                    6 => ("lfo2_freq", 20.0),
                    7 => ("ampeg_attack", 10.0), // Envelope times in seconds
                    8 => ("ampeg_decay", 10.0),
                    9 => ("ampeg_sustain", 100.0), // Sustain level
                    10 => ("ampeg_release", 10.0),
                    11 => ("fileg_attack", 10.0), // Filter envelope
                    12 => ("fileg_decay", 10.0),
                    13 => ("fileg_sustain", 100.0),
                    14 => ("fileg_release", 10.0),
                    15 => ("amplfo_depth", 100.0), // LFO depth
                    16 => ("fillfo_depth", 9600.0), // Filter LFO depth in cents
                    17 => ("pitchlfo_depth", 1200.0), // Pitch LFO depth in cents
                    _ => continue // Skip unknown modulation destinations
                };

                // Convert AKP amount (0-100) to bipolar range (-1 to 1) then scale
                let normalized_amount = (modulation.amount as f32 / 100.0) * 2.0 - 1.0;
                let scaled_amount = normalized_amount * scale_factor;

                sfz_content.push_str(&format!("{sfz_source}_to_{sfz_destination}={scaled_amount:.1}\n"));
            }

            // Additional common SFZ opcodes for better playback
            
            // Loop mode (if sample supports it)
            if keygroup.sample.is_some() {
                sfz_content.push_str("loop_mode=loop_continuous\n");
                sfz_content.push_str("loop_start=0\n");
                sfz_content.push_str("loop_end=0\n"); // Let SFZ player auto-detect
            }
            
            // Polyphony and voice management
            sfz_content.push_str("polyphony=64\n");
            sfz_content.push_str("note_polyphony=1\n"); // One voice per note (mono)
            
            // Pitch bend range (standard 2 semitones)
            sfz_content.push_str("bend_up=200\n"); // 200 cents = 2 semitones
            sfz_content.push_str("bend_down=-200\n");
            
            // Default amplitude envelope if none specified
            if keygroup.amp_env.is_none() {
                sfz_content.push_str("ampeg_attack=0.001\n");
                sfz_content.push_str("ampeg_decay=0.1\n");
                sfz_content.push_str("ampeg_sustain=100\n");
                sfz_content.push_str("ampeg_release=0.3\n");
            }

            sfz_content.push('\n');
        }

        sfz_content
    }

    /// Converts the parsed AkaiProgram into a Decent Sampler .dspreset XML string.
    pub fn to_dspreset_string(&self) -> String {
        let mut xml = String::new();
        
        // XML declaration
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<DecentSampler minVersion=\"1.0.0\">\n");
        
        // UI Section - Create knobs based on available parameters
        xml.push_str("  <ui>\n");
        xml.push_str("    <tab name=\"Main\">\n");
        xml.push_str("      <labeled-knob x=\"10\" y=\"20\" width=\"90\" height=\"100\" parameterName=\"ATTACK\" type=\"float\" minValue=\"0\" maxValue=\"5\" value=\"0.1\" textColor=\"AA000000\">\n");
        xml.push_str("        <label text=\"Attack\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"110\" y=\"20\" width=\"90\" height=\"100\" parameterName=\"DECAY\" type=\"float\" minValue=\"0\" maxValue=\"5\" value=\"0.5\" textColor=\"AA000000\">\n");
        xml.push_str("        <label text=\"Decay\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"210\" y=\"20\" width=\"90\" height=\"100\" parameterName=\"SUSTAIN\" type=\"float\" minValue=\"0\" maxValue=\"1\" value=\"0.7\" textColor=\"AA000000\">\n");
        xml.push_str("        <label text=\"Sustain\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"310\" y=\"20\" width=\"90\" height=\"100\" parameterName=\"RELEASE\" type=\"float\" minValue=\"0\" maxValue=\"10\" value=\"0.3\" textColor=\"AA000000\">\n");
        xml.push_str("        <label text=\"Release\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"410\" y=\"20\" width=\"90\" height=\"100\" parameterName=\"FILTER_CUTOFF\" type=\"float\" minValue=\"20\" maxValue=\"20000\" value=\"20000\" textColor=\"AA000000\">\n");
        xml.push_str("        <label text=\"Filter\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"510\" y=\"20\" width=\"90\" height=\"100\" parameterName=\"FILTER_RESONANCE\" type=\"float\" minValue=\"0\" maxValue=\"40\" value=\"0\" textColor=\"AA000000\">\n");
        xml.push_str("        <label text=\"Resonance\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("    </tab>\n");
        xml.push_str("  </ui>\n\n");
        
        // Groups section - contains all the samples
        xml.push_str("  <groups>\n");
        
        for (group_id, keygroup) in self.keygroups.iter().enumerate() {
            xml.push_str(&format!("    <group name=\"Group{}\"", group_id + 1));
            
            // Add global envelope parameters if available
            if let Some(env) = &keygroup.amp_env {
                let attack = if env.attack == 0 { 0.001 } else { (env.attack as f32 / 100.0 * 4.0).exp() * 0.001 };
                let decay = if env.decay == 0 { 0.1 } else { (env.decay as f32 / 100.0 * 4.0).exp() * 0.001 };
                let sustain = env.sustain as f32 / 100.0;
                let release = if env.release == 0 { 0.1 } else { (env.release as f32 / 100.0 * 5.0).exp() * 0.001 };
                
                xml.push_str(&format!(" attack=\"{:.3}\" decay=\"{:.3}\" sustain=\"{:.3}\" release=\"{:.3}\"",
                    attack, decay, sustain, release));
            }
            
            // Add volume if tune level is available
            if let Some(tune) = &keygroup.tune {
                let volume_db = (tune.level as f32 / 100.0) * 66.0 - 60.0;
                xml.push_str(&format!(" volume=\"{:.2}\"", volume_db));
            }
            
            xml.push_str(">\n");
            
            // Sample definition
            if let Some(sample) = &keygroup.sample {
                xml.push_str("      <sample ");
                xml.push_str(&format!("path=\"{}\" ", sample.filename));
                xml.push_str(&format!("loNote=\"{}\" hiNote=\"{}\" ", keygroup.low_key, keygroup.high_key));
                xml.push_str(&format!("loVel=\"{}\" hiVel=\"{}\" ", keygroup.low_vel, keygroup.high_vel));
                
                // Add tuning if available
                if let Some(tune) = &keygroup.tune {
                    if tune.semitone != 0 {
                        xml.push_str(&format!("tuning=\"{}\" ", tune.semitone));
                    }
                    if tune.fine_tune != 0 {
                        xml.push_str(&format!("fineTuning=\"{}\" ", tune.fine_tune));
                    }
                }
                
                xml.push_str("/>\n");
            }
            
            xml.push_str("    </group>\n");
        }
        
        xml.push_str("  </groups>\n\n");
        
        // Effects section
        xml.push_str("  <effects>\n");
        
        // Add filter if any keygroup has one
        let has_filter = self.keygroups.iter().any(|kg| kg.filter.is_some());
        if has_filter {
            xml.push_str("    <lowpass frequency=\"FILTER_CUTOFF\" resonance=\"FILTER_RESONANCE\" />\n");
        }
        
        // Add reverb by default
        xml.push_str("    <reverb roomSize=\"0.5\" damping=\"0.5\" wetLevel=\"0.3\" dryLevel=\"0.7\" width=\"1.0\" />\n");
        
        xml.push_str("  </effects>\n\n");
        
        // MIDI section - bind UI controls
        xml.push_str("  <midi>\n");
        xml.push_str("    <!-- MIDI CC bindings can be added here -->\n");
        xml.push_str("    <cc number=\"1\" parameter=\"FILTER_CUTOFF\" />\n");
        xml.push_str("    <cc number=\"2\" parameter=\"FILTER_RESONANCE\" />\n");
        xml.push_str("    <cc number=\"7\" parameter=\"MAIN_VOLUME\" />\n");
        xml.push_str("  </midi>\n\n");
        
        // Modulators section - map LFOs if available
        let has_lfo = self.keygroups.iter().any(|kg| kg.lfo1.is_some() || kg.lfo2.is_some());
        if has_lfo {
            xml.push_str("  <modulators>\n");
            
            for (group_id, keygroup) in self.keygroups.iter().enumerate() {
                if let Some(lfo) = &keygroup.lfo1 {
                    let lfo_freq = 0.1 * (300.0f32).powf(lfo.rate as f32 / 100.0);
                    let waveform = match lfo.waveform {
                        0 => "triangle",
                        1 => "sine",
                        2 => "square",
                        3 => "saw",
                        4 => "ramp", 
                        5 => "random",
                        _ => "sine"
                    };
                    
                    xml.push_str(&format!("    <lfo frequency=\"{:.2}\" waveform=\"{}\" target=\"FILTER_CUTOFF\" amount=\"0.3\" />\n",
                        lfo_freq, waveform));
                }
            }
            
            xml.push_str("  </modulators>\n\n");
        }
        
        // Tags for metadata
        xml.push_str("  <tags>\n");
        xml.push_str("    <tag name=\"author\" value=\"Rusty Samplers\" />\n");
        xml.push_str("    <tag name=\"description\" value=\"Converted from AKP format\" />\n");
        xml.push_str("    <tag name=\"conversion-tool\" value=\"Rusty Samplers v1.0\" />\n");
        xml.push_str("  </tags>\n\n");
        
        xml.push_str("</DecentSampler>\n");
        xml
    }
}

// --- Main Application Logic ---

fn main() -> Result<()> {
    println!("🎵 Rusty Samplers: Multi-Format Sampler Converter v1.0 🎵");
    println!();

    let args: Vec<String> = env::args().collect();
    
    // Handle different argument patterns
    match args.len() {
        1 => {
            println!("Usage:");
            println!("  Single file:  cargo run -- [OPTIONS] <path_to_akp_file>");
            println!("  Batch mode:   cargo run -- --batch [OPTIONS] <directory>");
            println!("  Help:         cargo run -- --help");
            println!();
            println!("Options:");
            println!("  --format sfz|ds     Output format (default: sfz)");
            return Ok(());
        }
        2 => {
            let arg = &args[1];
            match arg.as_str() {
                "--help" | "-h" => {
                    print_help();
                    return Ok(());
                }
                _ => {
                    // Single file conversion with default SFZ format
                    if let Err(e) = run_conversion(arg, OutputFormat::Sfz) {
                        eprintln!("❌ Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        3 => {
            let first_arg = &args[1];
            let second_arg = &args[2];
            
            match first_arg.as_str() {
                "--batch" | "-b" => {
                    if let Err(e) = run_batch_conversion(second_arg, OutputFormat::Sfz) {
                        eprintln!("❌ Batch Error: {}", e);
                        std::process::exit(1);
                    }
                }
                "--format" => {
                    eprintln!("❌ Missing file path after format option.");
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("❌ Invalid arguments. Use --help for usage information.");
                    std::process::exit(1);
                }
            }
        }
        4 => {
            let first_arg = &args[1];
            let second_arg = &args[2];
            let third_arg = &args[3];
            
            // Handle --format option
            if first_arg == "--format" {
                let format = parse_format(second_arg)?;
                if let Err(e) = run_conversion(third_arg, format) {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("❌ Invalid arguments. Use --help for usage information.");
                std::process::exit(1);
            }
        }
        5 => {
            let first_arg = &args[1];
            let second_arg = &args[2];
            let third_arg = &args[3];
            let fourth_arg = &args[4];
            
            // Handle --batch with --format
            if first_arg == "--batch" && second_arg == "--format" {
                let format = parse_format(third_arg)?;
                if let Err(e) = run_batch_conversion(fourth_arg, format) {
                    eprintln!("❌ Batch Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("❌ Invalid arguments. Use --help for usage information.");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("❌ Too many arguments. Use --help for usage information.");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn parse_format(format_str: &str) -> Result<OutputFormat> {
    match format_str.to_lowercase().as_str() {
        "sfz" => Ok(OutputFormat::Sfz),
        "ds" | "dspreset" | "decent" | "decentsampler" => Ok(OutputFormat::DecentSampler),
        _ => Err(AkpError::InvalidParameterValue("output_format".to_string(), 0))
    }
}

fn print_help() {
    println!("🎵 Rusty Samplers - Multi-Format Sampler Converter");
    println!();
    println!("USAGE:");
    println!("    cargo run -- [OPTIONS] <INPUT>");
    println!();
    println!("OPTIONS:");
    println!("    --format <FORMAT>    Output format: sfz, ds (default: sfz)");
    println!("    --batch, -b <DIR>    Convert all .akp files in directory");
    println!("    --help, -h           Show this help message");
    println!();
    println!("OUTPUT FORMATS:");
    println!("    sfz                  SFZ format (default)");
    println!("    ds, dspreset         Decent Sampler XML format");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run -- my_sample.akp");
    println!("    cargo run -- --format ds my_sample.akp");
    println!("    cargo run -- --batch ./samples/");
    println!("    cargo run -- --batch --format ds ./samples/");
    println!();
    println!("FEATURES:");
    println!("    • Comprehensive AKP chunk parsing");
    println!("    • Advanced SFZ and Decent Sampler parameter mapping");
    println!("    • Envelope, filter, and LFO conversion");
    println!("    • Modulation routing support");
    println!("    • Progress indicators and error handling");
    println!("    • Multi-format output support");
}

fn run_batch_conversion(directory: &str, format: OutputFormat) -> Result<()> {
    use std::fs;
    
    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' not found", directory)
        )));
    }
    
    if !dir_path.is_dir() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is not a directory", directory)
        )));
    }
    
    // Find all .akp files
    let mut akp_files = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("akp") {
            akp_files.push(path);
        }
    }
    
    if akp_files.is_empty() {
        println!("⚠️  No .akp files found in directory: {}", directory);
        return Ok(());
    }
    
    println!("🚀 Starting batch conversion of {} files...", akp_files.len());
    println!();
    
    let batch_progress = ProgressBar::new(akp_files.len() as u64);
    batch_progress.set_style(
        ProgressStyle::with_template(
            "🔄 [{bar:40.cyan/blue}] {pos:>3}/{len:3} files ({percent}%) {msg}"
        ).unwrap().progress_chars("█▉▊▋▌▍▎▏  ")
    );
    
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();
    
    for (_i, akp_file) in akp_files.iter().enumerate() {
        let file_name = akp_file.file_name().unwrap().to_string_lossy();
        batch_progress.set_message(format!("Processing {}", file_name));
        
        match run_conversion(&akp_file.to_string_lossy(), format) {
            Ok(()) => {
                success_count += 1;
                batch_progress.println(format!("✅ {}", file_name));
            }
            Err(e) => {
                error_count += 1;
                let error_msg = format!("{}: {}", file_name, e);
                errors.push(error_msg.clone());
                batch_progress.println(format!("❌ {}", error_msg));
            }
        }
        
        batch_progress.inc(1);
    }
    
    batch_progress.finish_with_message("Batch conversion complete!");
    
    println!();
    println!("📊 BATCH SUMMARY:");
    println!("   ✅ Successful: {}", success_count);
    println!("   ❌ Failed:     {}", error_count);
    println!("   📁 Total:      {}", akp_files.len());
    
    if !errors.is_empty() {
        println!();
        println!("⚠️  ERRORS:");
        for error in &errors {
            println!("   • {}", error);
        }
    }
    
    Ok(())
}

fn run_conversion(file_path_str: &str, format: OutputFormat) -> Result<()> {
    if !Path::new(file_path_str).exists() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not found", file_path_str)
        )));
    }
    
    // Create progress bar
    let progress = ProgressBar::new(100);
    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}"
        ).unwrap().progress_chars("█▉▊▋▌▍▎▏  ")
    );
    
    progress.set_message("Opening file...");
    progress.inc(10);
    
    let mut file = File::open(file_path_str)?;
    
    progress.set_message("Validating RIFF header...");
    progress.inc(10);
    validate_riff_header(&mut file)?;

    progress.set_message("Parsing chunks...");
    progress.inc(20);
    let mut program = AkaiProgram::default();
    let file_len = file.metadata()?.len();
    parse_top_level_chunks(&mut file, file_len, &mut program, &progress)?;

    progress.set_message("Validating structure...");
    progress.inc(10);
    
    // Check if we have any keygroups
    if program.keygroups.is_empty() {
        return Err(AkpError::MissingRequiredChunk("keygroup".to_string()));
    }
    
    // --- Generate and Save Output ---
    let format_name = match format {
        OutputFormat::Sfz => "SFZ",
        OutputFormat::DecentSampler => "Decent Sampler",
    };
    
    println!("-> Generating {} content...", format_name);
    let (output_content, file_extension) = match format {
        OutputFormat::Sfz => (program.to_sfz_string(), "sfz"),
        OutputFormat::DecentSampler => (program.to_dspreset_string(), "dspreset"),
    };
    
    let input_path = Path::new(file_path_str);
    let output_path = input_path.with_extension(file_extension);
    
    println!("-> Saving {} file to: {:?}", format_name, output_path);
    fs::write(&output_path, output_content)?;
    
    println!("\n--- Conversion Complete ---");
    println!("Successfully created {:?}.", output_path);

    Ok(())
}

// --- Parsing Functions (largely unchanged from v0.6) ---

pub fn validate_riff_header(file: &mut File) -> Result<()> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)
        .map_err(|_| AkpError::CorruptedChunk("RIFF".to_string(), "Failed to read RIFF signature".to_string()))?;
    
    if str::from_utf8(&buf).unwrap_or("") != "RIFF" {
        return Err(AkpError::InvalidRiffHeader);
    }
    
    // Skip file size
    file.seek(SeekFrom::Current(4))?;
    
    file.read_exact(&mut buf)
        .map_err(|_| AkpError::CorruptedChunk("APRG".to_string(), "Failed to read APRG signature".to_string()))?;
    
    if str::from_utf8(&buf).unwrap_or("") != "APRG" {
        return Err(AkpError::InvalidAprgSignature);
    }
    
    Ok(())
}

pub fn parse_top_level_chunks(file: &mut File, end_pos: u64, program: &mut AkaiProgram, progress: &ProgressBar) -> Result<()> {
    let mut processed = 0u64;
    
    while file.stream_position()? < end_pos {
        let current_pos = file.stream_position()?;
        let progress_percent = ((current_pos * 30) / end_pos) as u64; // 30% of total progress
        if processed != progress_percent {
            progress.set_position(20 + progress_percent); // Start at 20% + parsing progress
            processed = progress_percent;
        }
        
        let header = read_chunk_header(file)?;
        match header.id.as_str() {
            "prg " => {
                if header.size < 3 {
                    return Err(AkpError::InvalidChunkSize("prg".to_string(), header.size));
                }
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)
                    .map_err(|_| AkpError::CorruptedChunk("prg".to_string(), "Failed to read chunk data".to_string()))?;
                program.header = Some(parse_program_header(&mut Cursor::new(chunk_data))?);
            }
            "kgrp" => {
                if header.size == 0 {
                    return Err(AkpError::InvalidChunkSize("kgrp".to_string(), header.size));
                }
                let msg = format!("Parsing keygroup {}...", program.keygroups.len() + 1);
                progress.set_message("Parsing keygroup...");
                let kgrp_end_pos = file.stream_position()? + header.size as u64;
                let keygroup = parse_keygroup(file, kgrp_end_pos)?;
                program.keygroups.push(keygroup);
            }
            _ => {
                progress.println(format!("⚠️  Warning: Skipping unknown chunk type '{}'", header.id));
                file.seek(SeekFrom::Current(header.size as i64))?;
            }
        }
    }
    Ok(())
}

fn parse_keygroup(file: &mut File, end_pos: u64) -> Result<Keygroup> {
    let mut keygroup = Keygroup::default();
    let mut env_count = 0;
    let mut lfo_count = 0;

    while file.stream_position()? < end_pos {
        let header = read_chunk_header(file)?;
        let mut chunk_data = vec![0; header.size as usize];
        file.read_exact(&mut chunk_data)?;
        let mut cursor = Cursor::new(chunk_data);

        match header.id.as_str() {
            "zone" => {
                if header.size < 5 {
                    return Err(AkpError::InvalidChunkSize("zone".to_string(), header.size));
                }
                parse_zone_chunk(&mut cursor, &mut keygroup)?
            },
            "smpl" => {
                if header.size < 3 {
                    return Err(AkpError::InvalidChunkSize("smpl".to_string(), header.size));
                }
                keygroup.sample = Some(parse_smpl_chunk(&mut cursor)?)
            },
            "tune" => {
                if header.size < 5 {
                    return Err(AkpError::InvalidChunkSize("tune".to_string(), header.size));
                }
                keygroup.tune = Some(parse_tune_chunk(&mut cursor)?)
            },
            "filt" => {
                if header.size < 8 {
                    return Err(AkpError::InvalidChunkSize("filt".to_string(), header.size));
                }
                keygroup.filter = Some(parse_filt_chunk(&mut cursor)?)
            },
            "env " => {
                if header.size < 6 {
                    return Err(AkpError::InvalidChunkSize("env".to_string(), header.size));
                }
                let envelope = parse_env_chunk(&mut cursor)?;
                match env_count {
                    0 => keygroup.amp_env = Some(envelope),
                    1 => keygroup.filter_env = Some(envelope),
                    2 => keygroup.aux_env = Some(envelope),
                    _ => {} // Silently ignore extra envelopes in batch mode
                }
                env_count += 1;
            }
            "lfo " => {
                if header.size < 9 {
                    return Err(AkpError::InvalidChunkSize("lfo".to_string(), header.size));
                }
                let lfo = parse_lfo_chunk(&mut cursor)?;
                match lfo_count {
                    0 => keygroup.lfo1 = Some(lfo),
                    1 => keygroup.lfo2 = Some(lfo),
                    _ => {} // Silently ignore extra LFOs in batch mode
                }
                lfo_count += 1;
            }
            "mods" => {
                if header.size < 4 {
                    return Err(AkpError::InvalidChunkSize("mods".to_string(), header.size));
                }
                keygroup.mods.push(parse_mods_chunk(&mut cursor)?);
            }
            _ => {
                println!("Warning: Skipping unknown keygroup chunk type '{}'", header.id);
            }
        }
    }
    Ok(keygroup)
}

fn read_chunk_header(file: &mut File) -> Result<RiffChunkHeader> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    let id = str::from_utf8(&buf).unwrap_or("????").trim_end_matches('\0').to_string();
    let size = file.read_u32::<LittleEndian>()?;
    Ok(RiffChunkHeader { id, size })
}

fn parse_program_header(cursor: &mut Cursor<Vec<u8>>) -> Result<ProgramHeader> {
    cursor.seek(SeekFrom::Start(1))?;
    let midi_program_number = cursor.read_u8()?;
    let number_of_keygroups = cursor.read_u8()?;
    Ok(ProgramHeader { midi_program_number, number_of_keygroups })
}

fn parse_zone_chunk(cursor: &mut Cursor<Vec<u8>>, keygroup: &mut Keygroup) -> Result<()> {
    cursor.seek(SeekFrom::Start(1))?;
    keygroup.low_key = cursor.read_u8()?;
    keygroup.high_key = cursor.read_u8()?;
    keygroup.low_vel = cursor.read_u8()?;
    keygroup.high_vel = cursor.read_u8()?;
    
    // Validate key range
    if keygroup.low_key > keygroup.high_key {
        return Err(AkpError::InvalidKeyRange(keygroup.low_key, keygroup.high_key));
    }
    
    // Validate velocity range
    if keygroup.low_vel > keygroup.high_vel {
        return Err(AkpError::InvalidVelocityRange(keygroup.low_vel, keygroup.high_vel));
    }
    
    // Validate MIDI ranges
    if keygroup.high_key > 127 {
        return Err(AkpError::InvalidParameterValue("high_key".to_string(), keygroup.high_key));
    }
    if keygroup.high_vel > 127 {
        return Err(AkpError::InvalidParameterValue("high_vel".to_string(), keygroup.high_vel));
    }
    
    Ok(())
}

fn parse_smpl_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Sample> {
    cursor.seek(SeekFrom::Start(2))?;
    let mut buffer = Vec::new();
    cursor.read_to_end(&mut buffer)?;
    let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    let filename = String::from_utf8_lossy(&buffer[..end]).to_string();
    
    if filename.is_empty() {
        return Err(AkpError::CorruptedChunk("smpl".to_string(), "Empty sample filename".to_string()));
    }
    
    Ok(Sample { filename })
}

fn parse_tune_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Tune> {
    cursor.seek(SeekFrom::Start(2))?;
    let level = cursor.read_u8()?;
    let semitone = cursor.read_i8()?;
    let fine_tune = cursor.read_i8()?;
    Ok(Tune { level, semitone, fine_tune })
}

fn parse_filt_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Filter> {
    cursor.seek(SeekFrom::Start(2))?;
    let cutoff = cursor.read_u8()?;
    let resonance = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(7))?;
    let filter_type = cursor.read_u8()?;
    
    // Validate filter type (assuming 0-3 are valid)
    if filter_type > 3 {
        return Err(AkpError::InvalidParameterValue("filter_type".to_string(), filter_type));
    }
    
    Ok(Filter { cutoff, resonance, filter_type })
}

fn parse_env_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Envelope> {
    cursor.seek(SeekFrom::Start(2))?;
    let attack = cursor.read_u8()?;
    let decay = cursor.read_u8()?;
    let sustain = cursor.read_u8()?;
    let release = cursor.read_u8()?;
    Ok(Envelope { attack, decay, sustain, release })
}

fn parse_lfo_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Lfo> {
    cursor.seek(SeekFrom::Start(5))?;
    let waveform = cursor.read_u8()?;
    let rate = cursor.read_u8()?;
    let delay = cursor.read_u8()?;
    let depth = cursor.read_u8()?;
    Ok(Lfo { waveform, rate, delay, depth })
}

fn parse_mods_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Modulation> {
    cursor.seek(SeekFrom::Start(1))?;
    let source = cursor.read_u8()?;
    let destination = cursor.read_u8()?;
    let amount = cursor.read_u8()?;
    Ok(Modulation { source, destination, amount })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_zone_chunk_valid() {
        let data = vec![0, 60, 72, 64, 127]; // low_key=60, high_key=72, low_vel=64, high_vel=127
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();
        
        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(result.is_ok());
        assert_eq!(keygroup.low_key, 60);
        assert_eq!(keygroup.high_key, 72);
        assert_eq!(keygroup.low_vel, 64);
        assert_eq!(keygroup.high_vel, 127);
    }

    #[test]
    fn test_parse_zone_chunk_invalid_key_range() {
        let data = vec![0, 72, 60, 64, 127]; // low_key > high_key
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();
        
        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidKeyRange(72, 60))));
    }

    #[test]
    fn test_parse_zone_chunk_invalid_velocity_range() {
        let data = vec![0, 60, 72, 127, 64]; // low_vel > high_vel
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();
        
        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidVelocityRange(127, 64))));
    }

    #[test]
    fn test_parse_zone_chunk_invalid_key_value() {
        let data = vec![0, 60, 128, 64, 127]; // high_key > 127
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();
        
        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidParameterValue(_, 128))));
    }

    #[test]
    fn test_parse_smpl_chunk_valid() {
        let mut data = vec![0, 0]; // offset bytes
        data.extend_from_slice(b"test_sample.wav");
        data.push(0); // null terminator
        let mut cursor = Cursor::new(data);
        
        let result = parse_smpl_chunk(&mut cursor);
        assert!(result.is_ok());
        let sample = result.unwrap();
        assert_eq!(sample.filename, "test_sample.wav");
    }

    #[test]
    fn test_parse_smpl_chunk_empty_filename() {
        let data = vec![0, 0, 0]; // offset bytes + null terminator only
        let mut cursor = Cursor::new(data);
        
        let result = parse_smpl_chunk(&mut cursor);
        assert!(matches!(result, Err(AkpError::CorruptedChunk(_, _))));
    }

    #[test]
    fn test_parse_tune_chunk() {
        let data = vec![0, 0, 85, -12i8 as u8, 25]; // level=85, semitone=-12, fine_tune=25
        let mut cursor = Cursor::new(data);
        
        let result = parse_tune_chunk(&mut cursor);
        assert!(result.is_ok());
        let tune = result.unwrap();
        assert_eq!(tune.level, 85);
        assert_eq!(tune.semitone, -12);
        assert_eq!(tune.fine_tune, 25);
    }

    #[test]
    fn test_parse_filt_chunk_valid() {
        let data = vec![0, 0, 75, 25, 0, 0, 0, 2]; // cutoff=75, resonance=25, type=2
        let mut cursor = Cursor::new(data);
        
        let result = parse_filt_chunk(&mut cursor);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.cutoff, 75);
        assert_eq!(filter.resonance, 25);
        assert_eq!(filter.filter_type, 2);
    }

    #[test]
    fn test_parse_filt_chunk_invalid_type() {
        let data = vec![0, 0, 75, 25, 0, 0, 0, 5]; // invalid filter_type=5
        let mut cursor = Cursor::new(data);
        
        let result = parse_filt_chunk(&mut cursor);
        assert!(matches!(result, Err(AkpError::InvalidParameterValue(_, 5))));
    }

    #[test]
    fn test_parse_env_chunk() {
        let data = vec![0, 0, 10, 50, 80, 30]; // attack=10, decay=50, sustain=80, release=30
        let mut cursor = Cursor::new(data);
        
        let result = parse_env_chunk(&mut cursor);
        assert!(result.is_ok());
        let env = result.unwrap();
        assert_eq!(env.attack, 10);
        assert_eq!(env.decay, 50);
        assert_eq!(env.sustain, 80);
        assert_eq!(env.release, 30);
    }

    #[test]
    fn test_parse_mods_chunk() {
        let data = vec![0, 1, 5, 75]; // source=1, destination=5, amount=75
        let mut cursor = Cursor::new(data);
        
        let result = parse_mods_chunk(&mut cursor);
        assert!(result.is_ok());
        let modulation = result.unwrap();
        assert_eq!(modulation.source, 1);
        assert_eq!(modulation.destination, 5);
        assert_eq!(modulation.amount, 75);
    }

    #[test]
    fn test_akp_error_display() {
        let error = AkpError::InvalidKeyRange(72, 60);
        assert_eq!(
            error.to_string(),
            "Invalid key range: low_key (72) must be <= high_key (60)"
        );
        
        let error = AkpError::InvalidRiffHeader;
        assert_eq!(
            error.to_string(),
            "Invalid file format: Expected RIFF header but found different signature"
        );
    }

    #[test]
    fn test_sfz_generation_basic() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        
        keygroup.low_key = 60;
        keygroup.high_key = 72;
        keygroup.low_vel = 1;
        keygroup.high_vel = 127;
        keygroup.sample = Some(Sample { filename: "test.wav".to_string() });
        
        program.keygroups.push(keygroup);
        
        let sfz = program.to_sfz_string();
        assert!(sfz.contains("// Generated by Rusty Samplers"));
        assert!(sfz.contains("<region>"));
        assert!(sfz.contains("sample=test.wav"));
        assert!(sfz.contains("lokey=60"));
        assert!(sfz.contains("hikey=72"));
        assert!(sfz.contains("lovel=1"));
        assert!(sfz.contains("hivel=127"));
    }

    #[test]
    fn test_sfz_generation_with_tune() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        
        keygroup.tune = Some(Tune { 
            level: 75, 
            semitone: -12, 
            fine_tune: 25 
        });
        
        program.keygroups.push(keygroup);
        
        let sfz = program.to_sfz_string();
        assert!(sfz.contains("volume=-12.00")); // 75/100 * 48 - 48 = -12
        assert!(sfz.contains("tune=-12"));
        assert!(sfz.contains("fine_tune=25"));
    }

    #[test]
    fn test_sfz_generation_with_filter() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        
        keygroup.filter = Some(Filter {
            cutoff: 50,
            resonance: 25,
            filter_type: 1,
        });
        
        program.keygroups.push(keygroup);
        
        let sfz = program.to_sfz_string();
        assert!(sfz.contains("fil_type=lpf_2p"));
        assert!(sfz.contains("cutoff=632.5")); // 20 * 1000^(50/100) ≈ 632.5
        assert!(sfz.contains("resonance=6.0")); // 25 * 0.24 = 6.0
    }

    #[test]
    fn test_sfz_generation_with_envelope() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        
        keygroup.amp_env = Some(Envelope {
            attack: 20,
            decay: 40, 
            sustain: 80,
            release: 60,
        });
        
        program.keygroups.push(keygroup);
        
        let sfz = program.to_sfz_string();
        assert!(sfz.contains("ampeg_attack="));
        assert!(sfz.contains("ampeg_decay="));
        assert!(sfz.contains("ampeg_sustain=80"));
        assert!(sfz.contains("ampeg_release="));
    }

    #[test]
    fn test_sfz_generation_with_lfo() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        
        keygroup.lfo1 = Some(Lfo {
            waveform: 0,
            rate: 30,
            delay: 0,
            depth: 50,
        });
        
        program.keygroups.push(keygroup);
        
        let sfz = program.to_sfz_string();
        assert!(sfz.contains("lfo1_freq="));
    }

    #[test]
    fn test_sfz_generation_multiple_keygroups() {
        let mut program = AkaiProgram::default();
        
        // First keygroup
        let mut kg1 = Keygroup::default();
        kg1.low_key = 36;
        kg1.high_key = 60;
        kg1.sample = Some(Sample { filename: "low.wav".to_string() });
        
        // Second keygroup
        let mut kg2 = Keygroup::default();
        kg2.low_key = 61;
        kg2.high_key = 96;
        kg2.sample = Some(Sample { filename: "high.wav".to_string() });
        
        program.keygroups.push(kg1);
        program.keygroups.push(kg2);
        
        let sfz = program.to_sfz_string();
        
        // Should have two <region> sections
        assert_eq!(sfz.matches("<region>").count(), 2);
        assert!(sfz.contains("sample=low.wav"));
        assert!(sfz.contains("sample=high.wav"));
        assert!(sfz.contains("lokey=36"));
        assert!(sfz.contains("hikey=60"));
        assert!(sfz.contains("lokey=61"));
        assert!(sfz.contains("hikey=96"));
    }
}