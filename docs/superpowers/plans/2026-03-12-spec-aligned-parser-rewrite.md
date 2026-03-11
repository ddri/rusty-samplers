# Spec-Aligned AKP Parser Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite all AKP parser data structures, parsing functions, and output generators to match the burnit.co.uk byte-level specification exactly, verified against real S6000 factory files.

**Architecture:** Replace all types in `src/types.rs` with spec-aligned structs. Rewrite every parser function in `src/parser.rs` with correct byte offsets. Update both output generators (`src/sfz.rs`, `src/dspreset.rs`) to use new types and corrected value mappings. TDD: write failing tests first, then implement.

**Tech Stack:** Rust, byteorder crate (little-endian binary parsing), cargo test

**Spec:** `docs/superpowers/specs/2026-03-12-spec-aligned-parser-rewrite-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `src/types.rs` | Rewrite | All data structs: AkaiProgram, ProgramOutput, ProgramTuning, ProgramModulation, Keygroup, Zone, Envelope, FilterEnvelope, AuxEnvelope, Filter, Lfo. Conversion helpers. |
| `src/parser.rs` | Rewrite | All parse functions with spec-correct byte offsets. Top-level and keygroup routing. |
| `src/sfz.rs` | Modify | Update for new structs, fix filter/resonance/LFO/envelope mappings, zone iteration. |
| `src/dspreset.rs` | Modify | Same fixes as SFZ, DS-specific XML mappings. |
| `src/lib.rs` | Minor | Update re-exports if needed. |
| `src/error.rs` | No change | Existing error variants sufficient. |
| `src/bin/cli.rs` | No change | Uses `convert_file()`, unaffected. |
| `gui/src/main.rs` | No change | Uses `convert_file()`, unaffected. |
| `tests/integration_tests.rs` | No change | Tests error paths, unaffected. |
| `create_test_akp.py` | Rewrite | Generate test files with all spec-correct chunk types. |

---

## Chunk 1: Data Structures

### Task 1: Rewrite types.rs — New structs

**Files:**
- Rewrite: `src/types.rs`

This task replaces ALL structs and helpers in types.rs with spec-aligned versions. Since everything depends on these types, we write them all at once then fix compilation in subsequent tasks.

- [ ] **Step 1: Write the new types.rs**

Replace the entire contents of `src/types.rs` with:

```rust
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

impl Default for Lfo {
    fn default() -> Self {
        Self {
            waveform: 0,
            rate: 0,
            delay: 0,
            depth: 0,
            sync: 0,
            retrigger: 0,
            modwheel: 0,
            aftertouch: 0,
            rate_mod: 0,
            delay_mod: 0,
            depth_mod: 0,
        }
    }
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
    pub fn depth_normalized(&self) -> f32 {
        self.depth as f32 / 100.0
    }
}

