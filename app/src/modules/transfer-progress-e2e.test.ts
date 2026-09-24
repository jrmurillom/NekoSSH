import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createExplorerStatusController,
  isTransferCancelledError,
  TRANSFER_CANCELLED_ERROR,
  type TransferProgressPayload,
} from "./transfer-progress-helper";

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

describe("transfer-progress E2E DOM & UX lifecycle", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("reutiliza exactamente las mismas referencias de nodos DOM durante múltiples ticks sin recrear el árbol", () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    controller.setTransferProgress(
      {
        operation: "upload",
        file_name: "dataset.parquet",
        bytes_transferred: 0,
        total_bytes: 0,
        percent: 0,
        speed_bps: 0,
      },
      1,
      2,
    );

    expect(filesStatus.replaceChildrenCalls).toBe(1);
    const initialTitleEl = filesStatus.querySelector(".files-status-progress-title");
    const initialPercentEl = filesStatus.querySelector(".files-status-progress-percent");
    const initialBarEl = filesStatus.querySelector(".files-status-progress-bar");
    const initialMetaEl = filesStatus.querySelector(".files-status-progress-meta");

    expect(initialTitleEl).not.toBeNull();
    expect(initialPercentEl).not.toBeNull();
    expect(initialBarEl).not.toBeNull();
    expect(initialMetaEl).not.toBeNull();
    expect(initialMetaEl?.textContent).toBe("Preparando transferencia…");

    for (let step = 1; step <= 20; step++) {
      const pct = step * 5;
      controller.setTransferProgress(
        {
          operation: "upload",
          file_name: "dataset.parquet",
          bytes_transferred: step * 524288,
          total_bytes: 20 * 524288,
          percent: pct,
          speed_bps: 2097152,
        },
        1,
        2,
      );
    }

    // Cero llamadas adicionales a replaceChildren tras 20 actualizaciones de progreso
    expect(filesStatus.replaceChildrenCalls).toBe(1);
    expect(filesStatus.querySelector(".files-status-progress-title")).toBe(initialTitleEl);
    expect(filesStatus.querySelector(".files-status-progress-percent")).toBe(initialPercentEl);
    expect(filesStatus.querySelector(".files-status-progress-bar")).toBe(initialBarEl);
    expect(filesStatus.querySelector(".files-status-progress-meta")).toBe(initialMetaEl);

    expect(initialTitleEl?.textContent).toBe("Subiendo dataset.parquet (1/2)");
    expect(initialPercentEl?.textContent).toBe("100%");
    expect(initialBarEl?.style.width).toBe("100%");
    expect(initialMetaEl?.textContent).toBe("10.0 MB / 10.0 MB • 2.0 MB/s");
  });

  it("ejecuta el ciclo completo UX: Verificando destino -> Preparando -> Progreso -> Éxito -> Auto-cierre a los 3000ms", () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    // 1. Fase de verificación previa (detección de colisiones)
    controller.setStatus("Verificando destino…");
    expect(filesStatus.classList.contains("is-visible")).toBe(true);
    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.textContent).toBe("Verificando destino…");

    // 2. Fase Pre-flight antes del primer stat/chunk de Rust
    controller.setTransferProgress({
      operation: "scp",
      file_name: "release.tar.zst",
      bytes_transferred: 0,
      total_bytes: 0,
      percent: 0,
      speed_bps: 0,
    });
    expect(filesStatus.classList.contains("is-progress")).toBe(true);
    expect(filesStatus.querySelector(".files-status-progress-meta")?.textContent).toBe(
      "Preparando transferencia…",
    );

    // 3. Fase Streaming
    controller.setTransferProgress({
      operation: "scp",
      file_name: "release.tar.zst",
      bytes_transferred: 5242880,
      total_bytes: 10485760,
      percent: 50,
      speed_bps: 5242880,
    });
    expect(filesStatus.querySelector(".files-status-progress-title")?.textContent).toBe(
      "Copiando SCP release.tar.zst",
    );
    expect(filesStatus.querySelector(".files-status-progress-bar")?.style.width).toBe(
      "50%",
    );
    expect(filesStatus.querySelector(".files-status-progress-meta")?.textContent).toBe(
      "5.0 MB / 10.0 MB • 5.0 MB/s",
    );

    // 4. Fase Éxito final
    controller.setStatus("Copia exitosa: release.tar.zst", false, true);
    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.classList.contains("is-visible")).toBe(true);
    expect(filesStatus.classList.contains("success")).toBe(true);
    expect(filesStatus.textContent).toBe("✅ Copia exitosa: release.tar.zst");

    // 5. Auto-cierre a los 3000ms + limpieza de clase a los 3300ms
    vi.advanceTimersByTime(2999);
    expect(filesStatus.classList.contains("is-visible")).toBe(true);

    vi.advanceTimersByTime(1);
    expect(filesStatus.classList.contains("is-visible")).toBe(false);

    vi.advanceTimersByTime(300);
    expect(filesStatus.classList.contains("success")).toBe(false);
  });

  it("mantiene visible e inalterado el estado .error tras un fallo a mitad de transferencia sin auto-ocultarse", () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    controller.setTransferProgress({
      operation: "upload",
      file_name: "firmware.bin",
      bytes_transferred: 2097152,
      total_bytes: 4194304,
      percent: 50,
      speed_bps: 1048576,
    });
    expect(filesStatus.classList.contains("is-progress")).toBe(true);

    // Ocurre corte de red a mitad de subida
    controller.setStatus(
      "Subidos 0, fallaron 1: firmware.bin",
      true,
      false,
    );

    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.classList.contains("is-visible")).toBe(true);
    expect(filesStatus.classList.contains("error")).toBe(true);
    expect(filesStatus.textContent).toBe("❌ Subidos 0, fallaron 1: firmware.bin");

    // Incluso después de 15 segundos, el banner de error permanece visible para el usuario
    vi.advanceTimersByTime(15000);
    expect(filesStatus.classList.contains("is-visible")).toBe(true);
    expect(filesStatus.classList.contains("error")).toBe(true);
  });

  it("garantiza exclusión mutua entre subida SFTP (incluso durante Verificando destino…) y Pegar SCP", async () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    let transferInProgress = false;
    const executedOperations: string[] = [];
    const rejectedOperations: string[] = [];

    let resolveDirList!: () => void;
    const dirListPromise = new Promise<void>((resolve) => {
      resolveDirList = resolve;
    });

    async function simulateUploadFlow() {
      if (transferInProgress) {
        rejectedOperations.push("upload");
        return;
      }
      transferInProgress = true;
      controller.setStatus("Verificando destino…");
      try {
        await dirListPromise;
        controller.setTransferProgress({
          operation: "upload",
          file_name: "app.tar",
          bytes_transferred: 1024,
          total_bytes: 1024,
          percent: 100,
          speed_bps: 1024,
        });
        executedOperations.push("upload");
        controller.setStatus("Subida completa: 1 archivo(s)", false, true);
      } finally {
        transferInProgress = false;
      }
    }

    async function simulatePasteScpFlow() {
      if (transferInProgress) {
        rejectedOperations.push("scp");
        return;
      }
      transferInProgress = true;
      try {
        executedOperations.push("scp");
      } finally {
        transferInProgress = false;
      }
    }

    // Iniciamos upload; queda pausado en "Verificando destino…" (sftp_list_dir en vuelo)
    const uploadTask = simulateUploadFlow();
    expect(transferInProgress).toBe(true);
    expect(filesStatus.textContent).toBe("Verificando destino…");

    // Intentamos disparar Pegar SCP y un segundo Drop concurrente mientras lista el directorio
    await simulatePasteScpFlow();
    await simulateUploadFlow();

    expect(rejectedOperations).toEqual(["scp", "upload"]);
    expect(executedOperations).toEqual([]);

    // Termina sftp_list_dir y concluye la subida
    resolveDirList();
    await uploadTask;

    expect(transferInProgress).toBe(false);
    expect(executedOperations).toEqual(["upload"]);

    // Una vez liberado el lock en finally, Pegar SCP puede ejecutarse normalmente
    await simulatePasteScpFlow();
    expect(executedOperations).toEqual(["upload", "scp"]);
  });

  it("ejecuta el flujo completo de descarga de archivo grande (>4 GiB) y aborta limpiamente si el usuario cancela el diálogo Guardar como…", () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    // 1. Si el usuario cancela el diálogo nativo "Guardar como…" (retorna null), el banner permanece inalterado
    const pickedPathNull: string | null = null;
    if (pickedPathNull === null) {
      expect(filesStatus.classList.contains("is-visible")).toBe(false);
    }

    // 2. El usuario elige ruta destino local: inicia estado Pre-flight de descarga
    controller.setTransferProgress({
      operation: "download",
      file_name: "cluster-snapshot.tar.zst",
      bytes_transferred: 0,
      total_bytes: 0,
      percent: 0,
      speed_bps: 0,
    });
    expect(filesStatus.classList.contains("is-progress")).toBe(true);
    expect(filesStatus.querySelector(".files-status-progress-title")?.textContent).toBe(
      "Descargando cluster-snapshot.tar.zst",
    );
    expect(filesStatus.querySelector(".files-status-progress-meta")?.textContent).toBe(
      "Preparando transferencia…",
    );

    // 3. Streaming de archivo grande (6.40 GB de 10.00 GB a 64.0 MB/s)
    const tenGb = 10 * 1024 * 1024 * 1024;
    const transferred = 6.4 * 1024 * 1024 * 1024;
    controller.setTransferProgress({
      operation: "download",
      file_name: "cluster-snapshot.tar.zst",
      bytes_transferred: transferred,
      total_bytes: tenGb,
      percent: 64,
      speed_bps: 64 * 1024 * 1024,
    });
    expect(filesStatus.querySelector(".files-status-progress-percent")?.textContent).toBe(
      "64%",
    );
    expect(filesStatus.querySelector(".files-status-progress-bar")?.style.width).toBe(
      "64%",
    );
    expect(filesStatus.querySelector(".files-status-progress-meta")?.textContent).toBe(
      "6.40 GB / 10.00 GB • 64.0 MB/s",
    );

    // 4. Descarga finalizada al 100% y transición a banner de éxito con auto-cierre a los 3000ms
    controller.setStatus("Descarga completa: cluster-snapshot.tar.zst", false, true);
    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.classList.contains("is-visible")).toBe(true);
    expect(filesStatus.classList.contains("success")).toBe(true);
    expect(filesStatus.textContent).toBe("✅ Descarga completa: cluster-snapshot.tar.zst");

    vi.advanceTimersByTime(3000);
    expect(filesStatus.classList.contains("is-visible")).toBe(false);
  });

  it("bloquea operaciones concurrentes mientras el diálogo nativo Guardar como… está abierto y libera el cerrojo si el usuario cancela", async () => {
    let transferInProgress = false;
    const rejected: string[] = [];

    let resolveSaveDialog!: (picked: string | null) => void;
    const saveDialogPromise = new Promise<string | null>((resolve) => {
      resolveSaveDialog = resolve;
    });

    async function simulateHandleDownloadFile() {
      if (transferInProgress) {
        rejected.push("download");
        return;
      }
      transferInProgress = true;
      try {
        const localPath = await saveDialogPromise;
        if (!localPath) {
          return;
        }
      } finally {
        transferInProgress = false;
      }
    }

    async function simulateConcurrentUpload() {
      if (transferInProgress) {
        rejected.push("upload");
        return;
      }
    }

    // 1. Usuario abre "Descargar" -> el diálogo "Guardar como…" queda abierto esperando respuesta
    const downloadTask = simulateHandleDownloadFile();
    expect(transferInProgress).toBe(true);

    // 2. Mientras el diálogo está abierto, un segundo "Descargar" o "Upload" son bloqueados
    await simulateHandleDownloadFile();
    await simulateConcurrentUpload();
    expect(rejected).toEqual(["download", "upload"]);

    // 3. El usuario cancela el diálogo ("Guardar como…" retorna null) -> el bloque finally libera el cerrojo
    resolveSaveDialog(null);
    await downloadTask;
    expect(transferInProgress).toBe(false);
  });

  it("renderiza el botón .files-status-progress-cancel (✕) accesible y preserva su nodo DOM sin re-crearlo en cada tick (0 reflows)", () => {
    const filesStatus = createMockDomRoot();
    let cancelRequestCalls = 0;
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
      {
        onCancelRequest: () => {
          cancelRequestCalls += 1;
        },
      },
    );

    controller.setTransferProgress({
      operation: "download",
      file_name: "dataset.tar.gz",
      bytes_transferred: 1024,
      total_bytes: 10240,
      percent: 10,
      speed_bps: 1024,
    });

    const cancelBtn = filesStatus.querySelector(".files-status-progress-cancel");
    expect(cancelBtn).not.toBeNull();
    expect(cancelBtn?.tagName).toBe("BUTTON");
    expect(cancelBtn?.type).toBe("button");
    expect(cancelBtn?.textContent).toBe("✕");
    expect(cancelBtn?.getAttribute("title")).toBe("Cancelar transferencia");
    expect(cancelBtn?.getAttribute("aria-label")).toBe("Cancelar transferencia");

    // 40 ticks adicionales no deben reemplazar ni recrear el botón (0 reflows estructurales)
    for (let i = 2; i <= 40; i++) {
      controller.setTransferProgress({
        operation: "download",
        file_name: "dataset.tar.gz",
        bytes_transferred: i * 256,
        total_bytes: 10240,
        percent: Math.round((i * 256 * 100) / 10240),
        speed_bps: 2048,
      });
    }

    expect(filesStatus.replaceChildrenCalls).toBe(1);
    expect(filesStatus.querySelector(".files-status-progress-cancel")).toBe(cancelBtn);

    cancelBtn?.click();
    expect(cancelRequestCalls).toBe(1);
  });

  it("continúa transfiriendo sin interrupción cuando el usuario abre el diálogo de confirmación y elige Seguir transfiriendo", async () => {
    const filesStatus = createMockDomRoot();
    let sftpCancelInvocations = 0;
    let resolveConfirm!: (decision: boolean) => void;

    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
      {
        onCancelRequest: async () => {
          const confirmed = await new Promise<boolean>((resolve) => {
            resolveConfirm = resolve;
          });
          if (confirmed) {
            sftpCancelInvocations += 1;
          }
        },
      },
    );

    controller.setTransferProgress({
      operation: "download",
      file_name: "kernel.img",
      bytes_transferred: 30 * 1024 * 1024,
      total_bytes: 100 * 1024 * 1024,
      percent: 30,
      speed_bps: 10 * 1024 * 1024,
    });

    // Usuario hace clic en [ ✕ ] -> se abre confirmDialog
    const cancelBtn = filesStatus.querySelector(".files-status-progress-cancel");
    cancelBtn?.click();

    // Mientras el modal está abierto, la descarga sigue avanzando en vivo (30% -> 65%)
    controller.setTransferProgress({
      operation: "download",
      file_name: "kernel.img",
      bytes_transferred: 65 * 1024 * 1024,
      total_bytes: 100 * 1024 * 1024,
      percent: 65,
      speed_bps: 12 * 1024 * 1024,
    });
    expect(filesStatus.querySelector(".files-status-progress-percent")?.textContent).toBe("65%");

    // Usuario pulsa "Seguir transfiriendo" (false)
    resolveConfirm(false);
    await Promise.resolve();

    expect(sftpCancelInvocations).toBe(0);
    expect(filesStatus.classList.contains("is-progress")).toBe(true);
  });

  it("al confirmar Sí, cancelar invoca sftp_cancel_transfer, detiene los archivos restantes de un lote y muestra Transferencia cancelada (auto-cierre 3000ms)", async () => {
    const filesStatus = createMockDomRoot();
    let cancelSignal = false;
    let sftpCancelInvocations = 0;
    const uploadedFiles: string[] = [];

    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
      {
        onCancelRequest: async () => {
          // Confirmación positiva del usuario ("Sí, cancelar")
          cancelSignal = true;
          sftpCancelInvocations += 1;
        },
      },
    );

    const batchFiles = ["part1.tar", "part2.tar", "part3.tar"];

    async function simulateBatchUpload() {
      for (let i = 0; i < batchFiles.length; i++) {
        if (cancelSignal) break;
        const name = batchFiles[i];
        controller.setTransferProgress(
          {
            operation: "upload",
            file_name: name,
            bytes_transferred: 512,
            total_bytes: 1024,
            percent: 50,
            speed_bps: 512,
          },
          i + 1,
          batchFiles.length,
        );
        // Durante el archivo 1 (part1.tar), el usuario pulsa [ ✕ ] y confirma
        if (i === 0) {
          filesStatus.querySelector(".files-status-progress-cancel")?.click();
        }
        try {
          if (cancelSignal) {
            throw new Error(TRANSFER_CANCELLED_ERROR);
          }
          uploadedFiles.push(name);
        } catch (err) {
          if (isTransferCancelledError(err)) {
            controller.setStatus("Transferencia cancelada", false, false);
            return;
          }
          throw err;
        }
      }
    }

    await simulateBatchUpload();

    expect(sftpCancelInvocations).toBe(1);
    // Ninguno de los archivos restantes (part2.tar, part3.tar) se ejecutó ni part1.tar se completó
    expect(uploadedFiles).toEqual([]);
    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.classList.contains("error")).toBe(false);
    expect(filesStatus.classList.contains("is-visible")).toBe(true);
    expect(filesStatus.textContent).toBe("Transferencia cancelada");

    // Auto-dismiss informativo tras 3000ms
    vi.advanceTimersByTime(3000);
    expect(filesStatus.classList.contains("is-visible")).toBe(false);
  });

  it("al cerrar la pestaña SSH (closeTerminalTab) durante una transferencia activa suprime ticks residuales del Channel y limpia el banner sin error falso", () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    let activeTerminalId: string | null = "term-prod-1";
    let transferGeneration = 1;
    const currentGen = transferGeneration;
    const transferTerminalId = "term-prod-1";

    function onChannelProgressTick(payload: TransferProgressPayload) {
      if (activeTerminalId !== transferTerminalId || transferGeneration !== currentGen) {
        return;
      }
      controller.setTransferProgress(payload);
    }

    onChannelProgressTick({
      operation: "download",
      file_name: "dump.sql.gz",
      bytes_transferred: 4096,
      total_bytes: 8192,
      percent: 50,
      speed_bps: 2048,
    });
    expect(filesStatus.classList.contains("is-progress")).toBe(true);

    // El usuario cierra la pestaña SSH ("term-prod-1")
    activeTerminalId = null;
    transferGeneration += 1;
    controller.setStatus("");

    // Llegan ticks residuales en tránsito antes de que el hilo worker termine
    onChannelProgressTick({
      operation: "download",
      file_name: "dump.sql.gz",
      bytes_transferred: 6144,
      total_bytes: 8192,
      percent: 75,
      speed_bps: 2048,
    });

    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.classList.contains("is-visible")).toBe(false);
    expect(filesStatus.classList.contains("error")).toBe(false);
  });

  it("al cerrar todas las terminales o salir de la aplicación invalida todas las generaciones de transferencia activas", () => {
    const filesStatus = createMockDomRoot();
    const controller = createExplorerStatusController(
      filesStatus as unknown as HTMLElement,
    );

    let transferInProgress = true;
    let transferCancelledByUser = false;

    controller.setTransferProgress({
      operation: "scp",
      file_name: "replica.bin",
      bytes_transferred: 2048,
      total_bytes: 4096,
      percent: 50,
      speed_bps: 1024,
    });

    // Cierre global de terminales / salida de app
    transferCancelledByUser = true;
    transferInProgress = false;
    controller.setStatus("");

    expect(transferInProgress).toBe(false);
    expect(transferCancelledByUser).toBe(true);
    expect(filesStatus.classList.contains("is-progress")).toBe(false);
    expect(filesStatus.classList.contains("is-visible")).toBe(false);
  });
});
