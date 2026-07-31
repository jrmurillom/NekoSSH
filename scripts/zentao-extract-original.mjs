#!/usr/bin/env node
/**
 * Extract Definición original HTML from a ZenTao task description.
 * Usage: node scripts/zentao-extract-original.mjs "<desc html>"
 *    or: node scripts/zentao-extract-original.mjs --task <taskId>
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

export function extractOriginalFromDesc(desc) {
  if (!desc || typeof desc !== 'string') return '';

  const startIndex = desc.indexOf('[ENRICH-US-START]');
  const endIndex = desc.indexOf('[ENRICH-US-END]');
  
  if (startIndex !== -1 && endIndex !== -1 && endIndex > startIndex) {
    const original = desc.slice(0, startIndex) + desc.slice(endIndex + '[ENRICH-US-END]'.length);
    return original.replace(/<br>\s*$/i, '').trim();
  }

  const sectionMatch = desc.match(
    /<div[^>]*data-section="definicion-original"[^>]*>\s*<h3>[^<]*<\/h3>\s*([\s\S]*?)<\/div>/i,
  );
  if (sectionMatch) return sectionMatch[1].trim();

  const withoutEnrich = desc
    .replace(/<hr\s*\/?>\s*<div[^>]*data-section="enrich-us"[\s\S]*/i, '')
    .trim();
  return withoutEnrich.replace(/<h3>\s*Definici[oó]n original\s*<\/h3>/i, '').trim() || desc.trim();
}

async function fetchTaskDesc(taskId) {
  const mcpPath = path.join(os.homedir(), '.cursor', 'mcp.json');
  const mcp = JSON.parse(fs.readFileSync(mcpPath, 'utf8'));
  const zt = mcp.mcpServers?.zentao?.env;
  const base = `${zt.ZENTAO_URL.replace(/\/$/, '')}/api.php/v1`;
  const tokenRes = await fetch(`${base}/tokens`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify({ account: zt.ZENTAO_ACCOUNT, password: zt.ZENTAO_PASSWORD }),
  });
  const { token } = await tokenRes.json();
  const headers = { Token: token };

  const res = await fetch(`${base}/tasks/${taskId}`, { headers });
  if (res.ok) return (await res.json()).desc ?? '';

  throw new Error(
    `No se ha encontrado la task ${taskId}. Asegúrate de que el id exista en ZenTao y de que las credenciales en ~/.cursor/mcp.json sean correctas.`,
  );
}

const args = process.argv.slice(2);
if (args[0] === '--task') {
  const desc = await fetchTaskDesc(args[1]);
  process.stdout.write(extractOriginalFromDesc(desc));
} else if (args[0]) {
  process.stdout.write(extractOriginalFromDesc(args[0]));
} else {
  process.stdout.write(extractOriginalFromDesc(fs.readFileSync(0, 'utf8')));
}
