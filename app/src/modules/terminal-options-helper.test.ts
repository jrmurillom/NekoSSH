import { describe, it, expect } from "vitest";
import { Terminal } from "@xterm/xterm";
import {
  DEFAULT_TERMINAL_SCROLLBACK,
  buildTerminalOptions,
  type TerminalOptionsParams,
} from "./terminal-options-helper";

/**
 * Helper para escribir de forma asíncrona en xterm.js esperando a que el motor
 * de parsing termine de procesar el buffer.
 */
function writeAsync(term: Terminal, data: string): Promise<void> {
  return new Promise((resolve) => term.write(data, () => resolve()));
}

describe("terminal-options-helper", () => {
  describe("Nivel 1: Contrato y opciones puras de configuración", () => {
    it("debe definir DEFAULT_TERMINAL_SCROLLBACK como 10,000 líneas", () => {
      expect(DEFAULT_TERMINAL_SCROLLBACK).toBe(10000);
    });

    it("debe generar opciones completas con scrollback por defecto de 10,000 líneas", () => {
      const dummyTheme = { background: "#000000", foreground: "#ffffff" };
      const options = buildTerminalOptions({
        theme: dummyTheme,
        fontFamily: "Fira Code, monospace",
      });

      expect(options.scrollback).toBe(10000);
      expect(options.allowTransparency).toBe(true);
      expect(options.cursorBlink).toBe(true);
      expect(options.cursorStyle).toBe("block");
      expect(options.fontSize).toBe(14);
      expect(options.fontFamily).toBe("Fira Code, monospace");
      expect(options.theme).toEqual(dummyTheme);
    });

    it("debe aplicar fallback seguro si fontFamily está vacío o no se especifica", () => {
      const options = buildTerminalOptions({
        theme: {},
        fontFamily: "",
      });

      expect(options.fontFamily).toBe("monospace");
      expect(options.scrollback).toBe(10000);
    });

    it("debe permitir sobrescribir scrollback o fontSize si se requiere explícitamente", () => {
      const customParams: TerminalOptionsParams = {
        theme: {},
        fontFamily: "monospace",
        fontSize: 16,
        scrollback: 20000,
      };

      const options = buildTerminalOptions(customParams);
      expect(options.scrollback).toBe(20000);
      expect(options.fontSize).toBe(16);
    });
  });

  describe("Nivel 2: Comportamiento real del buffer en @xterm/xterm (tail -n2000)", () => {
    it("debe retener 2,000 líneas completas en el buffer sin truncar a 1,000", async () => {
      const options = buildTerminalOptions({
        theme: {},
        fontFamily: "monospace",
      });

      const term = new Terminal(options);

      // Simular la llegada de 2,000 líneas de un comando como 'tail -n2000 catalina.log'
      const totalLines = 2000;
      let batch = "";
      for (let i = 1; i <= totalLines; i++) {
        batch += `[CATALINA-LOG-ENTRY-${i}] Mensaje de depuración del servidor\r\n`;
      }

      await writeAsync(term, batch);

      // En xterm.js, term.buffer.active.length representa todas las líneas retenidas
      // (líneas de scrollback + líneas de la pantalla activa).
      // Con el default previo (1,000), length nunca superaba ~1,024.
      // Con scrollback: 10,000, debe retener holgadamente las 2,000 líneas generadas.
      expect(term.buffer.active.length).toBeGreaterThanOrEqual(totalLines);

      // Verificar que la primera línea aún existe en el buffer (no fue descartada)
      const firstLine = term.buffer.active.getLine(0)?.translateToString(true);
      expect(firstLine).toContain("[CATALINA-LOG-ENTRY-1]");

      term.dispose();
    });

    it("comportamiento contrastado: con scrollback de 1,000 sí se descartan las primeras 1,000 líneas", async () => {
      // Demostración explícita del bug original para contrastar
      const legacyTerm = new Terminal({ scrollback: 1000 });
      let batch = "";
      for (let i = 1; i <= 2000; i++) {
        batch += `[CATALINA-LOG-ENTRY-${i}]\r\n`;
      }

      await writeAsync(legacyTerm, batch);

      // Con 1,000 líneas de scrollback, la primera línea [CATALINA-LOG-ENTRY-1] YA NO EXISTE en el inicio
      const firstLineLegacy = legacyTerm.buffer.active.getLine(0)?.translateToString(true);
      expect(firstLineLegacy).not.toContain("[CATALINA-LOG-ENTRY-1]");
      // El buffer se estancó en 1000 + rows (1024)
      expect(legacyTerm.buffer.active.length).toBeLessThanOrEqual(1024);

      legacyTerm.dispose();
    });
  });

  describe("Nivel 3: Descarte circular y tope del buffer a 10,000 líneas", () => {
    it("debe respetar el límite de 10,000 líneas de scrollback al recibir más de 12,000 líneas", async () => {
      const options = buildTerminalOptions({
        theme: {},
        fontFamily: "monospace",
      });

      const term = new Terminal(options);
      const rows = term.rows; // Filas visibles del viewport (default 24)

      // Escribir 12,000 líneas continuas en bloques para rendimiento
      const chunkSize = 2000;
      for (let c = 0; c < 6; c++) {
        let chunk = "";
        for (let i = 1; i <= chunkSize; i++) {
          const lineNum = c * chunkSize + i;
          chunk += `ENTRY_NUM_${lineNum}\r\n`;
        }
        await writeAsync(term, chunk);
      }

      // El buffer activo total no debe exceder scrollback + viewport rows
      const maxExpectedCapacity = DEFAULT_TERMINAL_SCROLLBACK + rows;
      expect(term.buffer.active.length).toBeLessThanOrEqual(maxExpectedCapacity);

      // La primera entrada absoluta "ENTRY_NUM_1" debe haberse descartado por FIFO
      const lineZero = term.buffer.active.getLine(0)?.translateToString(true);
      expect(lineZero).not.toBe("ENTRY_NUM_1");
      // Debe haber avanzado aproximadamente a la entrada ~1976+
      expect(lineZero).toMatch(/^ENTRY_NUM_\d+/);
      const entryNum = parseInt(lineZero?.replace("ENTRY_NUM_", "") || "0", 10);
      expect(entryNum).toBeGreaterThan(1000);

      term.dispose();
    });
  });
});