impl Envelope {
    /// Convert AKP attack (0-100) to seconds (exponential curve).
    pub fn attack_time(&self) -> f32 {
        if self.attack == 0 { 0.0 } else { (self.attack as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    /// Convert AKP decay (0-100) to seconds (exponential curve).
    pub fn decay_time(&self) -> f32 {
        if self.decay == 0 { 0.0 } else { (self.decay as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    /// Convert AKP release (0-100) to seconds. Minimum 0.001s to avoid clicks.
    pub fn release_time(&self) -> f32 {
        if self.release == 0 { 0.001 } else { (self.release as f32 / 100.0 * 5.0).exp() * 0.001 }
    }

    /// Convert AKP sustain (0-100) to normalized 0.0-1.0.
    pub fn sustain_normalized(&self) -> f32 {
        self.sustain as f32 / 100.0
    }
}

impl FilterEnvelope {
    pub fn attack_time(&self) -> f32 {
        if self.attack == 0 { 0.0 } else { (self.attack as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    pub fn decay_time(&self) -> f32 {
        if self.decay == 0 { 0.0 } else { (self.decay as f32 / 100.0 * 4.0).exp() * 0.001 }
    }

    pub fn release_time(&self) -> f32 {
        if self.release == 0 { 0.001 } else { (self.release as f32 / 100.0 * 5.0).exp() * 0.001 }
    }

    pub fn sustain_normalized(&self) -> f32 {
        self.sustain as f32 / 100.0
    }
}

impl ProgramOutput {
    /// Convert AKP loudness (0-100) to dB. 0=-60dB, 85=0dB approx, 100=+6dB.
    pub fn volume_db(&self) -> f32 {
        (self.loudness as f32 / 100.0) * 66.0 - 60.0
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
            0 | 1 | 2 => "lpf_2p",           // 2-pole LP, 4-pole LP, 2-pole LP+
            3 | 4 | 5 => "bpf_2p",           // 2-pole BP, 4-pole BP, 2-pole BP+
            6 | 8 => "hpf_1p",               // 1-pole HP, 1-pole HP+
            7 => "hpf_2p",                    // 2-pole HP
            12 | 13 | 14 | 15 | 16 => "brf_2p", // Notch variants
            17 | 18 | 19 | 20 | 21 => "pkf_2p", // Peak variants
            _ => "lpf_2p",                    // Morphing, phaser, voweliser -> fallback
        }
    }
}
```

- [ ] **Step 2: Verify types.rs compiles in isolation**

Run: `cd /Users/davidryan/GitHub/rusty-samplers && cargo check --lib 2>&1 | head -50`

Expected: Compilation errors in parser.rs, sfz.rs, dspreset.rs referencing old types (Sample, Tune, Modulation, old Keygroup fields). This is expected — we fix those in subsequent tasks.

- [ ] **Step 3: Commit types.rs**

```bash
git add src/types.rs
git commit -m "types: rewrite all structs to match AKP spec"
```

---

## Chunk 2: Parser Rewrite

### Task 2: Rewrite parser.rs — All parse functions

**Files:**
- Rewrite: `src/parser.rs`

This rewrites every parse function with spec-correct byte offsets, adds new chunk parsers (out, tune, lfo1, lfo2, mods, kloc expanded, amp_env, filter_env, aux_env, zone expanded), and rewrites all unit tests.

- [ ] **Step 1: Write the new parser.rs**

Replace the entire contents of `src/parser.rs`. Key changes from current code:

**Top-level routing** — add `"out "`, `"tune"`, `"lfo "` (count-based → lfo1/lfo2), `"mods"` handlers.

**Keygroup routing** — remove `"tune"`, `"smpl"` (keep as silent skip), `"lfo "`, `"mods"`. Add count-based env dispatch (amp/filter/aux). Zone returns `Option<Zone>` pushed to `keygroup.zones`.

**All parse functions with correct offsets:**

```rust
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::str;
use indicatif::ProgressBar;

use crate::error::{AkpError, Result};
use crate::types::*;

const MAX_CHUNK_SIZE: u32 = 64 * 1024 * 1024;
const MAX_KEYGROUPS: usize = 1000;
const MAX_ZONES_PER_KEYGROUP: usize = 4;

pub fn validate_riff_header(file: &mut File) -> Result<()> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)
        .map_err(|_| AkpError::CorruptedChunk("RIFF".to_string(), "Failed to read RIFF signature".to_string()))?;

    if str::from_utf8(&buf).unwrap_or("") != "RIFF" {
        return Err(AkpError::InvalidRiffHeader);
    }

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
    let mut lfo_count = 0u8;

    while file.stream_position()? < end_pos {
        let current_pos = file.stream_position()?;
        if end_pos > 0 {
            let progress_percent = (current_pos * 30) / end_pos;
            if processed != progress_percent {
                progress.set_position(20 + progress_percent);
                processed = progress_percent;
            }
        }

        let header = read_chunk_header(file)?;

        if header.size > MAX_CHUNK_SIZE {
            return Err(AkpError::InvalidChunkSize(header.id, header.size));
        }

        let chunk_start = file.stream_position()?;
        if chunk_start + header.size as u64 > end_pos {
            return Err(AkpError::CorruptedChunk(
                header.id,
                "Chunk extends beyond container boundary".to_string(),
            ));
        }

        match header.id.as_str() {
            "prg " => {
                if header.size < 3 {
                    return Err(AkpError::InvalidChunkSize("prg".to_string(), header.size));
                }
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)?;
                program.header = Some(parse_program_header(&mut Cursor::new(chunk_data))?);
            }
            "out " => {
                if header.size < 8 {
                    return Err(AkpError::InvalidChunkSize("out".to_string(), header.size));
                }
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)?;
                program.output = Some(parse_out_chunk(&mut Cursor::new(chunk_data))?);
            }
            "tune" => {
                if header.size < 19 {
                    return Err(AkpError::InvalidChunkSize("tune".to_string(), header.size));
                }
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)?;
                program.tuning = Some(parse_tune_chunk(&mut Cursor::new(chunk_data))?);
            }
            "lfo " => {
                if header.size < 12 {
                    return Err(AkpError::InvalidChunkSize("lfo".to_string(), header.size));
                }
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)?;
                match lfo_count {
                    0 => program.lfo1 = Some(parse_lfo1_chunk(&mut Cursor::new(chunk_data))?),
                    1 => program.lfo2 = Some(parse_lfo2_chunk(&mut Cursor::new(chunk_data))?),
                    _ => {} // ignore extra LFOs
                }
                lfo_count += 1;
            }
            "mods" => {
                if header.size < 38 {
                    return Err(AkpError::InvalidChunkSize("mods".to_string(), header.size));
                }
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)?;
                program.modulation = Some(parse_mods_chunk(&mut Cursor::new(chunk_data))?);
            }
            "kgrp" => {
                if header.size == 0 {
                    return Err(AkpError::InvalidChunkSize("kgrp".to_string(), header.size));
                }
                if program.keygroups.len() >= MAX_KEYGROUPS {
                    return Err(AkpError::CorruptedChunk(
                        "kgrp".to_string(),
                        format!("Exceeded maximum of {MAX_KEYGROUPS} keygroups"),
                    ));
                }
                progress.set_message("Parsing keygroup...");
                let kgrp_end_pos = chunk_start + header.size as u64;
                let keygroup = parse_keygroup(file, kgrp_end_pos, progress)?;
                program.keygroups.push(keygroup);
            }
            _ => {
                progress.println(format!("Warning: Skipping unknown chunk type '{}'", header.id));
                file.seek(SeekFrom::Current(header.size as i64))?;
            }
        }
    }
    Ok(())
}

fn parse_keygroup(file: &mut File, end_pos: u64, progress: &ProgressBar) -> Result<Keygroup> {
    let mut keygroup = Keygroup::default();
    let mut env_count = 0u8;

    while file.stream_position()? < end_pos {
        let header = read_chunk_header(file)?;

        if header.size > MAX_CHUNK_SIZE {
            return Err(AkpError::InvalidChunkSize(header.id, header.size));
        }

        let chunk_start = file.stream_position()?;
        if chunk_start + header.size as u64 > end_pos {
            return Err(AkpError::CorruptedChunk(
                header.id,
                "Chunk extends beyond keygroup boundary".to_string(),
            ));
        }

        let mut chunk_data = vec![0; header.size as usize];
        file.read_exact(&mut chunk_data)?;
        let mut cursor = Cursor::new(chunk_data);

        match header.id.as_str() {
            "kloc" => {
                if header.size < 16 {
                    return Err(AkpError::InvalidChunkSize("kloc".to_string(), header.size));
                }
                parse_kloc_chunk(&mut cursor, &mut keygroup)?;
            }
            "env " => {
                if header.size < 18 {
                    return Err(AkpError::InvalidChunkSize("env".to_string(), header.size));
                }
                match env_count {
                    0 => keygroup.amp_env = Some(parse_amp_env_chunk(&mut cursor)?),
                    1 => keygroup.filter_env = Some(parse_filter_env_chunk(&mut cursor)?),
                    2 => keygroup.aux_env = Some(parse_aux_env_chunk(&mut cursor)?),
                    _ => {}
                }
                env_count += 1;
            }
            "filt" => {
                if header.size < 9 {
                    return Err(AkpError::InvalidChunkSize("filt".to_string(), header.size));
                }
                keygroup.filter = Some(parse_filt_chunk(&mut cursor)?);
            }
            "zone" => {
                if header.size < 2 {
                    return Err(AkpError::InvalidChunkSize("zone".to_string(), header.size));
                }
                if let Some(zone) = parse_zone_chunk(&mut cursor, header.size)? {
                    if keygroup.zones.len() < MAX_ZONES_PER_KEYGROUP {
                        keygroup.zones.push(zone);
                    }
                }
            }
            "smpl" => {
                // Not in spec — third-party tool artifact. Skip silently.
            }
            _ => {
                progress.println(format!("Warning: Skipping unknown keygroup chunk type '{}'", header.id));
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

pub fn parse_out_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<ProgramOutput> {
    cursor.seek(SeekFrom::Start(1))?;
    let loudness = cursor.read_u8()?;
    let amp_mod_1 = cursor.read_u8()?;
    let amp_mod_2 = cursor.read_u8()?;
    let pan_mod_1 = cursor.read_u8()?;
    let pan_mod_2 = cursor.read_u8()?;
    let pan_mod_3 = cursor.read_u8()?;
    let velocity_sensitivity = cursor.read_i8()?;
    Ok(ProgramOutput { loudness, amp_mod_1, amp_mod_2, pan_mod_1, pan_mod_2, pan_mod_3, velocity_sensitivity })
}

pub fn parse_tune_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<ProgramTuning> {
    cursor.seek(SeekFrom::Start(1))?;
    let semitone = cursor.read_i8()?;
    let fine = cursor.read_i8()?;
    let mut detune = [0i8; 12];
    for d in &mut detune {
        *d = cursor.read_i8()?;
    }
    let pitchbend_up = cursor.read_u8()?;
    let pitchbend_down = cursor.read_u8()?;
    let bend_mode = cursor.read_u8()?;
    let aftertouch = cursor.read_i8()?;
    Ok(ProgramTuning { semitone, fine, detune, pitchbend_up, pitchbend_down, bend_mode, aftertouch })
}

pub fn parse_lfo1_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Lfo> {
    cursor.seek(SeekFrom::Start(1))?;
    let waveform = cursor.read_u8()?;
    let rate = cursor.read_u8()?;
    let delay = cursor.read_u8()?;
    let depth = cursor.read_u8()?;
    let sync = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(7))?; // skip marker byte at offset 6
    let modwheel = cursor.read_u8()?;
    let aftertouch = cursor.read_u8()?;
    let rate_mod = cursor.read_i8()?;
    let delay_mod = cursor.read_i8()?;
    let depth_mod = cursor.read_i8()?;
    Ok(Lfo { waveform, rate, delay, depth, sync, retrigger: 0, modwheel, aftertouch, rate_mod, delay_mod, depth_mod })
}

pub fn parse_lfo2_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Lfo> {
    cursor.seek(SeekFrom::Start(1))?;
    let waveform = cursor.read_u8()?;
    let rate = cursor.read_u8()?;
    let delay = cursor.read_u8()?;
    let depth = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(6))?; // skip reserved byte at offset 5
    let retrigger = cursor.read_u8()?;
    // offsets 7-8 are reserved in LFO 2
    cursor.seek(SeekFrom::Start(9))?;
    let rate_mod = cursor.read_i8()?;
    let delay_mod = cursor.read_i8()?;
    let depth_mod = cursor.read_i8()?;
    Ok(Lfo { waveform, rate, delay, depth, sync: 0, retrigger, modwheel: 0, aftertouch: 0, rate_mod, delay_mod, depth_mod })
}

pub fn parse_mods_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<ProgramModulation> {
    // Source bytes at odd offsets: 5,7,9,11,13,15,17,19,21,23,25,27,29,31,33,35,37
    let offsets: [u64; 17] = [5,7,9,11,13,15,17,19,21,23,25,27,29,31,33,35,37];
    let mut sources = [0u8; 17];
    for (i, &offset) in offsets.iter().enumerate() {
        cursor.seek(SeekFrom::Start(offset))?;
        sources[i] = cursor.read_u8()?;
    }
    Ok(ProgramModulation {
        amp_mod_1_source: sources[0],
        amp_mod_2_source: sources[1],
        pan_mod_1_source: sources[2],
        pan_mod_2_source: sources[3],
        pan_mod_3_source: sources[4],
        lfo1_rate_mod_source: sources[5],
        lfo1_delay_mod_source: sources[6],
        lfo1_depth_mod_source: sources[7],
        lfo2_rate_mod_source: sources[8],
        lfo2_delay_mod_source: sources[9],
        lfo2_depth_mod_source: sources[10],
        pitch_mod_1_source: sources[11],
        pitch_mod_2_source: sources[12],
        amp_mod_source: sources[13],
        filter_mod_1_source: sources[14],
        filter_mod_2_source: sources[15],
        filter_mod_3_source: sources[16],
    })
}

pub fn parse_kloc_chunk(cursor: &mut Cursor<Vec<u8>>, keygroup: &mut Keygroup) -> Result<()> {
    cursor.seek(SeekFrom::Start(4))?;
    keygroup.low_key = cursor.read_u8()?;
    keygroup.high_key = cursor.read_u8()?;
    keygroup.semitone_tune = cursor.read_i8()?;
    keygroup.fine_tune = cursor.read_i8()?;
    keygroup.override_fx = cursor.read_u8()?;
    keygroup.fx_send_level = cursor.read_u8()?;
    keygroup.pitch_mod_1 = cursor.read_i8()?;
    keygroup.pitch_mod_2 = cursor.read_i8()?;
    keygroup.amp_mod = cursor.read_i8()?;
    keygroup.zone_crossfade = cursor.read_u8()?;
    keygroup.mute_group = cursor.read_u8()?;

    if keygroup.low_key > keygroup.high_key {
        return Err(AkpError::InvalidKeyRange(keygroup.low_key, keygroup.high_key));
    }
    if keygroup.high_key > 127 {
        return Err(AkpError::InvalidParameterValue("high_key".to_string(), keygroup.high_key));
    }

    Ok(())
}

pub fn parse_amp_env_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Envelope> {
    // Non-sequential: attack=1, decay=3, release=4, sustain=7
    cursor.seek(SeekFrom::Start(1))?;
    let attack = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(3))?;
    let decay = cursor.read_u8()?;
    let release = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(7))?;
    let sustain = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(10))?;
    let velocity_attack = cursor.read_i8()?;
    cursor.seek(SeekFrom::Start(12))?;
    let keyscale = cursor.read_i8()?;
    cursor.seek(SeekFrom::Start(14))?;
    let on_vel_release = cursor.read_i8()?;
    let off_vel_release = cursor.read_i8()?;
    Ok(Envelope { attack, decay, release, sustain, velocity_attack, keyscale, on_vel_release, off_vel_release })
}

