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
    let raw_filename = String::from_utf8_lossy(&name_buf[..end]).trim().to_string();
    let mut sample_name = sanitize_sample_path(&raw_filename);

    if sample_name.is_empty() {
        return Ok(None);
    }

    // AKP stores sample names without file extension — append .WAV
    // Using uppercase to match Akai convention (S6000 factory WAVs are .WAV).
    // Lowercase .wav silently works on macOS but breaks on case-sensitive filesystems (Linux).
    let has_audio_ext = sample_name.rsplit('.').next()
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "wav" | "aif" | "aiff" | "flac" | "ogg" | "mp3"));
    if !has_audio_ext {
        sample_name.push_str(".WAV");
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
        assert_eq!(zone.sample_name, "ABCDEFGHIJKLMNOPQRST.WAV");
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
