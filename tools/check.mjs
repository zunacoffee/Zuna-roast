// Fails if the app's JavaScript has a syntax error, so a broken build never ships.
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
const html = readFileSync(new URL('../app/index.html', import.meta.url), 'utf8');
const inline = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map(m => m[1]);
const files = { 'native.js': readFileSync(new URL('../app/native.js', import.meta.url), 'utf8') };
let ok = true;
for (const [name, code] of [...inline.map((c, i) => [`index.html script ${i + 1}`, c]), ...Object.entries(files)]) {
  try { new vm.Script(code, { filename: name }); console.log('ok  ', name); }
  catch (e) { ok = false; console.error('FAIL', name, '\n', e.message); }
}
if (!html.includes('<script src="native.js"></script>')) { ok = false; console.error('FAIL index.html does not load native.js'); }
process.exit(ok ? 0 : 1);
