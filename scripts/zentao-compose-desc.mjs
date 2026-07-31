#!/usr/bin/env node
/**
 * @deprecated Use zentao-enrich-push.mjs instead (UTF-8 safe, no pipe).
 * Compose ZenTao task description: preserve original + enrich-us planning block.
 */
import fs from 'node:fs';

console.error('Deprecated: use scripts/zentao-enrich-push.mjs');

const args = process.argv.slice(2);
const origIdx = args.indexOf('--original-file');
const enhIdx = args.indexOf('--enhanced-file');

if (origIdx < 0 || enhIdx < 0) {
  console.error(
    'Usage: node scripts/zentao-compose-desc.mjs --original-file <path> --enhanced-file <path>',
  );
  process.exit(1);
}

const original = fs.readFileSync(args[origIdx + 1], 'utf8').trim();
const enhanced = fs.readFileSync(args[enhIdx + 1], 'utf8').trim();

const html = `<div data-section="definicion-original">
<h3>Definición original</h3>
${original}
</div>
<hr>
<div data-section="enrich-us">
${enhanced}
</div>
`;

process.stdout.write(html);
