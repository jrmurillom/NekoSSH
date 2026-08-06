/**
 * Lógica pura de wallpapers por tema (normalización, mapa legacy, clasificación de migración).
 * La persistencia durable vive en SQLite + disco vía commands Rust.
 */

import { clampAndFormatOpacity } from "../bg-settings-helper";

export const BG_BY_THEME_KEY = "nekossh-bg-by-theme";
export const LEGACY_BG_URL_KEY = "nekossh-bg-url";
export const LEGACY_BG_LABEL_KEY = "nekossh-bg-label";
export const LEGACY_BG_OPACITY_KEY = "nekossh-bg-opacity";

export const DEFAULT_WALLPAPER_OPACITY = 0.3;

export type ThemeWallpaper = {
  url: string;
  label: string;
  opacity: number;
};

export type ThemeWallpaperMap = Record<string, ThemeWallpaper>;

export type LegacyWallpaperKeys = {
  url: string | null;
  label: string | null;
  opacity: string | null;
};

/** Clasifica un valor legacy/mapa para decidir cómo migrarlo al backend. */
export type WallpaperMigrationTarget =
  | { kind: "skip" }
  | { kind: "data_url"; dataUrl: string }
  | { kind: "http_url"; url: string }
  | { kind: "file_path"; path: string }
  | { kind: "opacity_only" };

export function classifyWallpaperUrlForMigration(url: string): WallpaperMigrationTarget {
  const trimmed = url.trim();
  if (!trimmed) return { kind: "opacity_only" };
  if (trimmed.startsWith("data:")) return { kind: "data_url", dataUrl: trimmed };
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return { kind: "http_url", url: trimmed };
  }
  // Ruta de disco o etiqueta no usable como origen
  if (/^[a-zA-Z]:[\\/]/.test(trimmed) || trimmed.startsWith("/") || trimmed.startsWith("\\\\")) {
    return { kind: "file_path", path: trimmed };
  }
  return { kind: "skip" };
}

export function emptyWallpaper(opacity = DEFAULT_WALLPAPER_OPACITY): ThemeWallpaper {
  return {
    url: "",
    label: "",
    opacity: clampAndFormatOpacity(opacity).numeric,
  };
}

export function normalizeWallpaper(entry: Partial<ThemeWallpaper> | null | undefined): ThemeWallpaper {
  if (!entry) return emptyWallpaper();
  return {
    url: typeof entry.url === "string" ? entry.url : "",
    label: typeof entry.label === "string" ? entry.label : "",
    opacity: clampAndFormatOpacity(
      typeof entry.opacity === "number" ? entry.opacity : DEFAULT_WALLPAPER_OPACITY,
    ).numeric,
  };
}

export function getThemeWallpaper(map: ThemeWallpaperMap, themeId: string): ThemeWallpaper {
  return normalizeWallpaper(map[themeId]);
}

/** Upsert del wallpaper de un tema; no toca los demás. */
export function setThemeWallpaper(
  map: ThemeWallpaperMap,
  themeId: string,
  entry: Partial<ThemeWallpaper>,
): ThemeWallpaperMap {
  return {
    ...map,
    [themeId]: normalizeWallpaper({
      ...getThemeWallpaper(map, themeId),
      ...entry,
    }),
  };
}

/** Quita imagen/etiqueta del tema y deja opacidad en default. */
export function clearThemeWallpaper(map: ThemeWallpaperMap, themeId: string): ThemeWallpaperMap {
  return {
    ...map,
    [themeId]: emptyWallpaper(),
  };
}

export function parseThemeWallpaperMap(raw: string | null | undefined): ThemeWallpaperMap {
  if (!raw) return {};
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};
    const out: ThemeWallpaperMap = {};
    for (const [themeId, value] of Object.entries(parsed as Record<string, unknown>)) {
      if (!themeId) continue;
      out[themeId] = normalizeWallpaper(value as Partial<ThemeWallpaper>);
    }
    return out;
  } catch {
    return {};
  }
}

export function serializeThemeWallpaperMap(map: ThemeWallpaperMap): string {
  return JSON.stringify(map);
}

/**
 * Migración one-shot desde claves globales.
 * Si hay legacy y el tema destino aún no tiene entry, lo crea.
 * `clearLegacy` indica que el caller debe borrar las claves globales.
 */
export function migrateLegacyWallpaper(
  map: ThemeWallpaperMap,
  legacy: LegacyWallpaperKeys,
  targetThemeId: string,
): { map: ThemeWallpaperMap; didMigrate: boolean; clearLegacy: boolean } {
  const hasLegacy =
    (legacy.url != null && legacy.url !== "") ||
    (legacy.label != null && legacy.label !== "") ||
    legacy.opacity != null;

  if (!hasLegacy) {
    return { map, didMigrate: false, clearLegacy: false };
  }

  if (Object.prototype.hasOwnProperty.call(map, targetThemeId)) {
    return { map, didMigrate: false, clearLegacy: true };
  }

  const opacity = clampAndFormatOpacity(
    legacy.opacity != null ? parseFloat(legacy.opacity) : DEFAULT_WALLPAPER_OPACITY,
  ).numeric;
  const url = legacy.url ?? "";
  const label =
    legacy.label ||
    (url && !url.startsWith("data:") ? url : "");

  return {
    map: setThemeWallpaper(map, targetThemeId, { url, label, opacity }),
    didMigrate: true,
    clearLegacy: true,
  };
}
