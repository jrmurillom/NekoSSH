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

export interface UploadItem {
  local_path: string;
  relative_path: string;
  is_dir: boolean;
  size: number;
}

export interface LocalUploadPlan {
  total_files: number;
  total_dirs: number;
  total_bytes: number;
  items: UploadItem[];
}

export interface UploadPlanSummary {
  total_files: number;
  total_dirs: number;
  total_bytes: number;
}

/** Formatea tamaño en bytes a string legible (B, KB, MB, GB). */
export function formatFileSize(bytes: number): string {
  if (bytes <= 0 || !Number.isFinite(bytes)) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const clamped = Math.min(i, units.length - 1);
  if (clamped === 0) return `${bytes} B`;
  const val = bytes / Math.pow(1024, clamped);
  return `${val.toFixed(val < 10 ? 1 : 0)} ${units[clamped]}`;
}

/**
 * Genera el texto descriptivo del diálogo de confirmación para carpetas o colecciones mixtas.
 */
export function formatUploadPlanImpact(
  summary: UploadPlanSummary,
  destPath: string,
  droppedRootNames: string[],
): string {
  const sizeStr = formatFileSize(summary.total_bytes);
  if (droppedRootNames.length === 1) {
    const singleName = baseName(droppedRootNames[0]);
    if (summary.total_dirs > 0) {
      const folderWord = summary.total_dirs === 1 ? "1 carpeta" : `${summary.total_dirs} carpetas`;
      const fileWord = summary.total_files === 1 ? "1 archivo" : `${summary.total_files} archivos`;
      return `Carpeta '${singleName}' (${folderWord}, ${fileWord} · ${sizeStr}) → ${destPath}`;
    }
    return `${singleName} (${sizeStr}) → ${destPath}`;
  }

  // Múltiples elementos en la raíz
  if (summary.total_dirs === 0) {
    return `${summary.total_files} archivos (${sizeStr}) → ${destPath}`;
  }
  const folderWord = summary.total_dirs === 1 ? "1 carpeta" : `${summary.total_dirs} carpetas`;
  const fileWord = summary.total_files === 1 ? "1 archivo" : `${summary.total_files} archivos`;
  return `${folderWord}, ${fileWord} (${sizeStr}) → ${destPath}`;
}

/**
 * Filtra los items del plan excluyendo carpetas o archivos raíz omitidos por el usuario en colisiones.
 */
export function filterPlanByExcludedRoots(
  plan: LocalUploadPlan,
  excludedRootNames: Set<string>,
): LocalUploadPlan {
  if (excludedRootNames.size === 0) return plan;
  const filteredItems = plan.items.filter((item) => {
    const rootName = item.relative_path.split("/")[0];
    return !excludedRootNames.has(rootName);
  });
  let total_files = 0;
  let total_dirs = 0;
  let total_bytes = 0;
  for (const item of filteredItems) {
    if (item.is_dir) {
      total_dirs++;
    } else {
      total_files++;
      total_bytes += item.size;
    }
  }
  return {
    total_files,
    total_dirs,
    total_bytes,
    items: filteredItems,
  };
}
