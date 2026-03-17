use crate::types::{AkaiProgram, EnvelopeTiming, mod_source_name};

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

        // Pre-compute filter state for UI and modulators sections
        let has_filter = self.keygroups.iter().any(|kg| kg.filter.is_some());
        let filter_env_ref = self.keygroups.iter()
            .find_map(|kg| kg.filter_env.as_ref().filter(|env| env.depth != 0));
        let has_filter_env = filter_env_ref.is_some();

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

        // Filter envelope ADSR knobs (only when filter effect and filter env both present)
        if has_filter && has_filter_env {
            let fenv = filter_env_ref.unwrap();
            let att_val = if fenv.attack == 0 { 0.001 } else { fenv.attack_time() };
            let dec_val = if fenv.decay == 0 { 0.1 } else { fenv.decay_time() };
            let sus_val = fenv.sustain_normalized();
            let rel_val = if fenv.release == 0 { 0.1 } else { fenv.release_time() };

            xml.push_str(&format!(
                "      <labeled-knob x=\"10\" y=\"140\" width=\"90\" height=\"100\" parameterName=\"FILT_ENV_ATTACK\" type=\"float\" minValue=\"0\" maxValue=\"5\" value=\"{att_val:.2}\" textColor=\"AA000000\">\n"
            ));
            xml.push_str("        <label text=\"Filt Att\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
            xml.push_str("      </labeled-knob>\n");
            xml.push_str(&format!(
                "      <labeled-knob x=\"110\" y=\"140\" width=\"90\" height=\"100\" parameterName=\"FILT_ENV_DECAY\" type=\"float\" minValue=\"0\" maxValue=\"5\" value=\"{dec_val:.2}\" textColor=\"AA000000\">\n"
            ));
            xml.push_str("        <label text=\"Filt Dec\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
            xml.push_str("      </labeled-knob>\n");
            xml.push_str(&format!(
                "      <labeled-knob x=\"210\" y=\"140\" width=\"90\" height=\"100\" parameterName=\"FILT_ENV_SUSTAIN\" type=\"float\" minValue=\"0\" maxValue=\"1\" value=\"{sus_val:.2}\" textColor=\"AA000000\">\n"
            ));
            xml.push_str("        <label text=\"Filt Sus\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
            xml.push_str("      </labeled-knob>\n");
            xml.push_str(&format!(
                "      <labeled-knob x=\"310\" y=\"140\" width=\"90\" height=\"100\" parameterName=\"FILT_ENV_RELEASE\" type=\"float\" minValue=\"0\" maxValue=\"10\" value=\"{rel_val:.2}\" textColor=\"AA000000\">\n"
            ));
            xml.push_str("        <label text=\"Filt Rel\" x=\"0\" y=\"80\" width=\"90\" height=\"30\" />\n");
            xml.push_str("      </labeled-knob>\n");
        }

        xml.push_str("    </tab>\n");
        xml.push_str("  </ui>\n\n");

        // Groups section
        if let Some(output) = &self.output {
            xml.push_str(&format!("  <groups volume=\"{:.2}\">\n", output.volume_db()));
        } else {
            xml.push_str("  <groups>\n");
        }

        for (group_id, keygroup) in self.keygroups.iter().enumerate() {
            xml.push_str(&format!("    <group name=\"Group{}\"", group_id + 1));

            if let Some(env) = &keygroup.amp_env {
                let attack = if env.attack == 0 { 0.001 } else { env.attack_time() };
                let decay = if env.decay == 0 { 0.1 } else { env.decay_time() };
                let sustain = env.sustain_normalized();
                let release = if env.release == 0 { 0.1 } else { env.release_time() };
                xml.push_str(&format!(" attack=\"{attack:.3}\" decay=\"{decay:.3}\" sustain=\"{sustain:.3}\" release=\"{release:.3}\""));
            }

            // Velocity sensitivity (DS range 0.0-1.0, negative not supported)
            if let Some(output) = &self.output {
                // AKP range (-100..100) → DS range (0.0..1.0), negative unsupported
                let vel_track = (output.velocity_sensitivity.max(0) as f32 / 100.0).min(1.0);
                if (vel_track - 1.0).abs() > f32::EPSILON {
                    xml.push_str(&format!(" ampVelTrack=\"{vel_track:.2}\""));
                }
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

        // Modulators section — collect all modulators, then wrap if non-empty
        let mut mod_xml = String::new();

        // LFO modulator
        if let Some(lfo) = &self.lfo1 {
            if lfo.depth > 0 {
                let amount = lfo.depth_normalized();
                mod_xml.push_str(&format!(
                    "    <lfo frequency=\"{:.2}\" waveform=\"{}\" target=\"FILTER_CUTOFF\" amount=\"{amount:.2}\" />\n",
                    lfo.rate_hz(), lfo.waveform_name()));
            }
        }

        // Filter envelope modulator (only when lowpass effect is present at effectIndex=0)
        if has_filter && has_filter_env {
            let fenv = filter_env_ref.unwrap();
            let attack = if fenv.attack == 0 { 0.001 } else { fenv.attack_time() };
            let decay = if fenv.decay == 0 { 0.1 } else { fenv.decay_time() };
            let sustain = fenv.sustain_normalized();
            let release = if fenv.release == 0 { 0.1 } else { fenv.release_time() };
            let mod_amount = fenv.depth as f32 / 100.0;

            mod_xml.push_str(&format!(
                "    <envelope attack=\"{attack:.3}\" decay=\"{decay:.3}\" sustain=\"{sustain:.3}\" release=\"{release:.3}\" modAmount=\"{mod_amount:.2}\" scope=\"voice\">\n"
            ));
            mod_xml.push_str(
                "      <binding type=\"effect\" level=\"instrument\" effectIndex=\"0\" parameter=\"FX_FILTER_FREQUENCY\" translation=\"table\" translationTable=\"0,33;0.3,150;0.4,450;0.5,1100;0.7,4100;0.9,11000;1.0,20000\" />\n"
            );
            mod_xml.push_str("    </envelope>\n");
        }

        // Modulation bindings from ProgramModulation
        if let Some(mods) = &self.modulation {
            // Velocity -> filter (filter_mod_1_source == 5)
            if mods.filter_mod_1_source == 5 {
                if let Some(filter) = self.keygroups.iter().find_map(|kg| kg.filter.as_ref()) {
                    if filter.mod_input_1 != 0 {
                        let amount = filter.mod_input_1 as f32 / 100.0;
                        mod_xml.push_str(&format!(
                            "    <velocity modAmount=\"{amount:.2}\">\n"
                        ));
                        mod_xml.push_str(
                            "      <binding type=\"effect\" level=\"instrument\" effectIndex=\"0\" parameter=\"FX_FILTER_FREQUENCY\" />\n"
                        );
                        mod_xml.push_str("    </velocity>\n");
                    }
                }
            }

            // Modwheel -> pan (pan_mod_3_source == 1)
            if mods.pan_mod_3_source == 1 {
                if let Some(output) = &self.output {
                    if output.pan_mod_3 > 0 {
                        let amount = output.pan_mod_3 as f32 / 100.0;
                        mod_xml.push_str(&format!(
                            "    <cc number=\"1\" modAmount=\"{amount:.2}\">\n"
                        ));
                        mod_xml.push_str(
                            "      <binding type=\"general\" level=\"instrument\" parameter=\"PAN\" />\n"
                        );
                        mod_xml.push_str("    </cc>\n");
                    }
                }
            }

            // Unsupported routes as XML comments
            let routes: &[(u8, &str, u8)] = &[
                (mods.amp_mod_1_source, "amp_mod_1", self.output.as_ref().map_or(0, |o| o.amp_mod_1)),
                (mods.amp_mod_2_source, "amp_mod_2", self.output.as_ref().map_or(0, |o| o.amp_mod_2)),
                (mods.pan_mod_1_source, "pan_mod_1", self.output.as_ref().map_or(0, |o| o.pan_mod_1)),
                (mods.pan_mod_2_source, "pan_mod_2", self.output.as_ref().map_or(0, |o| o.pan_mod_2)),
            ];

            for &(source, dest, amount) in routes {
                if source != 0 && amount != 0 {
                    mod_xml.push_str(&format!(
                        "    <!-- AKP modulation: {} \u{2192} {} (amount={}, not supported in DS) -->\n",
                        mod_source_name(source), dest, amount
                    ));
                }
            }

            // Also check pan_mod_3 for unsupported sources (not modwheel)
            if mods.pan_mod_3_source != 0 && mods.pan_mod_3_source != 1 {
                if let Some(output) = &self.output {
                    if output.pan_mod_3 > 0 {
                        mod_xml.push_str(&format!(
                            "    <!-- AKP modulation: {} \u{2192} pan_mod_3 (amount={}, not supported in DS) -->\n",
                            mod_source_name(mods.pan_mod_3_source), output.pan_mod_3
                        ));
                    }
                }
            }

            // Check filter mod sources for unsupported (not velocity)
            let filter_routes: &[(u8, &str, i8)] = &[
                (mods.filter_mod_2_source, "filter_mod_2", self.keygroups.first().and_then(|kg| kg.filter.as_ref()).map_or(0, |f| f.mod_input_2)),
                (mods.filter_mod_3_source, "filter_mod_3", self.keygroups.first().and_then(|kg| kg.filter.as_ref()).map_or(0, |f| f.mod_input_3)),
            ];

            for &(source, dest, amount) in filter_routes {
                if source != 0 && amount != 0 {
                    mod_xml.push_str(&format!(
                        "    <!-- AKP modulation: {} \u{2192} {} (amount={}, not supported in DS) -->\n",
                        mod_source_name(source), dest, amount
                    ));
                }
            }

            // Check filter_mod_1 for unsupported sources (not velocity)
            if mods.filter_mod_1_source != 0 && mods.filter_mod_1_source != 5 {
                if let Some(filter) = self.keygroups.first().and_then(|kg| kg.filter.as_ref()) {
                    if filter.mod_input_1 != 0 {
                        mod_xml.push_str(&format!(
                            "    <!-- AKP modulation: {} \u{2192} filter_mod_1 (amount={}, not supported in DS) -->\n",
                            mod_source_name(mods.filter_mod_1_source), filter.mod_input_1
                        ));
                    }
                }
            }
        }

        if !mod_xml.is_empty() {
            xml.push_str("  <modulators>\n");
            xml.push_str(&mod_xml);
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

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn test_dspreset_basic_structure() {
        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup { low_key: 36, high_key: 72, ..Default::default() };
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
        let keygroup = Keygroup { filter: Some(Filter { filter_type: 0, cutoff: 50, ..Default::default() }), ..Default::default() };
        program.keygroups.push(keygroup);

        let xml = program.to_dspreset_string();
        assert!(xml.contains("frequency=\"$FILTER_CUTOFF\""));
        assert!(xml.contains("resonance=\"$FILTER_RESONANCE\""));
    }

    #[test]
    fn test_dspreset_envelope_values() {
        let mut program = AkaiProgram::default();
        let keygroup = Keygroup { amp_env: Some(Envelope { attack: 20, decay: 40, sustain: 80, release: 60, ..Default::default() }), ..Default::default() };
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
        let mut program = AkaiProgram { lfo1: Some(Lfo { waveform: 0, rate: 50, depth: 75, ..Default::default() }), ..Default::default() };
        program.keygroups.push(Keygroup::default());

        let xml = program.to_dspreset_string();
        assert!(xml.contains("<modulators>"));
        assert!(xml.contains("waveform=\"sine\""));
        assert!(xml.contains("amount=\"0.75\""));
    }

    #[test]
    fn test_dspreset_volume_on_groups_wrapper() {
        let mut program = AkaiProgram {
            output: Some(ProgramOutput::default()), // loudness=85 → ~-4dB
            ..Default::default()
        };
        program.keygroups.push(Keygroup::default());

        let xml = program.to_dspreset_string();
        assert!(xml.contains("<groups volume=\""));
        assert!(!xml.contains("<group name=\"Group1\" volume=")); // not per-group
    }

    #[test]
    fn test_dspreset_amp_vel_track() {
        let mut program = AkaiProgram {
            output: Some(ProgramOutput { velocity_sensitivity: 25, ..Default::default() }),
            ..Default::default()
        };
        program.keygroups.push(Keygroup::default());

        let xml = program.to_dspreset_string();
        assert!(xml.contains("ampVelTrack=\"0.25\"")); // 25/100 = 0.25
    }

    #[test]
    fn test_dspreset_filter_envelope_modulator() {
        let mut program = AkaiProgram::default();
        program.keygroups.push(Keygroup {
            filter_env: Some(FilterEnvelope {
                attack: 10,
                decay: 30,
                sustain: 70,
                release: 20,
                depth: 50,
                ..Default::default()
            }),
            filter: Some(Filter::default()),
            ..Default::default()
        });

        let xml = program.to_dspreset_string();
        assert!(xml.contains("<envelope"), "Should contain envelope modulator");
        assert!(xml.contains("modAmount=\"0.50\""), "modAmount should be depth/100");
        assert!(xml.contains("FX_FILTER_FREQUENCY"), "Should target filter frequency");
        assert!(xml.contains("scope=\"voice\""), "Should have voice scope");
        assert!(xml.contains("translationTable="), "Should have translation table for frequency");
    }

    #[test]
    fn test_dspreset_filter_envelope_knobs() {
        let mut program = AkaiProgram::default();
        program.keygroups.push(Keygroup {
            filter_env: Some(FilterEnvelope {
                attack: 10,
                decay: 30,
                sustain: 70,
                release: 20,
                depth: 50,
                ..Default::default()
            }),
            filter: Some(Filter::default()),
            ..Default::default()
        });

        let xml = program.to_dspreset_string();
        assert!(xml.contains("FILT_ENV_ATTACK"), "Should have filter env attack knob");
        assert!(xml.contains("Filt Att"), "Should have filter attack label");
        assert!(xml.contains("Filt Dec"), "Should have filter decay label");
        assert!(xml.contains("Filt Sus"), "Should have filter sustain label");
        assert!(xml.contains("Filt Rel"), "Should have filter release label");
    }

    // ---- Output validation tests ----

    #[test]
    fn test_dspreset_output_passes_validation() {
        use crate::validate::validate_dspreset;

        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup {
            low_key: 36, high_key: 72,
            amp_env: Some(Envelope { attack: 20, decay: 40, sustain: 80, release: 60, ..Default::default() }),
            filter: Some(Filter { filter_type: 0, cutoff: 50, resonance: 6, ..Default::default() }),
            filter_env: Some(FilterEnvelope { attack: 10, decay: 30, sustain: 70, release: 20, depth: 50, ..Default::default() }),
            ..Default::default()
        };
        keygroup.zones.push(Zone { sample_name: "Piano_C3.wav".to_string(), low_vel: 1, high_vel: 127, ..Default::default() });
        program.keygroups.push(keygroup);
        program.output = Some(ProgramOutput::default());
        program.lfo1 = Some(Lfo { rate: 50, depth: 25, ..Default::default() });

        let xml = program.to_dspreset_string();
        let errors = validate_dspreset(&xml);
        assert!(errors.is_empty(), "Validation errors in dspreset output:\n{}\n---\n{xml}", errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn test_dspreset_boundary_values_pass_validation() {
        use crate::validate::validate_dspreset;

        let mut program = AkaiProgram::default();

        // Extreme values
        let mut kg1 = Keygroup {
            low_key: 0, high_key: 127,
            amp_env: Some(Envelope { attack: 0, decay: 0, sustain: 0, release: 0, ..Default::default() }),
            ..Default::default()
        };
        kg1.zones.push(Zone { sample_name: "test.wav".to_string(), low_vel: 0, high_vel: 127, ..Default::default() });
        program.keygroups.push(kg1);

        let mut kg2 = Keygroup {
            low_key: 0, high_key: 127,
            amp_env: Some(Envelope { attack: 100, decay: 100, sustain: 100, release: 100, ..Default::default() }),
            ..Default::default()
        };
        kg2.zones.push(Zone { sample_name: "test2.wav".to_string(), low_vel: 0, high_vel: 127, ..Default::default() });
        program.keygroups.push(kg2);

        // Loudness extremes
        program.output = Some(ProgramOutput { loudness: 100, ..Default::default() });

        let xml = program.to_dspreset_string();
        let errors = validate_dspreset(&xml);
        assert!(errors.is_empty(), "Validation errors with boundary values:\n{}\n---\n{xml}", errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn test_dspreset_zero_loudness_pass_validation() {
        use crate::validate::validate_dspreset;

        let mut program = AkaiProgram::default();
        let mut keygroup = Keygroup::default();
        keygroup.zones.push(Zone { sample_name: "test.wav".to_string(), ..Default::default() });
        program.keygroups.push(keygroup);
        program.output = Some(ProgramOutput { loudness: 0, ..Default::default() });

        let xml = program.to_dspreset_string();
        let errors = validate_dspreset(&xml);
        assert!(errors.is_empty(), "Validation errors with zero loudness:\n{}\n---\n{xml}", errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn test_dspreset_modulation_bindings() {
        let mut program = AkaiProgram {
            modulation: Some(ProgramModulation {
                filter_mod_1_source: 5, // VELOCITY
                pan_mod_3_source: 1,    // MODWHEEL
                amp_mod_2_source: 3,    // AFTERTOUCH (unsupported in DS)
                ..Default::default()
            }),
            output: Some(ProgramOutput {
                pan_mod_3: 50,
                amp_mod_2: 20,
                ..Default::default()
            }),
            ..Default::default()
        };
        program.keygroups.push(Keygroup {
            filter: Some(Filter { mod_input_1: 30, ..Default::default() }),
            ..Default::default()
        });

        let xml = program.to_dspreset_string();
        // Velocity -> filter
        assert!(xml.contains("<velocity"), "Should have velocity modulator");
        assert!(xml.contains("modAmount=\"0.30\""), "Velocity modAmount should be mod_input_1/100");
        // Modwheel -> pan
        assert!(xml.contains("<cc number=\"1\""), "Should have CC1 modulator for modwheel");
        assert!(xml.contains("parameter=\"PAN\""), "Modwheel should target PAN");
        // Unsupported route as comment
        assert!(xml.contains("<!-- AKP modulation: AFTERTOUCH"), "Unsupported routes should be XML comments");
        assert!(xml.contains("not supported in DS"), "Comment should mention DS limitation");
    }
}
