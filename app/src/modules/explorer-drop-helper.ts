/**
 * Helpers puros para el arrastrar y soltar del explorador SFTP.
 *
 * No tocan el DOM ni la red: resuelven la ruta destino a partir de la fila
 * bajo el cursor, detectan colisiones de nombre contra un listado dado y
 * formatean el texto de confirmación. Se prueban de forma aislada.
 */

/** Info mínima de la fila del árbol sobre la que se suelta (leída del dataset). */
export interface DropTargetRow {
  /** `row.dataset.path` de la fila bajo el cursor. */
  path: string;
  /** `row.dataset.isDir === "true"`. */
  isDir: boolean;
}

/** Deriva la carpeta padre de una ruta remota tipo POSIX. */
export function parentDir(remotePath: string): string {
  const norm = remotePath.replace(/\/+$/g, "");
  const idx = norm.lastIndexOf("/");
  if (idx <= 0) return "/";
  return norm.slice(0, idx);
}

/**
 * Resuelve la ruta de destino de la subida:
 * - Sobre una carpeta → esa carpeta.
 * - Sobre un archivo → la carpeta que lo contiene.
 * - Sin fila (fondo/listado) → el cwd actual del explorador.
 */
export function resolveDropTarget(
  row: DropTargetRow | null | undefined,
  explorerCwd: string,
): string {
  if (!row || !row.path) return explorerCwd;
  return row.isDir ? row.path : parentDir(row.path);
}

/** Extrae el nombre base de una ruta local (soporta separadores `/` y `\`). */
export function baseName(localPath: string): string {
  const norm = localPath.replace(/[\\/]+$/g, "");
  const idx = Math.max(norm.lastIndexOf("/"), norm.lastIndexOf("\\"));
  return idx >= 0 ? norm.slice(idx + 1) : norm;
}

/**
 * Dado un conjunto de rutas locales y los nombres ya existentes en el destino,
 * devuelve los nombres base que colisionarían (case-sensitive, semántica POSIX).
 */
export function detectCollisions(
  localPaths: string[],
  existingNames: Iterable<string>,
): string[] {
  const existing = new Set(existingNames);
  const collisions: string[] = [];
  for (const p of localPaths) {
    const name = baseName(p);
    if (existing.has(name)) collisions.push(name);
  }
  return collisions;
}

/** Texto de confirmación de subida: nombre único vs. cantidad. */
export function formatUploadConfirm(localPaths: string[], destPath: string): string {
  if (localPaths.length === 1) {
    return `${baseName(localPaths[0])} → ${destPath}`;
  }
  return `${localPaths.length} archivos → ${destPath}`;
}
