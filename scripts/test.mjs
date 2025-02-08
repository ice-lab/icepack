import { spawnSync } from 'child_process';
import fse from 'fs-extra';

const cratesDir = 'crates';
const crates = fse.readdirSync(cratesDir);

const packageScripts = [];
crates.forEach((crate) => {
  // Check the file if it is a directory.
  if (fse.statSync(cratesDir + '/' + crate).isDirectory()) {
    // Ingore crates which is temporary and use for binding.
    if (!crate.startsWith('.')) {
      packageScripts.push('--package', crate);
    }
  }
});

spawnSync('cargo', ['test', ...packageScripts], {
  cwd: process.cwd(),
  stdio: 'inherit',
});