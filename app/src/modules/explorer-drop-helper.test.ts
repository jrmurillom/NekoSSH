import { describe, expect, it } from "vitest";
import {
  baseName,
  detectCollisions,
  formatUploadConfirm,
  parentDir,
  resolveDropTarget,
} from "./explorer-drop-helper";

describe("explorer-drop-helper", () => {
  describe("parentDir", () => {
    it("devuelve la carpeta contenedora", () => {
      expect(parentDir("/var/www/index.html")).toBe("/var/www");
      expect(parentDir("/var/www")).toBe("/var");
    });
    it("colapsa a la raíz", () => {
      expect(parentDir("/index.html")).toBe("/");
      expect(parentDir("/")).toBe("/");
    });
  });

  describe("baseName", () => {
    it("soporta separadores POSIX y Windows", () => {
      expect(baseName("/home/user/config.yml")).toBe("config.yml");
      expect(baseName("C:\\Users\\Roberto\\logo.png")).toBe("logo.png");
      expect(baseName("solo.txt")).toBe("solo.txt");
    });
  });

  describe("resolveDropTarget", () => {
    it("sin fila usa el cwd", () => {
      expect(resolveDropTarget(null, "/srv")).toBe("/srv");
      expect(resolveDropTarget(undefined, "/srv")).toBe("/srv");
    });
    it("sobre carpeta usa esa ruta", () => {
      expect(resolveDropTarget({ path: "/srv/html", isDir: true }, "/srv")).toBe("/srv/html");
    });
    it("sobre archivo usa la carpeta contenedora", () => {
      expect(resolveDropTarget({ path: "/srv/html/index.html", isDir: false }, "/srv")).toBe(
        "/srv/html",
      );
    });
    it("fila sin path cae al cwd", () => {
      expect(resolveDropTarget({ path: "", isDir: true }, "/srv")).toBe("/srv");
    });
  });

  describe("detectCollisions", () => {
    it("detecta nombres ya existentes en el destino", () => {
      const collisions = detectCollisions(
        ["/local/index.html", "/local/nuevo.txt"],
        ["index.html", "otro.txt"],
      );
      expect(collisions).toEqual(["index.html"]);
    });
    it("sin colisiones devuelve vacío", () => {
      expect(detectCollisions(["/local/a.txt"], ["b.txt"])).toEqual([]);
    });
  });

  describe("formatUploadConfirm", () => {
    it("un archivo muestra su nombre y destino", () => {
      expect(formatUploadConfirm(["/local/config.yml"], "/var/www")).toBe(
        "config.yml → /var/www",
      );
    });
    it("varios archivos muestran la cantidad y destino", () => {
      expect(formatUploadConfirm(["/a/x.txt", "/a/y.txt", "/a/z.txt"], "/var/www")).toBe(
        "3 archivos → /var/www",
      );
    });
  });
});