pub fn parse_filter_env_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<FilterEnvelope> {
    cursor.seek(SeekFrom::Start(1))?;
    let attack = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(3))?;
    let decay = cursor.read_u8()?;
    let release = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(7))?;
    let sustain = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(9))?;
    let depth = cursor.read_i8()?;
    let velocity_attack = cursor.read_i8()?;
    cursor.seek(SeekFrom::Start(12))?;
    let keyscale = cursor.read_i8()?;
    cursor.seek(SeekFrom::Start(14))?;
    let on_vel_release = cursor.read_i8()?;
    let off_vel_release = cursor.read_i8()?;
    Ok(FilterEnvelope { attack, decay, release, sustain, depth, velocity_attack, keyscale, on_vel_release, off_vel_release })
}

pub fn parse_aux_env_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<AuxEnvelope> {
    cursor.seek(SeekFrom::Start(1))?;
    let rate_1 = cursor.read_u8()?;
    let rate_2 = cursor.read_u8()?;
    let rate_3 = cursor.read_u8()?;
    let rate_4 = cursor.read_u8()?;
    let level_1 = cursor.read_u8()?;
    let level_2 = cursor.read_u8()?;
    let level_3 = cursor.read_u8()?;
    let level_4 = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(10))?;
    let vel_rate_1 = cursor.read_i8()?;
    cursor.seek(SeekFrom::Start(12))?;
    let key_rate_2_4 = cursor.read_i8()?;
    cursor.seek(SeekFrom::Start(14))?;
    let vel_rate_4 = cursor.read_i8()?;
    let off_vel_rate_4 = cursor.read_i8()?;
    let vel_output_level = cursor.read_i8()?;
    Ok(AuxEnvelope { rate_1, rate_2, rate_3, rate_4, level_1, level_2, level_3, level_4, vel_rate_1, key_rate_2_4, vel_rate_4, off_vel_rate_4, vel_output_level })
}

pub fn parse_filt_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Filter> {
    cursor.seek(SeekFrom::Start(1))?;
    let filter_type = cursor.read_u8()?;
    let cutoff = cursor.read_u8()?;
    let resonance = cursor.read_u8()?;
    let keyboard_track = cursor.read_i8()?;
    let mod_input_1 = cursor.read_i8()?;
    let mod_input_2 = cursor.read_i8()?;
    let mod_input_3 = cursor.read_i8()?;
    let headroom = cursor.read_u8()?;

    if filter_type > 25 {
        return Err(AkpError::InvalidParameterValue("filter_type".to_string(), filter_type));
    }

    Ok(Filter { filter_type, cutoff, resonance, keyboard_track, mod_input_1, mod_input_2, mod_input_3, headroom })
}

pub fn parse_zone_chunk(cursor: &mut Cursor<Vec<u8>>, chunk_size: u32) -> Result<Option<Zone>> {
    cursor.seek(SeekFrom::Start(1))?;
    let name_len = cursor.read_u8()? as usize;

    // Zones with name_len=0 are sample parameter blocks — skip
    if name_len == 0 {
        return Ok(None);
    }

    if name_len > 20 {
        return Err(AkpError::CorruptedChunk("zone".to_string(), format!("name_len {name_len} exceeds max 20")));
    }

    if chunk_size < 46 {
        return Err(AkpError::InvalidChunkSize("zone".to_string(), chunk_size));
    }

    // Read sample name (offsets 2-21, 20 bytes max)
    let mut name_buf = [0u8; 20];
    cursor.read_exact(&mut name_buf)?;
    let end = name_buf.iter().position(|&b| b == 0).unwrap_or(name_len.min(20));
    let raw_filename = String::from_utf8_lossy(&name_buf[..end]).to_string();
    let sample_name = sanitize_sample_path(&raw_filename);

    if sample_name.is_empty() {
        return Ok(None);
    }

    // Read remaining fields at their spec offsets
    cursor.seek(SeekFrom::Start(34))?;
    let low_vel = cursor.read_u8()?;
    let high_vel = cursor.read_u8()?;
    let fine_tune = cursor.read_i8()?;
    let semitone_tune = cursor.read_i8()?;
    let filter = cursor.read_i8()?;
    let pan = cursor.read_i8()?;
    let playback = cursor.read_u8()?;
    let output = cursor.read_u8()?;
    let level = cursor.read_i8()?;
    let keyboard_track = cursor.read_u8()?;
    let vel_to_start = cursor.read_i16::<LittleEndian>()?;

    // Treat 0,0 velocity as full range
    let (low_vel, high_vel) = if low_vel == 0 && high_vel == 0 {
        (0, 127)
    } else {
        if low_vel > high_vel {
            return Err(AkpError::InvalidVelocityRange(low_vel, high_vel));
        }
        if high_vel > 127 {
            return Err(AkpError::InvalidParameterValue("high_vel".to_string(), high_vel));
        }
        (low_vel, high_vel)
    };

    Ok(Some(Zone {
        sample_name,
        low_vel,
        high_vel,
        fine_tune,
        semitone_tune,
        filter,
        pan,
        playback,
        output,
        level,
        keyboard_track,
        vel_to_start,
    }))
}

