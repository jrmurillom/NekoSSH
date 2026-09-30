import { describe, expect, it, vi } from "vitest";
import {
  baseName,
  detectCollisions,
  filterPlanByExcludedRoots,
  formatUploadPlanImpact,
  type LocalUploadPlan,
} from "./explorer-drop-helper";
import {
  createExplorerStatusController,
  type TransferProgressPayload,
} from "./transfer-progress-helper";

// --- Mock DOM Minimal para Status Bar ---
interface MockClassList {
  add: (...tokens: string[]) => void;
  remove: (...tokens: string[]) => void;
  toggle: (token: string, force?: boolean) => boolean;
  contains: (token: string) => boolean;
}

interface MockElement {
  tagName: string;
  className: string;
  classList: MockClassList;
  style: { width: string };
  textContent: string;
  children: MockElement[];
  ownerDocument: { createElement: (tag: string) => HTMLElement };
  appendChild: (child: MockElement) => MockElement;
  replaceChildren: (...nodes: MockElement[]) => void;
  setAttribute: (name: string, value: string) => void;
  getAttribute: (name: string) => string | null;
  addEventListener: (event: string, listener: (ev: { stopPropagation: () => void }) => void) => void;
  click: () => void;
}

function createMockDomElement(tag = "DIV"): MockElement {
  const classSet = new Set<string>();
  const listeners = new Map<string, Array<(ev: { stopPropagation: () => void }) => void>>();
  let rawText = "";

  const doc = {
    createElement(t: string): HTMLElement {
      return createMockDomElement(t) as unknown as HTMLElement;
    },
  };

  const el: MockElement = {
    tagName: tag.toUpperCase(),
    className: "",
    classList: {
      add: (...tokens: string[]) => tokens.forEach((t) => classSet.add(t)),
      remove: (...tokens: string[]) => tokens.forEach((t) => classSet.delete(t)),
      toggle: (token: string, force?: boolean) => {
        const shouldAdd = force !== undefined ? force : !classSet.has(token);
        if (shouldAdd) classSet.add(token);
        else classSet.delete(token);
        return shouldAdd;
      },
      contains: (token: string) => classSet.has(token),
    },
    style: { width: "" },
    attributes: new Map<string, string>(),
    children: [],
    ownerDocument: doc,
    get textContent(): string {
      if (el.children.length === 0) return rawText;
      return el.children.map((c) => c.textContent).join("");
    },
    set textContent(val: string) {
      rawText = val;
      el.children = [];
    },
    appendChild(child: MockElement) {
      el.children.push(child);
      return child;
    },
    replaceChildren(...nodes: MockElement[]) {
      rawText = "";
      el.children = [...nodes];
    },
    setAttribute(name: string, value: string) {
      (el as any).attributes.set(name, value);
    },
    getAttribute(name: string) {
      return (el as any).attributes.get(name) ?? null;
    },
    addEventListener(event: string, listener: (ev: { stopPropagation: () => void }) => void) {
      const list = listeners.get(event) ?? [];
      list.push(listener);
      listeners.set(event, list);
    },
    click() {
      const list = listeners.get("click") ?? [];
      list.forEach((fn) => fn({ stopPropagation: () => {} }));
    },
  };

  return el;
}

function joinRemote(dir: string, name: string): string {
  if (dir === "/" || dir === "") return `/${name}`;
  return `${dir.replace(/\/+$/g, "")}/${name}`;
}

