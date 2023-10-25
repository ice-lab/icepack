import gitclone from 'git-clone/promise.js';
import { rimraf } from 'rimraf';
import ora from 'ora';
import fse from 'fs-extra';

const REPO = 'git@github.com:web-infra-dev/rspack.git';
const DEST = 'crates/.rspack_crates/';
const CEHECKOUT = 'v0.3.6';

function createSpinner(
  text,
  options = {},
) {
  const spinner = ora({
    text,
    stream: process.stdout,
    isEnabled: process.stdout.isTTY,
    interval: 200,
    ...options,
  });
  spinner.start();
  return spinner;
}

async function getRspackCrates(repo, dest, checkout) {
  let cloneError = null;
  const cloneDest = `${dest}/.temp`;
  const spinner = createSpinner('Cloning rspack repo...');
  // Step 1: remove dir.
  await rimraf(dest);
  // Step2: clone git repo.
  await gitclone(repo, cloneDest, { checkout }).catch((err) => {cloneError = err;});
  if (!cloneError) {
    // Step3: only copy crates dir to the dest.
    spinner.text = 'Copying crates to the dest...';
    fse.copySync(cloneDest + '/crates', dest);
    // Step4: remove useless files.
    spinner.text = 'Clean up...';
    await rimraf(cloneDest);
    spinner.succeed('Cloning rspack repo succeed.');
  } else {
    spinner.fail('Cloning rspack repo failed.');
    // Clean temp dir if clone failed.
    await rimraf(cloneDest);
    console.log(cloneError);
  }
}

getRspackCrates(REPO, DEST, CEHECKOUT);