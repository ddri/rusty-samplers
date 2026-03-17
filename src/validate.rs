//! Output validation for generated SFZ and DecentSampler preset files.
//!
//! These validators check structural correctness of converter output:
//! sample paths, MIDI ranges, envelope parameters, filter values, etc.
//! Run on every `cargo test` to catch bugs before they reach a human ear.

/// A single validation error with context.
#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Known audio file extensions (case-insensitive check).
fn has_audio_extension(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "wav" | "aif" | "aiff" | "flac" | "ogg" | "mp3"))
}

/// Check a string for null bytes or control characters (except newline/tab).
fn has_control_chars(s: &str) -> bool {
    s.bytes().any(|b| b < 0x20 && b != b'\n' && b != b'\r' && b != b'\t')
}

// ============================================================
// SFZ Validator
// ============================================================

/// Validate a generated SFZ string. Returns a list of errors (empty = valid).
pub fn validate_sfz(sfz: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut in_region = false;
    let mut region_has_sample = false;
    let mut region_num = 0u32;

    for line in sfz.lines() {
        let line = line.trim();

        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        if line == "<region>" {
            // Check previous region had a sample
            if in_region && !region_has_sample {
                errors.push(ValidationError {
                    field: format!("region_{region_num}"),
                    message: "region has no sample= line".into(),
                });
            }
            in_region = true;
            region_has_sample = false;
            region_num += 1;
            continue;
        }

        if line.starts_with('<') {
            // <global>, <group>, etc. — check previous region
            if in_region && !region_has_sample {
                errors.push(ValidationError {
                    field: format!("region_{region_num}"),
                    message: "region has no sample= line".into(),
                });
            }
            in_region = false;
            continue;
        }

        // Parse opcode=value
        if let Some((opcode, raw_value)) = line.split_once('=') {
            let opcode = opcode.trim();
            let value = raw_value.trim();

            match opcode {
                "sample" => {
                    region_has_sample = true;
                    if raw_value != raw_value.trim() {
                        errors.push(ValidationError {
                            field: "sample".into(),
                            message: format!("untrimmed whitespace: '{raw_value}'"),
                        });
                    }
                    if !has_audio_extension(value) {
                        errors.push(ValidationError {
                            field: "sample".into(),
                            message: format!("missing audio extension: '{value}'"),
                        });
                    }
                    if has_control_chars(value) {
                        errors.push(ValidationError {
                            field: "sample".into(),
                            message: format!("contains control characters: '{value}'"),
                        });
                    }
                }
                "lokey" | "hikey" => {
                    if let Ok(v) = value.parse::<i32>() {
                        if !(0..=127).contains(&v) {
                            errors.push(ValidationError {
                                field: opcode.into(),
                                message: format!("{v} out of range 0-127"),
                            });
                        }
                    }
                }
                "lovel" | "hivel" => {
                    if let Ok(v) = value.parse::<i32>() {
                        if !(0..=127).contains(&v) {
                            errors.push(ValidationError {
                                field: opcode.into(),
                                message: format!("{v} out of range 0-127"),
                            });
                        }
                    }
                }
                "ampeg_attack" | "ampeg_decay" | "ampeg_release"
                | "fileg_attack" | "fileg_decay" | "fileg_release" => {
                    if let Ok(v) = value.parse::<f64>() {
                        if !(0.0..=100.0).contains(&v) {
                            errors.push(ValidationError {
                                field: opcode.into(),
                                message: format!("{v} out of range 0-100s"),
                            });
                        }
                    }
                }
                "ampeg_sustain" | "fileg_sustain" => {
                    if let Ok(v) = value.parse::<f64>() {
                        if !(0.0..=100.0).contains(&v) {
                            errors.push(ValidationError {
                                field: opcode.into(),
                                message: format!("{v} out of range 0-100"),
                            });
                        }
                    }
                }
                "cutoff" => {
                    if let Ok(v) = value.parse::<f64>() {
                        if !(1.0..=22000.0).contains(&v) {
                            errors.push(ValidationError {
                                field: "cutoff".into(),
                                message: format!("{v} out of range 1-22000 Hz"),
                            });
                        }
                    }
                }
                "resonance" => {
                    if let Ok(v) = value.parse::<f64>() {
                        if !(0.0..=40.0).contains(&v) {
                            errors.push(ValidationError {
                                field: "resonance".into(),
                                message: format!("{v} out of range 0-40 dB"),
                            });
                        }
                    }
                }
                "amplitude" => {
                    if let Ok(v) = value.parse::<f64>() {
                        if !(0.0..=100.0).contains(&v) {
                            errors.push(ValidationError {
                                field: "amplitude".into(),
                                message: format!("{v} out of range 0-100"),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Check last region
    if in_region && !region_has_sample {
        errors.push(ValidationError {
            field: format!("region_{region_num}"),
            message: "region has no sample= line".into(),
        });
    }

    // Cross-region checks: lokey <= hikey, lovel <= hivel
    validate_sfz_ranges(sfz, &mut errors);

    errors
}

/// Check lokey<=hikey and lovel<=hivel within each region.
fn validate_sfz_ranges(sfz: &str, errors: &mut Vec<ValidationError>) {
    let mut region_num = 0u32;
    let mut lokey: Option<i32> = None;
    let mut hikey: Option<i32> = None;
    let mut lovel: Option<i32> = None;
    let mut hivel: Option<i32> = None;

    for line in sfz.lines() {
        let line = line.trim();
        if line == "<region>" {
            // Check previous region
            check_range_pair(region_num, "key", lokey, hikey, errors);
            check_range_pair(region_num, "vel", lovel, hivel, errors);
            region_num += 1;
            lokey = None;
            hikey = None;
            lovel = None;
            hivel = None;
            continue;
        }
        if let Some((opcode, value)) = line.split_once('=') {
            match opcode.trim() {
                "lokey" => lokey = value.trim().parse().ok(),
                "hikey" => hikey = value.trim().parse().ok(),
                "lovel" => lovel = value.trim().parse().ok(),
                "hivel" => hivel = value.trim().parse().ok(),
                _ => {}
            }
        }
    }
    // Check last region
    check_range_pair(region_num, "key", lokey, hikey, errors);
    check_range_pair(region_num, "vel", lovel, hivel, errors);
}

fn check_range_pair(region: u32, name: &str, lo: Option<i32>, hi: Option<i32>, errors: &mut Vec<ValidationError>) {
    if region == 0 { return; }
    if let (Some(l), Some(h)) = (lo, hi) {
        if l > h {
            errors.push(ValidationError {
                field: format!("region_{region}"),
                message: format!("lo{name}={l} > hi{name}={h}"),
            });
        }
    }
}

// ============================================================
// DecentSampler Validator
// ============================================================

/// Validate a generated DecentSampler preset string. Returns a list of errors (empty = valid).
pub fn validate_dspreset(xml: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Well-formed structure checks
    if !xml.contains("<?xml") {
        errors.push(ValidationError { field: "structure".into(), message: "missing <?xml declaration".into() });
    }
    if !xml.contains("<DecentSampler") {
        errors.push(ValidationError { field: "structure".into(), message: "missing <DecentSampler> root element".into() });
    }
    if !xml.contains("</DecentSampler>") {
        errors.push(ValidationError { field: "structure".into(), message: "missing </DecentSampler> closing tag".into() });
    }

    // Check each line for attribute-level values
    for line in xml.lines() {
        let line = line.trim();

        // Sample path attributes
        for path in extract_attr_values(line, "path") {
            if path != path.trim() {
                errors.push(ValidationError { field: "path".into(), message: format!("untrimmed: '{path}'") });
            }
            if !has_audio_extension(&path) && !path.starts_with('$') {
                errors.push(ValidationError { field: "path".into(), message: format!("missing audio extension: '{path}'") });
            }
        }

        // Note ranges
        for val in extract_attr_values(line, "loNote") {
            if let Ok(v) = val.parse::<i32>() {
                if !(0..=127).contains(&v) {
                    errors.push(ValidationError { field: "loNote".into(), message: format!("{v} out of range 0-127") });
                }
            }
        }
        for val in extract_attr_values(line, "hiNote") {
            if let Ok(v) = val.parse::<i32>() {
                if !(0..=127).contains(&v) {
                    errors.push(ValidationError { field: "hiNote".into(), message: format!("{v} out of range 0-127") });
                }
            }
        }
        for val in extract_attr_values(line, "loVel") {
            if let Ok(v) = val.parse::<i32>() {
                if !(0..=127).contains(&v) {
                    errors.push(ValidationError { field: "loVel".into(), message: format!("{v} out of range 0-127") });
                }
            }
        }
        for val in extract_attr_values(line, "hiVel") {
            if let Ok(v) = val.parse::<i32>() {
                if !(0..=127).contains(&v) {
                    errors.push(ValidationError { field: "hiVel".into(), message: format!("{v} out of range 0-127") });
                }
            }
        }

        // Volume on <groups> element
        if line.starts_with("<groups") {
            for val in extract_attr_values(line, "volume") {
                if let Ok(v) = val.parse::<f64>() {
                    if !(-60.0..=0.0).contains(&v) {
                        errors.push(ValidationError { field: "volume".into(), message: format!("{v} out of range -60 to 0 dB") });
                    }
                }
            }
        }

        // Envelope attributes on <group> elements
        if line.starts_with("<group") {
            for val in extract_attr_values(line, "attack") {
                if let Ok(v) = val.parse::<f64>() {
                    if v < 0.0 {
                        errors.push(ValidationError { field: "attack".into(), message: format!("{v} is negative") });
                    }
                }
            }
            for val in extract_attr_values(line, "decay") {
                if let Ok(v) = val.parse::<f64>() {
                    if v < 0.0 {
                        errors.push(ValidationError { field: "decay".into(), message: format!("{v} is negative") });
                    }
                }
            }
            for val in extract_attr_values(line, "release") {
                if let Ok(v) = val.parse::<f64>() {
                    if v < 0.0 {
                        errors.push(ValidationError { field: "release".into(), message: format!("{v} is negative") });
                    }
                }
            }
            for val in extract_attr_values(line, "sustain") {
                if let Ok(v) = val.parse::<f64>() {
                    if !(0.0..=1.0).contains(&v) {
                        errors.push(ValidationError { field: "sustain".into(), message: format!("{v} out of range 0-1") });
                    }
                }
            }
            for val in extract_attr_values(line, "ampVelTrack") {
                if let Ok(v) = val.parse::<f64>() {
                    if !(0.0..=1.0).contains(&v) {
                        errors.push(ValidationError { field: "ampVelTrack".into(), message: format!("{v} out of range 0-1") });
                    }
                }
            }
        }
    }

    errors
}

/// Extract all values for a given attribute name from a line.
/// e.g. `extract_attr_values(r#"path="foo.wav" path="bar.wav""#, "path")` → ["foo.wav", "bar.wav"]
fn extract_attr_values(line: &str, attr: &str) -> Vec<String> {
    let mut results = Vec::new();
    let pattern = format!("{attr}=\"");
    let mut search = line;
    while let Some(start) = search.find(&pattern) {
        let after = &search[start + pattern.len()..];
        if let Some(end) = after.find('"') {
            results.push(after[..end].to_string());
            search = &after[end + 1..];
        } else {
            break;
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sfz_passes() {
        let sfz = r#"// comment
<region>
sample=Piano_C3.wav
lokey=36
hikey=72
lovel=0
hivel=127
ampeg_attack=0.010
ampeg_decay=0.500
ampeg_sustain=80
ampeg_release=0.300
"#;
        let errors = validate_sfz(sfz);
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_sfz_missing_extension() {
        let sfz = "<region>\nsample=Piano_C3\nlokey=60\nhikey=72\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.field == "sample" && e.message.contains("extension")));
    }

    #[test]
    fn test_sfz_untrimmed_sample() {
        let sfz = "<region>\nsample= Piano_C3.wav \nlokey=60\nhikey=72\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.field == "sample" && e.message.contains("untrimmed")));
    }

    #[test]
    fn test_sfz_key_out_of_range() {
        let sfz = "<region>\nsample=test.wav\nlokey=128\nhikey=64\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.field == "lokey" && e.message.contains("128")));
    }

    #[test]
    fn test_sfz_inverted_range() {
        let sfz = "<region>\nsample=test.wav\nlokey=80\nhikey=60\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.message.contains("lokey=80 > hikey=60")));
    }

    #[test]
    fn test_sfz_region_without_sample() {
        let sfz = "<region>\nlokey=60\nhikey=72\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.message.contains("no sample= line")));
    }

    #[test]
    fn test_sfz_envelope_range() {
        let sfz = "<region>\nsample=t.wav\nampeg_sustain=150\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.field == "ampeg_sustain"));
    }

    #[test]
    fn test_sfz_cutoff_range() {
        let sfz = "<region>\nsample=t.wav\ncutoff=0.5\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.field == "cutoff"));
    }

    #[test]
    fn test_sfz_resonance_range() {
        let sfz = "<region>\nsample=t.wav\nresonance=50.0\n";
        let errors = validate_sfz(sfz);
        assert!(errors.iter().any(|e| e.field == "resonance"));
    }

    #[test]
    fn test_valid_dspreset_passes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<DecentSampler minVersion="1.0.0">
  <groups volume="-1.41">
    <group name="Group1" attack="0.010" decay="0.500" sustain="0.800" release="0.300">
      <sample path="piano.wav" loNote="36" hiNote="72" loVel="0" hiVel="127" />
    </group>
  </groups>
</DecentSampler>
"#;
        let errors = validate_dspreset(xml);
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_dspreset_missing_structure() {
        let xml = "<groups><group><sample path=\"t.wav\" /></group></groups>";
        let errors = validate_dspreset(xml);
        assert!(errors.iter().any(|e| e.message.contains("<?xml")));
        assert!(errors.iter().any(|e| e.message.contains("<DecentSampler>")));
        assert!(errors.iter().any(|e| e.message.contains("</DecentSampler>")));
    }

    #[test]
    fn test_dspreset_path_no_extension() {
        let xml = r#"<?xml version="1.0"?><DecentSampler><groups><group><sample path="test" loNote="60" hiNote="72" /></group></groups></DecentSampler>"#;
        let errors = validate_dspreset(xml);
        assert!(errors.iter().any(|e| e.field == "path" && e.message.contains("extension")));
    }

    #[test]
    fn test_dspreset_volume_out_of_range() {
        let xml = "<?xml version=\"1.0\"?>\n<DecentSampler>\n<groups volume=\"5.0\">\n</groups>\n</DecentSampler>\n";
        let errors = validate_dspreset(xml);
        assert!(errors.iter().any(|e| e.field == "volume"));
    }

    #[test]
    fn test_dspreset_sustain_out_of_range() {
        let xml = "<?xml version=\"1.0\"?>\n<DecentSampler>\n<groups>\n<group sustain=\"1.5\">\n</group>\n</groups>\n</DecentSampler>\n";
        let errors = validate_dspreset(xml);
        assert!(errors.iter().any(|e| e.field == "sustain"));
    }

    #[test]
    fn test_dspreset_note_out_of_range() {
        let xml = "<?xml version=\"1.0\"?>\n<DecentSampler>\n<groups>\n<group>\n<sample path=\"t.wav\" loNote=\"130\" hiNote=\"72\" />\n</group>\n</groups>\n</DecentSampler>\n";
        let errors = validate_dspreset(xml);
        assert!(errors.iter().any(|e| e.field == "loNote"));
    }

    #[test]
    fn test_extract_attr_values() {
        let line = r#"<sample path="foo.wav" loNote="60" hiNote="72" />"#;
        assert_eq!(extract_attr_values(line, "path"), vec!["foo.wav"]);
        assert_eq!(extract_attr_values(line, "loNote"), vec!["60"]);
        assert_eq!(extract_attr_values(line, "hiNote"), vec!["72"]);
        assert!(extract_attr_values(line, "missing").is_empty());
    }

    #[test]
    fn test_has_audio_extension() {
        assert!(has_audio_extension("test.wav"));
        assert!(has_audio_extension("test.WAV"));
        assert!(has_audio_extension("test.aif"));
        assert!(has_audio_extension("test.flac"));
        assert!(!has_audio_extension("test"));
        assert!(!has_audio_extension("test.txt"));
        // Dots in note names shouldn't match
        assert!(!has_audio_extension("BRASS 02-C.1"));
    }
}
