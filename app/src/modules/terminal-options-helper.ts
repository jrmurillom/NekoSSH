import type { ITerminalOptions, ITheme } from "@xterm/xterm";

/**
 * Capacidad por defecto del buffer de scrollback para emuladores xterm en NekoSSH.
 * 10,000 líneas representa el punto dulce técnico de la industria entre retención profunda
 * de logs (p. ej. tail -n2000 catalina.log) y un uso eficiente de memoria RAM (~10-20 MB por shell).
 */
export const DEFAULT_TERMINAL_SCROLLBACK = 10000;

export interface TerminalOptionsParams {
  theme: ITheme;
  fontFamily?: string;
  fontSize?: number;
  scrollback?: number;
}

/**
 * Construye y normaliza la configuración de inicialización para instancias de `@xterm/xterm`.
 * Centraliza las opciones visuales y de capacidad de buffer para garantizar coherencia
 * entre el shell principal y cualquier shell hijo en cuadrícula.
 */
export function buildTerminalOptions(params: TerminalOptionsParams): ITerminalOptions {
  const resolvedFontFamily =
    params.fontFamily && params.fontFamily.trim().length > 0
      ? params.fontFamily
      : "monospace";

  return {
    allowTransparency: true,
    cursorBlink: true,
    cursorStyle: "block",
    theme: { ...params.theme },
    fontFamily: resolvedFontFamily,
    fontSize: params.fontSize ?? 14,
    scrollback: params.scrollback ?? DEFAULT_TERMINAL_SCROLLBACK,
  };
}
