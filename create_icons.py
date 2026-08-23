import struct, zlib

def create_png(path, width=32, height=32):
    def chunk(chunk_type, data):
        c = chunk_type + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xFFFFFFFF)
    
    header = b'\x89PNG\r\n\x1a\n'
    ihdr = chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
    raw = b''
    for y in range(height):
        raw += b'\x00'  # filter none
        for x in range(width):
            raw += bytes([59, 130, 246, 255])  # blue EF color RGBA
    idat = chunk(b'IDAT', zlib.compress(raw))
    iend = chunk(b'IEND', b'')
    
    with open(path, 'wb') as f:
        f.write(header + ihdr + idat + iend)

create_png('/Users/macbookpro/Project/email-forensic-desktop/src-tauri/icons/32x32.png', 32, 32)
create_png('/Users/macbookpro/Project/email-forensic-desktop/src-tauri/icons/128x128.png', 128, 128)
create_png('/Users/macbookpro/Project/email-forensic-desktop/src-tauri/icons/128x128@2x.png', 256, 256)
create_png('/Users/macbookpro/Project/email-forensic-desktop/src-tauri/icons/icon.png', 512, 512)
print('Icons created')
