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
