import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createExplorerStatusController,
  formatBytes,
  formatEta,
  formatSpeed,
  formatTransferHeader,
  formatTransferMetrics,
  isTransferCancelledError,
  TRANSFER_CANCELLED_ERROR,
  type ExplorerStatusController,
  type TransferProgressPayload,
} from "./transfer-progress-helper";

// --- Mock DOM Minimal para pruebas E2E de WebView ---
interface MockClassList {
  add: (...tokens: string[]) => void;
  remove: (...tokens: string[]) => void;
  toggle: (token: string, force?: boolean) => boolean;
  contains: (token: string) => boolean;
  toArray: () => string[];
}

interface MockElement {
  tagName: string;
  type: string;
  title: string;
  className: string;
  classList: MockClassList;
  style: { width: string };
  attributes: Map<string, string>;
  children: MockElement[];
  replaceChildrenCalls: number;
  ownerDocument: { createElement: (tag: string) => HTMLElement };
  textContent: string;
  appendChild: (child: MockElement) => MockElement;
  replaceChildren: (...nodes: MockElement[]) => void;
  setAttribute: (name: string, value: string) => void;
  getAttribute: (name: string) => string | null;
  addEventListener: (event: string, listener: (ev: { stopPropagation: () => void }) => void) => void;
  click: () => void;
  querySelector: (cls: string) => MockElement | null;
}

function createMockDomRoot(): MockElement {
  const doc = {
    createElement(tag: string): HTMLElement {
      return createElement(tag) as unknown as HTMLElement;
    },
  };

  function createElement(tag: string): MockElement {
    const classSet = new Set<string>();
    const listeners = new Map<string, Array<(ev: { stopPropagation: () => void }) => void>>();
    let rawText = "";

    const el: MockElement = {
      tagName: tag.toUpperCase(),
      type: "",
      get title() {
        return el.attributes.get("title") ?? "";
      },
      set title(val: string) {
        el.attributes.set("title", val);
      },
      get className() {
        return Array.from(classSet).join(" ");
      },
      set className(val: string) {
        classSet.clear();
        for (const token of val.split(/\s+/).filter(Boolean)) {
          classSet.add(token);
        }
      },
      classList: {
        add: (...tokens: string[]) => {
          for (const t of tokens) if (t) classSet.add(t);
        },
        remove: (...tokens: string[]) => {
          for (const t of tokens) classSet.delete(t);
        },
        toggle: (token: string, force?: boolean) => {
          const shouldAdd = force !== undefined ? force : !classSet.has(token);
          if (shouldAdd) classSet.add(token);
          else classSet.delete(token);
          return shouldAdd;
        },
        contains: (token: string) => classSet.has(token),
        toArray: () => Array.from(classSet),
      },
      style: { width: "" },
      attributes: new Map<string, string>(),
      children: [],
      replaceChildrenCalls: 0,
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
        el.replaceChildrenCalls += 1;
        rawText = "";
        el.children = [...nodes];
      },
      setAttribute(name: string, value: string) {
        el.attributes.set(name, value);
      },
      getAttribute(name: string) {
        return el.attributes.get(name) ?? null;
      },
      addEventListener(event: string, listener: (ev: { stopPropagation: () => void }) => void) {
        const list = listeners.get(event) ?? [];
        list.push(listener);
        listeners.set(event, list);
      },
      click() {
        const list = listeners.get("click") ?? [];
        for (const fn of list) {
          fn({ stopPropagation: () => {} });
        }
      },
      querySelector(selector: string): MockElement | null {
        const cls = selector.startsWith(".") ? selector.slice(1) : selector;
        for (const child of el.children) {
          if (child.classList.contains(cls)) return child;
          const nested = child.querySelector(selector);
          if (nested) return nested;
        }
        return null;
      },
    };

    return el;
  }

  return createElement("div");
}

