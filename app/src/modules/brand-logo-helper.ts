/**
 * Resuelve el asset de logo de marca según el id de tema conceptual.
 * La función es pura: recibe el mapa de URLs (inyectado por Vite o tests).
 */

export const BRAND_LOGO_THEME_IDS = [
  "nekossh",
  "hatsune-miku",
  "rei-ayanami",
  "neon-evangelion",
  "cyberpunk-david",
  "cyberpunk-lucy",
  "persona5",
  "sailor-moon",
] as const;

export type BrandLogoThemeId = (typeof BRAND_LOGO_THEME_IDS)[number];

const FALLBACK_THEME_ID: BrandLogoThemeId = "nekossh";

export function resolveBrandLogoThemeId(themeId: string): BrandLogoThemeId {
  return (BRAND_LOGO_THEME_IDS as readonly string[]).includes(themeId)
    ? (themeId as BrandLogoThemeId)
    : FALLBACK_THEME_ID;
}

/**
 * Devuelve la URL del PNG de logo para `themeId`.
 * Si el id no está en el catálogo o falta en `logos`, usa el de `nekossh`.
 */
export function resolveBrandLogoUrl(
  themeId: string,
  logos: Record<string, string>,
): string {
  const resolvedId = resolveBrandLogoThemeId(themeId);
  return logos[resolvedId] ?? logos[FALLBACK_THEME_ID] ?? "";
}
