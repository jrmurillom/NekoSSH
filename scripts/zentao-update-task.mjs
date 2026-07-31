#!/usr/bin/env node
/**
 * Update a ZenTao task description via REST API PUT /tasks/{id}.
 * Usage: node scripts/zentao-update-task.mjs <taskId> [--file path]
 *   If --file omitted, reads description body from stdin.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const args = process.argv.slice(2);
const taskId = args[0];
const fileIdx = args.indexOf('--file');
const filePath = fileIdx >= 0 ? args[fileIdx + 1] : null;

if (!taskId) {
  console.error('Usage: node scripts/zentao-update-task.mjs <taskId> [--file path]');
  process.exit(1);
}

const desc = filePath
  ? fs.readFileSync(filePath, 'utf8')
  : fs.readFileSync(0, 'utf8');

// Prefer zentao-enrich-push.mjs on Windows — piping stdin corrupts UTF-8 in PowerShell.

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
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ account: zt.ZENTAO_ACCOUNT, password: zt.ZENTAO_PASSWORD }),
  });
  if (!res.ok) throw new Error(`Token request failed: ${res.status}`);
  const data = await res.json();
  return data.token;
}

const token = await getToken();
const res = await fetch(`${base}/tasks/${taskId}`, {
  method: 'PUT',
  headers: { Token: token, 'Content-Type': 'application/json; charset=utf-8' },
  body: JSON.stringify({ desc }),
});

const text = await res.text();
let body;
try {
  body = JSON.parse(text);
} catch {
  body = text;
}

console.log(JSON.stringify({ ok: res.ok, status: res.status, body }, null, 2));
process.exit(res.ok ? 0 : 1);
