import sharp from "sharp";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");
const source = path.join(iconsDir, "icon.png");

async function run() {
  // Load source and trim transparent/white padding
  const img = sharp(source);
  const meta = await img.metadata();
  console.log(`Source: ${meta.width}x${meta.height}`);

  // Trim whitespace around the dice
  const trimmed = await sharp(source).trim().toBuffer();
  const trimMeta = await sharp(trimmed).metadata();
  console.log(`Trimmed: ${trimMeta.width}x${trimMeta.height}`);

  // Add ~5% padding back (so the icon isn't edge-to-edge)
  const maxDim = Math.max(trimMeta.width, trimMeta.height);
  const pad = Math.round(maxDim * 0.05);
  const base = await sharp(trimmed)
    .extend({
      top: pad,
      bottom: pad,
      left: pad,
      right: pad,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .toBuffer();

  // Desktop icon sizes
  const sizes = [
    { name: "32x32.png", size: 32 },
    { name: "64x64.png", size: 64 },
    { name: "128x128.png", size: 128 },
    { name: "128x128@2x.png", size: 256 },
    { name: "icon.png", size: 1024 },
  ];

  // Windows Store logo sizes
  const storeSizes = [
    { name: "StoreLogo.png", size: 50 },
    { name: "Square30x30Logo.png", size: 30 },
    { name: "Square44x44Logo.png", size: 44 },
    { name: "Square71x71Logo.png", size: 71 },
    { name: "Square89x89Logo.png", size: 89 },
    { name: "Square107x107Logo.png", size: 107 },
    { name: "Square142x142Logo.png", size: 142 },
    { name: "Square150x150Logo.png", size: 150 },
    { name: "Square284x284Logo.png", size: 284 },
    { name: "Square310x310Logo.png", size: 310 },
  ];

  for (const { name, size } of [...sizes, ...storeSizes]) {
    await sharp(base)
      .resize(size, size, {
        fit: "contain",
        background: { r: 0, g: 0, b: 0, alpha: 0 },
      })
      .png()
      .toFile(path.join(iconsDir, name));
    console.log(`  ✓ ${name} (${size}x${size})`);
  }

  // Generate .ico (Windows) — 16, 32, 48, 256 sizes embedded
  const icoSizes = [16, 32, 48, 256];
  const icoBuffers = await Promise.all(
    icoSizes.map((s) =>
      sharp(base)
        .resize(s, s, {
          fit: "contain",
          background: { r: 0, g: 0, b: 0, alpha: 0 },
        })
        .png()
        .toBuffer(),
    ),
  );

  // Build ICO file manually (simple ICO format)
  const icoHeader = Buffer.alloc(6);
  icoHeader.writeUInt16LE(0, 0); // reserved
  icoHeader.writeUInt16LE(1, 2); // ICO type
  icoHeader.writeUInt16LE(icoSizes.length, 4);

  let offset = 6 + icoSizes.length * 16;
  const entries = [];
  for (let i = 0; i < icoSizes.length; i++) {
    const entry = Buffer.alloc(16);
    entry.writeUInt8(icoSizes[i] === 256 ? 0 : icoSizes[i], 0); // width (0 = 256)
    entry.writeUInt8(icoSizes[i] === 256 ? 0 : icoSizes[i], 1); // height
    entry.writeUInt8(0, 2); // color palette
    entry.writeUInt8(0, 3); // reserved
    entry.writeUInt16LE(1, 4); // color planes
    entry.writeUInt16LE(32, 6); // bits per pixel
    entry.writeUInt32LE(icoBuffers[i].length, 8); // size
    entry.writeUInt32LE(offset, 12); // offset
    entries.push(entry);
    offset += icoBuffers[i].length;
  }

  const ico = Buffer.concat([icoHeader, ...entries, ...icoBuffers]);
  const fs = await import("fs");
  fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);
  console.log("  ✓ icon.ico");

  console.log("\nDone! All icons regenerated with reduced padding.");
}

run().catch(console.error);
