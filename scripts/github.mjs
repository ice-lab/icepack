import path from 'path';
import gitclone from 'git-clone/promise.js';
import { rimraf } from 'rimraf';
import ora from 'ora';
import yaml from 'js-yaml';
import fse from 'fs-extra';

export function getGithubInfo() {
  try {
    const content = yaml.load(
      fse.readFileSync('.github/actions/clone-crates/action.yml', 'utf-8')
    );
    return ['repo', 'dest', 'temp', 'ref'].reduce((info, key) => {
      info[key] = content.inputs[key].default;
      return info;
    }, {});
  } catch (e) {
    console.log(e);
    return {};
  }
}

export async function overwriteContent(content, dest) {
  fse.writeFileSync(dest, content);
}

export async function copyAndCleanUp(temp, dest, spinner) {
  const updateSpinner = text => {
    if (spinner) {
      spinner.text = text;
    }
  }

  updateSpinner('Copying crates to the dest...');
  
  fse.copySync(path.join(temp, 'crates'), dest);
  
  const pkg = JSON.parse(fse.readFileSync(path.join(temp, 'package.json'), 'utf-8'));

  // Update build.rs content
  const buildRsPath = path.join(dest, 'rspack_loader_swc/build.rs');
  const buildRsContent = fse.readFileSync(buildRsPath, 'utf-8')
    .replace('"../../Cargo.toml"', '"../../../Cargo.toml"');
  fse.writeFileSync(buildRsPath, buildRsContent);

  // Write package.json
  fse.writeFileSync(
    path.join(dest, '../package.json'), 
    JSON.stringify({ version: pkg.version }, null, 2)
  );

  updateSpinner('Clean up...');
  await rimraf(temp);

  if (process.env.IS_GITHUB) {
    await Promise.all(
      ['node_binding', 'bench'].map(dir => 
        rimraf(path.join(dest, dir))
      )
    );
  }

  spinner?.succeed('Cloning rspack repo succeed.');
}

export function createSpinner(text, options = {}) {
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
  const { repo, dest, temp, ref } = getGithubInfo();
  const spinner = createSpinner('Cloning rspack repo...');

  try {
    await rimraf(dest);
    await gitclone(`git@github.com:${repo}.git`, temp, { checkout: ref });
    await copyAndCleanUp(temp, dest, spinner);
  } catch (err) {
    spinner.fail('Cloning rspack repo failed.');
    await rimraf(temp);
    console.log(err);
  }
}

export default getGithubInfo();