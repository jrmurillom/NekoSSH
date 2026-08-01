/**
 * Chrome overlays: confirm dialog (A1) + context menu (B3).
 * Look: docs/design/DESIGN.md § Confirmaciones / Menús contextuales.
 */
import type { IconNode } from "lucide";
import { icon } from "./icons";

export type ConfirmDialogOptions = {
  title: string;
  message: string;
  /** Simple mono impact line (mutually exclusive with detailFilename/detailFullPath). */
  impact?: string;
  /** Basename shown by default; pair with detailFullPath for collapsible full path. */
  detailFilename?: string;
  /** Full path shown in a readonly textarea when the user expands “ver ruta completa”. */
  detailFullPath?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};

export type AlertDialogOptions = {
  title: string;
  message: string;
  okLabel?: string;
};

export type ContextMenuItem = {
  id: string;
  label: string;
  icon?: IconNode;
  danger?: boolean;
  separatorBefore?: boolean;
};

let dialogRoot: HTMLElement | null = null;
let menuEl: HTMLElement | null = null;
let menuOutsideHandler: ((e: MouseEvent) => void) | null = null;
let menuKeyHandler: ((e: KeyboardEvent) => void) | null = null;

function ensureDialogRoot(): HTMLElement {
  if (dialogRoot && document.body.contains(dialogRoot)) {
    if (dialogRoot.querySelector(".chrome-dialog-detail")) return dialogRoot;
    dialogRoot.remove();
    dialogRoot = null;
  }
  dialogRoot = document.createElement("div");
  dialogRoot.id = "chrome-dialog-root";
  dialogRoot.className = "chrome-dialog-root";
  dialogRoot.hidden = true;
  dialogRoot.innerHTML = `
    <div class="chrome-dialog-overlay" data-chrome-dialog-dismiss></div>
    <div class="chrome-dialog" role="dialog" aria-modal="true" aria-labelledby="chrome-dialog-title">
      <h3 id="chrome-dialog-title" class="chrome-dialog-title"></h3>
      <p class="chrome-dialog-message"></p>
      <p class="chrome-dialog-impact" hidden></p>
      <div class="chrome-dialog-detail" hidden>
        <p class="chrome-dialog-detail-filename"></p>
        <button type="button" class="chrome-dialog-path-toggle">ver ruta completa</button>
        <textarea class="chrome-dialog-path-full" readonly rows="3" hidden></textarea>
      </div>
      <div class="chrome-dialog-actions"></div>
    </div>
  `;
  document.body.appendChild(dialogRoot);
  return dialogRoot;
}

function closeDialog() {
  const root = ensureDialogRoot();
  root.hidden = true;
  root.classList.remove("is-open");
}

function resetDialogDetail(root: HTMLElement) {
  const detailEl = root.querySelector(".chrome-dialog-detail") as HTMLElement | null;
  const filenameEl = root.querySelector(".chrome-dialog-detail-filename") as HTMLElement | null;
  const toggleBtn = root.querySelector(".chrome-dialog-path-toggle") as HTMLButtonElement | null;
  const pathFull = root.querySelector(".chrome-dialog-path-full") as HTMLTextAreaElement | null;
  if (detailEl) detailEl.hidden = true;
  if (filenameEl) filenameEl.textContent = "";
  if (toggleBtn) {
    toggleBtn.textContent = "ver ruta completa";
    toggleBtn.setAttribute("aria-expanded", "false");
  }
  if (pathFull) {
    pathFull.hidden = true;
    pathFull.value = "";
  }
}

/** A1 glass confirm. Resolves true if confirmed. */
export function confirmDialog(opts: ConfirmDialogOptions): Promise<boolean> {
  return new Promise((resolve) => {
    const root = ensureDialogRoot();
    const titleEl = root.querySelector(".chrome-dialog-title") as HTMLElement;
    const msgEl = root.querySelector(".chrome-dialog-message") as HTMLElement;
    const impactEl = root.querySelector(".chrome-dialog-impact") as HTMLElement;
    const detailEl = root.querySelector(".chrome-dialog-detail") as HTMLElement;
    const filenameEl = root.querySelector(".chrome-dialog-detail-filename") as HTMLElement;
    const toggleBtn = root.querySelector(".chrome-dialog-path-toggle") as HTMLButtonElement;
    const pathFull = root.querySelector(".chrome-dialog-path-full") as HTMLTextAreaElement;
    const actions = root.querySelector(".chrome-dialog-actions") as HTMLElement;
    const overlay = root.querySelector("[data-chrome-dialog-dismiss]") as HTMLElement;

    titleEl.textContent = opts.title;
    msgEl.textContent = opts.message;

    const usePathDetail =
      typeof opts.detailFilename === "string" &&
      opts.detailFilename.length > 0 &&
      typeof opts.detailFullPath === "string";

    resetDialogDetail(root);
    if (usePathDetail) {
      impactEl.hidden = true;
      impactEl.textContent = "";
      detailEl.hidden = false;
      filenameEl.textContent = opts.detailFilename!;
      pathFull.value = opts.detailFullPath!;
      pathFull.hidden = true;
      toggleBtn.textContent = "ver ruta completa";
      toggleBtn.setAttribute("aria-expanded", "false");
    } else if (opts.impact) {
      impactEl.hidden = false;
      impactEl.textContent = opts.impact;
    } else {
      impactEl.hidden = true;
      impactEl.textContent = "";
    }

    actions.replaceChildren();
    const cancelBtn = document.createElement("button");
    cancelBtn.type = "button";
    cancelBtn.className = "btn chrome-btn-ghost";
    cancelBtn.textContent = opts.cancelLabel ?? "Cancelar";

    const confirmBtn = document.createElement("button");
    confirmBtn.type = "button";
    confirmBtn.className = opts.danger !== false ? "btn chrome-btn-danger" : "btn chrome-btn-primary";
    confirmBtn.textContent = opts.confirmLabel ?? "Confirmar";

    const onTogglePath = (e: Event) => {
      e.preventDefault();
      e.stopPropagation();
      const expanded = pathFull.hidden;
      pathFull.hidden = !expanded;
      toggleBtn.textContent = expanded ? "ocultar ruta" : "ver ruta completa";
      toggleBtn.setAttribute("aria-expanded", expanded ? "true" : "false");
    };

    const finish = (value: boolean) => {
      document.removeEventListener("keydown", onKey);
      overlay.removeEventListener("click", onOverlay);
      toggleBtn.removeEventListener("click", onTogglePath);
      closeDialog();
      resolve(value);
    };

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        finish(false);
      } else if (e.key === "Enter") {
        const t = e.target as HTMLElement | null;
        if (t?.closest?.(".chrome-dialog-path-toggle") || t?.closest?.(".chrome-dialog-path-full")) {
          return;
        }
        e.preventDefault();
        finish(true);
      }
    };

    const onOverlay = () => finish(false);

    if (usePathDetail) {
      toggleBtn.addEventListener("click", onTogglePath);
    }
    cancelBtn.addEventListener("click", () => finish(false));
    confirmBtn.addEventListener("click", () => finish(true));
    overlay.addEventListener("click", onOverlay);
    document.addEventListener("keydown", onKey);

    actions.append(cancelBtn, confirmBtn);
    root.hidden = false;
    root.classList.add("is-open");
    queueMicrotask(() => confirmBtn.focus());
  });
}

