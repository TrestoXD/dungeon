#!/usr/bin/env python3
"""Convierte PNG a bytes RGBA crudos (assets/<nombre>.rgba): 4 bytes por píxel, R G B A.

Uso:
    python3 tools/convert_textures.py ruta/a/waifu.png          # un archivo suelto
    python3 tools/convert_textures.py <carpeta> nombre1 nombre2  # varios de una carpeta

Cualquier PNG vale (RGB, RGBA, paleta, gris); si no tiene alpha se rellena con 255.
Paredes, suelo y techo necesitan lado potencia de 2; los sprites pueden tener cualquier tamaño.
Requiere Pillow.
"""
import sys
from pathlib import Path
from PIL import Image

out = Path(__file__).resolve().parent.parent / "assets"
out.mkdir(exist_ok=True)

args = sys.argv[1:]
if not args:
    sys.exit(__doc__)

first = Path(args[0])
if first.suffix.lower() == ".png":
    files = [first]
else:
    names = args[1:] or ["cobblestone", "stone"]
    files = [first / f"{n}.png" for n in names]

for f in files:
    im = Image.open(f).convert("RGBA")
    w, h = im.size
    dest = out / f"{f.stem}.rgba"
    dest.write_bytes(im.tobytes())
    a = im.getchannel("A").tobytes()
    pow2 = w & (w - 1) == 0 and h & (h - 1) == 0
    print(f"{dest.name}: {w}x{h}  {w*h*4} bytes  alpha {min(a)}..{max(a)}"
          f"  {'(potencia de 2: vale para paredes)' if pow2 else '(solo para sprites)'}")
    print(f"   -> en sprite.rs:  const WAIFU_SIZE: Option<(usize, usize)> = Some(({w}, {h}));")