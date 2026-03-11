# Spec-Aligned AKP Parser Rewrite

**Date**: 2026-03-12
**Scope**: S5000/S6000 models only
**References**: burnit.co.uk/AKPspec/, ConvertWithMoss (git-moss/ConvertWithMoss)

## Goal

Rewrite the AKP parser to match the burnit.co.uk byte-level specification exactly. Parse and store all spec fields. Fix all output generators to use correct values. Verified against real S6000 factory AKP files.

## Context

The parser was originally written from incomplete format assumptions. Testing against real Akai S6000 factory files revealed that while basic parsing now works (after adding kloc, rewriting zone, fixing filt), many byte offsets are wrong and critical fields are misread. A spec cross-check confirmed:

- Envelope ADSR offsets are wrong (sustain and release swapped, attack off by 1)
- LFO offsets are wrong (reading from offset 5, spec says 1)
- LFO waveform enum is inverted (0=sine not triangle)
- Tune chunk reads a "level" field that doesn't exist in the spec
- Mods chunk is misinterpreted as multiple small entries (it's one 38-byte flat block)
- tune/lfo/mods are top-level program chunks, not inside kgrp
- Zone sample name field is 14 bytes, spec says 20
- Resonance range is 0-12, not 0-100
- Filter type 0 is "2-pole LP" (active), not "off"
- Only 1 zone per keygroup stored, spec supports 4
- Output chunk (loudness, velocity sensitivity) is skipped entirely
- Aux envelope has completely different structure from amp/filter (rate/level, not ADSR)

## Phase 1: Data Structures (`src/types.rs`)

Replace all existing structs with spec-aligned versions. Every field from the burnit.co.uk spec gets a corresponding Rust field.

### AkaiProgram (top-level)

```rust
pub struct AkaiProgram {
    pub header: Option<ProgramHeader>,
    pub output: Option<ProgramOutput>,         // NEW — from out chunk
    pub tuning: Option<ProgramTuning>,         // RENAMED — from top-level tune chunk
    pub lfo1: Option<Lfo>,                     // MOVED — from top-level lfo chunk
    pub lfo2: Option<Lfo>,                     // MOVED — from top-level lfo chunk
    pub modulation: Option<ProgramModulation>, // REWRITTEN — from top-level mods chunk
    pub keygroups: Vec<Keygroup>,
}
```

### ProgramHeader (prg, 6 bytes) — unchanged

```rust
pub struct ProgramHeader {
    pub midi_program_number: u8,  // 0-127
    pub number_of_keygroups: u8,  // 1-99
}
```

### ProgramOutput (out, 8 bytes) — NEW

```rust
pub struct ProgramOutput {
    pub loudness: u8,              // 0-100, default 85
    pub amp_mod_1: u8,             // 0-100
    pub amp_mod_2: u8,             // 0-100
    pub pan_mod_1: u8,             // 0-100
    pub pan_mod_2: u8,             // 0-100
    pub pan_mod_3: u8,             // 0-100
    pub velocity_sensitivity: i8,  // -100..+100, default +25
}
```

### ProgramTuning (tune, 22 bytes) — REWRITTEN

Replaces old `Tune` struct which had a non-existent "level" field. The volume concept lives in `ProgramOutput.loudness`, not here.

```rust
pub struct ProgramTuning {
    pub semitone: i8,          // -36..+36, offset 1
    pub fine: i8,              // -50..+50, offset 2
    pub detune: [i8; 12],      // per-note C through B, offsets 3-14
    pub pitchbend_up: u8,      // 0-24 semitones, offset 15
    pub pitchbend_down: u8,    // 0-24 semitones, offset 16
    pub bend_mode: u8,         // 0=NORMAL 1=HELD, offset 17
    pub aftertouch: i8,        // -12..+12, offset 18
}
```

### Lfo (lfo, 12 bytes) — EXPANDED

Offsets fixed from 5-8 to 1-4. Added sync, modwheel, aftertouch, retrigger, and mod fields.

**Important**: LFO 1 and LFO 2 have different layouts at offsets 5-8:
- LFO 1: offset 5 = sync, offset 7 = modwheel, offset 8 = aftertouch
- LFO 2: offset 5 = reserved (0x01), offset 6 = retrigger, offsets 7-8 = reserved (0x00)

We use a single struct with all fields. The parser reads both variants into it: for LFO 1 it reads sync/modwheel/aftertouch; for LFO 2 it reads retrigger and zeroes the LFO1-only fields.

Default waveform differs: LFO 1 defaults to 1 (TRIANGLE), LFO 2 defaults to 0 (SINE).

```rust
pub struct Lfo {
    pub waveform: u8,      // offset 1. 0=SINE,1=TRI,2=SQ,3=SQ+,4=SQ-,5=SAW_BI,6=SAW_UP,7=SAW_DN,8=RANDOM
    pub rate: u8,          // offset 2. 0-100
    pub delay: u8,         // offset 3. 0-100
    pub depth: u8,         // offset 4. 0-100
    pub sync: u8,          // offset 5. 0=OFF, 1=ON (LFO 1 only)
    pub retrigger: u8,     // offset 6. 0=OFF, 1=ON (LFO 2 only)
    pub modwheel: u8,      // offset 7. 0-100 (LFO 1 only, always 0 for LFO 2)
    pub aftertouch: u8,    // offset 8. 0-100 (LFO 1 only, always 0 for LFO 2)
    pub rate_mod: i8,      // offset 9. -100..+100
    pub delay_mod: i8,     // offset 10. -100..+100
    pub depth_mod: i8,     // offset 11. -100..+100
}
```

### ProgramModulation (mods, 38 bytes) — REWRITTEN

Was multiple small source/dest/amount entries. Now a single flat struct matching the spec's fixed routing assignments. Each field is a modulation source index (0=NO SOURCE through 14=dEXTERNAL).

The 38-byte block has a `[destination_id, source_id]` pairing at each position. The even offsets contain fixed destination identifiers that we skip during parsing — only the source values at odd offsets are stored.

Default values match the burnit.co.uk spec defaults (e.g., amp_mod_1=KEYBOARD(6), pitch_mod_1=LFO1(7), amp_mod=VELOCITY(5)).

```rust
pub struct ProgramModulation {
    pub amp_mod_1_source: u8,         // offset 5, default 6 (KEYBOARD)
    pub amp_mod_2_source: u8,         // offset 7
    pub pan_mod_1_source: u8,         // offset 9
    pub pan_mod_2_source: u8,         // offset 11
    pub pan_mod_3_source: u8,         // offset 13
    pub lfo1_rate_mod_source: u8,     // offset 15
    pub lfo1_delay_mod_source: u8,    // offset 17
    pub lfo1_depth_mod_source: u8,    // offset 19
    pub lfo2_rate_mod_source: u8,     // offset 21
    pub lfo2_delay_mod_source: u8,    // offset 23
    pub lfo2_depth_mod_source: u8,    // offset 25
    pub pitch_mod_1_source: u8,       // offset 27
    pub pitch_mod_2_source: u8,       // offset 29
    pub amp_mod_source: u8,           // offset 31
    pub filter_mod_1_source: u8,      // offset 33
    pub filter_mod_2_source: u8,      // offset 35
    pub filter_mod_3_source: u8,      // offset 37
}
```

### Keygroup — EXPANDED

```rust
pub struct Keygroup {
    // From kloc (16 bytes)
    pub low_key: u8,             // offset 4, default 21
    pub high_key: u8,            // offset 5, default 127
    pub semitone_tune: i8,       // offset 6, -36..+36
    pub fine_tune: i8,           // offset 7, -50..+50
    pub override_fx: u8,         // offset 8, 0=OFF,1=FX1,2=FX2,3=RV3,4=RV4
    pub fx_send_level: u8,       // offset 9, 0-100
    pub pitch_mod_1: i8,         // offset 10, -100..+100
    pub pitch_mod_2: i8,         // offset 11, -100..+100
    pub amp_mod: i8,             // offset 12, -100..+100
    pub zone_crossfade: u8,      // offset 13, 0=OFF,1=ON
    pub mute_group: u8,          // offset 14

    // Up to 4 sample zones
    pub zones: Vec<Zone>,

    // Envelopes — three different types
    pub amp_env: Option<Envelope>,
    pub filter_env: Option<FilterEnvelope>,
    pub aux_env: Option<AuxEnvelope>,

    // Filter
    pub filter: Option<Filter>,
}
```

Default impl: `low_key=21, high_key=127` (spec says range 21-127, default low=21).

### Zone (zone, 46 bytes) — NEW struct

Was flat fields on Keygroup. Now its own struct with all spec fields. Up to 4 per keygroup.

```rust
pub struct Zone {
    pub sample_name: String,     // name_len at offset 1, name bytes at offsets 2-21 (20 chars max, null-padded)
    pub low_vel: u8,             // offset 34, 0-127
    pub high_vel: u8,            // offset 35, 0-127
    pub fine_tune: i8,           // offset 36, -50..+50
    pub semitone_tune: i8,       // offset 37, -36..+36
    pub filter: i8,              // offset 38, -100..+100
    pub pan: i8,                 // offset 39, -50..+50 (L50..R50)
    pub playback: u8,            // offset 40, 0=NO LOOP,1=ONE SHOT,2=LOOP IN REL,3=LOOP UNTIL REL,4=AS SAMPLE
    pub output: u8,              // offset 41, 0-24
    pub level: i8,               // offset 42, -100..+100
    pub keyboard_track: u8,      // offset 43, 0=OFF,1=ON
    pub vel_to_start: i16,       // offsets 44-45, -9999..9999
}
```

### Envelope (env, 18 bytes) — FIXED offsets

```rust
pub struct Envelope {
    pub attack: u8,              // offset 1
    pub decay: u8,               // offset 3
    pub release: u8,             // offset 4
    pub sustain: u8,             // offset 7
    pub velocity_attack: i8,     // offset 10
    pub keyscale: i8,            // offset 12
    pub on_vel_release: i8,      // offset 14
    pub off_vel_release: i8,     // offset 15
}
```

### FilterEnvelope (env, 18 bytes) — NEW separate struct

Same layout as Envelope but has a `depth` field at offset 9.

```rust
pub struct FilterEnvelope {
    pub attack: u8,              // offset 1
    pub decay: u8,               // offset 3
    pub release: u8,             // offset 4
    pub sustain: u8,             // offset 7
    pub depth: i8,               // offset 9, -100..+100
    pub velocity_attack: i8,     // offset 10
    pub keyscale: i8,            // offset 12
    pub on_vel_release: i8,      // offset 14
    pub off_vel_release: i8,     // offset 15
}
```

### AuxEnvelope (env, 18 bytes) — NEW struct

Completely different from ADSR. Four-stage rate/level envelope.

```rust
pub struct AuxEnvelope {
    pub rate_1: u8,              // offset 1
    pub rate_2: u8,              // offset 2
    pub rate_3: u8,              // offset 3
    pub rate_4: u8,              // offset 4
    pub level_1: u8,             // offset 5
    pub level_2: u8,             // offset 6
    pub level_3: u8,             // offset 7
    pub level_4: u8,             // offset 8
    pub vel_rate_1: i8,          // offset 10
    pub key_rate_2_4: i8,        // offset 12
    pub vel_rate_4: i8,          // offset 14
    pub off_vel_rate_4: i8,      // offset 15
    pub vel_output_level: i8,    // offset 16
}
```

### Filter (filt, 10 bytes) — EXPANDED

```rust
pub struct Filter {
    pub filter_type: u8,         // offset 1, 0-25 (0=2-pole LP, active filter)
    pub cutoff: u8,              // offset 2, 0-100
    pub resonance: u8,           // offset 3, 0-12
    pub keyboard_track: i8,      // offset 4, -36..+36
    pub mod_input_1: i8,         // offset 5, -100..+100
    pub mod_input_2: i8,         // offset 6, -100..+100
    pub mod_input_3: i8,         // offset 7, -100..+100
    pub headroom: u8,            // offset 8, 0-5 (0=0dB..5=30dB)
}
```

### Removed structs

- `Tune` — replaced by `ProgramTuning` (no "level" field)
- `Sample` — replaced by `Zone.sample_name`
- `Modulation` — replaced by `ProgramModulation`

### Kept unchanged

- `RiffChunkHeader`
- `OutputFormat`
- `AkpError` (may need minor additions)

## Phase 2: Parser Rewrites (`src/parser.rs`)

### Top-level chunk routing

`parse_top_level_chunks` match block becomes:

```
"prg " → parse_program_header → program.header
"out " → parse_out_chunk → program.output          // NEW (was skipped)
"tune" → parse_tune_chunk → program.tuning          // NEW at top level
"lfo " → count-based: 0 → parse_lfo1_chunk, 1 → parse_lfo2_chunk  // MOVED to top level
"mods" → parse_mods_chunk → program.modulation       // MOVED to top level
"kgrp" → parse_keygroup → program.keygroups.push()
_      → skip with warning
```

### Keygroup-level chunk routing

`parse_keygroup` match block becomes:

```
"kloc" → parse_kloc_chunk                            // EXPANDED
"env " → count-based dispatch:
           0 → parse_amp_env_chunk → keygroup.amp_env
           1 → parse_filter_env_chunk → keygroup.filter_env
           2 → parse_aux_env_chunk → keygroup.aux_env
"filt" → parse_filt_chunk → keygroup.filter          // EXPANDED
"zone" → parse_zone_chunk → keygroup.zones.push()    // Returns Zone, multiple allowed
_      → skip with warning
```

Removed from keygroup: `"tune"`, `"smpl"`, `"lfo "`, `"mods"`.

Note: `"smpl"` is not in the spec. It was a fallback for third-party-created files. Keep it in the keygroup parser as a silent skip (no warning) for forward compatibility with third-party files, but do not extract data from it — zone chunks are the canonical source for sample names.

### Individual parser functions

Each function listed with its byte offset changes:

**`parse_program_header`** — No change. Offsets 1-2 are correct.

**`parse_out_chunk`** — NEW.
- Offset 1: loudness (u8)
- Offset 2: amp_mod_1 (u8)
- Offset 3: amp_mod_2 (u8)
- Offset 4: pan_mod_1 (u8)
- Offset 5: pan_mod_2 (u8)
- Offset 6: pan_mod_3 (u8)
- Offset 7: velocity_sensitivity (i8)
- Minimum chunk size: 8

**`parse_tune_chunk`** — REWRITTEN.
- Offset 1: semitone (i8)
- Offset 2: fine (i8)
- Offsets 3-14: detune[0..12] (i8 array)
- Offset 15: pitchbend_up (u8)
- Offset 16: pitchbend_down (u8)
- Offset 17: bend_mode (u8)
- Offset 18: aftertouch (i8)
- Minimum chunk size: 19

**`parse_lfo1_chunk`** — REWRITTEN (was `parse_lfo_chunk`).
- Seek to offset 1 (was 5).
- Offset 1: waveform (u8)
- Offset 2: rate (u8)
- Offset 3: delay (u8)
- Offset 4: depth (u8)
- Offset 5: sync (u8)
- Offset 6: marker (skip)
- Offset 7: modwheel (u8)
- Offset 8: aftertouch (u8)
- Offset 9: rate_mod (i8)
- Offset 10: delay_mod (i8)
- Offset 11: depth_mod (i8)
- retrigger = 0 (not present in LFO 1)
- Minimum chunk size: 12

**`parse_lfo2_chunk`** — NEW (different layout at offsets 5-8).
- Offsets 1-4: same as LFO 1 (waveform, rate, delay, depth)
- Offset 5: reserved (skip)
- Offset 6: retrigger (u8)
- Offset 7-8: reserved (skip — NOT modwheel/aftertouch)
- Offset 9: rate_mod (i8)
- Offset 10: delay_mod (i8)
- Offset 11: depth_mod (i8)
- sync = 0, modwheel = 0, aftertouch = 0 (not present in LFO 2)
- Minimum chunk size: 12

**`parse_mods_chunk`** — REWRITTEN.
- Single 38-byte flat block. Source bytes at odd offsets starting from 5, with padding bytes between.
- Offsets: 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35, 37
- Minimum chunk size: 38

**`parse_kloc_chunk`** — EXPANDED.
- Offsets 4-5: low_key, high_key (unchanged)
- Offset 6: semitone_tune (i8)
- Offset 7: fine_tune (i8)
- Offset 8: override_fx (u8)
- Offset 9: fx_send_level (u8)
- Offset 10: pitch_mod_1 (i8)
- Offset 11: pitch_mod_2 (i8)
- Offset 12: amp_mod (i8)
- Offset 13: zone_crossfade (u8)
- Offset 14: mute_group (u8)
- Minimum chunk size: 18 (unchanged)

**`parse_amp_env_chunk`** — REWRITTEN (was `parse_env_chunk`).
- Non-sequential reads:
- Offset 1: attack (u8)
- Offset 3: decay (u8)
- Offset 4: release (u8)
- Offset 7: sustain (u8)
- Offset 10: velocity_attack (i8)
- Offset 12: keyscale (i8)
- Offset 14: on_vel_release (i8)
- Offset 15: off_vel_release (i8)
- Minimum chunk size: 18

**`parse_filter_env_chunk`** — NEW.
- Same offsets as amp env, plus:
- Offset 9: depth (i8)
- Minimum chunk size: 18

**`parse_aux_env_chunk`** — NEW.
- Sequential reads from offset 1:
- Offsets 1-4: rate_1 through rate_4 (u8)
- Offsets 5-8: level_1 through level_4 (u8)
- Offset 10: vel_rate_1 (i8)
- Offset 12: key_rate_2_4 (i8)
- Offset 14: vel_rate_4 (i8)
- Offset 15: off_vel_rate_4 (i8)
- Offset 16: vel_output_level (i8)
- Minimum chunk size: 18

**`parse_filt_chunk`** — EXPANDED.
- Offsets 1-3 unchanged (filter_type, cutoff, resonance)
- Offset 4: keyboard_track (i8)
- Offset 5: mod_input_1 (i8)
- Offset 6: mod_input_2 (i8)
- Offset 7: mod_input_3 (i8)
- Offset 8: headroom (u8)
- Minimum chunk size: 4 → 9

**`parse_zone_chunk`** — EXPANDED.
- Offset 1: name_len (u8). If 0, skip zone (sample parameter block).
- Offsets 2-21: sample name (20 bytes, was 14)
- Offset 34: low_vel (u8)
- Offset 35: high_vel (u8)
- Offset 36: fine_tune (i8)
- Offset 37: semitone_tune (i8)
- Offset 38: filter (i8)
- Offset 39: pan (i8)
- Offset 40: playback (u8)
- Offset 41: output (u8)
- Offset 42: level (i8)
- Offset 43: keyboard_track (u8)
- Offsets 44-45: vel_to_start (i16, little-endian)
- Returns `Option<Zone>` (None if name_len=0)
- Validate: if name_len > 20, return error (buffer overflow protection)
- Minimum chunk size: 2 for initial name_len read. If name_len > 0, enforce chunk_size >= 46 before reading further fields.
- Note: real files may have 46 or 48 byte zones (ConvertWithMoss observed both). Accept either size — read up to offset 45, ignore trailing bytes.

**`sanitize_sample_path`** — No change.

## Phase 3: Output Generator Updates

### SFZ (`src/sfz.rs`)

**Structural changes:**
- Iterate `keygroup.zones` instead of checking `keygroup.sample`. Each zone with a sample name becomes a `<region>`.
- Key range from keygroup, velocity range from zone.

**Value fixes:**
- Filter type 0 is now "2-pole LP" (active filter). Remove the `if filter_type > 0` guard. All filter types are active.
- Full filter type mapping (26 types → SFZ equivalents, see docs/akp-format-reference.md).
- Resonance scaling: `(resonance as f32 / 12.0) * 40.0` (was `/ 100.0`). This is linear; ConvertWithMoss uses cube-root scaling. Linear is a reasonable approximation — may need tuning against real presets.
- LFO waveform: fix `waveform_name()` to match spec enum (0=sine, 1=triangle, add 3-8).
- Envelope values: now reading correct offsets, sustain/release no longer swapped.

**New fields wired up:**
- `program.output.loudness` → global `volume` header or per-region offset
- `program.tuning.pitchbend_up/down` → `bend_up`, `bend_down` (was hardcoded 200)
- `zone.pan` → `pan` opcode
- `zone.level` → `volume` offset
- `zone.playback` → `loop_mode` mapping (0=no_loop, 1=one_shot, 2=loop_continuous, 3=loop_sustain, 4=use sample header)
- `zone.fine_tune` / `zone.semitone_tune` → `tune`, `transpose`
- `keygroup.semitone_tune` / `keygroup.fine_tune` from kloc → combined with zone tune
- Envelope velocity modulation → `ampeg_vel2attack`, etc.
- Filter env depth → `fileg_depth` (was hardcoded 2400)

**Removed:**
- Hardcoded `bend_up=200` / `bend_down=-200` → use program tuning values
- Hardcoded `loop_mode=loop_continuous` → use zone playback field
- Old `Tune.volume_db()` helper → replaced by `ProgramOutput.loudness` scaling

### Decent Sampler (`src/dspreset.rs`)

Same core fixes as SFZ, plus DS-specific mappings:
- Envelope values corrected (same offset fixes)
- Filter/resonance scaling fixed (same formula)
- LFO waveform mapping fixed (same enum)
- Zone iteration: each zone becomes a `<sample>` within a `<group>`, key range on group, velocity range on sample
- `ProgramOutput.loudness` → `<DecentSampler>` global volume attribute
- `ProgramOutput.velocity_sensitivity` → velocity curve/sensitivity on sample elements
- Zone-level tuning (`zone.fine_tune`, `zone.semitone_tune`) applied per-sample, combined additively with keygroup tuning from kloc

### Conversion helpers (`src/types.rs`)

- `Lfo::waveform_name()` — fix enum mapping, add values 3-8
- `Lfo::rate_hz()` — unchanged
- `Envelope::attack_time()`, `decay_time()`, `release_time()`, `sustain_normalized()` — unchanged (time curve is a guess, but consistent)
- Remove `Tune::volume_db()` — volume concept moves to output loudness
- Add `ProgramOutput::volume_db()` — `(loudness / 100.0) * 66.0 - 60.0` or similar
- Add `Filter::cutoff_hz()` — move cutoff conversion from sfz.rs to types.rs for reuse

## Phase 4: Unit Tests (`src/parser.rs`)

Every parser test rewritten with spec-correct byte layouts.

### Tests to rewrite:
- `test_parse_zone_*` — 20-byte name buffer, return Zone struct, multi-zone
- `test_parse_filt_*` — add extra fields, threshold at 26
- `test_parse_env_*` → split into `test_parse_amp_env_*`, `test_parse_filter_env_*`, `test_parse_aux_env_*`
- `test_parse_lfo_*` — new at correct offsets

### New tests:
- `test_parse_out_chunk` — loudness, velocity sensitivity
- `test_parse_tune_chunk` — semitone, fine, pitchbend, detune array
- `test_parse_mods_chunk` — 38-byte flat block, source values at odd offsets
- `test_parse_kloc_expanded` — all fields including tune, fx, crossfade
- `test_parse_filter_env_with_depth` — filter env depth field
- `test_parse_aux_env_rate_level` — rate/level layout, not ADSR
- `test_parse_zone_multi` — 4 zones in one keygroup
- `test_parse_zone_20char_name` — sample name at max length

### SFZ/dspreset tests:
- Update all existing tests for new struct constructors (Zone instead of Sample, etc.)
- Add test for filter type 0 producing `lpf_2p` output
- Add test for resonance scaling with 0-12 input range
- Add test for multi-zone keygroup producing multiple `<region>` blocks

### Integration tests (`tests/integration_tests.rs`):
- No changes expected. They test error paths (invalid RIFF, missing keygroup) which are unaffected.

### Real file verification:
- Run all 4 S6000 factory files after rewrite
- Confirm: amp envelope sustain ~2 (not 39) for piano presets
- Confirm: LFO waveform is triangle (value 1) not sine
- Confirm: sample names unchanged (still within 14 chars for these files)
- Spot-check SFZ output for correct pitchbend range, filter type, resonance values

## Phase 5: Test Data Generator (`create_test_akp.py`)

Update to emit all chunk types with spec-correct layouts:
- prg + out + tune + lfo + lfo + mods at top level
- kgrp containing kloc + 3x env + filt + 4x zone
- All fields at correct offsets with realistic default values

## Deferred (out of scope)

- Z4/Z8 extra filter chunks
- MPC4000 pad assignment chunk
- Envelope time curve accuracy (keep current exponential, document uncertainty)
- Program-level mods → SFZ modulation routing mapping (parse and store, defer output mapping to future work)
- RIFF size=0 detection for S5000 vs Z4/Z8 distinction

## Files Modified

- `src/types.rs` — Complete struct rewrite
- `src/parser.rs` — All parser functions rewritten, top-level routing changed
- `src/sfz.rs` — Output generator updated for new structs + value fixes
- `src/dspreset.rs` — Output generator updated for new structs + value fixes
- `src/lib.rs` — Re-exports updated for new/renamed types. Public API: `AkaiProgram`, `OutputFormat`, `AkpError`, `Result`, `validate_riff_header`, `parse_top_level_chunks`, `convert_file`. Internal types (Zone, Envelope, etc.) don't need public re-export — CLI and GUI only use `convert_file` and `OutputFormat`.
- `create_test_akp.py` — Updated for correct chunk layouts

## Files NOT Modified

- `src/error.rs` — No new error variants needed
- `src/bin/cli.rs` — Uses `convert_file()`, unaffected by internal struct changes
- `gui/src/main.rs` — Uses `convert_file()`, unaffected
- `tests/integration_tests.rs` — Tests error paths, unaffected

## Verification Checklist

1. `cargo test` — all tests pass
2. `cargo clippy` — clean on both crates (library + gui)
3. `cd gui && cargo check` — GUI compiles with new types
4. Run 4 real S6000 AKP files — all parse without errors
5. Spot-check SFZ: envelope sustain ~2 for piano, correct pitchbend, filter type, resonance
6. Spot-check Decent Sampler: same value correctness
7. No regressions: files that parsed before still parse
