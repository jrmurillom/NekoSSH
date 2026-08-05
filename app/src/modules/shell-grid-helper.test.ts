import { describe, expect, it } from "vitest";
import {
  MAX_CHILD_SHELLS,
  canAddChildShell,
  childShellLabel,
  focusIndexAfterClose,
  gridDensityClass,
} from "./shell-grid-helper";

describe("canAddChildShell", () => {
  it("permite abrir hijos hasta el máximo", () => {
    expect(canAddChildShell(1)).toBe(true);
    expect(canAddChildShell(2)).toBe(true);
    expect(canAddChildShell(3)).toBe(true);
  });

  it("bloquea al llegar a padre + 3 hijos", () => {
    expect(canAddChildShell(1 + MAX_CHILD_SHELLS)).toBe(false);
  });
});

describe("gridDensityClass", () => {
  it("mapea el número de celdas a la clase de densidad", () => {
    expect(gridDensityClass(1)).toBe("term-grid cells-1");
    expect(gridDensityClass(2)).toBe("term-grid cells-2");
    expect(gridDensityClass(3)).toBe("term-grid cells-3");
    expect(gridDensityClass(4)).toBe("term-grid cells-4");
  });

  it("satura fuera de rango", () => {
    expect(gridDensityClass(0)).toBe("term-grid cells-1");
    expect(gridDensityClass(9)).toBe("term-grid cells-4");
  });
});

describe("childShellLabel", () => {
  it("numera los hijos después del padre", () => {
    expect(childShellLabel(1)).toBe("Shell 1");
    expect(childShellLabel(2)).toBe("Shell 2");
  });
});

describe("focusIndexAfterClose", () => {
  it("devuelve el padre cuando solo queda una celda", () => {
    expect(focusIndexAfterClose(1, 1)).toBe(0);
  });

  it("mantiene la posición cuando hay vecino a la derecha", () => {
    expect(focusIndexAfterClose(1, 3)).toBe(1);
  });

  it("retrocede al último si se cerró la última celda", () => {
    expect(focusIndexAfterClose(3, 3)).toBe(2);
  });
});
