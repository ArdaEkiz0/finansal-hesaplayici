// Sifir bagimlilikla logo uretir: build/icon.ico (16/32/48/256) + public/icon.png (512)
// Tasarim: public/logo.svg ile ayni — mavi-mor gradient, yuvarlak kare, beyaz ₺ isareti.
const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

const ROOT = path.join(__dirname, "..");
const OUT_DIR = path.join(ROOT, "build");
fs.mkdirSync(OUT_DIR, { recursive: true });

const C0 = [0x3b, 0x82, 0xf6]; // #3b82f6
const C1 = [0x8b, 0x5c, 0xf6]; // #8b5cf6
const RADIUS = 26 / 120;

// ₺ isareti (normalize koordinat, logo.svg oranlari)
function glyph(x, y) {
  const inRect = (x0, x1, y0, y1) => x >= x0 && x < x1 && y >= y0 && y < y1;
  if (inRect(0.33, 0.67, 0.325, 0.375)) return true; // ust cubuk
  if (inRect(0.465, 0.535, 0.325, 0.60)) return true; // govde
  if (inRect(0.38, 0.62, 0.51, 0.545)) return true; // alt cizgi 1
  if (inRect(0.38, 0.62, 0.555, 0.59)) return true; // alt cizgi 2
  return false;
}

function roundedRect(x, y) {
  const r = RADIUS;
  const cx = Math.min(Math.max(x, r), 1 - r);
  const cy = Math.min(Math.max(y, r), 1 - r);
  const dx = x - cx;
  const dy = y - cy;
  return dx * dx + dy * dy <= r * r;
}

function render(size) {
  const SS = 4;
  const big = size * SS;
  const acc = new Float32Array(size * size * 4);
  for (let py = 0; py < big; py++) {
    const y = (py + 0.5) / big;
    for (let px = 0; px < big; px++) {
      const x = (px + 0.5) / big;
      let r = 0, g = 0, b = 0, a = 0;
      if (roundedRect(x, y)) {
        const t = Math.min(1, Math.max(0, (x + y) / 2));
        r = C0[0] + (C1[0] - C0[0]) * t;
        g = C0[1] + (C1[1] - C0[1]) * t;
        b = C0[2] + (C1[2] - C0[2]) * t;
        a = 255;
        if (glyph(x, y)) {
          r = 255; g = 255; b = 255;
        }
      }
      const ox = Math.floor(px / SS);
      const oy = Math.floor(py / SS);
      const o = (oy * size + ox) * 4;
      acc[o] += r; acc[o + 1] += g; acc[o + 2] += b; acc[o + 3] += a;
    }
  }
  const n = SS * SS;
  const out = Buffer.alloc(size * size * 4);
  for (let k = 0; k < size * size * 4; k++) out[k] = Math.round(acc[k] / n);
  return out;
}

// --- minimal PNG kodlayici (RGBA, filtre 0) ---
const CRC_TABLE = (() => {
  const t = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c;
  }
  return t;
})();

function crc32(buf) {
  let c = -1;
  for (let k = 0; k < buf.length; k++) c = CRC_TABLE[(c ^ buf[k]) & 0xff] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}

function chunk(type, data) {
  const td = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([td, data])), 0);
  return Buffer.concat([len, td, data, crc]);
}

function encodePNG(size, rgba) {
  const raw = Buffer.alloc(size * (1 + size * 4));
  for (let y = 0; y < size; y++) {
    raw[y * (1 + size * 4)] = 0;
    rgba.copy(raw, y * (1 + size * 4) + 1, y * size * 4, (y + 1) * size * 4);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// --- ICO (PNG sikistirmali girdiler) ---
function encodeICO(pngs) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngs.length, 4);
  const dir = Buffer.alloc(16 * pngs.length);
  let offset = 6 + 16 * pngs.length;
  pngs.forEach((p, k) => {
    const o = k * 16;
    dir[o] = p.size >= 256 ? 0 : p.size;
    dir[o + 1] = p.size >= 256 ? 0 : p.size;
    dir[o + 2] = 0; dir[o + 3] = 0;
    dir.writeUInt16LE(1, o + 4);
    dir.writeUInt16LE(32, o + 6);
    dir.writeUInt32LE(p.data.length, o + 8);
    dir.writeUInt32LE(offset, o + 12);
    offset += p.data.length;
  });
  return Buffer.concat([header, dir, ...pngs.map((p) => p.data)]);
}

const sizes = [256, 48, 32, 16];
const pngs = sizes.map((s) => ({ size: s, data: encodePNG(s, render(s)) }));
fs.writeFileSync(path.join(OUT_DIR, "icon.ico"), encodeICO(pngs));
fs.writeFileSync(path.join(ROOT, "public", "icon.png"), encodePNG(512, render(512)));
for (const p of pngs) console.log(`icon ${p.size}x${p.size}: ${(p.data.length / 1024).toFixed(1)} KB`);
console.log("OK: build/icon.ico + public/icon.png");
