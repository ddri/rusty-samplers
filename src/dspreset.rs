use crate::types::AkaiProgram;

impl AkaiProgram {
    /// Converts the parsed AkaiProgram into a Decent Sampler .dspreset XML string.
    pub fn to_dspreset_string(&self) -> String {
        let mut xml = String::new();

        // XML declaration
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

        for (_group_id, keygroup) in self.keygroups.iter().enumerate() {
            xml.push_str(&format!("    <group name=\"Group{}\"", _group_id + 1));

            if let Some(env) = &keygroup.amp_env {
                let attack = if env.attack == 0 { 0.001 } else { (env.attack as f32 / 100.0 * 4.0).exp() * 0.001 };
                let decay = if env.decay == 0 { 0.1 } else { (env.decay as f32 / 100.0 * 4.0).exp() * 0.001 };
                let sustain = env.sustain as f32 / 100.0;
                let release = if env.release == 0 { 0.1 } else { (env.release as f32 / 100.0 * 5.0).exp() * 0.001 };

                xml.push_str(&format!(" attack=\"{:.3}\" decay=\"{:.3}\" sustain=\"{:.3}\" release=\"{:.3}\"",
                    attack, decay, sustain, release));
            }

            if let Some(tune) = &keygroup.tune {
                let volume_db = (tune.level as f32 / 100.0) * 66.0 - 60.0;
                xml.push_str(&format!(" volume=\"{:.2}\"", volume_db));
            }

            xml.push_str(">\n");

            if let Some(sample) = &keygroup.sample {
                xml.push_str("      <sample ");
                xml.push_str(&format!("path=\"{}\" ", sample.filename));
                xml.push_str(&format!("loNote=\"{}\" hiNote=\"{}\" ", keygroup.low_key, keygroup.high_key));
                xml.push_str(&format!("loVel=\"{}\" hiVel=\"{}\" ", keygroup.low_vel, keygroup.high_vel));

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

        let has_filter = self.keygroups.iter().any(|kg| kg.filter.is_some());
        if has_filter {
            xml.push_str("    <lowpass frequency=\"FILTER_CUTOFF\" resonance=\"FILTER_RESONANCE\" />\n");
        }

        xml.push_str("    <reverb roomSize=\"0.5\" damping=\"0.5\" wetLevel=\"0.3\" dryLevel=\"0.7\" width=\"1.0\" />\n");

        xml.push_str("  </effects>\n\n");

        // MIDI section
        xml.push_str("  <midi>\n");
        xml.push_str("    <!-- MIDI CC bindings can be added here -->\n");
        xml.push_str("    <cc number=\"1\" parameter=\"FILTER_CUTOFF\" />\n");
        xml.push_str("    <cc number=\"2\" parameter=\"FILTER_RESONANCE\" />\n");
        xml.push_str("    <cc number=\"7\" parameter=\"MAIN_VOLUME\" />\n");
        xml.push_str("  </midi>\n\n");

        // Modulators section
        let has_lfo = self.keygroups.iter().any(|kg| kg.lfo1.is_some() || kg.lfo2.is_some());
        if has_lfo {
            xml.push_str("  <modulators>\n");

            for (_group_id, keygroup) in self.keygroups.iter().enumerate() {
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
