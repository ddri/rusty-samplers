#!/usr/bin/env python3
"""
Creates a minimal valid AKP file for testing the rusty-samplers converter.
This creates the basic RIFF/APRG structure that the parser expects.
"""

import struct

def create_test_akp():
    # RIFF header
    riff_header = b'RIFF'
    
    # APRG signature 
    aprg_sig = b'APRG'
    
    # Minimal program chunk (prg)
    prg_chunk = b'prg\x00'  # chunk ID
    prg_size = struct.pack('<L', 8)  # 8 bytes of data
    prg_data = b'\x01\x01\x00\x00\x00\x00\x00\x00'  # minimal program data
    
    # Calculate total file size
    content = aprg_sig + prg_chunk + prg_size + prg_data
    file_size = struct.pack('<L', len(content))
    
    # Complete file
    akp_data = riff_header + file_size + content
    
    with open('test_sample.akp', 'wb') as f:
        f.write(akp_data)
    
    print(f"✅ Created test_sample.akp ({len(akp_data)} bytes)")
    print("🧪 This file can be used to test the AKP parser")

if __name__ == "__main__":
    create_test_akp()