/** A1 single-button alert (replaces window.alert). */
export function alertDialog(opts: AlertDialogOptions): Promise<void> {
  return new Promise((resolve) => {
    const root = ensureDialogRoot();
    const titleEl = root.querySelector(".chrome-dialog-title") as HTMLElement;
    const msgEl = root.querySelector(".chrome-dialog-message") as HTMLElement;
    const impactEl = root.querySelector(".chrome-dialog-impact") as HTMLElement;
    const actions = root.querySelector(".chrome-dialog-actions") as HTMLElement;
    const overlay = root.querySelector("[data-chrome-dialog-dismiss]") as HTMLElement;

    titleEl.textContent = opts.title;
    msgEl.textContent = opts.message;
    impactEl.hidden = true;
    resetDialogDetail(root);

    actions.replaceChildren();
    const okBtn = document.createElement("button");
    okBtn.type = "button";
    okBtn.className = "btn chrome-btn-primary";
    okBtn.textContent = opts.okLabel ?? "Entendido";

    const finish = () => {
      document.removeEventListener("keydown", onKey);
      overlay.removeEventListener("click", onOverlay);
      closeDialog();
      resolve();
    };

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" || e.key === "Enter") {
        e.preventDefault();
        finish();
      }
    };
    const onOverlay = () => finish();

    okBtn.addEventListener("click", finish);
    overlay.addEventListener("click", onOverlay);
    document.addEventListener("keydown", onKey);

    actions.append(okBtn);
    root.hidden = false;
    root.classList.add("is-open");
    queueMicrotask(() => okBtn.focus());
  });
}

function closeContextMenu() {
  if (menuOutsideHandler) {
    document.removeEventListener("mousedown", menuOutsideHandler, true);
    menuOutsideHandler = null;
  }
  if (menuKeyHandler) {
    document.removeEventListener("keydown", menuKeyHandler, true);
    menuKeyHandler = null;
  }
  if (menuEl) {
    menuEl.remove();
    menuEl = null;
  }
}

/** B3 context menu. Resolves selected item id, or null if dismissed. */
export function showContextMenu(
  clientX: number,
  clientY: number,
  items: ContextMenuItem[],
): Promise<string | null> {
  closeContextMenu();
  return new Promise((resolve) => {
    const menu = document.createElement("div");
    menu.className = "chrome-context-menu";
    menu.setAttribute("role", "menu");

    const finish = (id: string | null) => {
      closeContextMenu();
      resolve(id);
    };

    for (const item of items) {
      if (item.separatorBefore) {
        const sep = document.createElement("div");
        sep.className = "chrome-context-sep";
        sep.setAttribute("role", "separator");
        menu.appendChild(sep);
      }
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "chrome-context-item" + (item.danger ? " is-danger" : "");
      btn.setAttribute("role", "menuitem");
      if (item.icon) {
        btn.appendChild(icon(item.icon, { size: 14, className: "icon--sm" }));
      }
      const label = document.createElement("span");
      label.textContent = item.label;
      btn.appendChild(label);
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        finish(item.id);
      });
      menu.appendChild(btn);
    }

    document.body.appendChild(menu);
    menuEl = menu;

    const pad = 8;
    const rect = menu.getBoundingClientRect();
    let left = clientX;
    let top = clientY;
    if (left + rect.width > window.innerWidth - pad) {
      left = Math.max(pad, window.innerWidth - rect.width - pad);
    }
    if (top + rect.height > window.innerHeight - pad) {
      top = Math.max(pad, window.innerHeight - rect.height - pad);
    }
    menu.style.left = `${left}px`;
    menu.style.top = `${top}px`;

    menuOutsideHandler = (e: MouseEvent) => {
      if (!menu.contains(e.target as Node)) finish(null);
    };
    menuKeyHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        finish(null);
      }
    };
    // Defer so the opening contextmenu event doesn't instantly close.
    queueMicrotask(() => {
      document.addEventListener("mousedown", menuOutsideHandler!, true);
      document.addEventListener("keydown", menuKeyHandler!, true);
    });
  });
}
