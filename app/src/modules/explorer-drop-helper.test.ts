import { describe, expect, it } from "vitest";
import {
  baseName,
  detectCollisions,
  filterPlanByExcludedRoots,
  formatFileSize,
  formatUploadConfirm,
  formatUploadPlanImpact,
  LocalUploadPlan,
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

  describe("formatFileSize", () => {
    it("formatea 0 bytes y negativos", () => {
      expect(formatFileSize(0)).toBe("0 B");
      expect(formatFileSize(-1)).toBe("0 B");
    });
    it("formatea bytes, KB, MB y GB", () => {
      expect(formatFileSize(500)).toBe("500 B");
      expect(formatFileSize(1024)).toBe("1.0 KB");
      expect(formatFileSize(1048576 * 4.2)).toBe("4.2 MB");
      expect(formatFileSize(1073741824 * 2)).toBe("2.0 GB");
    });
  });

  describe("formatUploadPlanImpact", () => {
    it("carpeta individual muestra nombre, carpetas, archivos y tamaño", () => {
      const summary = { total_files: 125, total_dirs: 3, total_bytes: 4404019 };
      const impact = formatUploadPlanImpact(summary, "/var/www", ["C:\\Users\\Roberto\\Desktop\\techpeople"]);
      expect(impact).toBe("Carpeta 'techpeople' (3 carpetas, 125 archivos · 4.2 MB) → /var/www");
    });

    it("archivo individual muestra su nombre y tamaño", () => {
      const summary = { total_files: 1, total_dirs: 0, total_bytes: 2048 };
      const impact = formatUploadPlanImpact(summary, "/var/www", ["/local/index.html"]);
      expect(impact).toBe("index.html (2.0 KB) → /var/www");
    });

    it("selección mixta muestra conteo de carpetas, archivos y tamaño", () => {
      const summary = { total_files: 50, total_dirs: 2, total_bytes: 1048576 };
      const impact = formatUploadPlanImpact(summary, "/home/neko", ["/local/img", "/local/logo.png"]);
      expect(impact).toBe("2 carpetas, 50 archivos (1.0 MB) → /home/neko");
    });

    it("múltiples archivos sin carpetas muestra conteo de archivos y tamaño", () => {
      const summary = { total_files: 5, total_dirs: 0, total_bytes: 5120 };
      const impact = formatUploadPlanImpact(summary, "/home/neko", ["/a/1.txt", "/a/2.txt"]);
      expect(impact).toBe("5 archivos (5.0 KB) → /home/neko");
    });
  });

  describe("filterPlanByExcludedRoots", () => {
    it("excluye carpetas y todos sus archivos anidados si la raíz fue omitida", () => {
      const plan: LocalUploadPlan = {
        total_files: 3,
        total_dirs: 2,
        total_bytes: 300,
        items: [
          { local_path: "/a/techpeople", relative_path: "techpeople", is_dir: true, size: 0 },
          { local_path: "/a/techpeople/css", relative_path: "techpeople/css", is_dir: true, size: 0 },
          { local_path: "/a/techpeople/css/a.css", relative_path: "techpeople/css/a.css", is_dir: false, size: 100 },
          { local_path: "/a/techpeople/index.html", relative_path: "techpeople/index.html", is_dir: false, size: 100 },
          { local_path: "/a/keep.txt", relative_path: "keep.txt", is_dir: false, size: 100 },
        ],
      };

      const filtered = filterPlanByExcludedRoots(plan, new Set(["techpeople"]));
      expect(filtered.total_files).toBe(1);
      expect(filtered.total_dirs).toBe(0);
      expect(filtered.total_bytes).toBe(100);
      expect(filtered.items.length).toBe(1);
      expect(filtered.items[0].relative_path).toBe("keep.txt");
    });

    it("si no hay exclusiones devuelve el plan intacto", () => {
      const plan: LocalUploadPlan = {
        total_files: 1,
        total_dirs: 0,
        total_bytes: 50,
        items: [{ local_path: "/a/1.txt", relative_path: "1.txt", is_dir: false, size: 50 }],
      };
      const filtered = filterPlanByExcludedRoots(plan, new Set());
      expect(filtered).toBe(plan);
    });
  });
});
