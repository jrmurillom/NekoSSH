import { describe, expect, it } from "vitest";
import { normalizeRemotePath, getParentRemotePath } from "./sftp-path-helper";

describe("sftp-path-helper", () => {
  describe("normalizeRemotePath", () => {
    it("debe normalizar la raíz y rutas vacías", () => {
      expect(normalizeRemotePath("")).toBe("/");
      expect(normalizeRemotePath("/")).toBe("/");
      expect(normalizeRemotePath("   /   ")).toBe("/");
    });

    it("debe remover slashes duplicados y trailing slashes", () => {
      expect(normalizeRemotePath("/var//www///html/")).toBe("/var/www/html");
      expect(normalizeRemotePath("\\var\\www\\")).toBe("/var/www");
    });
  });

  describe("getParentRemotePath", () => {
    it("debe calcular la ruta padre correctamente", () => {
      expect(getParentRemotePath("/var/www/html")).toBe("/var/www");
      expect(getParentRemotePath("/var/www")).toBe("/var");
      expect(getParentRemotePath("/var")).toBe("/");
      expect(getParentRemotePath("/")).toBe("/");
    });
  });
});
