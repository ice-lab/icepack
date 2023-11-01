import fse from 'fs-extra';

const DEST = 'crates/.rspack_crates/';
const crates = fse.readdirSync(DEST);

crates.forEach((crate) => {
  // Ingore crates which is temporary and use for binding.
  console.log('crates', crate);
});