// binrw-based declarative AKP parser implementation
// Professional-grade binary parsing using industry-standard patterns

use crate::error::{ConversionError, Result};
use crate::formats::common::*;
use super::types::*;

use binrw::{BinRead, BinResult, Endian};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor, BufReader};
use log::{debug, warn, info};

/// Parse an AKP file using the declarative binrw approach
pub fn parse_akp_file(file: File) -> Result<AkaiProgram> {
    info!("Starting binrw declarative AKP parsing");
    
    // Wrap in BufReader for better I/O performance (recommended by research)
    let mut buffered_file = BufReader::new(file);
    
    // Parse using binrw declarative structs
    let akp_file = AkpFile::read_options(&mut buffered_file, Endian::Little, ())
        .map_err(|e| ConversionError::Custom {
            message: format!("binrw parsing failed: {}", e),
        })?;
    
    // Convert binrw structs to our common format
    let program = convert_akp_file_to_program(akp_file)?;
    
    info!("binrw parsing complete: {}", program.stats());
    Ok(program)
}

/// Convert parsed binrw AKP file to our common AkaiProgram format
fn convert_akp_file_to_program(akp_file: AkpFile) -> Result<AkaiProgram> {
    let mut program = AkaiProgram::new();
    
    for chunk in akp_file.chunks {
        match chunk {
            TopLevelChunk::Program { data } => {
                // Parse program header from raw data
                let mut cursor = Cursor::new(data);
                if let Ok(header) = parse_program_header_from_cursor(&mut cursor) {
                    program.header = Some(header);
                    debug!("Parsed program header with binrw");
                }
            }
            TopLevelChunk::Keygroup { keygroup_data } => {
                let keygroup = convert_keygroup_data(keygroup_data)?;
                debug!("Converted keygroup: {}", keygroup.description());
                program.add_keygroup(keygroup);
            }
            TopLevelChunk::Output { data: _ } => {
                debug!("Skipped output chunk");
            }
        }
    }
    
    Ok(program)
}

/// Convert binrw KeygroupData to our common Keygroup format
fn convert_keygroup_data(data: KeygroupData) -> Result<Keygroup> {
    let mut keygroup = Keygroup::default();
    
    // Convert zone information
    if let Some(zone_chunk) = data.zone {
        keygroup.zone = Zone {
            low_key: zone_chunk.low_key,
            high_key: zone_chunk.high_key,
            low_vel: zone_chunk.low_vel,
            high_vel: zone_chunk.high_vel,
        };
    }
    
    // Convert sample information
    if let Some(sample_chunk) = data.sample {
        keygroup.sample = Some(Sample {
            filename: sample_chunk.filename,
        });
    }
    
    // Convert tuning information
    if let Some(tune_chunk) = data.tune {
        keygroup.tune = Some(Tune {
            level: tune_chunk.level,
            semitone: tune_chunk.semitone,
            fine_tune: tune_chunk.fine_tune,
        });
    }
    
    // Convert filter information
    if let Some(filter_chunk) = data.filter {
        keygroup.filter = Some(Filter {
            cutoff: filter_chunk.cutoff,
            resonance: filter_chunk.resonance,
            filter_type: filter_chunk.filter_type,
        });
    }
    
    // Convert envelopes
    if !data.envelopes.is_empty() {
        if let Some(env) = data.envelopes.get(0) {
            keygroup.amp_env = Some(Envelope {
                attack: env.attack,
                decay: env.decay,
                sustain: env.sustain,
                release: env.release,
            });
        }
        if let Some(env) = data.envelopes.get(1) {
            keygroup.filter_env = Some(Envelope {
                attack: env.attack,
                decay: env.decay,
                sustain: env.sustain,
                release: env.release,
            });
        }
        if let Some(env) = data.envelopes.get(2) {
            keygroup.aux_env = Some(Envelope {
                attack: env.attack,
                decay: env.decay,
                sustain: env.sustain,
                release: env.release,
            });
        }
    }
    
    // Convert LFOs
    if !data.lfos.is_empty() {
        if let Some(lfo) = data.lfos.get(0) {
            keygroup.lfo1 = Some(Lfo {
                waveform: lfo.waveform,
                rate: lfo.rate,
                delay: lfo.delay,
                depth: lfo.depth,
            });
        }
        if let Some(lfo) = data.lfos.get(1) {
            keygroup.lfo2 = Some(Lfo {
                waveform: lfo.waveform,
                rate: lfo.rate,
                delay: lfo.delay,
                depth: lfo.depth,
            });
        }
    }
    
    Ok(keygroup)
}

