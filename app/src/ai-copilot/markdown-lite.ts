/**
 * Renderizador Markdown técnico ligero y determinista para NekoAI.
 * Regla de oro: escapa HTML crudo primero para garantizar 0% XSS antes de aplicar formato.
 */

export function escapeHtml(raw: string): string {
  return String(raw)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function formatInlineMarkdown(escapedLine: string): string {
  // 1. Extraer código inline primero para no interpretar ** dentro de backticks
  const codeTokens: string[] = [];
  const withPlaceholders = escapedLine.replace(/`([^`\n]+)`/g, (_m, code) => {
    const idx = codeTokens.push(`<code class="copilot-md-inline-code">${code}</code>`) - 1;
    return `\u0000CODE${idx}\u0000`;
  });

  // 2. Negritas **texto** y cursivas *texto*
  const formatted = withPlaceholders
    .replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>")
    .replace(/(^|[^*])\*([^*\n]+)\*/g, "$1<em>$2</em>");

  // 3. Restaurar código inline
  return formatted.replace(/\u0000CODE(\d+)\u0000/g, (_m, idx) => codeTokens[Number(idx)] ?? "");
}

function isTableSeparatorLine(line: string): boolean {
  const trimmed = line.trim();
  if (!trimmed.includes("|")) return false;
  const cells = trimmed
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((c) => c.trim());
  return cells.length > 0 && cells.every((c) => /^:?-{2,}:?$/.test(c));
}

function splitTableCells(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((c) => c.trim());
}

/**
 * Convierte Markdown técnico confiable a HTML seguro con bloques de código copiables y tablas GFM.
 */
export function renderSafeMarkdown(rawInput: string): string {
  const normalized = String(rawInput ?? "").replace(/\r\n?/g, "\n").trim();
  if (!normalized) return "";

  const escaped = escapeHtml(normalized);
  const lines = escaped.split("\n");
  const out: string[] = [];

  let i = 0;
  let blockIdSeq = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Bloques cercados ```lang
    if (line.trim().startsWith("```")) {
      const lang = line.trim().slice(3).trim() || "shell";
      const codeLines: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].trim().startsWith("```")) {
        codeLines.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) i += 1; // saltar cierre ```
      const codeContent = codeLines.join("\n");
      const bid = `md-code-${blockIdSeq++}`;
      out.push(
        `<div class="copilot-md-codewrap">` +
          `<div class="copilot-md-codehead">` +
            `<span>${lang}</span>` +
            `<button type="button" class="copilot-btn-xs copilot-copy-code-btn" data-copy-target="${bid}">Copiar</button>` +
          `</div>` +
          `<pre id="${bid}" class="copilot-code-block">${codeContent}</pre>` +
        `</div>`,
      );
      continue;
    }

    // Tablas GFM: línea con `|` seguida de separador `|---|`
    if (
      line.includes("|") &&
      i + 1 < lines.length &&
      isTableSeparatorLine(lines[i + 1])
    ) {
      const headers = splitTableCells(line);
      i += 2; // saltar header y separador
      const rows: string[][] = [];
      while (i < lines.length && lines[i].includes("|") && lines[i].trim() !== "") {
        rows.push(splitTableCells(lines[i]));
        i += 1;
      }
      const thHtml = headers.map((h) => `<th>${formatInlineMarkdown(h)}</th>`).join("");
      const bodyHtml = rows
        .map(
          (row) =>
            `<tr>${headers.map((_h, colIdx) => `<td>${formatInlineMarkdown(row[colIdx] ?? "")}</td>`).join("")}</tr>`,
        )
        .join("");
      out.push(
        `<div class="copilot-md-table-wrap"><table class="copilot-md-table"><thead><tr>${thHtml}</tr></thead><tbody>${bodyHtml}</tbody></table></div>`,
      );
      continue;
    }

    // Encabezados #, ##, ###
    const headingMatch = line.match(/^(#{1,4})\s+(.+)$/);
    if (headingMatch) {
      const level = Math.min(headingMatch[1].length + 1, 5);
      out.push(`<h${level} class="copilot-md-h">${formatInlineMarkdown(headingMatch[2])}</h${level}>`);
      i += 1;
      continue;
    }

    // Cita / alerta técnica (`> texto` -> ya escapado como `&gt;`)
    if (line.trimStart().startsWith("&gt; ")) {
      const quoteLines: string[] = [];
      while (i < lines.length && lines[i].trimStart().startsWith("&gt;")) {
        quoteLines.push(lines[i].trimStart().replace(/^&gt;\s?/, ""));
        i += 1;
      }
      out.push(
        `<blockquote class="copilot-md-quote">${quoteLines.map((l) => formatInlineMarkdown(l)).join("<br/>")}</blockquote>`,
      );
      continue;
    }

    // Listas desordenadas (- / *)
    if (/^\s*[-*]\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\s*[-*]\s+/.test(lines[i])) {
        items.push(`<li>${formatInlineMarkdown(lines[i].replace(/^\s*[-*]\s+/, ""))}</li>`);
        i += 1;
      }
      out.push(`<ul class="copilot-md-list">${items.join("")}</ul>`);
      continue;
    }

    // Listas numeradas (1. )
    if (/^\s*\d+\.\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\s*\d+\.\s+/.test(lines[i])) {
        items.push(`<li>${formatInlineMarkdown(lines[i].replace(/^\s*\d+\.\s+/, ""))}</li>`);
        i += 1;
      }
      out.push(`<ol class="copilot-md-list">${items.join("")}</ol>`);
      continue;
    }

    // Línea vacía separadora
    if (line.trim() === "") {
      i += 1;
      continue;
    }

    // Párrafo estándar
    const paraLines: string[] = [];
    while (
      i < lines.length &&
      lines[i].trim() !== "" &&
      !lines[i].trim().startsWith("```") &&
      !/^(#{1,4})\s+/.test(lines[i]) &&
      !/^\s*[-*]\s+/.test(lines[i]) &&
      !/^\s*\d+\.\s+/.test(lines[i]) &&
      !lines[i].trimStart().startsWith("&gt; ")
    ) {
      paraLines.push(formatInlineMarkdown(lines[i]));
      i += 1;
    }
    out.push(`<p class="copilot-md-p">${paraLines.join("<br/>")}</p>`);
  }

  return out.join("\n");
}
