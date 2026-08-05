/** Lógica pura del grid de shells por contexto de pestaña (padre + hijos). */

/** Máximo de shells hijos por contexto (padre + 3 = 4 celdas). */
export const MAX_CHILD_SHELLS = 3;

/** `paneCount` incluye al padre. */
export function canAddChildShell(paneCount: number): boolean {
  return paneCount - 1 < MAX_CHILD_SHELLS;
}

/** Clase de densidad del grid: cells-1 .. cells-4. */
export function gridDensityClass(paneCount: number): string {
  const cells = Math.min(Math.max(paneCount, 1), MAX_CHILD_SHELLS + 1);
  return `term-grid cells-${cells}`;
}

/** Etiqueta del nuevo shell hijo: el padre es "Principal". */
export function childShellLabel(paneCount: number): string {
  return `Shell ${Math.max(paneCount, 1)}`;
}

/**
 * Índice de la celda que recibe el foco tras cerrar un hijo.
 * Nunca devuelve un índice fuera de rango; 0 (padre) si no queda otro.
 */
export function focusIndexAfterClose(closedIndex: number, remainingCount: number): number {
  if (remainingCount <= 1) return 0;
  return Math.min(closedIndex, remainingCount - 1);
}
