use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::str;
use indicatif::ProgressBar;

use crate::error::{AkpError, Result};
use crate::types::*;

/// Maximum allowed size for a single chunk (64 MB).
const MAX_CHUNK_SIZE: u32 = 64 * 1024 * 1024;

/// Maximum number of keygroups per program.
const MAX_KEYGROUPS: usize = 1000;

/// Maximum number of modulation entries per keygroup.
const MAX_MODS_PER_KEYGROUP: usize = 256;

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
                file.read_exact(&mut chunk_data)
                    .map_err(|_| AkpError::CorruptedChunk("prg".to_string(), "Failed to read chunk data".to_string()))?;
                program.header = Some(parse_program_header(&mut Cursor::new(chunk_data))?);
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
    let mut env_count = 0;
    let mut lfo_count = 0;

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
                    _ => {}
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
                    _ => {}
                }
                lfo_count += 1;
            }
            "mods" => {
                if header.size < 4 {
                    return Err(AkpError::InvalidChunkSize("mods".to_string(), header.size));
                }
                if keygroup.mods.len() >= MAX_MODS_PER_KEYGROUP {
                    return Err(AkpError::CorruptedChunk(
                        "mods".to_string(),
                        format!("Exceeded maximum of {MAX_MODS_PER_KEYGROUP} modulations per keygroup"),
                    ));
                }
                keygroup.mods.push(parse_mods_chunk(&mut cursor)?);
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

pub fn parse_zone_chunk(cursor: &mut Cursor<Vec<u8>>, keygroup: &mut Keygroup) -> Result<()> {
    cursor.seek(SeekFrom::Start(1))?;
    keygroup.low_key = cursor.read_u8()?;
    keygroup.high_key = cursor.read_u8()?;
    keygroup.low_vel = cursor.read_u8()?;
    keygroup.high_vel = cursor.read_u8()?;

    if keygroup.low_key > keygroup.high_key {
        return Err(AkpError::InvalidKeyRange(keygroup.low_key, keygroup.high_key));
    }

    if keygroup.low_vel > keygroup.high_vel {
        return Err(AkpError::InvalidVelocityRange(keygroup.low_vel, keygroup.high_vel));
    }

    if keygroup.high_key > 127 {
        return Err(AkpError::InvalidParameterValue("high_key".to_string(), keygroup.high_key));
    }
    if keygroup.high_vel > 127 {
        return Err(AkpError::InvalidParameterValue("high_vel".to_string(), keygroup.high_vel));
    }

    Ok(())
}

pub fn parse_smpl_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Sample> {
    cursor.seek(SeekFrom::Start(2))?;
    let mut buffer = Vec::new();
    cursor.read_to_end(&mut buffer)?;
    let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    let raw_filename = String::from_utf8_lossy(&buffer[..end]).to_string();

    if raw_filename.is_empty() {
        return Err(AkpError::CorruptedChunk("smpl".to_string(), "Empty sample filename".to_string()));
    }

    let filename = sanitize_sample_path(&raw_filename);

    if filename.is_empty() {
        return Err(AkpError::CorruptedChunk("smpl".to_string(), "Sample filename is empty after sanitization".to_string()));
    }

    Ok(Sample { filename })
}

/// Sanitize a sample path from an AKP file.
/// Normalizes separators, removes `..` components and leading `/` or drive letters,
/// but preserves legitimate subdirectory structure (e.g., `Strings/Violin_C3.wav`).
fn sanitize_sample_path(raw: &str) -> String {
    // Normalize backslashes to forward slashes
    let normalized = raw.replace('\\', "/");

    // Strip drive letter prefix (e.g., "C:/")
    let without_drive = if normalized.len() >= 3
        && normalized.as_bytes()[0].is_ascii_alphabetic()
        && &normalized[1..3] == ":/"
    {
        &normalized[3..]
    } else {
        &normalized
    };

    // Filter out dangerous components, preserve legitimate subdirs
    let clean: Vec<&str> = without_drive
        .split('/')
        .filter(|component| {
            !component.is_empty() && *component != "." && *component != ".."
        })
        .collect();

    clean.join("/")
}

pub fn parse_tune_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Tune> {
    cursor.seek(SeekFrom::Start(2))?;
    let level = cursor.read_u8()?;
    let semitone = cursor.read_i8()?;
    let fine_tune = cursor.read_i8()?;
    Ok(Tune { level, semitone, fine_tune })
}

pub fn parse_filt_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Filter> {
    cursor.seek(SeekFrom::Start(2))?;
    let cutoff = cursor.read_u8()?;
    let resonance = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(7))?;
    let filter_type = cursor.read_u8()?;

    if filter_type > 3 {
        return Err(AkpError::InvalidParameterValue("filter_type".to_string(), filter_type));
    }

    Ok(Filter { cutoff, resonance, filter_type })
}

