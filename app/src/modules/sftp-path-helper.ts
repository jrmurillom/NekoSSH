/**
 * Helper de manipulación y resolución de rutas para el explorador SFTP.
 */

export function normalizeRemotePath(path: string): string {
  const trimmed = path.trim().replace(/\\/g, "/");
  if (!trimmed || trimmed === "/") return "/";

  // Reemplazar slashes duplicados
  const normalized = trimmed.replace(/\/+/g, "/");
  return normalized.endsWith("/") && normalized.length > 1 ? normalized.slice(0, -1) : normalized;
}

export function getParentRemotePath(currentPath: string): string {
  const norm = normalizeRemotePath(currentPath);
  if (norm === "/") return "/";

  const lastSlashIndex = norm.lastIndexOf("/");
  if (lastSlashIndex <= 0) return "/";

  return norm.substring(0, lastSlashIndex);
}
