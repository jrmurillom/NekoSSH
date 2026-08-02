/**
 * Helper para resolución de URLs de imagen de fondo y normalización de opacidad.
 */

export function resolveBackgroundUrl(
  rawUrl: string,
  convertFn?: (path: string) => string
): string {
  const trimmed = rawUrl.trim();
  if (!trimmed) return "";

  // URLs remotas o Data URIs se preservan sin cambios
  if (
    trimmed.startsWith("http://") ||
    trimmed.startsWith("https://") ||
    trimmed.startsWith("data:")
  ) {
    return trimmed;
  }

  // Rutas locales de disco (Windows/Unix): convertir con convertFileSrc si está disponible
  if (convertFn) {
    return convertFn(trimmed);
  }

  return trimmed;
}

export function clampAndFormatOpacity(value: number): { numeric: number; formatted: string } {
  let num = isNaN(value) ? 0.3 : value;
  num = Math.max(0, Math.min(1, num));
  return {
    numeric: num,
    formatted: num.toFixed(2),
  };
}

export function applyBackgroundStyle(
  element: { style: { backgroundImage: string; opacity: string } },
  url: string,
  opacity: number,
  convertFn?: (path: string) => string
): void {
  const resolvedUrl = resolveBackgroundUrl(url, convertFn);
  if (resolvedUrl) {
    element.style.backgroundImage = `url("${resolvedUrl}")`;
    element.style.opacity = opacity.toString();
  } else {
    element.style.backgroundImage = "";
    element.style.opacity = "0";
  }
}