pub fn parse_env_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Envelope> {
    cursor.seek(SeekFrom::Start(2))?;
    let attack = cursor.read_u8()?;
    let decay = cursor.read_u8()?;
    let sustain = cursor.read_u8()?;
    let release = cursor.read_u8()?;
    Ok(Envelope { attack, decay, sustain, release })
}

pub fn parse_lfo_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Lfo> {
    cursor.seek(SeekFrom::Start(5))?;
    let waveform = cursor.read_u8()?;
    let rate = cursor.read_u8()?;
    let delay = cursor.read_u8()?;
    let depth = cursor.read_u8()?;
    Ok(Lfo { waveform, rate, delay, depth })
}

pub fn parse_mods_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Modulation> {
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
        let data = vec![0, 60, 72, 64, 127];
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
        let data = vec![0, 72, 60, 64, 127];
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();

        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidKeyRange(72, 60))));
    }

    #[test]
    fn test_parse_zone_chunk_invalid_velocity_range() {
        let data = vec![0, 60, 72, 127, 64];
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();

        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidVelocityRange(127, 64))));
    }

    #[test]
    fn test_parse_zone_chunk_invalid_key_value() {
        let data = vec![0, 60, 128, 64, 127];
        let mut cursor = Cursor::new(data);
        let mut keygroup = Keygroup::default();

        let result = parse_zone_chunk(&mut cursor, &mut keygroup);
        assert!(matches!(result, Err(AkpError::InvalidParameterValue(_, 128))));
    }

    #[test]
    fn test_parse_smpl_chunk_valid() {
        let mut data = vec![0, 0];
        data.extend_from_slice(b"test_sample.wav");
        data.push(0);
        let mut cursor = Cursor::new(data);

        let result = parse_smpl_chunk(&mut cursor);
        assert!(result.is_ok());
        let sample = result.unwrap();
        assert_eq!(sample.filename, "test_sample.wav");
    }

    #[test]
    fn test_parse_smpl_chunk_empty_filename() {
        let data = vec![0, 0, 0];
        let mut cursor = Cursor::new(data);

        let result = parse_smpl_chunk(&mut cursor);
        assert!(matches!(result, Err(AkpError::CorruptedChunk(_, _))));
    }

    #[test]
    fn test_parse_tune_chunk() {
        let data = vec![0, 0, 85, -12i8 as u8, 25];
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
        let data = vec![0, 0, 75, 25, 0, 0, 0, 2];
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
        let data = vec![0, 0, 75, 25, 0, 0, 0, 5];
        let mut cursor = Cursor::new(data);

        let result = parse_filt_chunk(&mut cursor);
        assert!(matches!(result, Err(AkpError::InvalidParameterValue(_, 5))));
    }

    #[test]
    fn test_parse_env_chunk() {
        let data = vec![0, 0, 10, 50, 80, 30];
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
        let data = vec![0, 1, 5, 75];
        let mut cursor = Cursor::new(data);

        let result = parse_mods_chunk(&mut cursor);
        assert!(result.is_ok());
        let modulation = result.unwrap();
        assert_eq!(modulation.source, 1);
        assert_eq!(modulation.destination, 5);
        assert_eq!(modulation.amount, 75);
    }
}