describe("explorer-folder-upload.e2e", () => {
  it("ejecuta el ciclo de vida completo de subida recursiva de carpetas (drop-to-finish)", async () => {
    const statusRoot = createMockDomElement("DIV");
    const cancelSpy = vi.fn();
    const controller = createExplorerStatusController(statusRoot as unknown as HTMLElement, {
      onCancelRequest: cancelSpy,
    });

    const droppedPaths = ["C:\\Users\\Roberto\\Desktop\\techpeople"];
    const dest = "/var/www/html";

    // 1. Simulación de sftp_scan_local_upload_items
    const scanResult: LocalUploadPlan = {
      total_files: 3,
      total_dirs: 3,
      total_bytes: 4096 + 2048 + 1024,
      items: [
        { local_path: "C:\\Desktop\\techpeople", relative_path: "techpeople", is_dir: true, size: 0 },
        { local_path: "C:\\Desktop\\techpeople\\assets", relative_path: "techpeople/assets", is_dir: true, size: 0 },
        { local_path: "C:\\Desktop\\techpeople\\_next", relative_path: "techpeople/_next", is_dir: true, size: 0 },
        { local_path: "C:\\Desktop\\techpeople\\index.html", relative_path: "techpeople/index.html", is_dir: false, size: 4096 },
        { local_path: "C:\\Desktop\\techpeople\\assets\\style.css", relative_path: "techpeople/assets/style.css", is_dir: false, size: 2048 },
        { local_path: "C:\\Desktop\\techpeople\\_next\\app.js", relative_path: "techpeople/_next/app.js", is_dir: false, size: 1024 },
      ],
    };

    // 2. Verificación del texto de confirmación presentado al usuario
    const impact = formatUploadPlanImpact(scanResult, dest, droppedPaths);
    expect(impact).toBe("Carpeta 'techpeople' (3 carpetas, 3 archivos · 7.0 KB) → /var/www/html");

    // 3. Simulación de ejecución de subida en main.ts
    const createdDirs: string[] = [];
    const uploadedFiles: Array<{ local: string; remote: string; size: number }> = [];

    let fileIndex = 0;
    for (const item of scanResult.items) {
      const remoteItemPath = joinRemote(dest, item.relative_path);

      if (item.is_dir) {
        // sftp_ensure_remote_dir
        createdDirs.push(remoteItemPath);
        continue;
      }

      fileIndex++;
      uploadedFiles.push({ local: item.local_path, remote: remoteItemPath, size: item.size });

      // Emisión de progreso en tiempo real
      controller.setTransferProgress(
        {
          operation: "upload",
          file_name: item.relative_path,
          bytes_transferred: item.size,
          total_bytes: item.size,
          percent: 100,
          speed_bps: 1024 * 100,
        },
        fileIndex,
        scanResult.total_files,
      );

      expect(statusRoot.textContent).toContain(`(${fileIndex}/${scanResult.total_files})`);
      expect(statusRoot.textContent).toContain(item.relative_path);
    }

    // Estado final exitoso
    controller.setStatus(`Subida completa: ${uploadedFiles.length} archivo(s), ${scanResult.total_dirs} carpeta(s)`, false, true);
    expect(statusRoot.textContent).toContain("Subida completa: 3 archivo(s), 3 carpeta(s)");

    // Verificación de orden y rutas remotas
    expect(createdDirs).toEqual([
      "/var/www/html/techpeople",
      "/var/www/html/techpeople/assets",
      "/var/www/html/techpeople/_next",
    ]);

    expect(uploadedFiles.map((u) => u.remote)).toEqual([
      "/var/www/html/techpeople/index.html",
      "/var/www/html/techpeople/assets/style.css",
      "/var/www/html/techpeople/_next/app.js",
    ]);
  });

  it("aborta limpiamente la subida secuencial cuando el usuario solicita cancelación a mitad de lote", async () => {
    const statusRoot = createMockDomElement("DIV");
    let transferCancelledByUser = false;

    const controller = createExplorerStatusController(statusRoot as unknown as HTMLElement, {
      onCancelRequest: () => {
        transferCancelledByUser = true;
      },
    });

    const plan: LocalUploadPlan = {
      total_files: 5,
      total_dirs: 1,
      total_bytes: 5000,
      items: [
        { local_path: "/a/f", relative_path: "f", is_dir: true, size: 0 },
        { local_path: "/a/f/1.txt", relative_path: "f/1.txt", is_dir: false, size: 1000 },
        { local_path: "/a/f/2.txt", relative_path: "f/2.txt", is_dir: false, size: 1000 },
        { local_path: "/a/f/3.txt", relative_path: "f/3.txt", is_dir: false, size: 1000 },
        { local_path: "/a/f/4.txt", relative_path: "f/4.txt", is_dir: false, size: 1000 },
        { local_path: "/a/f/5.txt", relative_path: "f/5.txt", is_dir: false, size: 1000 },
      ],
    };

    let uploadedCount = 0;
    for (const item of plan.items) {
      if (transferCancelledByUser) break;

      if (item.is_dir) continue;

      uploadedCount++;
      // Simular que el usuario pulsa cancelar en el archivo 2
      if (uploadedCount === 2) {
        controller.setTransferProgress(
          {
            operation: "upload",
            file_name: item.relative_path,
            bytes_transferred: 500,
            total_bytes: item.size,
            percent: 50,
            speed_bps: 1024,
          },
          uploadedCount,
          plan.total_files,
        );
        transferCancelledByUser = true;
      }
    }

    if (transferCancelledByUser) {
      controller.setStatus("Transferencia cancelada", false, false);
    }

    expect(uploadedCount).toBe(2);
    expect(statusRoot.textContent).toBe("Transferencia cancelada");
  });

  it("excluye en cascada las carpetas y archivos anidados cuando el usuario elige omitir una colisión", () => {
    const existingNames = ["techpeople"];
    const droppedPaths = ["C:\\Desktop\\techpeople", "C:\\Desktop\\nuevo.txt"];

    const collisions = detectCollisions(droppedPaths, existingNames);
    expect(collisions).toEqual(["techpeople"]);

    const plan: LocalUploadPlan = {
      total_files: 4,
      total_dirs: 2,
      total_bytes: 400,
      items: [
        { local_path: "C:\\Desktop\\techpeople", relative_path: "techpeople", is_dir: true, size: 0 },
        { local_path: "C:\\Desktop\\techpeople\\assets", relative_path: "techpeople/assets", is_dir: true, size: 0 },
        { local_path: "C:\\Desktop\\techpeople\\assets\\style.css", relative_path: "techpeople/assets/style.css", is_dir: false, size: 100 },
        { local_path: "C:\\Desktop\\techpeople\\index.html", relative_path: "techpeople/index.html", is_dir: false, size: 100 },
        { local_path: "C:\\Desktop\\nuevo.txt", relative_path: "nuevo.txt", is_dir: false, size: 200 },
      ],
    };

    // Usuario decide "Omitir" para "techpeople"
    const excludedRoots = new Set<string>(["techpeople"]);
    const effectivePlan = filterPlanByExcludedRoots(plan, excludedRoots);

    expect(effectivePlan.total_files).toBe(1);
    expect(effectivePlan.total_dirs).toBe(0);
    expect(effectivePlan.total_bytes).toBe(200);
    expect(effectivePlan.items.length).toBe(1);
    expect(effectivePlan.items[0].relative_path).toBe("nuevo.txt");
  });
});
