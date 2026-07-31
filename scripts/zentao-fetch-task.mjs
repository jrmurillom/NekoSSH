#!/usr/bin/env node
/**
 * Fetch a ZenTao task by id only (REST API). Reads credentials from ~/.cursor/mcp.json.
 * Usage: node scripts/zentao-fetch-task.mjs <taskId>
 *
 * No fallback by execution list or name. If the id is missing → clear not-found message.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const taskId = process.argv[2];
if (!taskId) {
  console.error('Usage: node scripts/zentao-fetch-task.mjs <taskId>');
  process.exit(1);
}

if (!/^\d+$/.test(String(taskId))) {
  console.error(
    `No se ha encontrado la task "${taskId}". Asegúrate de pasar solo el id numérico (ej. enrich-us 6).`,
  );
  process.exit(1);
}

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

async function apiGet(token, urlPath) {
  const res = await fetch(`${base}${urlPath}`, {
    headers: { Token: token },
  });
  const text = await res.text();
  let body;
  try {
    body = JSON.parse(text);
  } catch {
    body = text;
  }
  return { ok: res.ok, status: res.status, body };
}

const notFoundMessage = `No se ha encontrado la task ${taskId}. Asegúrate de que el id exista en ZenTao y de que las credenciales en ~/.cursor/mcp.json sean correctas.`;

const token = await getToken();
const result = await apiGet(token, `/tasks/${taskId}`);

if (!result.ok) {
  const payload = {
    ok: false,
    status: result.status === 404 ? 404 : result.status,
    body: {
      error: 'Task not found',
      message: notFoundMessage,
      taskId: String(taskId),
    },
  };
  console.log(JSON.stringify(payload, null, 2));
  console.error(notFoundMessage);
  process.exit(1);
}

console.log(JSON.stringify({ ok: true, status: 200, body: result.body }, null, 2));
