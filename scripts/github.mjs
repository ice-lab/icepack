import gitclone from 'git-clone/promise.js';
import { rimraf } from 'rimraf';
import ora from 'ora';
import yaml from 'js-yaml';
import fse from 'fs-extra';

export function getGithubInfo() {
  const info = {};
  try {
    const content = yaml.load(
      fse.readFileSync('.github/actions/clone-crates/action.yml',
      'utf-8',
    ));
    ['repo', 'dest', 'temp', 'ref'].forEach((key) => {
      info[key] = content.inputs[key].default;
    });
  } catch (e) {
    console.log(e);
  }
  return info;
}

export async function copyAndCleanUp(temp, dest, spinner) {
  // Step3: only copy crates dir to the dest.
  if (spinner) {
    spinner.text = 'Copying crates to the dest...';
  }
  fse.copySync(temp + '/crates', dest);
  const pkg = JSON.parse(fse.readFileSync(temp + '/package.json', 'utf-8'));
  console.log('version: ', pkg.version);
  // Step4: remove useless files.
  if (spinner) {
    spinner.text = 'Clean up...';
  }
  await rimraf(temp);
  if (process.env.IS_GITHUB) {
    await Promise.all(['node_binding', 'bench'].map(async (dir) => {
      // Remove useless crates in github action to reduce the check time.
      await rimraf(dest + '/' + dir);
    }));
  }
  if (spinner) {
    spinner.succeed('Cloning rspack repo succeed.');
  }
}

export function createSpinner(
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

export async function getRspackCrates() {
  const {
    repo,
    dest,
    temp,
    ref,
  } = getGithubInfo();
  let cloneError = null;
  const spinner = createSpinner('Cloning rspack repo...');
  // Step 1: remove dir.
  await rimraf(dest);
  // Step2: clone git repo.
  await gitclone(`git@github.com:${repo}.git`, temp, { checkout: ref }).catch((err) => {cloneError = err;});
  if (!cloneError) {
    copyAndCleanUp(temp, dest, spinner);
  } else {
    spinner.fail('Cloning rspack repo failed.');
    // Clean temp dir if clone failed.
    await rimraf(temp);
    console.log(cloneError);
  }
}

export default getGithubInfo();