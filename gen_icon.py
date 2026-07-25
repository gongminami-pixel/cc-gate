#!/usr/bin/env python3
"""Generate cc-x-llm app icon: 2 left dots + 3 right dots + 6 connecting lines."""

import struct, zlib, math, os

SIZE = 512
OUT_DIR = os.path.join(os.path.dirname(__file__), "src-tauri", "icons")
os.makedirs(OUT_DIR, exist_ok=True)

# ── Colors ────────────────────────────────────────────────────
BG         = (0, 0, 0, 0)          # transparent
DOT_LEFT   = (10, 132, 255, 255)   # macOS blue (agent)
DOT_RIGHT  = (142, 142, 147, 255)  # gray (LLM)
LINE       = (142, 142, 147, 200)   # gray semi-transparent line (thicker)

# ── Geometry ─────��────────────────────────────────────────────
margin = 80
left_x = margin + 60
right_x = SIZE - margin - 60
left_dots  = [(left_x, 190), (left_x, 322)]
right_dots = [(right_x, 130), (right_x, 256), (right_x, 382)]
dot_radius = 30
line_width = 5.5

# ── Anti-aliasing helpers ──────────────────────────────────────
def circle_mask(cx, cy, r, x, y):
    d = math.hypot(x - cx, y - cy)
    if d < r - 1.0: return 1.0
    if d > r + 1.0: return 0.0
    return max(0.0, (r + 1.0 - d) / 2.0)

def draw_line(pixels, x1, y1, x2, y2, color, width):
    """Anti-aliased thick line from (x1,y1) to (x2,y2)."""
    dx, dy = x2 - x1, y2 - y1
    length = math.hypot(dx, dy)
    if length < 1: return
    ux, uy = dx / length, dy / length
    nx, ny = -uy, ux  # perpendicular

    x0, y0 = int(min(x1, x2) - width - 2), int(min(y1, y2) - width - 2)
    x1e, y1e = int(max(x1, x2) + width + 2), int(max(y1, y2) + width + 2)

    for y in range(max(0, y0), min(SIZE, y1e)):
        for x in range(max(0, x0), min(SIZE, x1e)):
            px, py = x - x1, y - y1
            proj = px * ux + py * uy
            if 0 <= proj <= length:
                dist = abs(px * nx + py * ny)
                if dist <= width + 1.0:
                    alpha = 1.0 if dist <= width - 1.0 else max(0.0, (width + 1.0 - dist) / 2.0)
                    blend_pixel(pixels, x, y, color, alpha)

def blend_pixel(pixels, x, y, color, alpha):
    """Alpha-blend color onto pixel (x,y)."""
    if x < 0 or x >= SIZE or y < 0 or y >= SIZE: return
    cr, cg, cb, ca = color
    pr, pg, pb, pa = pixels[y][x]
    a = alpha * ca / 255.0
    # premultiplied alpha blend
    nr = int(pr * (1 - a) + cr * a)
    ng = int(pg * (1 - a) + cg * a)
    nb = int(pb * (1 - a) + cb * a)
    na = max(pa, int(ca * alpha))
    pixels[y][x] = (min(255, nr), min(255, ng), min(255, nb), min(255, na))

# ── Build pixel buffer ────────────────────────────────────────
pixels = [[BG for _ in range(SIZE)] for _ in range(SIZE)]

# Draw 6 connecting lines
for (lx, ly) in left_dots:
    for (rx, ry) in right_dots:
        draw_line(pixels, lx, ly, rx, ry, LINE, line_width)

# Draw dots (on top of lines for clean look)
for (dx, dy) in left_dots:
    for y in range(max(0, dy - dot_radius - 2), min(SIZE, dy + dot_radius + 2)):
        for x in range(max(0, dx - dot_radius - 2), min(SIZE, dx + dot_radius + 2)):
            m = circle_mask(dx, dy, dot_radius, x, y)
            if m > 0:
                blend_pixel(pixels, x, y, DOT_LEFT, min(1.0, m))

for (dx, dy) in right_dots:
    for y in range(max(0, dy - dot_radius - 2), min(SIZE, dy + dot_radius + 2)):
        for x in range(max(0, dx - dot_radius - 2), min(SIZE, dx + dot_radius + 2)):
            m = circle_mask(dx, dy, dot_radius, x, y)
            if m > 0:
                blend_pixel(pixels, x, y, DOT_RIGHT, min(1.0, m))

# ── Encode PNG ─────────────────────────────────────────────────
def make_png(w, h, pixels):
    def chunk(ctype, data):
        c = ctype + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)
    raw = b''
    for row in pixels:
        raw += b'\x00'
        for r, g, b, a in row:
            raw += struct.pack('BBBB', r, g, b, a)
    ihdr = struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr) + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b'')

def scale_down(big, sz):
    ratio = SIZE / sz
    small = []
    for y in range(sz):
        row = []
        for x in range(sz):
            sx, sy = int(x * ratio), int(y * ratio)
            row.append(big[sy][sx])
        small.append(row)
    return make_png(sz, sz, small)

# ── Write all sizes ───────────────────────────────────────────
for name, sz in [("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512)]:
    data = make_png(sz, sz, pixels) if sz == SIZE else scale_down(pixels, sz)
    path = os.path.join(OUT_DIR, name)
    with open(path, "wb") as f:
        f.write(data)
    print(f"  {name}  {sz}×{sz}  {len(data):,} bytes")

# icon.icns (just copy png for Tauri compatibility)
import shutil
shutil.copy(os.path.join(OUT_DIR, "icon.png"), os.path.join(OUT_DIR, "icon.icns"))
print(f"  icon.icns")
print(f"\nDone! {len(os.listdir(OUT_DIR))} icon files in {OUT_DIR}")
