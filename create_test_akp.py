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

    # out chunk (8 bytes)
    out_data = bytearray(8)
    out_data[1] = 85   # loudness
    out_data[7] = 25   # velocity_sensitivity (i8)
    out_chunk = make_chunk(b'out ', bytes(out_data))

    # tune chunk (22 bytes)
    tune_data = bytearray(22)
    tune_data[15] = 2   # pitchbend_up
    tune_data[16] = 2   # pitchbend_down
    tune_chunk = make_chunk(b'tune', bytes(tune_data))

    # lfo chunk 1 (12 bytes)
    lfo1_data = bytearray(12)
    lfo1_data[1] = 1   # waveform (TRIANGLE)
    lfo1_data[2] = 30  # rate
    lfo1_data[4] = 50  # depth
    lfo1_chunk = make_chunk(b'lfo ', bytes(lfo1_data))

    # lfo chunk 2 (12 bytes)
    lfo2_data = bytearray(12)
    lfo2_data[1] = 0   # waveform (SINE)
    lfo2_chunk = make_chunk(b'lfo ', bytes(lfo2_data))

    # mods chunk (38 bytes)
    mods_data = bytearray(38)
    mods_data[5] = 6    # amp_mod_1_source = KEYBOARD
    mods_data[27] = 7   # pitch_mod_1_source = LFO1
    mods_data[31] = 5   # amp_mod_source = VELOCITY
    mods_chunk = make_chunk(b'mods', bytes(mods_data))

    # -- Keygroup contents --

    # kloc chunk (16 bytes)
    kloc_data = bytearray(16)
    kloc_data[4] = 36   # low_key
    kloc_data[5] = 96   # high_key
    kloc_chunk = make_chunk(b'kloc', bytes(kloc_data))

    # amp env (18 bytes): attack=1, decay=3, release=4, sustain=7
    amp_env_data = bytearray(18)
    amp_env_data[1] = 10   # attack
    amp_env_data[3] = 50   # decay
    amp_env_data[4] = 30   # release
    amp_env_data[7] = 80   # sustain
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

    # filt chunk (10 bytes)
    filt_data = bytearray(10)
    filt_data[1] = 0    # filter_type (2-pole LP)
    filt_data[2] = 75   # cutoff
    filt_data[3] = 6    # resonance (0-12 range)
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
