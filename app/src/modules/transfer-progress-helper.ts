export interface TransferProgressPayload {
  operation: "upload" | "scp" | "download";
  file_name: string;
  bytes_transferred: number;
  total_bytes: number;
  percent: number;
  speed_bps: number;
}

/**
 * Formatea una cantidad de bytes en una unidad legible (B, KB, MB, GB).
 */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return "0 B";
  }
  const kb = 1024;
  const mb = kb * 1024;
  const gb = mb * 1024;

  if (bytes >= gb) {
    return `${(bytes / gb).toFixed(2)} GB`;
  }
  if (bytes >= mb) {
    return `${(bytes / mb).toFixed(1)} MB`;
  }
  if (bytes >= kb) {
    return `${(bytes / kb).toFixed(1)} KB`;
  }
  return `${Math.round(bytes)} B`;
}

/**
 * Formatea la velocidad de transferencia en bytes por segundo (B/s, KB/s, MB/s, GB/s).
 */
export function formatSpeed(speedBps: number): string {
  if (!Number.isFinite(speedBps) || speedBps <= 0) {
    return "0 B/s";
  }
  return `${formatBytes(speedBps)}/s`;
}

/**
 * Acota el porcentaje dentro del rango entero [0, 100].
 */
export function clampPercent(percent: number): number {
  if (!Number.isFinite(percent)) {
    return 0;
  }
  return Math.max(0, Math.min(100, Math.round(percent)));
}

/**
 * Construye el encabezado de progreso diferenciado por operación y lote.
 */
export function formatTransferHeader(
  payload: TransferProgressPayload,
  batchIndex?: number,
  batchTotal?: number,
): { titleText: string; percentText: string } {
  const actionLabel =
    payload.operation === "scp"
      ? "Copiando SCP"
      : payload.operation === "download"
        ? "Descargando"
        : "Subiendo";
  const batchSuffix =
    batchIndex !== undefined && batchTotal !== undefined && batchTotal > 1
      ? ` (${batchIndex}/${batchTotal})`
      : "";
  const pct = clampPercent(payload.percent);
  return {
    titleText: `${actionLabel} ${payload.file_name}${batchSuffix}`,
    percentText: `${pct}%`,
  };
}

/**
 * Construye la línea secundaria de métricas (bytes transferidos / totales y velocidad actual).
 * En estado pre-flight (total_bytes === 0 && percent === 0) muestra "Preparando transferencia…".
 */
export function formatTransferMetrics(payload: TransferProgressPayload): string {
  if (payload.total_bytes === 0 && payload.percent === 0) {
    return "Preparando transferencia…";
  }
  const transferred = formatBytes(payload.bytes_transferred);
  const total = formatBytes(payload.total_bytes);
  if (payload.speed_bps > 0) {
    return `${transferred} / ${total} • ${formatSpeed(payload.speed_bps)}`;
  }
  return `${transferred} / ${total}`;
}

export const TRANSFER_CANCELLED_ERROR = "TRANSFER_CANCELLED";

/**
 * Determina si un error devuelto por Tauri/Rust corresponde a una cancelación intencional de transferencia.
 */
export function isTransferCancelledError(err: unknown): boolean {
  if (!err) {
    return false;
  }
  return String(err).includes(TRANSFER_CANCELLED_ERROR);
}

export interface StatusProgressRefs {
  root: HTMLElement;
  titleEl: HTMLElement;
  percentEl: HTMLElement;
  cancelBtnEl: HTMLButtonElement;
  barEl: HTMLElement;
  metaEl: HTMLElement;
}

export interface ExplorerStatusOptions {
  onCancelRequest?: () => void;
}

export interface ExplorerStatusController {
  setStatus: (message: string, isError?: boolean, isSuccess?: boolean) => void;
  setTransferProgress: (
    payload: TransferProgressPayload,
    batchIndex?: number,
    batchTotal?: number,
  ) => void;
}

/**
 * Controlador DOM modular para el banner `#files-status`.
 * Reutiliza referencias DOM durante los ticks de progreso (0 reflows estructurales)
 * y gestiona el temporizador de auto-cierre (3000ms para éxito/info, persistente en error).
 */
