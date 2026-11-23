"""Makes the repo copies (ui-v3, 1440 wide, 256 colours) and the small embedded copies (JPEG) of the screens."""
import base64
import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "target" / "p1" / "ui3"
DST = ROOT / "docs" / "design" / "ui-v3"
EMB = SRC / "embed"
DST.mkdir(parents=True, exist_ok=True)
EMB.mkdir(parents=True, exist_ok=True)

b64 = {}
total_png = total_jpg = 0
for png in sorted(SRC.glob("*.png")):
    im = Image.open(png).convert("RGB")
    big = im.resize((1440, round(im.height * 1440 / im.width)), Image.LANCZOS)
    out = DST / png.name
    big.quantize(256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(out, optimize=True)
    total_png += out.stat().st_size
    small = im.resize((1000, round(im.height * 1000 / im.width)), Image.LANCZOS)
    jpg = EMB / (png.stem + ".jpg")
    small.save(jpg, quality=74, optimize=True)
    total_jpg += jpg.stat().st_size
    b64[png.stem] = base64.b64encode(jpg.read_bytes()).decode("ascii")
    print(f"{png.name}: png {out.stat().st_size // 1024} KB, jpg {jpg.stat().st_size // 1024} KB")
(EMB / "b64.json").write_text(json.dumps(b64), encoding="utf-8")
print(f"repo PNGs {total_png // 1024} KB; embedded JPEGs {total_jpg // 1024} KB")
