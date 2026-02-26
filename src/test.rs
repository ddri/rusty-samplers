#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_program_initialization() {
        let program = AkaiProgram::default();
        assert_eq!(program.keygroups.len(), 0);
        assert_eq!(program.name, "Default Program");
    }

    #[test]
    fn test_sfz_conversion() {
        let mut program = AkaiProgram::default();
        
        // Create a test keygroup
        let mut keygroup = Keygroup::default();
        keygroup.sample.path = "test_sample.wav".to_string();
        keygroup.tune.root_key = 60; // Middle C
        keygroup.zone.key_low = 60;
        keygroup.zone.key_high = 60;
        
        program.keygroups.push(keygroup);
        
        let sfz_output = program.to_sfz_string();
        
        // Basic checks
        assert!(sfz_output.contains("sample=test_sample.wav"));
        assert!(sfz_output.contains("lokey=60"));
        assert!(sfz_output.contains("hikey=60"));
        assert!(sfz_output.contains("pitch_keycenter=60"));
    }

    #[test]
    fn test_decent_sampler_conversion() {
        let mut program = AkaiProgram::default();
        
        // Create a test keygroup  
        let mut keygroup = Keygroup::default();
        keygroup.sample.path = "test_sample.wav".to_string();
        keygroup.tune.root_key = 60;
        keygroup.zone.key_low = 60;
        keygroup.zone.key_high = 60;
        
        program.keygroups.push(keygroup);
        
        let ds_output = program.to_dspreset_string();
        
        // Basic checks
        assert!(ds_output.contains("<DecentSampler>"));
        assert!(ds_output.contains("test_sample.wav"));
        assert!(ds_output.contains("rootNote=\"60\""));
        assert!(ds_output.contains("</DecentSampler>"));
    }

    #[test]
    fn test_error_handling() {
        // Test invalid RIFF header
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // Not RIFF
        let mut cursor = Cursor::new(invalid_data);
        
        let result = validate_riff_header(&mut cursor);
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                AkpError::InvalidRiffHeader => {}, // Expected
                _ => panic!("Wrong error type"),
            }
        }
    }
}