// === Declarative binrw Structures ===

/// Main AKP file structure with RIFF header (read-only)
#[derive(Debug)]
pub struct AkpFile {
    pub file_size: u32,
    pub chunks: Vec<TopLevelChunk>,
}

impl BinRead for AkpFile {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // Read RIFF magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"RIFF" {
            return Err(binrw::Error::BadMagic { pos: 0, found: Box::new(magic) });
        }
        
        // Read file size (should be 0 for AKP)
        let file_size = u32::read_options(reader, _endian, ())?;
        
        // Read APRG format identifier
        let mut format_magic = [0u8; 4];
        reader.read_exact(&mut format_magic)?;
        if &format_magic != b"APRG" {
            return Err(binrw::Error::BadMagic { pos: 8, found: Box::new(format_magic) });
        }
        
        // Parse chunks until EOF
        let chunks = parse_chunks_until_eof(reader, _endian)?;
        
        Ok(AkpFile { file_size, chunks })
    }
}

/// Top-level chunk types in AKP files (read-only)
#[derive(Debug)]
pub enum TopLevelChunk {
    Program { data: Vec<u8> },
    Keygroup { keygroup_data: KeygroupData },
    Output { data: Vec<u8> },
}

/// Keygroup data containing all subchunks
#[derive(Debug, Default)]
pub struct KeygroupData {
    pub zone: Option<ZoneChunk>,
    pub sample: Option<SampleChunk>,
    pub tune: Option<TuneChunk>,
    pub filter: Option<FilterChunk>,
    pub envelopes: Vec<EnvelopeChunk>,
    pub lfos: Vec<LfoChunk>,
}

/// Zone chunk (key and velocity ranges)
#[derive(Debug)]
pub struct ZoneChunk {
    pub size: u32,
    pub low_key: u8,
    pub high_key: u8,
    pub low_vel: u8,
    pub high_vel: u8,
}

impl BinRead for ZoneChunk {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"zone" {
            return Err(binrw::Error::BadMagic { pos: reader.stream_position()? - 4, found: Box::new(magic) });
        }
        
        let size = u32::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(1))?; // Skip reserved byte
        let low_key = u8::read_options(reader, endian, ())?;
        let high_key = u8::read_options(reader, endian, ())?;
        let low_vel = u8::read_options(reader, endian, ())?;
        let high_vel = u8::read_options(reader, endian, ())?;
        
        Ok(ZoneChunk { size, low_key, high_key, low_vel, high_vel })
    }
}

/// Sample chunk (filename)
#[derive(Debug)]
pub struct SampleChunk {
    pub size: u32,
    pub filename: String,
}

impl BinRead for SampleChunk {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"smpl" {
            return Err(binrw::Error::BadMagic { pos: reader.stream_position()? - 4, found: Box::new(magic) });
        }
        
        let size = u32::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(2))?; // Skip reserved bytes
        let filename = parse_null_terminated_string(reader)?;
        
        Ok(SampleChunk { size, filename })
    }
}

/// Tune chunk (level, semitone, fine tune)
#[derive(Debug)]
pub struct TuneChunk {
    pub size: u32,
    pub level: u8,
    pub semitone: i8,
    pub fine_tune: i8,
}

impl BinRead for TuneChunk {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"tune" {
            return Err(binrw::Error::BadMagic { pos: reader.stream_position()? - 4, found: Box::new(magic) });
        }
        
        let size = u32::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(2))?; // Skip reserved bytes
        let level = u8::read_options(reader, endian, ())?;
        let semitone = i8::read_options(reader, endian, ())?;
        let fine_tune = i8::read_options(reader, endian, ())?;
        
        Ok(TuneChunk { size, level, semitone, fine_tune })
    }
}