export function createExplorerStatusController(
  filesStatus: HTMLElement,
  options?: ExplorerStatusOptions,
): ExplorerStatusController {
  let statusDismissTimer: ReturnType<typeof setTimeout> | null = null;
  let statusProgressRefs: StatusProgressRefs | null = null;

  function clearTimers() {
    if (statusDismissTimer) {
      clearTimeout(statusDismissTimer);
      statusDismissTimer = null;
    }
  }

  function setTransferProgress(
    payload: TransferProgressPayload,
    batchIndex?: number,
    batchTotal?: number,
  ): void {
    clearTimers();

    if (
      !statusProgressRefs ||
      statusProgressRefs.root !== filesStatus ||
      !filesStatus.classList.contains("is-progress")
    ) {
      const doc = filesStatus.ownerDocument ?? document;
      const headerEl = doc.createElement("div");
      headerEl.className = "files-status-progress-header";

      const titleEl = doc.createElement("span");
      titleEl.className = "files-status-progress-title";

      const percentEl = doc.createElement("span");
      percentEl.className = "files-status-progress-percent";

      const cancelBtnEl = doc.createElement("button");
      cancelBtnEl.type = "button";
      cancelBtnEl.className = "files-status-progress-cancel";
      cancelBtnEl.title = "Cancelar transferencia";
      cancelBtnEl.setAttribute("aria-label", "Cancelar transferencia");
      cancelBtnEl.textContent = "✕";
      cancelBtnEl.addEventListener("click", (ev) => {
        ev.stopPropagation();
        options?.onCancelRequest?.();
      });

      headerEl.appendChild(titleEl);
      headerEl.appendChild(percentEl);
      headerEl.appendChild(cancelBtnEl);

      const trackEl = doc.createElement("div");
      trackEl.className = "files-status-progress-track";

      const barEl = doc.createElement("div");
      barEl.className = "files-status-progress-bar";
      trackEl.appendChild(barEl);

      const metaEl = doc.createElement("div");
      metaEl.className = "files-status-progress-meta";

      filesStatus.replaceChildren(headerEl, trackEl, metaEl);
      statusProgressRefs = {
        root: filesStatus,
        titleEl,
        percentEl,
        cancelBtnEl,
        barEl,
        metaEl,
      };
    }

    const header = formatTransferHeader(payload, batchIndex, batchTotal);
    const pct = clampPercent(payload.percent);

    statusProgressRefs.titleEl.textContent = header.titleText;
    statusProgressRefs.percentEl.textContent = header.percentText;
    statusProgressRefs.barEl.style.width = `${pct}%`;
    statusProgressRefs.metaEl.textContent = formatTransferMetrics(payload);

    filesStatus.classList.remove("error", "success");
    filesStatus.classList.add("is-visible", "is-progress");
    filesStatus.setAttribute("title", `${header.titleText} — ${header.percentText}`);
  }

  function setStatus(message: string, isError = false, isSuccess = false): void {
    clearTimers();
    filesStatus.classList.remove("is-progress");
    statusProgressRefs = null;

    if (!message) {
      filesStatus.classList.remove("is-visible", "error", "success");
      filesStatus.textContent = "";
      filesStatus.setAttribute("title", "");
      return;
    }

    let displayMessage = message;
    if (isSuccess) {
      displayMessage = `✅ ${message}`;
    } else if (isError) {
      displayMessage = `❌ ${message}`;
    }

    filesStatus.textContent = displayMessage;
    filesStatus.setAttribute("title", displayMessage);
    filesStatus.classList.toggle("error", isError);
    filesStatus.classList.toggle("success", isSuccess);
    filesStatus.classList.add("is-visible");

    if (!isError) {
      statusDismissTimer = setTimeout(() => {
        filesStatus.classList.remove("is-visible");
        setTimeout(() => {
          filesStatus.classList.remove("error", "success");
        }, 300);
        statusDismissTimer = null;
      }, 3000);
    }
  }

  return {
    setStatus,
    setTransferProgress,
  };
}

