// Legacy AKP parser implementation
// Original manual parsing approach using byteorder

use crate::error::{ConversionError, Result, RecoveryContext, recovery};
use crate::formats::common::*;
use super::types::*;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::str;
use log::{debug, warn};

/// Parse an AKP file using the legacy manual parsing approach with recovery
pub fn parse_akp_file(mut file: File) -> Result<AkaiProgram> {
    debug!("Starting legacy AKP parsing with recovery");
    
    let mut recovery_context = RecoveryContext::new();
    
    validate_riff_header(&mut file)?;
    
    let mut program = AkaiProgram::new();
    let file_len = file.metadata()?.len();
    parse_top_level_chunks_with_recovery(&mut file, file_len, &mut program, &mut recovery_context)?;
    
    if recovery_context.chunks_recovered > 0 || recovery_context.chunks_skipped > 0 {
        warn!("Recovery statistics: {}", recovery_context.stats());
        for warning in &recovery_context.warnings {
            warn!("{}", warning);
        }
    }
    
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

/// Parse top-level chunks in the AKP file with malformed file recovery
fn parse_top_level_chunks_with_recovery(
    file: &mut File, 
    end_pos: u64, 
    program: &mut AkaiProgram, 
    recovery_context: &mut RecoveryContext
) -> Result<()> {
    let mut chunk_count = 0;
    
    while file.stream_position()? < end_pos {
        recovery_context.file_position = file.stream_position()?;
        
        let header = match read_chunk_header(file) {
            Ok(h) => h,
            Err(e) => {
                warn!("Malformed chunk header at offset {}: {}", recovery_context.file_position, e);
                
                // Attempt recovery by finding next valid chunk
                match recovery::skip_to_next_chunk(file, recovery_context.file_position, recovery_context) {
                    Ok(Some(_pos)) => continue, // Found recovery point, continue parsing
                    Ok(None) => break, // No recovery possible, stop parsing
                    Err(_) => return Err(e), // Recovery failed with error
                }
            }
        };
        
        chunk_count += 1;
        recovery_context.total_chunks_processed += 1;
        
        debug!("Processing chunk '{}' (size: {} bytes)", header.id, header.size);
        
        let parse_result = match header.id.as_str() {
            "prg " => parse_program_chunk(file, &header, program),
            "kgrp" => parse_keygroup_chunk_with_recovery(file, &header, program, recovery_context),
            "out " => {
                // Skip output chunk for now
                file.seek(SeekFrom::Current(header.size as i64))?;
                debug!("Skipped output chunk");
                Ok(())
            }
            _ => {
                warn!("Unknown chunk type '{}', skipping", header.id);
                file.seek(SeekFrom::Current(header.size as i64))?;
                recovery_context.record_skip(&header.id, "unknown chunk type");
                Ok(())
            }
        };
        
        // Handle chunk parsing errors with recovery
        if let Err(e) = parse_result {
            if e.is_recoverable() {
                warn!("Recoverable error in chunk '{}': {}", header.id, e);
                recovery_context.record_recovery(&header.id, &e.to_string());
                
                // Skip malformed chunk and continue
                let skip_bytes = header.size as i64;
                if let Err(seek_err) = file.seek(SeekFrom::Current(skip_bytes)) {
                    warn!("Failed to skip malformed chunk: {}", seek_err);
                    break;
                }
            } else {
                return Err(e); // Fatal error, stop parsing
            }
        }
    }
    
    debug!("Processed {} top-level chunks with recovery", chunk_count);
    Ok(())
}

/// Parse program chunk with error handling
fn parse_program_chunk(file: &mut File, header: &RiffChunkHeader, program: &mut AkaiProgram) -> Result<()> {
    let mut chunk_data = vec![0; header.size as usize];
    file.read_exact(&mut chunk_data)?;
    program.header = Some(parse_program_header(&mut Cursor::new(chunk_data))?);
    debug!("Parsed program header");
    Ok(())
}

/// Parse keygroup chunk with enhanced recovery
fn parse_keygroup_chunk_with_recovery(
    file: &mut File, 
    header: &RiffChunkHeader, 
    program: &mut AkaiProgram, 
    recovery_context: &mut RecoveryContext
) -> Result<()> {
    let kgrp_end_pos = file.stream_position()? + header.size as u64;
    
    match parse_keygroup_with_recovery(file, kgrp_end_pos, recovery_context) {
        Ok(keygroup) => {
            debug!("Parsed keygroup: {}", keygroup.description());
            program.add_keygroup(keygroup);
            Ok(())
        }
        Err(e) if e.is_recoverable() => {
            warn!("Malformed keygroup, attempting recovery: {}", e);
            
            // Create keygroup with default values
            let recovered_keygroup = recovery::recover_keygroup_with_partial_data(&[], recovery_context);
            program.add_keygroup(recovered_keygroup);
            
            // Skip to end of malformed keygroup chunk
            file.seek(SeekFrom::Start(kgrp_end_pos))?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Parse a keygroup chunk with recovery capabilities
fn parse_keygroup_with_recovery(file: &mut File, end_pos: u64, recovery_context: &mut RecoveryContext) -> Result<Keygroup> {
    let mut keygroup = Keygroup::default();
    let mut env_count = 0;
    let mut lfo_count = 0;
    let mut subchunk_count = 0;
    let mut available_chunks = Vec::new();

    while file.stream_position()? < end_pos {
        let header = match read_chunk_header(file) {
            Ok(h) => h,
            Err(e) => {
                warn!("Malformed subchunk header in keygroup: {}", e);
                
                // Try recovery within keygroup
                let current_pos = file.stream_position()?;
                match recovery::skip_to_next_chunk(file, current_pos, recovery_context) {
                    Ok(Some(_pos)) => continue,
                    Ok(None) => break, // No more valid chunks
                    Err(_) => return Err(ConversionError::recoverable_chunk_error("kgrp", file.stream_position()?, "malformed subchunk")),
                }
            }
        };
        
        let mut chunk_data = vec![0; header.size as usize];
        if let Err(e) = file.read_exact(&mut chunk_data) {
            warn!("Failed to read subchunk '{}' data: {}", header.id, e);
            recovery_context.record_skip(&header.id, "failed to read chunk data");
            continue;
        }
        
        subchunk_count += 1;
        available_chunks.push(header.id.clone());
        
        let parse_result = match header.id.as_str() {
            "zone" => {
                parse_zone_chunk(&mut Cursor::new(chunk_data), &mut keygroup)
            }
            "smpl" => {
                match parse_smpl_chunk(&mut Cursor::new(chunk_data)) {
                    Ok(sample) => {
                        keygroup.sample = Some(sample);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            "tune" => {
                match parse_tune_chunk(&mut Cursor::new(chunk_data)) {
                    Ok(tune) => {
                        keygroup.tune = Some(tune);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            "filt" => {
                match parse_filt_chunk(&mut Cursor::new(chunk_data)) {
                    Ok(filter) => {
                        keygroup.filter = Some(filter);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            "env " => {
                match parse_env_chunk(&mut Cursor::new(chunk_data)) {
                    Ok(envelope) => {
                        match env_count {
                            0 => keygroup.amp_env = Some(envelope),
                            1 => keygroup.filter_env = Some(envelope),
                            2 => keygroup.aux_env = Some(envelope),
                            _ => {} // Ignore extra envelopes
                        }
                        env_count += 1;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            "lfo " => {
                match parse_lfo_chunk(&mut Cursor::new(chunk_data)) {
                    Ok(lfo) => {
                        match lfo_count {
                            0 => keygroup.lfo1 = Some(lfo),
                            1 => keygroup.lfo2 = Some(lfo),
                            _ => {} // Ignore extra LFOs
                        }
                        lfo_count += 1;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => {
                warn!("Unknown subchunk type '{}' in keygroup", header.id);
                recovery_context.record_skip(&header.id, "unknown subchunk type");
                continue;
            }
        };
        
        if let Err(e) = parse_result {
            warn!("Failed to parse subchunk '{}': {}", header.id, e);
            recovery_context.record_skip(&header.id, &e.to_string());
        }
    }
    
    // Apply recovery strategies for missing chunks
    let chunk_refs: Vec<&str> = available_chunks.iter().map(|s| s.as_str()).collect();
    if chunk_refs.is_empty() || !chunk_refs.contains(&"zone") || !chunk_refs.contains(&"smpl") {
        let recovered_data = recovery::recover_keygroup_with_partial_data(&chunk_refs, recovery_context);
        
        // Merge recovered data with what we parsed
        if keygroup.zone.low_key == 0 && keygroup.zone.high_key == 0 {
            keygroup.zone = recovered_data.zone;
        }
        if keygroup.sample.is_none() {
            keygroup.sample = recovered_data.sample;
        }
    }
    
    debug!("Parsed keygroup with {} subchunks", subchunk_count);
    Ok(keygroup)
}

/// Parse a keygroup chunk containing nested subchunks (legacy function for compatibility)
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

/// Parse an AKP file from any reader that implements Read + Seek
/// This is a bridge function for the plugin system
pub fn parse_akp_reader<R: Read + Seek>(reader: &mut R) -> Result<AkaiProgram> {
    // Convert the generic reader to a temporary file-like interface
    // This is a temporary solution until we refactor the legacy parser
    
    // Read all data into memory first
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).map_err(ConversionError::Io)?;
    
    // Create a temporary file from the buffer
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("temp_akp_{}.akp", std::process::id()));
    
    std::fs::write(&temp_path, &buffer).map_err(ConversionError::Io)?;
    
    // Parse using the existing file-based parser
    let file = File::open(&temp_path).map_err(ConversionError::Io)?;
    let result = parse_akp_file(file);
    
    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_path);
    
    result
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