/// Sanitize a sample path from an AKP file.
fn sanitize_sample_path(raw: &str) -> String {
    let normalized = raw.replace('\\', "/");

    let without_drive = if normalized.len() >= 3
        && normalized.as_bytes()[0].is_ascii_alphabetic()
        && &normalized[1..3] == ":/"
    {
        &normalized[3..]
    } else {
        &normalized
    };

    let clean: Vec<&str> = without_drive
        .split('/')
        .filter(|component| {
            !component.is_empty() && *component != "." && *component != ".."
        })
        .collect();

    clean.join("/")
}
```

- [ ] **Step 2: Write parser unit tests**

Append to the bottom of `src/parser.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ---- Zone tests ----

    fn make_zone_data(name: &[u8], low_vel: u8, high_vel: u8) -> Vec<u8> {
        let mut data = vec![0u8; 48];
        let name_len = name.len().min(20) as u8;
        data[1] = name_len;
        data[2..2 + name_len as usize].copy_from_slice(&name[..name_len as usize]);
        data[34] = low_vel;
        data[35] = high_vel;
        data
    }

    #[test]
    fn test_parse_zone_extracts_sample_name() {
        let data = make_zone_data(b"Piano_C3.wav", 0, 127);
        let mut cursor = Cursor::new(data);
        let zone = parse_zone_chunk(&mut cursor, 48).unwrap().unwrap();
        assert_eq!(zone.sample_name, "Piano_C3.wav");
        assert_eq!(zone.low_vel, 0);
        assert_eq!(zone.high_vel, 127);
    }

    #[test]
    fn test_parse_zone_20char_name() {
        let data = make_zone_data(b"ABCDEFGHIJKLMNOPQRST", 1, 127);
        let mut cursor = Cursor::new(data);
        let zone = parse_zone_chunk(&mut cursor, 48).unwrap().unwrap();
        assert_eq!(zone.sample_name, "ABCDEFGHIJKLMNOPQRST");
    }

    #[test]
    fn test_parse_zone_zero_vel_full_range() {
        let data = make_zone_data(b"test.wav", 0, 0);
        let mut cursor = Cursor::new(data);
        let zone = parse_zone_chunk(&mut cursor, 48).unwrap().unwrap();
        assert_eq!(zone.low_vel, 0);
        assert_eq!(zone.high_vel, 127);
    }

    #[test]
    fn test_parse_zone_invalid_velocity_range() {
        let data = make_zone_data(b"test.wav", 127, 64);
        let mut cursor = Cursor::new(data);
        let result = parse_zone_chunk(&mut cursor, 48);
        assert!(matches!(result, Err(AkpError::InvalidVelocityRange(127, 64))));
    }

    #[test]
    fn test_parse_zone_no_name_skipped() {
        let mut data = vec![0u8; 48];
        data[1] = 0;
        let mut cursor = Cursor::new(data);
        let result = parse_zone_chunk(&mut cursor, 48).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_zone_name_len_overflow() {
        let mut data = vec![0u8; 48];
        data[1] = 21; // exceeds max 20
        let mut cursor = Cursor::new(data);
        let result = parse_zone_chunk(&mut cursor, 48);
        assert!(matches!(result, Err(AkpError::CorruptedChunk(_, _))));
    }

    #[test]
    fn test_parse_zone_extra_fields() {
        let mut data = make_zone_data(b"test.wav", 20, 100);
        data[36] = (-5i8) as u8;   // fine_tune
        data[37] = 3;               // semitone_tune
        data[38] = (-10i8) as u8;  // filter
        data[39] = (-25i8) as u8;  // pan (L25)
        data[40] = 3;               // playback (LOOP UNTIL REL)
        data[41] = 0;               // output
        data[42] = 10;              // level
        data[43] = 1;               // keyboard_track
        data[44] = 0;               // vel_to_start low
        data[45] = 0;               // vel_to_start high
        let mut cursor = Cursor::new(data);
        let zone = parse_zone_chunk(&mut cursor, 48).unwrap().unwrap();
        assert_eq!(zone.fine_tune, -5);
        assert_eq!(zone.semitone_tune, 3);
        assert_eq!(zone.filter, -10);
        assert_eq!(zone.pan, -25);
        assert_eq!(zone.playback, 3);
        assert_eq!(zone.level, 10);
        assert_eq!(zone.keyboard_track, 1);
    }

    // ---- kloc tests ----

    #[test]
    fn test_parse_kloc_chunk_expanded() {
        let mut data = vec![0u8; 16];
        data[4] = 36;               // low_key
        data[5] = 72;               // high_key
        data[6] = (-12i8) as u8;   // semitone_tune
        data[7] = 25;               // fine_tune
        data[8] = 1;                // override_fx (FX1)
        data[9] = 50;               // fx_send_level
        data[10] = (-30i8) as u8;  // pitch_mod_1
        data[11] = 0;               // pitch_mod_2
        data[12] = (-50i8) as u8;  // amp_mod
        data[13] = 1;               // zone_crossfade (ON)
        data[14] = 3;               // mute_group
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();
        parse_kloc_chunk(&mut cursor, &mut keygroup).unwrap();
        assert_eq!(keygroup.low_key, 36);
        assert_eq!(keygroup.high_key, 72);
        assert_eq!(keygroup.semitone_tune, -12);
        assert_eq!(keygroup.fine_tune, 25);
        assert_eq!(keygroup.override_fx, 1);
        assert_eq!(keygroup.fx_send_level, 50);
        assert_eq!(keygroup.pitch_mod_1, -30);
        assert_eq!(keygroup.zone_crossfade, 1);
        assert_eq!(keygroup.mute_group, 3);
    }

    #[test]
    fn test_parse_kloc_chunk_invalid_range() {
        let mut data = vec![0u8; 16];
        data[4] = 80;
        data[5] = 40;
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();
        let result = parse_kloc_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidKeyRange(80, 40))));
    }

    // ---- out chunk tests ----

    #[test]
    fn test_parse_out_chunk() {
        let data = vec![0, 85, 10, 20, 30, 40, 50, 25];
        let mut cursor = Cursor::new(data);
        let out = parse_out_chunk(&mut cursor).unwrap();
        assert_eq!(out.loudness, 85);
        assert_eq!(out.amp_mod_1, 10);
        assert_eq!(out.velocity_sensitivity, 25);
    }

    // ---- tune chunk tests ----

    #[test]
    fn test_parse_tune_chunk() {
        let mut data = vec![0u8; 22];
        data[1] = (-12i8) as u8;    // semitone
        data[2] = 25;                // fine
        // detune: offsets 3-14
        data[3] = (-5i8) as u8;     // C detune
        data[15] = 2;                // pitchbend_up
        data[16] = 2;                // pitchbend_down
        data[17] = 0;                // bend_mode
        data[18] = (-6i8) as u8;    // aftertouch
        let mut cursor = Cursor::new(data);
        let tune = parse_tune_chunk(&mut cursor).unwrap();
        assert_eq!(tune.semitone, -12);
        assert_eq!(tune.fine, 25);
        assert_eq!(tune.detune[0], -5); // C
        assert_eq!(tune.pitchbend_up, 2);
        assert_eq!(tune.pitchbend_down, 2);
        assert_eq!(tune.aftertouch, -6);
    }

    // ---- LFO tests ----

    #[test]
    fn test_parse_lfo1_chunk() {
        let mut data = vec![0u8; 12];
        data[1] = 1;                // waveform (TRIANGLE)
        data[2] = 50;               // rate
        data[3] = 20;               // delay
        data[4] = 75;               // depth
        data[5] = 1;                // sync (ON)
        data[7] = 80;               // modwheel
        data[8] = 40;               // aftertouch
        data[9] = (-10i8) as u8;   // rate_mod
        data[10] = 0;               // delay_mod
        data[11] = (-20i8) as u8;  // depth_mod
        let mut cursor = Cursor::new(data);
        let lfo = parse_lfo1_chunk(&mut cursor).unwrap();
        assert_eq!(lfo.waveform, 1);
        assert_eq!(lfo.rate, 50);
        assert_eq!(lfo.depth, 75);
        assert_eq!(lfo.sync, 1);
        assert_eq!(lfo.modwheel, 80);
        assert_eq!(lfo.aftertouch, 40);
        assert_eq!(lfo.retrigger, 0); // not in LFO 1
        assert_eq!(lfo.rate_mod, -10);
    }

    #[test]
    fn test_parse_lfo2_chunk() {
        let mut data = vec![0u8; 12];
        data[1] = 0;                // waveform (SINE)
        data[2] = 30;               // rate
        data[3] = 10;               // delay
        data[4] = 60;               // depth
        data[6] = 1;                // retrigger (ON)
        data[9] = 5;                // rate_mod
        let mut cursor = Cursor::new(data);
        let lfo = parse_lfo2_chunk(&mut cursor).unwrap();
        assert_eq!(lfo.waveform, 0);
        assert_eq!(lfo.rate, 30);
        assert_eq!(lfo.retrigger, 1);
        assert_eq!(lfo.sync, 0);       // not in LFO 2
        assert_eq!(lfo.modwheel, 0);   // not in LFO 2
        assert_eq!(lfo.aftertouch, 0); // not in LFO 2
    }

    // ---- mods tests ----

    #[test]
    fn test_parse_mods_chunk() {
        let mut data = vec![0u8; 38];
        data[5] = 6;    // amp_mod_1_source = KEYBOARD
        data[27] = 7;   // pitch_mod_1_source = LFO1
        data[31] = 5;   // amp_mod_source = VELOCITY
        let mut cursor = Cursor::new(data);
        let mods = parse_mods_chunk(&mut cursor).unwrap();
        assert_eq!(mods.amp_mod_1_source, 6);
        assert_eq!(mods.pitch_mod_1_source, 7);
        assert_eq!(mods.amp_mod_source, 5);
        assert_eq!(mods.pan_mod_1_source, 0);
    }

    // ---- Envelope tests ----

    #[test]
    fn test_parse_amp_env_chunk() {
        let mut data = vec![0u8; 18];
        data[1] = 10;                // attack
        data[3] = 50;                // decay
        data[4] = 30;                // release
        data[7] = 80;                // sustain
        data[10] = (-20i8) as u8;   // velocity_attack
        data[12] = 5;                // keyscale
        data[14] = (-10i8) as u8;   // on_vel_release
        data[15] = (-5i8) as u8;    // off_vel_release
        let mut cursor = Cursor::new(data);
        let env = parse_amp_env_chunk(&mut cursor).unwrap();
        assert_eq!(env.attack, 10);
        assert_eq!(env.decay, 50);
        assert_eq!(env.release, 30);
        assert_eq!(env.sustain, 80);
        assert_eq!(env.velocity_attack, -20);
        assert_eq!(env.keyscale, 5);
        assert_eq!(env.on_vel_release, -10);
        assert_eq!(env.off_vel_release, -5);
    }

    #[test]
    fn test_parse_filter_env_chunk() {
        let mut data = vec![0u8; 18];
        data[1] = 5;                 // attack
        data[3] = 60;                // decay
        data[4] = 40;                // release
        data[7] = 70;                // sustain
        data[9] = (-50i8) as u8;    // depth
        data[10] = (-15i8) as u8;   // velocity_attack
        let mut cursor = Cursor::new(data);
        let env = parse_filter_env_chunk(&mut cursor).unwrap();
        assert_eq!(env.attack, 5);
        assert_eq!(env.decay, 60);
        assert_eq!(env.release, 40);
        assert_eq!(env.sustain, 70);
        assert_eq!(env.depth, -50);
        assert_eq!(env.velocity_attack, -15);
    }

    #[test]
    fn test_parse_aux_env_chunk() {
        let mut data = vec![0u8; 18];
        data[1] = 10;  // rate_1
        data[2] = 20;  // rate_2
        data[3] = 30;  // rate_3
        data[4] = 40;  // rate_4
        data[5] = 50;  // level_1
        data[6] = 60;  // level_2
        data[7] = 70;  // level_3
        data[8] = 80;  // level_4
        data[10] = (-10i8) as u8; // vel_rate_1
        data[12] = 5;  // key_rate_2_4
        data[14] = (-20i8) as u8; // vel_rate_4
        data[15] = (-30i8) as u8; // off_vel_rate_4
        data[16] = (-40i8) as u8; // vel_output_level
        let mut cursor = Cursor::new(data);
        let env = parse_aux_env_chunk(&mut cursor).unwrap();
        assert_eq!(env.rate_1, 10);
        assert_eq!(env.rate_4, 40);
        assert_eq!(env.level_1, 50);
        assert_eq!(env.level_4, 80);
        assert_eq!(env.vel_rate_1, -10);
        assert_eq!(env.vel_output_level, -40);
    }

    // ---- filt tests ----

    #[test]
    fn test_parse_filt_chunk_expanded() {
        let data = vec![0, 2, 75, 8, 10, (-20i8) as u8, 30, (-40i8) as u8, 3];
        let mut cursor = Cursor::new(data);
        let filter = parse_filt_chunk(&mut cursor).unwrap();
        assert_eq!(filter.filter_type, 2);
        assert_eq!(filter.cutoff, 75);
        assert_eq!(filter.resonance, 8);
        assert_eq!(filter.keyboard_track, 10);
        assert_eq!(filter.mod_input_1, -20);
        assert_eq!(filter.mod_input_2, 30);
        assert_eq!(filter.mod_input_3, -40);
        assert_eq!(filter.headroom, 3);
    }

    #[test]
    fn test_parse_filt_chunk_type_zero_is_valid() {
        let data = vec![0, 0, 100, 0, 0, 0, 0, 0, 0];
        let mut cursor = Cursor::new(data);
        let filter = parse_filt_chunk(&mut cursor).unwrap();
        assert_eq!(filter.filter_type, 0); // 2-pole LP, active
        assert_eq!(filter.cutoff, 100);
    }

    #[test]
    fn test_parse_filt_chunk_invalid_type() {
        let data = vec![0, 26, 75, 8, 0, 0, 0, 0, 0];
        let mut cursor = Cursor::new(data);
        let result = parse_filt_chunk(&mut cursor);
        assert!(matches!(result, Err(AkpError::InvalidParameterValue(_, 26))));
    }

    // ---- Sanitize path tests ----

    #[test]
    fn test_sanitize_sample_path_basic() {
        assert_eq!(sanitize_sample_path("test.wav"), "test.wav");
    }

    #[test]
    fn test_sanitize_sample_path_backslash() {
        assert_eq!(sanitize_sample_path("Strings\\Violin_C3.wav"), "Strings/Violin_C3.wav");
    }

    #[test]
    fn test_sanitize_sample_path_dotdot() {
        assert_eq!(sanitize_sample_path("../../etc/passwd"), "etc/passwd");
    }

    #[test]
    fn test_sanitize_sample_path_drive_letter() {
        assert_eq!(sanitize_sample_path("C:/Samples/test.wav"), "Samples/test.wav");
    }

    // ---- Waveform name tests ----

    #[test]
    fn test_lfo_waveform_name() {
        assert_eq!((Lfo { waveform: 0, ..Default::default() }).waveform_name(), "sine");
        assert_eq!((Lfo { waveform: 1, ..Default::default() }).waveform_name(), "triangle");
        assert_eq!((Lfo { waveform: 2, ..Default::default() }).waveform_name(), "square");
        assert_eq!((Lfo { waveform: 8, ..Default::default() }).waveform_name(), "random");
    }

    // ---- Filter helper tests ----

    #[test]
    fn test_filter_sfz_type() {
        let f = |t: u8| Filter { filter_type: t, ..Default::default() }.sfz_filter_type();
        assert_eq!(f(0), "lpf_2p");
        assert_eq!(f(3), "bpf_2p");
        assert_eq!(f(7), "hpf_2p");
        assert_eq!(f(12), "brf_2p");
        assert_eq!(f(17), "pkf_2p");
        assert_eq!(f(25), "lpf_2p"); // voweliser fallback
    }

    #[test]
    fn test_filter_resonance_db() {
        let f = Filter { resonance: 12, ..Default::default() };
        assert!((f.resonance_db() - 40.0).abs() < 0.01);
        let f = Filter { resonance: 0, ..Default::default() };
        assert!((f.resonance_db() - 0.0).abs() < 0.01);
    }

    // ---- Program header test ----

    #[test]
    fn test_parse_program_header() {
        let data = vec![0, 5, 11]; // flags=0, midi_pgm=5, num_keygroups=11
        let mut cursor = Cursor::new(data);
        let header = parse_program_header(&mut cursor).unwrap();
        assert_eq!(header.midi_program_number, 5);
        assert_eq!(header.number_of_keygroups, 11);
    }
}
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd /Users/davidryan/GitHub/rusty-samplers && cargo test --lib -- parser::tests 2>&1 | tail -30`

Expected: All parser tests PASS. Compilation errors may still exist in sfz.rs and dspreset.rs (fixed in next tasks).

- [ ] **Step 4: Commit parser.rs**

```bash
git add src/parser.rs
git commit -m "parser: rewrite all functions to match AKP spec byte offsets"
```

---

## Chunk 3: Output Generators

### Task 3: Update sfz.rs for new types

**Files:**
- Rewrite: `src/sfz.rs`

Update SFZ generator to iterate zones, use new struct fields, fix filter/resonance/LFO mappings.

- [ ] **Step 1: Write the new sfz.rs**

```rust
use crate::types::AkaiProgram;