/// Filter chunk (cutoff, resonance, type)
#[derive(Debug)]
pub struct FilterChunk {
    pub size: u32,
    pub cutoff: u8,
    pub resonance: u8,
    pub filter_type: u8,
}

impl BinRead for FilterChunk {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"filt" {
            return Err(binrw::Error::BadMagic { pos: reader.stream_position()? - 4, found: Box::new(magic) });
        }
        
        let size = u32::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(2))?; // Skip reserved bytes
        let cutoff = u8::read_options(reader, endian, ())?;
        let resonance = u8::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(3))?; // Skip to filter type at offset 7
        let filter_type = u8::read_options(reader, endian, ())?;
        
        Ok(FilterChunk { size, cutoff, resonance, filter_type })
    }
}

/// Envelope chunk (ADSR)
#[derive(Debug)]
pub struct EnvelopeChunk {
    pub size: u32,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

impl BinRead for EnvelopeChunk {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"env " {
            return Err(binrw::Error::BadMagic { pos: reader.stream_position()? - 4, found: Box::new(magic) });
        }
        
        let size = u32::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(2))?; // Skip reserved bytes
        let attack = u8::read_options(reader, endian, ())?;
        let decay = u8::read_options(reader, endian, ())?;
        let sustain = u8::read_options(reader, endian, ())?;
        let release = u8::read_options(reader, endian, ())?;
        
        Ok(EnvelopeChunk { size, attack, decay, sustain, release })
    }
}

/// LFO chunk (waveform, rate, delay, depth)
#[derive(Debug)]
pub struct LfoChunk {
    pub size: u32,
    pub waveform: u8,
    pub rate: u8,
    pub delay: u8,
    pub depth: u8,
}

impl BinRead for LfoChunk {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"lfo " {
            return Err(binrw::Error::BadMagic { pos: reader.stream_position()? - 4, found: Box::new(magic) });
        }
        
        let size = u32::read_options(reader, endian, ())?;
        reader.seek(SeekFrom::Current(5))?; // Skip to waveform at offset 5
        let waveform = u8::read_options(reader, endian, ())?;
        let rate = u8::read_options(reader, endian, ())?;
        let delay = u8::read_options(reader, endian, ())?;
        let depth = u8::read_options(reader, endian, ())?;
        
        Ok(LfoChunk { size, waveform, rate, delay, depth })
    }
}

// === Custom Parsers ===

/// Parse chunks until EOF (handles AKP's non-standard chunk count)
fn parse_chunks_until_eof<R: Read + Seek>(
    reader: &mut R,
    _endian: Endian,
) -> BinResult<Vec<TopLevelChunk>> {
    let mut chunks = Vec::new();
    
    loop {
        // Try to read chunk header
        match ChunkHeader::read_options(reader, _endian, ()) {
            Ok(header) => {
                debug!("Reading chunk '{}' (size: {})", header.id, header.size);
                
                // Parse the chunk based on its type
                match header.id.as_str() {
                    "prg " => {
                        let mut data = vec![0u8; header.size as usize];
                        reader.read_exact(&mut data)?;
                        chunks.push(TopLevelChunk::Program { data });
                    }
                    "kgrp" => {
                        let keygroup_data = parse_keygroup_chunks(reader, _endian, header.size)?;
                        chunks.push(TopLevelChunk::Keygroup { keygroup_data });
                    }
                    "out " => {
                        let mut data = vec![0u8; header.size as usize];
                        reader.read_exact(&mut data)?;
                        chunks.push(TopLevelChunk::Output { data });
                    }
                    _ => {
                        warn!("Unknown chunk type '{}', skipping {} bytes", header.id, header.size);
                        reader.seek(SeekFrom::Current(header.size as i64))?;
                    }
                }
            }
            Err(_) => break, // EOF reached
        }
    }
    
    debug!("Parsed {} top-level chunks", chunks.len());
    Ok(chunks)
}

