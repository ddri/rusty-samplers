#!/usr/bin/env python3
"""
Creates a minimal valid AKP file for testing the rusty-samplers converter.
Generates all spec chunk types with correct byte layouts.
"""

import struct


def make_chunk(chunk_id: bytes, data: bytes) -> bytes:
    """Build a RIFF chunk: 4-byte ID + LE uint32 size + data."""
    return chunk_id + struct.pack('<I', len(data)) + data


def create_test_akp():
    # prg chunk (6 bytes): byte 0=flags, 1=MIDI pgm#, 2=# keygroups
    prg_data = bytearray(6)
    prg_data[1] = 1   # MIDI program number
    prg_data[2] = 1   # 1 keygroup
    prg_chunk = make_chunk(b'prg ', bytes(prg_data))

    # out chunk (8 bytes): loudness, amp_mod_1, amp_mod_2, pan_mod_1-3, vel_sens
    out_data = bytearray(8)
    out_data[1] = 85   # loudness
    out_data[2] = 30   # amp_mod_1 amount (u8, 0-100)
    out_data[3] = 20   # amp_mod_2 amount (u8, 0-100)
    out_data[4] = 40   # pan_mod_1 amount (u8, 0-100)
    out_data[5] = 25   # pan_mod_2 amount (u8, 0-100)
    out_data[6] = 50   # pan_mod_3 amount (u8, 0-100)
    out_data[7] = 25   # velocity_sensitivity (i8)
    out_chunk = make_chunk(b'out ', bytes(out_data))

    # tune chunk (22 bytes): semitone, fine, 12 detune, pbend_up, pbend_down, bend_mode, aftertouch
    tune_data = bytearray(22)
    tune_data[15] = 2   # pitchbend_up (semitones)
    tune_data[16] = 2   # pitchbend_down (semitones)
    tune_data[18] = 3   # aftertouch -> pitch (i8, -12..+12 semitones) — hardwired route
    tune_chunk = make_chunk(b'tune', bytes(tune_data))

    # lfo chunk 1 (12 bytes): waveform, rate, delay, depth, sync, -, modwheel, aftertouch, rate_mod, delay_mod, depth_mod
    lfo1_data = bytearray(12)
    lfo1_data[1] = 1   # waveform (TRIANGLE)
    lfo1_data[2] = 30  # rate
    lfo1_data[4] = 50  # depth
    lfo1_data[7] = 60  # modwheel shortcut (u8, 0-100) — hardwired route
    lfo1_data[8] = 40  # aftertouch shortcut (u8, 0-100) — hardwired route
    lfo1_data[9] = 15  # rate_mod amount (i8) — flexible route amount
    lfo1_data[10] = 10 # delay_mod amount (i8)
    lfo1_data[11] = 20 # depth_mod amount (i8)
    lfo1_chunk = make_chunk(b'lfo ', bytes(lfo1_data))

    # lfo chunk 2 (12 bytes)
    lfo2_data = bytearray(12)
    lfo2_data[1] = 0   # waveform (SINE)
    lfo2_chunk = make_chunk(b'lfo ', bytes(lfo2_data))

    # mods chunk (38 bytes): source assignments at odd offsets 5-37
    mods_data = bytearray(38)
    mods_data[5] = 6    # amp_mod_1_source = KEYBOARD (6)
    mods_data[7] = 3    # amp_mod_2_source = AFTERTOUCH (3)
    mods_data[9] = 8    # pan_mod_1_source = LFO2 (8)
    mods_data[11] = 6   # pan_mod_2_source = KEYBOARD (6)
    mods_data[13] = 1   # pan_mod_3_source = MODWHEEL (1)
    mods_data[15] = 6   # lfo1_rate_mod_source = KEYBOARD (6)
    mods_data[27] = 7   # pitch_mod_1_source = LFO1 (7)
    mods_data[29] = 11  # pitch_mod_2_source = AUX_ENV (11)
    mods_data[31] = 5   # amp_mod_source = VELOCITY (5)
    mods_data[33] = 5   # filter_mod_1_source = VELOCITY (5)
    mods_data[35] = 8   # filter_mod_2_source = LFO2 (8)
    mods_data[37] = 9   # filter_mod_3_source = AMP_ENV (9)
    mods_chunk = make_chunk(b'mods', bytes(mods_data))

    # -- Keygroup contents --

    # kloc chunk (16 bytes): low_key, high_key, semitone, fine, override_fx, fx_send,
    #   pitch_mod_1, pitch_mod_2, amp_mod, zone_crossfade, ...
    kloc_data = bytearray(16)
    kloc_data[4] = 36   # low_key
    kloc_data[5] = 96   # high_key
    kloc_data[10] = 50  # pitch_mod_1 amount (i8, -100..+100) — flexible route
    kloc_data[11] = 30  # pitch_mod_2 amount (i8, -100..+100) — flexible route
    kloc_data[12] = 80  # amp_mod amount (i8, -100..+100) — flexible route (velocity)
    kloc_chunk = make_chunk(b'kloc', bytes(kloc_data))

    # amp env (18 bytes): attack, -, decay, release, -, -, -, sustain, -, -,
    #   vel_attack, -, key_scale, -, on_vel_release, off_vel_release, ...
    amp_env_data = bytearray(18)
    amp_env_data[1] = 10   # attack
    amp_env_data[3] = 50   # decay
    amp_env_data[4] = 30   # release
    amp_env_data[7] = 80   # sustain
    amp_env_data[10] = 20  # velocity_attack (i8, -100..+100) — hardwired route
    amp_env_data[12] = 15  # keyboard_scale (i8, -100..+100) — hardwired route
    amp_env_data[14] = 10  # on_vel_release (i8, -100..+100) — hardwired route
    amp_env_data[15] = 5   # off_vel_release (i8, -100..+100) — hardwired route
    amp_env_chunk = make_chunk(b'env ', bytes(amp_env_data))

    # filter env (18 bytes)
    filt_env_data = bytearray(18)
    filt_env_data[1] = 5    # attack
    filt_env_data[3] = 60   # decay
    filt_env_data[4] = 40   # release
    filt_env_data[7] = 70   # sustain
    filt_env_data[9] = 50   # depth (i8, positive)
    filt_env_chunk = make_chunk(b'env ', bytes(filt_env_data))

    # aux env (18 bytes)
    aux_env_data = bytearray(18)
    aux_env_data[1] = 10   # rate_1
    aux_env_data[5] = 100  # level_1
    aux_env_chunk = make_chunk(b'env ', bytes(aux_env_data))

    # filt chunk (10 bytes): filter_type, cutoff, resonance, keyboard_track,
    #   mod_input_1, mod_input_2, mod_input_3, headroom
    filt_data = bytearray(10)
    filt_data[1] = 0    # filter_type (2-pole LP)
    filt_data[2] = 75   # cutoff
    filt_data[3] = 6    # resonance (0-12 range)
    filt_data[4] = 10   # keyboard_track (i8, -100..+100)
    filt_data[5] = 40   # mod_input_1 (i8, -100..+100) — flexible route (velocity)
    filt_data[6] = 30   # mod_input_2 (i8, -100..+100) — flexible route (LFO2)
    filt_data[7] = 20   # mod_input_3 (i8, -100..+100) — flexible route (AMP_ENV)
    filt_chunk = make_chunk(b'filt', bytes(filt_data))

    # zone chunk (48 bytes)
    zone_data = bytearray(48)
    sample_name = b'Piano_C3'
    zone_data[1] = len(sample_name)
    zone_data[2:2 + len(sample_name)] = sample_name
    zone_data[34] = 1     # low_vel
    zone_data[35] = 127   # high_vel
    zone_data[40] = 4     # playback (AS SAMPLE)
    zone_data[43] = 1     # keyboard_track (ON)
    zone_chunk = make_chunk(b'zone', bytes(zone_data))

    # Assemble keygroup
    kgrp_inner = kloc_chunk + amp_env_chunk + filt_env_chunk + aux_env_chunk + filt_chunk + zone_chunk
    kgrp_chunk = make_chunk(b'kgrp', kgrp_inner)

    # Assemble RIFF/APRG file
    content = b'APRG' + prg_chunk + out_chunk + tune_chunk + lfo1_chunk + lfo2_chunk + mods_chunk + kgrp_chunk
    file_size = struct.pack('<I', len(content))
    akp_data = b'RIFF' + file_size + content

    with open('test_sample.akp', 'wb') as f:
        f.write(akp_data)

    print(f"Created test_sample.akp ({len(akp_data)} bytes)")


if __name__ == "__main__":
    create_test_akp()
