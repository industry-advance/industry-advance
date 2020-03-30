import zlib
import sys
import javaobj

with open(sys.argv[1], "rb") as f:
    decompressed = zlib.decompress(f.read())

# Look for magic header
if not decompressed.startswith("MSAV".encode("ASCII")):
    raise Exception("Invalid save file")
else:
    save = bytearray(decompressed)
    print(save[:4])  # Print header

# Version (not sure if format version or map version)
print(save[4])

with open("save", "wb") as f:
    f.write(save)