impl AkaiProgram {
    pub fn to_sfz_string(&self) -> String {
        let mut sfz = String::new();
        sfz.push_str("// Generated by Rusty Samplers\n\n");

        // Global header from program-level data
        if let Some(tuning) = &self.tuning {
            sfz.push_str(&format!("bend_up={}\n", tuning.pitchbend_up as i32 * 100));
            sfz.push_str(&format!("bend_down=-{}\n", tuning.pitchbend_down as i32 * 100));
            sfz.push('\n');
        }

        for keygroup in &self.keygroups {
            // Each zone becomes a <region>
            let zones: Vec<_> = if keygroup.zones.is_empty() {
                // No zones — emit one region with no sample (keeps envelope/filter data visible)
                vec![None]
            } else {
                keygroup.zones.iter().map(Some).collect()
            };

            for zone in &zones {
                sfz.push_str("<region>\n");

                // Sample + velocity from zone
                if let Some(z) = zone {
                    sfz.push_str(&format!("sample={}\n", z.sample_name));
                    sfz.push_str(&format!("lokey={}\nhikey={}\n", keygroup.low_key, keygroup.high_key));
                    sfz.push_str(&format!("lovel={}\nhivel={}\n", z.low_vel, z.high_vel));

                    // Zone-level tuning (additive with keygroup)
                    let semitone = keygroup.semitone_tune as i16 + z.semitone_tune as i16;
                    let fine = keygroup.fine_tune as i16 + z.fine_tune as i16;
                    if semitone != 0 {
                        sfz.push_str(&format!("transpose={semitone}\n"));
                    }
                    if fine != 0 {
                        sfz.push_str(&format!("tune={fine}\n"));
                    }

                    // Zone pan
                    if z.pan != 0 {
                        sfz.push_str(&format!("pan={}\n", z.pan));
                    }

                    // Zone level
                    if z.level != 0 {
                        sfz.push_str(&format!("volume={}\n", z.level));
                    }

                    // Playback mode from zone
                    match z.playback {
                        0 => sfz.push_str("loop_mode=no_loop\n"),
                        1 => sfz.push_str("loop_mode=one_shot\n"),
                        2 => sfz.push_str("loop_mode=loop_continuous\n"),
                        3 => sfz.push_str("loop_mode=loop_sustain\n"),
                        _ => {} // 4=AS SAMPLE, use sample header default
                    }
                } else {
                    sfz.push_str(&format!("lokey={}\nhikey={}\n", keygroup.low_key, keygroup.high_key));
                }

                // Loudness from program output
                if let Some(output) = &self.output {
                    if output.loudness != 85 {
                        sfz.push_str(&format!("amplitude={}\n", output.loudness));
                    }
                }

                // Amp envelope
                if let Some(env) = &keygroup.amp_env {
                    sfz.push_str(&format!("ampeg_attack={:.3}\n", env.attack_time()));
                    sfz.push_str(&format!("ampeg_decay={:.3}\n", env.decay_time()));
                    sfz.push_str(&format!("ampeg_sustain={}\n", env.sustain));
                    sfz.push_str(&format!("ampeg_release={:.3}\n", env.release_time()));

                    if env.velocity_attack != 0 {
                        sfz.push_str(&format!("ampeg_vel2attack={}\n", env.velocity_attack));
                    }
                }

                // Filter
                if let Some(filter) = &keygroup.filter {
                    sfz.push_str(&format!("fil_type={}\n", filter.sfz_filter_type()));
                    sfz.push_str(&format!("cutoff={:.1}\n", filter.cutoff_hz()));
                    sfz.push_str(&format!("resonance={:.1}\n", filter.resonance_db()));

                    if filter.keyboard_track != 0 {
                        // Convert -36..+36 semitones to cents
                        sfz.push_str(&format!("fil_keytrack={}\n", filter.keyboard_track as i32 * 100));
                    }
                }

                // Filter envelope
                if let Some(env) = &keygroup.filter_env {
                    sfz.push_str(&format!("fileg_attack={:.3}\n", env.attack_time()));
                    sfz.push_str(&format!("fileg_decay={:.3}\n", env.decay_time()));
                    sfz.push_str(&format!("fileg_sustain={}\n", env.sustain));
                    sfz.push_str(&format!("fileg_release={:.3}\n", env.release_time()));

                    if env.depth != 0 {
                        // Convert depth (-100..100) to cents
                        let depth_cents = env.depth as f32 / 100.0 * 9600.0;
                        sfz.push_str(&format!("fileg_depth={:.0}\n", depth_cents));
                    }
                }

                // LFOs from program level
                if let Some(lfo) = &self.lfo1 {
                    if lfo.depth > 0 {
                        sfz.push_str(&format!("lfo1_freq={:.2}\n", lfo.rate_hz()));
                        sfz.push_str(&format!("lfo1_wave={}\n", lfo.waveform_name()));
                        let depth_cents = lfo.depth_normalized() * 100.0;
                        sfz.push_str(&format!("lfo1_pitch={depth_cents:.1}\n"));

                        if lfo.delay > 0 {
                            let delay_time = (lfo.delay as f32 / 100.0) * 10.0;
                            sfz.push_str(&format!("lfo1_delay={delay_time:.2}\n"));
                        }
                    }
                }

                if let Some(lfo) = &self.lfo2 {
                    if lfo.depth > 0 {
                        sfz.push_str(&format!("lfo2_freq={:.2}\n", lfo.rate_hz()));
                        sfz.push_str(&format!("lfo2_wave={}\n", lfo.waveform_name()));
                        let depth_cents = lfo.depth_normalized() * 100.0;
                        sfz.push_str(&format!("lfo2_pitch={depth_cents:.1}\n"));

                        if lfo.delay > 0 {
                            let delay_time = (lfo.delay as f32 / 100.0) * 10.0;
                            sfz.push_str(&format!("lfo2_delay={delay_time:.2}\n"));
                        }
                    }
                }

                sfz.push('\n');
            }
        }

        sfz
    }
}

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn test_sfz_generation_basic() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.low_key = 60;
        keygroup.high_key = 72;
        keygroup.zones.push(Zone {
            sample_name: "test.wav".to_string(),
            low_vel: 1,
            high_vel: 127,
            ..Default::default()
        });
        program.keygroups.push(keygroup);

        let sfz = program.to_sfz_string();
        assert!(sfz.contains("<region>"));
        assert!(sfz.contains("sample=test.wav"));
        assert!(sfz.contains("lokey=60"));
        assert!(sfz.contains("hikey=72"));
        assert!(sfz.contains("lovel=1"));
        assert!(sfz.contains("hivel=127"));
    }

    #[test]
    fn test_sfz_filter_type_zero_is_active() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.filter = Some(Filter { filter_type: 0, cutoff: 50, ..Default::default() });
        keygroup.zones.push(Zone { sample_name: "t.wav".to_string(), ..Default::default() });
        program.keygroups.push(keygroup);

        let sfz = program.to_sfz_string();
        assert!(sfz.contains("fil_type=lpf_2p"));
        assert!(sfz.contains("cutoff="));
    }

    #[test]
    fn test_sfz_resonance_from_0_12_range() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.filter = Some(Filter { filter_type: 0, cutoff: 50, resonance: 6, ..Default::default() });
        keygroup.zones.push(Zone { sample_name: "t.wav".to_string(), ..Default::default() });
        program.keygroups.push(keygroup);

        let sfz = program.to_sfz_string();
        assert!(sfz.contains("resonance=20.0")); // 6/12 * 40 = 20
    }

    #[test]
    fn test_sfz_multi_zone_keygroup() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.low_key = 36;
        keygroup.high_key = 72;
        keygroup.zones.push(Zone { sample_name: "soft.wav".to_string(), low_vel: 0, high_vel: 63, ..Default::default() });
        keygroup.zones.push(Zone { sample_name: "loud.wav".to_string(), low_vel: 64, high_vel: 127, ..Default::default() });
        program.keygroups.push(keygroup);

        let sfz = program.to_sfz_string();
        assert_eq!(sfz.matches("<region>").count(), 2);
        assert!(sfz.contains("sample=soft.wav"));
        assert!(sfz.contains("sample=loud.wav"));
        assert!(sfz.contains("hivel=63"));
        assert!(sfz.contains("lovel=64"));
    }

    #[test]
    fn test_sfz_pitchbend_from_tuning() {
        let mut program = AkaiProgram::default();
        program.tuning = Some(ProgramTuning { pitchbend_up: 12, pitchbend_down: 12, ..Default::default() });
        program.keygroups.push(Keygroup::default());

        let sfz = program.to_sfz_string();
        assert!(sfz.contains("bend_up=1200"));
        assert!(sfz.contains("bend_down=-1200"));
    }

    #[test]
    fn test_sfz_envelope_with_velocity() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.amp_env = Some(Envelope {
            attack: 20,
            decay: 40,
            sustain: 80,
            release: 60,
            velocity_attack: -20,
            ..Default::default()
        });
        keygroup.zones.push(Zone { sample_name: "t.wav".to_string(), ..Default::default() });
        program.keygroups.push(keygroup);

        let sfz = program.to_sfz_string();
        assert!(sfz.contains("ampeg_vel2attack=-20"));
        assert!(sfz.contains("ampeg_sustain=80"));
    }

    #[test]
    fn test_sfz_filter_env_depth() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.filter_env = Some(FilterEnvelope {
            attack: 5,
            decay: 60,
            sustain: 70,
            release: 40,
            depth: 50,
            ..Default::default()
        });
        keygroup.zones.push(Zone { sample_name: "t.wav".to_string(), ..Default::default() });
        program.keygroups.push(keygroup);

        let sfz = program.to_sfz_string();
        assert!(sfz.contains("fileg_depth=4800")); // 50/100 * 9600 = 4800
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd /Users/davidryan/GitHub/rusty-samplers && cargo test --lib -- sfz::tests 2>&1 | tail -20`

Expected: All SFZ tests PASS.

- [ ] **Step 3: Commit sfz.rs**

```bash
git add src/sfz.rs
git commit -m "sfz: update output for spec-aligned types and value mappings"
```

### Task 4: Update dspreset.rs for new types

**Files:**
- Rewrite: `src/dspreset.rs`

Same core changes as SFZ: iterate zones, use new struct fields, fix filter/resonance/LFO mappings.

- [ ] **Step 1: Write the new dspreset.rs**

```rust
use crate::types::AkaiProgram;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
}

