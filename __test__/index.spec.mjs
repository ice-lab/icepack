import test from 'ava'

import { getCssModulesLocalIdent } from '../index.js'

test('template [hash]', (t) => {
  t.is(getCssModulesLocalIdent('src/pages/index.module.css', 'test' ,'[hash]'), '_58deea1c54f94c19c993');
})
test('template [path][name][ext]__[local]', (t) => {
  t.is(getCssModulesLocalIdent('src/pages/index.module.css', 'test' ,'[path][name][ext]__[local]'), 'src-pages-index-module-css__test');
})