// Legacy AKP parser implementation
// Original manual parsing approach using byteorder

use crate::error::{ConversionError, Result};
use crate::formats::common::*;
use super::types::*;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::str;
use log::{debug, warn};

/// Parse an AKP file using the legacy manual parsing approach
pub fn parse_akp_file(mut file: File) -> Result<AkaiProgram> {
    debug!("Starting legacy AKP parsing");
    
    validate_riff_header(&mut file)?;
    
    let mut program = AkaiProgram::new();
    let file_len = file.metadata()?.len();
    parse_top_level_chunks(&mut file, file_len, &mut program)?;
    
    debug!("Legacy parsing complete: {}", program.stats());
    Ok(program)
}

/// Validate RIFF header and APRG format identifier
fn validate_riff_header(file: &mut File) -> Result<()> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    let header = str::from_utf8(&buf)?.trim_end_matches('\0');
    if header != "RIFF" {
        return Err(ConversionError::InvalidRiffHeader { 
            found: header.to_string() 
        });
    }
    
    file.seek(SeekFrom::Current(4))?; // Skip file size (always 0 in AKP)
    file.read_exact(&mut buf)?;
    let format = str::from_utf8(&buf)?.trim_end_matches('\0');
    if format != "APRG" {
        return Err(ConversionError::InvalidFormat { 
            found: format.to_string() 
        });
    }
    debug!("RIFF/APRG header validated");
    Ok(())
}

/// Parse top-level chunks in the AKP file
fn parse_top_level_chunks(file: &mut File, end_pos: u64, program: &mut AkaiProgram) -> Result<()> {
    let mut chunk_count = 0;
    
    while file.stream_position()? < end_pos {
        let header = read_chunk_header(file)?;
        chunk_count += 1;
        
        debug!("Processing chunk '{}' (size: {} bytes)", header.id, header.size);
        
        match header.id.as_str() {
            "prg " => {
                let mut chunk_data = vec![0; header.size as usize];
                file.read_exact(&mut chunk_data)?;
                program.header = Some(parse_program_header(&mut Cursor::new(chunk_data))?);
                debug!("Parsed program header");
            }
            "kgrp" => {
                let kgrp_end_pos = file.stream_position()? + header.size as u64;
                let keygroup = parse_keygroup(file, kgrp_end_pos)?;
                debug!("Parsed keygroup: {}", keygroup.description());
                program.add_keygroup(keygroup);
            }
            "out " => {
                // Skip output chunk for now
                file.seek(SeekFrom::Current(header.size as i64))?;
                debug!("Skipped output chunk");
            }
            _ => {
                warn!("Unknown chunk type '{}', skipping", header.id);
                file.seek(SeekFrom::Current(header.size as i64))?;
            }
        }
    }
    
    debug!("Processed {} top-level chunks", chunk_count);
    Ok(())
}

/// Parse a keygroup chunk containing nested subchunks
fn parse_keygroup(file: &mut File, end_pos: u64) -> Result<Keygroup> {
    let mut keygroup = Keygroup::default();
    let mut env_count = 0;
    let mut lfo_count = 0;
    let mut subchunk_count = 0;

    while file.stream_position()? < end_pos {
        let header = read_chunk_header(file)?;
        let mut chunk_data = vec![0; header.size as usize];
        file.read_exact(&mut chunk_data)?;
        let mut cursor = Cursor::new(chunk_data);
        subchunk_count += 1;

        match header.id.as_str() {
            "zone" => {
                parse_zone_chunk(&mut cursor, &mut keygroup)?;
                debug!("Parsed zone chunk");
            }
            "smpl" => {
                keygroup.sample = Some(parse_smpl_chunk(&mut cursor)?);
                debug!("Parsed sample chunk");
            }
            "tune" => {
                keygroup.tune = Some(parse_tune_chunk(&mut cursor)?);
                debug!("Parsed tune chunk");
            }
            "filt" => {
                keygroup.filter = Some(parse_filt_chunk(&mut cursor)?);
                debug!("Parsed filter chunk");
            }
            "env " => {
                let envelope = parse_env_chunk(&mut cursor)?;
                match env_count {
                    0 => keygroup.amp_env = Some(envelope),
                    1 => keygroup.filter_env = Some(envelope),
                    2 => keygroup.aux_env = Some(envelope),
                    _ => warn!("Extra envelope chunk found (count: {})", env_count),
                }
                env_count += 1;
                debug!("Parsed envelope chunk #{}", env_count);
            }
            "lfo " => {
                let lfo = parse_lfo_chunk(&mut cursor)?;
                match lfo_count {
                    0 => keygroup.lfo1 = Some(lfo),
                    1 => keygroup.lfo2 = Some(lfo),
                    _ => warn!("Extra LFO chunk found (count: {})", lfo_count),
                }
                lfo_count += 1;
                debug!("Parsed LFO chunk #{}", lfo_count);
            }
            _ => {
                warn!("Unknown keygroup subchunk '{}', skipping", header.id);
            }
        }
    }
    
    debug!("Keygroup parsing complete: {} subchunks", subchunk_count);
    Ok(keygroup)
}

