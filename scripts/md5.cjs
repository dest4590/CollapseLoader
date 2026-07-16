// md5.cjs – computes MD5 hash of a .jar file
// Usage: node md5.cjs <path-to-file.jar>

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

function usage() {
  console.error('Usage: node md5.cjs <file.jar>');
  process.exit(1);
}

const [, , target] = process.argv;
if (!target) usage();

const filePath = path.resolve(target);
if (!fs.existsSync(filePath)) {
  console.error(`File not found: ${filePath}`);
  process.exit(1);
}
if (path.extname(filePath).toLowerCase() !== '.jar') {
  console.error('Error: file must have .jar extension');
  process.exit(1);
}

const hash = crypto.createHash('md5');
const stream = fs.createReadStream(filePath);
stream.on('error', err => {
  console.error('Read error:', err.message);
  process.exit(1);
});
stream.on('data', chunk => hash.update(chunk));
stream.on('end', () => {
  const digest = hash.digest('hex');
  console.log(`MD5(${path.basename(filePath)}) = ${digest}`);
});