impl AkaiProgram {
    pub fn to_dspreset_string(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<DecentSampler minVersion=\"1.0.0\">\n");

        // UI Section
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

        // Groups section
        xml.push_str("  <groups>\n");

        for (group_id, keygroup) in self.keygroups.iter().enumerate() {
            xml.push_str(&format!("    <group name=\"Group{}\"", group_id + 1));

            if let Some(env) = &keygroup.amp_env {
                let attack = if env.attack == 0 { 0.001 } else { env.attack_time() };
                let decay = if env.decay == 0 { 0.1 } else { env.decay_time() };
                let sustain = env.sustain_normalized();
                let release = if env.release == 0 { 0.1 } else { env.release_time() };
                xml.push_str(&format!(" attack=\"{attack:.3}\" decay=\"{decay:.3}\" sustain=\"{sustain:.3}\" release=\"{release:.3}\""));
            }

            // Volume from program output
            if let Some(output) = &self.output {
                xml.push_str(&format!(" volume=\"{:.2}\"", output.volume_db()));
            }

            xml.push_str(">\n");

            // Each zone becomes a <sample>
            for zone in &keygroup.zones {
                xml.push_str("      <sample ");
                xml.push_str(&format!("path=\"{}\" ", xml_escape(&zone.sample_name)));
                xml.push_str(&format!("loNote=\"{}\" hiNote=\"{}\" ", keygroup.low_key, keygroup.high_key));
                xml.push_str(&format!("loVel=\"{}\" hiVel=\"{}\" ", zone.low_vel, zone.high_vel));

                let semitone = keygroup.semitone_tune as i16 + zone.semitone_tune as i16;
                let fine = keygroup.fine_tune as i16 + zone.fine_tune as i16;
                if semitone != 0 {
                    xml.push_str(&format!("tuning=\"{semitone}\" "));
                }
                if fine != 0 {
                    xml.push_str(&format!("fineTuning=\"{fine}\" "));
                }

                if zone.pan != 0 {
                    // DS pan: -100 to 100
                    xml.push_str(&format!("pan=\"{}\" ", zone.pan as i32 * 2));
                }

                xml.push_str("/>\n");
            }

            xml.push_str("    </group>\n");
        }

