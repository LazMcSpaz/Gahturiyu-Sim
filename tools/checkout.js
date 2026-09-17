/* A read-only copy of the engine at some other commit, for comparing against.
   git archive into a temp dir; the sweep worker loads build.js from there. */
const fs = require('fs');
const os = require('os');
const path = require('path');
const { execSync } = require('child_process');

module.exports = function checkout(ref) {
  const repo = path.join(__dirname, '..');
  const sha = execSync(`git rev-parse --short ${ref}`, { cwd: repo }).toString().trim();
  const dir = path.join(os.tmpdir(), `gahturiyu-${sha}`);
  if (!fs.existsSync(path.join(dir, 'build.js'))) {
    fs.mkdirSync(dir, { recursive: true });
    execSync(`git archive ${ref} | tar -x -C "${dir}"`, { cwd: repo });
  }
  return dir;
};