/// Read a RIFF chunk header
fn read_chunk_header(file: &mut File) -> Result<RiffChunkHeader> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    let id = str::from_utf8(&buf).unwrap_or("????").trim_end_matches('\0').to_string();
    let size = file.read_u32::<LittleEndian>()?;
    Ok(RiffChunkHeader { id, size })
}

/// Parse program header chunk
fn parse_program_header(cursor: &mut Cursor<Vec<u8>>) -> Result<ProgramHeader> {
    cursor.seek(SeekFrom::Start(1))?; // Skip reserved byte
    let midi_program_number = cursor.read_u8()?;
    let number_of_keygroups = cursor.read_u8()?;
    Ok(ProgramHeader { midi_program_number, number_of_keygroups })
}

/// Parse zone chunk (key and velocity ranges)
fn parse_zone_chunk(cursor: &mut Cursor<Vec<u8>>, keygroup: &mut Keygroup) -> Result<()> {
    cursor.seek(SeekFrom::Start(1))?; // Skip reserved byte
    keygroup.zone.low_key = cursor.read_u8()?;
    keygroup.zone.high_key = cursor.read_u8()?;
    keygroup.zone.low_vel = cursor.read_u8()?;
    keygroup.zone.high_vel = cursor.read_u8()?;
    Ok(())
}

/// Parse sample chunk (filename)
fn parse_smpl_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Sample> {
    cursor.seek(SeekFrom::Start(2))?; // Skip reserved bytes
    let mut buffer = Vec::new();
    cursor.read_to_end(&mut buffer)?;
    let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    let filename = String::from_utf8_lossy(&buffer[..end]).to_string();
    Ok(Sample { filename })
}

/// Parse tune chunk (level, semitone, fine tune)
fn parse_tune_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Tune> {
    cursor.seek(SeekFrom::Start(2))?; // Skip reserved bytes
    let level = cursor.read_u8()?;
    let semitone = cursor.read_i8()?;
    let fine_tune = cursor.read_i8()?;
    Ok(Tune { level, semitone, fine_tune })
}

/// Parse filter chunk (cutoff, resonance, type)
fn parse_filt_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Filter> {
    cursor.seek(SeekFrom::Start(2))?; // Skip reserved bytes
    let cutoff = cursor.read_u8()?;
    let resonance = cursor.read_u8()?;
    cursor.seek(SeekFrom::Start(7))?; // Skip to filter type
    let filter_type = cursor.read_u8()?;
    Ok(Filter { cutoff, resonance, filter_type })
}

/// Parse envelope chunk (ADSR)
fn parse_env_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Envelope> {
    cursor.seek(SeekFrom::Start(2))?; // Skip reserved bytes
    let attack = cursor.read_u8()?;
    let decay = cursor.read_u8()?;
    let sustain = cursor.read_u8()?;
    let release = cursor.read_u8()?;
    Ok(Envelope { attack, decay, sustain, release })
}

/// Parse LFO chunk (waveform, rate, delay, depth)
fn parse_lfo_chunk(cursor: &mut Cursor<Vec<u8>>) -> Result<Lfo> {
    cursor.seek(SeekFrom::Start(5))?; // Skip to waveform
    let waveform = cursor.read_u8()?;
    let rate = cursor.read_u8()?;
    let delay = cursor.read_u8()?;
    let depth = cursor.read_u8()?;
    Ok(Lfo { waveform, rate, delay, depth })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_header_parsing() {
        let data = b"test\x08\x00\x00\x00";
        let mut file = Cursor::new(data.to_vec());
        
        // We need to mock File behavior, but this is complex
        // This test would need refactoring to work with Cursor instead of File
    }

    #[test]
    fn test_envelope_parsing() {
        let data = vec![0, 0, 10, 20, 30, 40]; // Skip first 2 bytes, then ADSR values
        let mut cursor = Cursor::new(data);
        let envelope = parse_env_chunk(&mut cursor).unwrap();
        
        assert_eq!(envelope.attack, 10);
        assert_eq!(envelope.decay, 20);
        assert_eq!(envelope.sustain, 30);
        assert_eq!(envelope.release, 40);
    }

    #[test]
    fn test_tune_parsing() {
        let data = vec![0, 0, 75, 12, (-5i8) as u8]; // Skip 2, level 75, semitone 12, fine -5
        let mut cursor = Cursor::new(data);
        let tune = parse_tune_chunk(&mut cursor).unwrap();
        
        assert_eq!(tune.level, 75);
        assert_eq!(tune.semitone, 12);
        assert_eq!(tune.fine_tune, -5);
    }
}