        xml.push_str("  </groups>\n\n");

        // Effects section
        xml.push_str("  <effects>\n");
        let has_filter = self.keygroups.iter().any(|kg| kg.filter.is_some());
        if has_filter {
            xml.push_str("    <lowpass frequency=\"$FILTER_CUTOFF\" resonance=\"$FILTER_RESONANCE\" />\n");
        }
        xml.push_str("    <reverb roomSize=\"0.5\" damping=\"0.5\" wetLevel=\"0.3\" dryLevel=\"0.7\" width=\"1.0\" />\n");
        xml.push_str("  </effects>\n\n");

        // MIDI section
        xml.push_str("  <midi>\n");
        xml.push_str("    <cc number=\"1\" parameter=\"FILTER_CUTOFF\" />\n");
        xml.push_str("    <cc number=\"2\" parameter=\"FILTER_RESONANCE\" />\n");
        xml.push_str("    <cc number=\"7\" parameter=\"MAIN_VOLUME\" />\n");
        xml.push_str("  </midi>\n\n");

        // Modulators section
        if let Some(lfo) = &self.lfo1 {
            if lfo.depth > 0 {
                xml.push_str("  <modulators>\n");
                let amount = lfo.depth_normalized();
                xml.push_str(&format!(
                    "    <lfo frequency=\"{:.2}\" waveform=\"{}\" target=\"FILTER_CUTOFF\" amount=\"{amount:.2}\" />\n",
                    lfo.rate_hz(), lfo.waveform_name()));
                xml.push_str("  </modulators>\n\n");
            }
        }

        // Tags
        xml.push_str("  <tags>\n");
        xml.push_str("    <tag name=\"author\" value=\"Rusty Samplers\" />\n");
        xml.push_str("    <tag name=\"description\" value=\"Converted from AKP format\" />\n");
        xml.push_str("    <tag name=\"conversion-tool\" value=\"Rusty Samplers v1.0\" />\n");
        xml.push_str("  </tags>\n\n");

