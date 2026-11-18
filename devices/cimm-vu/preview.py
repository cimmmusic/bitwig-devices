"""Render panel.sksl offline with Skia: previews/*.png (no Bitwig needed)."""
import struct, sys
from pathlib import Path
import skia

HERE = Path(__file__).resolve().parent
src = (HERE / "panel.sksl").read_text()
eff = skia.RuntimeEffect.MakeForShader("uniform float3 iResolution;\n" + src)
if eff is None:
    sys.exit("shader failed to compile")
W, H = 982, 328
(HERE / "previews").mkdir(exist_ok=True)
for name, vals in {
    "vu-rest": [0, 0, 0, 0],
    "vu-0vu": [1.0, 0.8, 1.26 + 4, 1.05],          # L at 0 VU, hold +2, amber LED
    "vu-hot": [1.9, 1.5, 2.1 + 8, 1.8 + 4],        # near +6, red / amber LEDs
    "vu-quiet": [0.18, 0.25, 0.35, 0.4],           # around -15 / -12 VU
}.items():
    shader = eff.makeShader(skia.Data.MakeWithCopy(struct.pack("7f", W, H, 1.0, *vals)))
    surface = skia.Surface(W, H)
    paint = skia.Paint()
    paint.setShader(shader)
    surface.getCanvas().drawRect(skia.Rect(0, 0, W, H), paint)
    surface.makeImageSnapshot().save(str(HERE / "previews" / f"{name}.png"))
print("rendered")
