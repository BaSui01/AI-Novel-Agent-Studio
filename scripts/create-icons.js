const fs = require('fs');
const path = require('path');

const iconsDir = path.join(__dirname, '../src-tauri/icons');
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

// 1. Create 32x32 Uncompressed BMP ICO Payload
const bmpHeaderSize = 40;
const pixelDataSize = 32 * 32 * 4; // 4096 bytes
const andMaskSize = (32 * 32) / 8; // 128 bytes
const imageSize = bmpHeaderSize + pixelDataSize + andMaskSize;

const icoHeader = Buffer.alloc(6);
icoHeader.writeUInt16LE(0, 0);
icoHeader.writeUInt16LE(1, 2);
icoHeader.writeUInt16LE(1, 4);

const icoEntry = Buffer.alloc(16);
icoEntry.writeUInt8(32, 0);
icoEntry.writeUInt8(32, 1);
icoEntry.writeUInt8(0, 2);
icoEntry.writeUInt8(0, 3);
icoEntry.writeUInt16LE(1, 4);
icoEntry.writeUInt16LE(32, 6);
icoEntry.writeUInt32LE(imageSize, 8);
icoEntry.writeUInt32LE(22, 12);

const bmpHeader = Buffer.alloc(40);
bmpHeader.writeUInt32LE(40, 0);          // biSize
bmpHeader.writeInt32LE(32, 4);           // biWidth
bmpHeader.writeInt32LE(64, 8);           // biHeight (32 * 2 for ICO BMP format)
bmpHeader.writeUInt16LE(1, 12);          // biPlanes
bmpHeader.writeUInt16LE(32, 14);         // biBitCount
bmpHeader.writeUInt32LE(0, 16);          // biCompression BI_RGB
bmpHeader.writeUInt32LE(pixelDataSize, 20); // biSizeImage

const pixels = Buffer.alloc(pixelDataSize, 0xAA); // Colored BGRA fill
const andMask = Buffer.alloc(andMaskSize, 0x00);   // Transparent mask

const icoBuffer = Buffer.concat([icoHeader, icoEntry, bmpHeader, pixels, andMask]);

fs.writeFileSync(path.join(iconsDir, 'icon.ico'), icoBuffer);
fs.writeFileSync(path.join(iconsDir, '32x32.png'), pixels);
fs.writeFileSync(path.join(iconsDir, '128x128.png'), pixels);
fs.writeFileSync(path.join(iconsDir, '128x128@2x.png'), pixels);
fs.writeFileSync(path.join(iconsDir, 'icon.icns'), pixels);

console.log('Valid BMP-ICO generated successfully');
