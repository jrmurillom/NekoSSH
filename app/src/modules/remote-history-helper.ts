export interface HistoryCommandItem {
  date: string;
  command: string;
}

/**
 * Copia un comando de historial al portapapeles local usando la API nativa del navegador.
 */
export async function copyCommandToClipboard(command: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(command);
  } catch (err) {
    console.error("Error al copiar al portapapeles:", err);
  }
}

/**
 * Procesa las líneas crudas del historial remoto (leídas mediante tail/head).
 * Soporta Zsh (timestamp inline `: ts:0;cmd`) y Bash (timestamp en la línea anterior `#ts`).
 */
export function parseRemoteHistoryLines(rawLines: string[]): HistoryCommandItem[] {
  const items: HistoryCommandItem[] = [];
  let pendingTimestamp: string | null = null;

  for (const line of rawLines) {
    const trimmed = line.trim();
    if (trimmed.length === 0) continue;

    // 1. Formato Zsh extendido: ": 1627999999:0;comando"
    if (trimmed.startsWith(": ")) {
      const match = trimmed.match(/^:\s*(\d+):[^;]*;(.*)$/);
      if (match) {
        const ts = parseInt(match[1], 10) * 1000;
        const dateStr = new Date(ts).toLocaleString();
        items.push({ date: dateStr, command: match[2] });
        pendingTimestamp = null; // Limpiar cualquier timestamp de Bash huérfano
        continue;
      }
    }

    // 2. Formato Bash extendido: línea de timestamp "#1627999999"
    if (trimmed.startsWith("#")) {
      const match = trimmed.match(/^#(\d+)$/);
      if (match) {
        const ts = parseInt(match[1], 10) * 1000;
        pendingTimestamp = new Date(ts).toLocaleString();
        continue;
      }
    }

    // 3. Comando plano (Bash sin timestamp o comando normal tras timestamp de Bash)
    const date = pendingTimestamp ? pendingTimestamp : "N/D";
    items.push({ date, command: line });
    pendingTimestamp = null; // Consumir timestamp
  }

  return items;
}