        xml.push_str("</DecentSampler>\n");
        xml
    }
}

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn test_dspreset_basic_structure() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.low_key = 36;
        keygroup.high_key = 72;
        keygroup.zones.push(Zone {
            sample_name: "piano.wav".to_string(),
            low_vel: 1,
            high_vel: 127,
            ..Default::default()
        });
        program.keygroups.push(keygroup);

        let xml = program.to_dspreset_string();
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<DecentSampler"));
        assert!(xml.contains("</DecentSampler>"));
        assert!(xml.contains("<group name=\"Group1\""));
        assert!(xml.contains("path=\"piano.wav\""));
        assert!(xml.contains("loNote=\"36\""));
        assert!(xml.contains("hiNote=\"72\""));
    }

    #[test]
    fn test_dspreset_filter_binding_uses_dollar_prefix() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.filter = Some(Filter { filter_type: 0, cutoff: 50, ..Default::default() });
        program.keygroups.push(keygroup);

        let xml = program.to_dspreset_string();
        assert!(xml.contains("frequency=\"$FILTER_CUTOFF\""));
        assert!(xml.contains("resonance=\"$FILTER_RESONANCE\""));
    }

    #[test]
    fn test_dspreset_envelope_values() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.amp_env = Some(Envelope { attack: 20, decay: 40, sustain: 80, release: 60, ..Default::default() });
        program.keygroups.push(keygroup);

        let xml = program.to_dspreset_string();
        assert!(xml.contains("attack=\""));
        assert!(xml.contains("sustain=\"0.800\""));
    }

    #[test]
    fn test_dspreset_multi_zone() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.zones.push(Zone { sample_name: "soft.wav".to_string(), low_vel: 0, high_vel: 63, ..Default::default() });
        keygroup.zones.push(Zone { sample_name: "loud.wav".to_string(), low_vel: 64, high_vel: 127, ..Default::default() });
        program.keygroups.push(keygroup);

        let xml = program.to_dspreset_string();
        assert!(xml.contains("path=\"soft.wav\""));
        assert!(xml.contains("path=\"loud.wav\""));
    }

    #[test]
    fn test_dspreset_lfo_from_program_level() {
        let mut program = AkaiProgram::default();
        program.lfo1 = Some(Lfo { waveform: 0, rate: 50, depth: 75, ..Default::default() });
        program.keygroups.push(Keygroup::default());

        let xml = program.to_dspreset_string();
        assert!(xml.contains("<modulators>"));
        assert!(xml.contains("waveform=\"sine\""));
        assert!(xml.contains("amount=\"0.75\""));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd /Users/davidryan/GitHub/rusty-samplers && cargo test --lib -- dspreset::tests 2>&1 | tail -20`

Expected: All dspreset tests PASS.

- [ ] **Step 3: Commit dspreset.rs**

```bash
git add src/dspreset.rs
git commit -m "dspreset: update output for spec-aligned types and zone iteration"
```

---

## Chunk 4: Integration and Verification

### Task 5: Fix lib.rs re-exports and run full test suite

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Check if lib.rs needs updates**

The current re-exports are:
```rust
pub use error::{AkpError, Result};
pub use types::{AkaiProgram, OutputFormat};
pub use parser::{validate_riff_header, parse_top_level_chunks};
```

These types and functions all still exist with the same signatures, so lib.rs should need no change. Verify compilation.

- [ ] **Step 2: Run full test suite**

Run: `cd /Users/davidryan/GitHub/rusty-samplers && cargo test 2>&1 | tail -40`

Expected: ALL tests pass (unit + integration). If integration tests fail, diagnose and fix.

- [ ] **Step 3: Run clippy on both crates**

Run: `cd /Users/davidryan/GitHub/rusty-samplers && cargo clippy 2>&1 | tail -20`
Run: `cd /Users/davidryan/GitHub/rusty-samplers/gui && cargo clippy 2>&1 | tail -20`

Expected: Clean on both. GUI should compile since it only uses `convert_file()` and `OutputFormat`.

- [ ] **Step 4: Commit any fixes**

```bash
git add src/lib.rs  # only if modified
git commit -m "fix: resolve compilation issues from parser rewrite"
```

### Task 6: Test against real S6000 factory files

**Files:**
- No code changes (verification only)

- [ ] **Step 1: Build and run against all 4 test files**

```bash
cd /Users/davidryan/GitHub/rusty-samplers
cargo build -p rusty-samplers --bin rusty-samplers-cli
for f in test_akp_files/*.AKP; do
    echo "=== $(basename "$f") ==="
    ./target/debug/rusty-samplers-cli "$f" 2>&1 | head -5
done
```

Expected: All 4 files parse without errors, each creates an .sfz file.

- [ ] **Step 2: Spot-check SFZ output correctness**

```bash
head -30 test_akp_files/ST_GRAND_PF.sfz
```

Verify:
- Sample names present (e.g., `C1-PNO93L -S`)
- Key ranges present and reasonable (e.g., `lokey=22 hikey=39`)
- Velocity ranges from zones (not hardcoded 0-127 for all)
- Pitchbend from program tuning (not hardcoded 200)
- Envelope sustain should be a small number (raw byte, ~2-3 for piano), not 39

- [ ] **Step 3: Test Decent Sampler output**

```bash
for f in test_akp_files/*.AKP; do
    echo "=== $(basename "$f") ==="
    ./target/debug/rusty-samplers-cli --format ds "$f" 2>&1 | head -5
done
head -30 test_akp_files/ST_GRAND_PF.dspreset
```

Expected: All produce .dspreset files with valid XML structure.

### Task 7: Update test data generator

**Files:**
- Rewrite: `create_test_akp.py`

- [ ] **Step 1: Write updated create_test_akp.py**

```python
#!/usr/bin/env python3
"""
Creates a minimal valid AKP file for testing the rusty-samplers converter.
Generates all spec chunk types with correct byte layouts.
"""

import struct


def make_chunk(chunk_id: bytes, data: bytes) -> bytes:
    """Build a RIFF chunk: 4-byte ID + LE uint32 size + data."""
    return chunk_id + struct.pack('<I', len(data)) + data


def create_test_akp():
    # prg chunk (6 bytes): byte 0=flags, 1=MIDI pgm#, 2=# keygroups
    prg_data = bytearray(6)
    prg_data[1] = 1   # MIDI program number
    prg_data[2] = 1   # 1 keygroup
    prg_chunk = make_chunk(b'prg ', bytes(prg_data))

    # out chunk (8 bytes)
    out_data = bytearray(8)
    out_data[1] = 85   # loudness
    out_data[7] = 25   # velocity_sensitivity (i8)
    out_chunk = make_chunk(b'out ', bytes(out_data))

    # tune chunk (22 bytes)
    tune_data = bytearray(22)
    tune_data[15] = 2   # pitchbend_up
    tune_data[16] = 2   # pitchbend_down
    tune_chunk = make_chunk(b'tune', bytes(tune_data))

    # lfo chunk 1 (12 bytes)
    lfo1_data = bytearray(12)
    lfo1_data[1] = 1   # waveform (TRIANGLE)
    lfo1_data[2] = 30  # rate
    lfo1_data[4] = 50  # depth
    lfo1_chunk = make_chunk(b'lfo ', bytes(lfo1_data))

    # lfo chunk 2 (12 bytes)
    lfo2_data = bytearray(12)
    lfo2_data[1] = 0   # waveform (SINE)
    lfo2_chunk = make_chunk(b'lfo ', bytes(lfo2_data))

    # mods chunk (38 bytes)
    mods_data = bytearray(38)
    mods_data[5] = 6    # amp_mod_1_source = KEYBOARD
    mods_data[27] = 7   # pitch_mod_1_source = LFO1
    mods_data[31] = 5   # amp_mod_source = VELOCITY
    mods_chunk = make_chunk(b'mods', bytes(mods_data))

    # -- Keygroup contents --

    # kloc chunk (16 bytes)
    kloc_data = bytearray(16)
    kloc_data[4] = 36   # low_key
    kloc_data[5] = 96   # high_key
    kloc_chunk = make_chunk(b'kloc', bytes(kloc_data))

    # amp env (18 bytes): attack=1, decay=3, release=4, sustain=7
    amp_env_data = bytearray(18)
    amp_env_data[1] = 10   # attack
    amp_env_data[3] = 50   # decay
    amp_env_data[4] = 30   # release
    amp_env_data[7] = 80   # sustain
    amp_env_chunk = make_chunk(b'env ', bytes(amp_env_data))

    # filter env (18 bytes)
    filt_env_data = bytearray(18)
    filt_env_data[1] = 5    # attack
    filt_env_data[3] = 60   # decay
    filt_env_data[4] = 40   # release
    filt_env_data[7] = 70   # sustain
    filt_env_data[9] = 50   # depth (i8, positive)
    filt_env_chunk = make_chunk(b'env ', bytes(filt_env_data))

    # aux env (18 bytes)
    aux_env_data = bytearray(18)
    aux_env_data[1] = 10   # rate_1
    aux_env_data[5] = 100  # level_1
    aux_env_chunk = make_chunk(b'env ', bytes(aux_env_data))

    # filt chunk (10 bytes)
    filt_data = bytearray(10)
    filt_data[1] = 0    # filter_type (2-pole LP)
    filt_data[2] = 75   # cutoff
    filt_data[3] = 6    # resonance (0-12 range)
    filt_chunk = make_chunk(b'filt', bytes(filt_data))

    # zone chunk (48 bytes)
    zone_data = bytearray(48)
    sample_name = b'Piano_C3'
    zone_data[1] = len(sample_name)
    zone_data[2:2 + len(sample_name)] = sample_name
    zone_data[34] = 1     # low_vel
    zone_data[35] = 127   # high_vel
    zone_data[40] = 4     # playback (AS SAMPLE)
    zone_data[43] = 1     # keyboard_track (ON)
    zone_chunk = make_chunk(b'zone', bytes(zone_data))

    # Assemble keygroup
    kgrp_inner = kloc_chunk + amp_env_chunk + filt_env_chunk + aux_env_chunk + filt_chunk + zone_chunk
    kgrp_chunk = make_chunk(b'kgrp', kgrp_inner)

    # Assemble RIFF/APRG file
    content = b'APRG' + prg_chunk + out_chunk + tune_chunk + lfo1_chunk + lfo2_chunk + mods_chunk + kgrp_chunk
    file_size = struct.pack('<I', len(content))
    akp_data = b'RIFF' + file_size + content

    with open('test_sample.akp', 'wb') as f:
        f.write(akp_data)

    print(f"Created test_sample.akp ({len(akp_data)} bytes)")


if __name__ == "__main__":
    create_test_akp()
```

- [ ] **Step 2: Generate test file and verify it parses**

```bash
cd /Users/davidryan/GitHub/rusty-samplers
python3 create_test_akp.py
./target/debug/rusty-samplers-cli test_sample.akp
cat test_sample.sfz
```

Expected: Parses without errors. SFZ shows sample=Piano_C3, correct key/vel ranges, filter, envelope data.

- [ ] **Step 3: Commit**

```bash
git add create_test_akp.py
git commit -m "test: update AKP generator with all spec-correct chunk types"
```

### Task 8: Final verification and cleanup

- [ ] **Step 1: Run complete test suite one final time**

```bash
cd /Users/davidryan/GitHub/rusty-samplers && cargo test 2>&1
```

Expected: All tests pass.

- [ ] **Step 2: Run clippy on both crates**

```bash
cargo clippy 2>&1
cd gui && cargo clippy 2>&1
```

Expected: Clean on both.

- [ ] **Step 3: Commit any remaining fixes**

Stage any modified files by name, then:
```bash
git commit -m "chore: final cleanup after spec-aligned parser rewrite"
```
