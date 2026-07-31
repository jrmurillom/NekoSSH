#!/usr/bin/env node
/**
 * Compose (original + enrich-us planning) and PUT to ZenTao — single process, UTF-8 safe.
 *
 * Usage:
 *   node scripts/zentao-enrich-push.mjs <taskId> --enhanced-file <planning.html> \
 *     --original-file <original.html>
 *
 *   node scripts/zentao-enrich-push.mjs <taskId> --enhanced-file <planning.html> \
 *     --use-task-original
 *
 * Planning HTML = PO-only (see scripts/templates/zentao-planning-enhanced.html).
 * Do NOT pipe on Windows.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { extractOriginalFromDesc } from './zentao-extract-original.mjs';

const args = process.argv.slice(2);
const taskId = args[0];
const origIdx = args.indexOf('--original-file');
const enhIdx = args.indexOf('--enhanced-file');
const useTaskOriginal = args.includes('--use-task-original');

if (!taskId || enhIdx < 0 || (origIdx < 0 && !useTaskOriginal)) {
  console.error(`Usage:
  node scripts/zentao-enrich-push.mjs <taskId> --enhanced-file <path> --original-file <path>
  node scripts/zentao-enrich-push.mjs <taskId> --enhanced-file <path> --use-task-original`);
  process.exit(1);
}

const enhanced = fs.readFileSync(args[enhIdx + 1], 'utf8').trim();

const mcpPath = path.join(os.homedir(), '.cursor', 'mcp.json');
const mcp = JSON.parse(fs.readFileSync(mcpPath, 'utf8'));
const zt = mcp.mcpServers?.zentao?.env;
if (!zt?.ZENTAO_URL || !zt?.ZENTAO_ACCOUNT || !zt?.ZENTAO_PASSWORD) {
  console.error('Missing zentao env in ~/.cursor/mcp.json');
  process.exit(1);
}

const base = `${zt.ZENTAO_URL.replace(/\/$/, '')}/api.php/v1`;

async function getToken() {
  const res = await fetch(`${base}/tokens`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify({ account: zt.ZENTAO_ACCOUNT, password: zt.ZENTAO_PASSWORD }),
  });
  if (!res.ok) throw new Error(`Token request failed: ${res.status}`);
  return (await res.json()).token;
}

async function fetchTask(taskId, token) {
  const res = await fetch(`${base}/tasks/${taskId}`, { headers: { Token: token } });
  if (!res.ok) {
    throw new Error(
      `No se ha encontrado la task ${taskId}. Asegúrate de que el id exista en ZenTao y de que las credenciales en ~/.cursor/mcp.json sean correctas.`,
    );
  }
  return res.json();
}

const token = await getToken();
let original;
if (useTaskOriginal) {
  const task = await fetchTask(taskId, token);
  original = extractOriginalFromDesc(task.desc ?? '');
} else {
  original = fs.readFileSync(args[origIdx + 1], 'utf8').trim();
}

const desc = `${original}
<br>
[ENRICH-US-START]
<hr>
<div data-section="enrich-us">
${enhanced}
</div>
[ENRICH-US-END]
`;

const res = await fetch(`${base}/tasks/${taskId}`, {
  method: 'PUT',
  headers: {
    Token: token,
    'Content-Type': 'application/json; charset=utf-8',
    Accept: 'application/json; charset=utf-8',
  },
  body: JSON.stringify({ desc }),
});

const text = await res.text();
let body;
try {
  body = JSON.parse(text);
} catch {
  body = text;
}

const hasMojibake = /(?:\?\?|Ã.|Â)/.test(body?.desc ?? '');
console.log(
  JSON.stringify({ ok: res.ok, status: res.status, encodingOk: !hasMojibake, taskId }, null, 2),
);
if (!res.ok) process.exit(1);