describe("E2E Direct P2P Transfer & Streaming Fallback in WebView DOM", () => {
  let filesStatusEl: MockElement;
  let cancelRequested: boolean;
  let controller: ExplorerStatusController;

  beforeEach(() => {
    vi.useFakeTimers();
    cancelRequested = false;
    filesStatusEl = createMockDomRoot();
    filesStatusEl.className = "files-status";

    controller = createExplorerStatusController(filesStatusEl as unknown as HTMLElement, {
      onCancelRequest: () => {
        cancelRequested = true;
      },
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("6.1 simula el ciclo completo de transferencia P2P: pre-flight, telemetría MB/s con ETA y finalización exitosa", () => {
    // 1. Pre-flight inicial (0 bytes)
    controller.setTransferProgress({
      operation: "scp",
      file_name: "prod-db-10gb.tar.gz",
      bytes_transferred: 0,
      total_bytes: 10 * 1024 * 1024 * 1024,
      percent: 0,
      speed_bps: 0,
    });

    expect(filesStatusEl.classList.contains("is-visible")).toBe(true);
    expect(filesStatusEl.classList.contains("is-progress")).toBe(true);

    const titleEl = filesStatusEl.querySelector(".files-status-progress-title");
    const percentEl = filesStatusEl.querySelector(".files-status-progress-percent");
    const barEl = filesStatusEl.querySelector(".files-status-progress-bar");
    const metaEl = filesStatusEl.querySelector(".files-status-progress-meta");

    expect(titleEl?.textContent).toBe("Copiando SCP prod-db-10gb.tar.gz");
    expect(percentEl?.textContent).toBe("0%");
    expect(barEl?.style.width).toBe("0%");
    expect(metaEl?.textContent).toBe("0 B / 10.00 GB");

    // 2. Primera ráfaga de telemetría P2P (25% a 125 MB/s)
    const bytes25 = 2.5 * 1024 * 1024 * 1024;
    const speed125 = 125 * 1024 * 1024;
    controller.setTransferProgress({
      operation: "scp",
      file_name: "prod-db-10gb.tar.gz",
      bytes_transferred: bytes25,
      total_bytes: 10 * 1024 * 1024 * 1024,
      percent: 25,
      speed_bps: speed125,
      eta_seconds: 61,
    });

    expect(percentEl?.textContent).toBe("25%");
    expect(barEl?.style.width).toBe("25%");
    expect(metaEl?.textContent).toContain("2.50 GB / 10.00 GB • 125.0 MB/s • ETA 1:01");

    // 3. Segunda ráfaga de telemetría a alta velocidad (75% a 250 MB/s)
    const bytes75 = 7.5 * 1024 * 1024 * 1024;
    const speed250 = 250 * 1024 * 1024;
    controller.setTransferProgress({
      operation: "scp",
      file_name: "prod-db-10gb.tar.gz",
      bytes_transferred: bytes75,
      total_bytes: 10 * 1024 * 1024 * 1024,
      percent: 75,
      speed_bps: speed250,
      eta_seconds: 10,
    });

    expect(percentEl?.textContent).toBe("75%");
    expect(barEl?.style.width).toBe("75%");
    expect(metaEl?.textContent).toContain("7.50 GB / 10.00 GB • 250.0 MB/s • ETA 0:10");

    // 4. Finalización al 100% y éxito
    controller.setTransferProgress({
      operation: "scp",
      file_name: "prod-db-10gb.tar.gz",
      bytes_transferred: 10 * 1024 * 1024 * 1024,
      total_bytes: 10 * 1024 * 1024 * 1024,
      percent: 100,
      speed_bps: speed250,
    });
    expect(percentEl?.textContent).toBe("100%");
    expect(barEl?.style.width).toBe("100%");

    controller.setStatus("Copia exitosa: prod-db-10gb.tar.gz", false, true);

    expect(filesStatusEl.classList.contains("is-progress")).toBe(false);
    expect(filesStatusEl.classList.contains("success")).toBe(true);
    expect(filesStatusEl.textContent).toContain("Copia exitosa: prod-db-10gb.tar.gz");
  });

  it("6.2 simula cancelación cooperativa a mitad de transferencia mediante el botón [ ✕ ] y limpieza reactiva", () => {
    // Iniciar transferencia
    controller.setTransferProgress({
      operation: "scp",
      file_name: "massive-archive.iso",
      bytes_transferred: 3 * 1024 * 1024 * 1024,
      total_bytes: 10 * 1024 * 1024 * 1024,
      percent: 30,
      speed_bps: 100 * 1024 * 1024,
    });

    const cancelBtn = filesStatusEl.querySelector(".files-status-progress-cancel");
    expect(cancelBtn).not.toBeNull();

    // El usuario hace click en el botón de cancelar en el banner
    cancelBtn?.click();
    expect(cancelRequested).toBe(true);

    // Backend responde con el código de cancelación cooperativa
    const backendError = new Error(`Tauri invoke failed: ${TRANSFER_CANCELLED_ERROR}`);
    expect(isTransferCancelledError(backendError)).toBe(true);

    // Frontend refleja el estado limpio sin error espurio
    controller.setStatus("Transferencia cancelada", false, false);
    expect(filesStatusEl.classList.contains("is-progress")).toBe(false);
    expect(filesStatusEl.classList.contains("error")).toBe(false);
    expect(filesStatusEl.textContent).toBe("Transferencia cancelada");
  });

  it("6.3 simula conmutación transparente a túnel de streaming socket pipe cuando no hay conectividad directa", () => {
    // Simular el inicio de transferencia en modo Streaming Pipe Fallback
    // En este modo, el stream fluye a velocidad de socket local/relay (~45 MB/s)
    const totalBytes = 500 * 1024 * 1024; // 500 MB
    const chunkSpeed = 45 * 1024 * 1024; // 45 MB/s

    controller.setTransferProgress({
      operation: "scp",
      file_name: "data-package.tar",
      bytes_transferred: 100 * 1024 * 1024,
      total_bytes: totalBytes,
      percent: 20,
      speed_bps: chunkSpeed,
      eta_seconds: 9,
    });

    const metaEl = filesStatusEl.querySelector(".files-status-progress-meta");
    const barEl = filesStatusEl.querySelector(".files-status-progress-bar");

    expect(barEl?.style.width).toBe("20%");
    expect(metaEl?.textContent).toContain("100.0 MB / 500.0 MB • 45.0 MB/s • ETA 0:09");

    // Progreso avanza sin saltos
    controller.setTransferProgress({
      operation: "scp",
      file_name: "data-package.tar",
      bytes_transferred: 400 * 1024 * 1024,
      total_bytes: totalBytes,
      percent: 80,
      speed_bps: chunkSpeed,
      eta_seconds: 2,
    });
    expect(barEl?.style.width).toBe("80%");
    expect(metaEl?.textContent).toContain("400.0 MB / 500.0 MB • 45.0 MB/s • ETA 0:02");

    // Finaliza con éxito
    controller.setStatus("Copia exitosa: data-package.tar", false, true);
    expect(filesStatusEl.classList.contains("success")).toBe(true);
    expect(filesStatusEl.textContent).toContain("data-package.tar");
  });

  it("gestiona errores de conexión remota o credenciales inválidas sin dejar UI colgada", () => {
    controller.setTransferProgress({
      operation: "scp",
      file_name: "confidential.pem",
      bytes_transferred: 0,
      total_bytes: 1024,
      percent: 0,
      speed_bps: 0,
    });

    // Ocurre un fallo real no-cancelación
    const realError = "Host unreachable or authentication rejected";
    expect(isTransferCancelledError(realError)).toBe(false);

    controller.setStatus(`Error al copiar: ${realError}`, true, false);

    expect(filesStatusEl.classList.contains("is-progress")).toBe(false);
    expect(filesStatusEl.classList.contains("error")).toBe(true);
    expect(filesStatusEl.textContent).toContain("Host unreachable or authentication rejected");
  });
});
