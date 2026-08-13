import { describe, expect, it } from "vitest";
import {
  computeVisibleNodes,
  isFilterActive,
  nodeNameMatches,
  type FilterableNode,
} from "./explorer-name-filter";

function dir(name: string, children: FilterableNode[], expanded = true): FilterableNode {
  return { name, isDir: true, expanded, children };
}

function file(name: string): FilterableNode {
  return { name, isDir: false, expanded: false, children: [] };
}

describe("explorer-name-filter", () => {
  describe("isFilterActive", () => {
    it("es falso para consulta vacía o solo espacios", () => {
      expect(isFilterActive("")).toBe(false);
      expect(isFilterActive("   ")).toBe(false);
    });
    it("es verdadero cuando hay texto", () => {
      expect(isFilterActive("ng")).toBe(true);
    });
  });

  describe("nodeNameMatches", () => {
    it("coincide por substring sin distinguir mayúsculas", () => {
      expect(nodeNameMatches("NginX.conf", "ngin")).toBe(true);
      expect(nodeNameMatches("README.md", "ME")).toBe(true);
    });
    it("no coincide cuando no está el substring", () => {
      expect(nodeNameMatches("index.html", "ngin")).toBe(false);
    });
    it("consulta vacía coincide con todo", () => {
      expect(nodeNameMatches("cualquiera", "")).toBe(true);
    });
  });

  describe("computeVisibleNodes", () => {
    it("consulta vacía deja todo lo pintado visible", () => {
      const roots = [file("a.txt"), dir("conf", [file("b.txt")])];
      const visible = computeVisibleNodes(roots, "");
      expect(visible.size).toBe(3);
    });

    it("filtra entradas del nivel actual por substring case-insensitive", () => {
      const nginx = file("nginx.conf");
      const readme = file("README.md");
      const visible = computeVisibleNodes([nginx, readme], "NGIN");
      expect(visible.has(nginx)).toBe(true);
      expect(visible.has(readme)).toBe(false);
    });

    it("mantiene el padre visible cuando un hijo coincide (padre ancla)", () => {
      const child = file("nginx.conf");
      const parent = dir("conf.d", [child]);
      const visible = computeVisibleNodes([parent], "nginx");
      expect(visible.has(parent)).toBe(true);
      expect(visible.has(child)).toBe(true);
    });

    it("filtra hijos que no coinciden aunque el padre esté expandido", () => {
      const match = file("nginx.conf");
      const noMatch = file("mime.types");
      const parent = dir("conf.d", [match, noMatch]);
      const visible = computeVisibleNodes([parent], "nginx");
      expect(visible.has(match)).toBe(true);
      expect(visible.has(noMatch)).toBe(false);
    });

    it("no inspecciona hijos de carpetas colapsadas", () => {
      const hidden = file("nginx.conf");
      const collapsed = dir("conf.d", [hidden], false);
      const visible = computeVisibleNodes([collapsed], "nginx");
      expect(visible.has(collapsed)).toBe(false);
      expect(visible.has(hidden)).toBe(false);
    });

    it("devuelve conjunto vacío cuando nada coincide", () => {
      const roots = [file("a.txt"), dir("conf", [file("b.txt")])];
      const visible = computeVisibleNodes(roots, "zzz");
      expect(visible.size).toBe(0);
    });
  });
});
