"""Generate the development icon from simple repository-owned geometric shapes."""
import struct
import zlib
from pathlib import Path
size=128
pixels=[]
for y in range(size):
    pixels.append(0)
    for x in range(size):
        color=(21,43,69,255)
        if (x-23)**2+(y-23)**2<4:color=(21,43,69,255)
        line=(35<=x<=41 and 32<=y<=96) or (38<=x<=81 and 59<=y<=65) or (77<=x<=83 and 31<=y<=62)
        circle=any((x-cx)**2+(y-cy)**2<=11**2 for cx,cy in [(38,28),(38,99),(80,28)])
        inner=any((x-cx)**2+(y-cy)**2<=5**2 for cx,cy in [(38,28),(38,99),(80,28)])
        if line or circle:color=(100,180,255,255)
        if inner:color=(21,43,69,255)
        pixels.extend(color)
def chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
path=Path(__file__).resolve().parents[1]/'src-tauri/icons/icon.png'
path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',size,size,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(bytes(pixels)))+chunk(b'IEND',b''))