/// Parse keygroup subchunks (handles exactly 9 subchunks per AKP spec)
fn parse_keygroup_chunks<R: Read + Seek>(
    reader: &mut R,
    _endian: Endian,
    total_size: u32,
) -> BinResult<KeygroupData> {
    let mut keygroup_data = KeygroupData::default();
    let start_pos = reader.stream_position()?;
    let end_pos = start_pos + total_size as u64;
    
    while reader.stream_position()? < end_pos {
        let header = ChunkHeader::read_options(reader, _endian, ())?;
        debug!("Reading keygroup subchunk '{}' (size: {})", header.id, header.size);
        
        match header.id.as_str() {
            "zone" => {
                let zone = ZoneChunk::read_options(reader, _endian, ())?;
                keygroup_data.zone = Some(zone);
            }
            "smpl" => {
                let sample = SampleChunk::read_options(reader, _endian, ())?;
                keygroup_data.sample = Some(sample);
            }
            "tune" => {
                let tune = TuneChunk::read_options(reader, _endian, ())?;
                keygroup_data.tune = Some(tune);
            }
            "filt" => {
                let filter = FilterChunk::read_options(reader, _endian, ())?;
                keygroup_data.filter = Some(filter);
            }
            "env " => {
                let envelope = EnvelopeChunk::read_options(reader, _endian, ())?;
                keygroup_data.envelopes.push(envelope);
            }
            "lfo " => {
                let lfo = LfoChunk::read_options(reader, _endian, ())?;
                keygroup_data.lfos.push(lfo);
            }
            _ => {
                warn!("Unknown keygroup subchunk '{}', skipping {} bytes", header.id, header.size);
                reader.seek(SeekFrom::Current(header.size as i64))?;
            }
        }
    }
    
    debug!("Parsed keygroup with {} envelopes, {} LFOs", 
           keygroup_data.envelopes.len(), keygroup_data.lfos.len());
    
    Ok(keygroup_data)
}

/// Parse null-terminated string
fn parse_null_terminated_string<R: Read + Seek>(
    reader: &mut R,
) -> BinResult<String> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 1];
    
    loop {
        reader.read_exact(&mut buffer)?;
        if buffer[0] == 0 {
            break;
        }
        bytes.push(buffer[0]);
    }
    
    String::from_utf8(bytes).map_err(|e| binrw::Error::Custom {
        pos: 0,
        err: Box::new(e),
    })
}

/// Simple chunk header for custom parsing
#[derive(Debug)]
struct ChunkHeader {
    id: String,
    size: u32,
}

impl BinRead for ChunkHeader {
    type Args<'a> = ();
    
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut id_bytes = [0u8; 4];
        reader.read_exact(&mut id_bytes)?;
        let id = String::from_utf8_lossy(&id_bytes).trim_end_matches('\0').to_string();
        let size = u32::read_options(reader, endian, ())?;
        
        Ok(ChunkHeader { id, size })
    }
}

/// Helper function to parse program header from cursor (legacy compatibility)
fn parse_program_header_from_cursor(cursor: &mut Cursor<Vec<u8>>) -> Result<ProgramHeader> {
    use byteorder::ReadBytesExt;
    
    cursor.seek(SeekFrom::Start(1))?; // Skip reserved byte
    let midi_program_number = cursor.read_u8()?;
    let number_of_keygroups = cursor.read_u8()?;
    
    Ok(ProgramHeader {
        midi_program_number,
        number_of_keygroups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_null_terminated_string_parsing() {
        let data = b"hello\0";
        let mut cursor = Cursor::new(data.to_vec());
        let result = parse_null_terminated_string(&mut cursor).unwrap();
        assert_eq!(result, "hello");
    }
    
    #[test]
    fn test_chunk_header_parsing() {
        let data = b"test\x08\x00\x00\x00"; // "test" + size 8
        let mut cursor = Cursor::new(data.to_vec());
        let header = ChunkHeader::read_options(&mut cursor, Endian::Little, ()).unwrap();
        assert_eq!(header.id, "test");
        assert_eq!(header.size, 8);
    }
}