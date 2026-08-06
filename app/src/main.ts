// --- NekoSSH Frontend Controller (Cyber-Sakura Estética) ---
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { AppIcons, icon, setButtonIcon } from "./icons";
import { alertDialog, confirmDialog, showContextMenu } from "./overlays";
import { initSnippetsUi } from "./snippets-ui";
import { stripTrailingPasteNoise } from "./strip-trailing-paste";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { clampAndFormatOpacity, calculateTerminalOverlayOpacity, resolveBackgroundUrl, resolveBackgroundApply } from "./bg-settings-helper";
import { parseRemoteHistoryLines, copyCommandToClipboard } from "./modules/remote-history-helper";
import {
  MAX_CHILD_SHELLS,
  canAddChildShell,
  childShellLabel,
  focusIndexAfterClose,
  gridDensityClass,
} from "./modules/shell-grid-helper";
import { resolveBrandLogoUrl } from "./modules/brand-logo-helper";
import {
  BG_BY_THEME_KEY,
  DEFAULT_WALLPAPER_OPACITY,
  LEGACY_BG_LABEL_KEY,
  LEGACY_BG_OPACITY_KEY,
  LEGACY_BG_URL_KEY,
  parseThemeWallpaperMap,
  type ThemeWallpaper,
} from "./modules/theme-wallpaper-helper";
import logoNekossh from "./assets/logos/nekossh.png";
import logoHatsuneMiku from "./assets/logos/hatsune-miku.png";
import logoReiAyanami from "./assets/logos/rei-ayanami.png";
import logoNeonEvangelion from "./assets/logos/neon-evangelion.png";
import logoCyberpunkDavid from "./assets/logos/cyberpunk-david.png";
import logoCyberpunkLucy from "./assets/logos/cyberpunk-lucy.png";
import logoPersona5 from "./assets/logos/persona5.png";
import logoSailorMoon from "./assets/logos/sailor-moon.png";

const BRAND_LOGO_URLS: Record<string, string> = {
  nekossh: logoNekossh,
  "hatsune-miku": logoHatsuneMiku,
  "rei-ayanami": logoReiAyanami,
  "neon-evangelion": logoNeonEvangelion,
  "cyberpunk-david": logoCyberpunkDavid,
  "cyberpunk-lucy": logoCyberpunkLucy,
  persona5: logoPersona5,
  "sailor-moon": logoSailorMoon,
};

const THEME_TERMINAL_COLORS: Record<string, Record<string, string>> = {
  "nekossh": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#f8f8f2",
    cursor: "#ff69b4",
    cursorAccent: "#080409",
    selectionBackground: "rgba(255, 105, 180, 0.3)",
    black: "#000000",
    red: "#ff3131",
    green: "#39ff14",
    yellow: "#ffb86c",
    blue: "#bd93f9",
    magenta: "#ff69b4",
    cyan: "#00ffff",
    white: "#f8f8f2"
  },
  "hatsune-miku": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#e8f4f2",
    cursor: "#39c5bb",
    cursorAccent: "#060d0d",
    selectionBackground: "rgba(57, 197, 187, 0.3)",
    black: "#000000",
    red: "#ff4444",
    green: "#39ff14",
    yellow: "#e6db74",
    blue: "#66d9ef",
    magenta: "#e84f8a",
    cyan: "#39c5bb",
    white: "#e8f4f2"
  },
  "rei-ayanami": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#dce6f0",
    cursor: "#4a7dbd",
    cursorAccent: "#060810",
    selectionBackground: "rgba(74, 125, 189, 0.3)",
    black: "#000000",
    red: "#e74c3c",
    green: "#39ff14",
    yellow: "#f0c674",
    blue: "#4a7dbd",
    magenta: "#c0392b",
    cyan: "#6c8ebf",
    white: "#dce6f0"
  },
  "neon-evangelion": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#e8e6f0",
    cursor: "#66ff00",
    cursorAccent: "#0a0418",
    selectionBackground: "rgba(102, 255, 0, 0.3)",
    black: "#000000",
    red: "#ff3131",
    green: "#66ff00",
    yellow: "#ff6600",
    blue: "#9b59b6",
    magenta: "#cc44ff",
    cyan: "#a3ff66",
    white: "#e8e6f0"
  },
  "cyberpunk-david": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#f0ece0",
    cursor: "#f5c518",
    cursorAccent: "#0a0a06",
    selectionBackground: "rgba(245, 197, 24, 0.3)",
    black: "#000000",
    red: "#e63946",
    green: "#39ff14",
    yellow: "#f5c518",
    blue: "#ff8c00",
    magenta: "#e63946",
    cyan: "#fad961",
    white: "#f0ece0"
  },
  "cyberpunk-lucy": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#f0e8f5",
    cursor: "#e040fb",
    cursorAccent: "#0a0610",
    selectionBackground: "rgba(224, 64, 251, 0.3)",
    black: "#000000",
    red: "#ff3131",
    green: "#39ff14",
    yellow: "#f0a0ff",
    blue: "#29b6f6",
    magenta: "#e040fb",
    cyan: "#29b6f6",
    white: "#f0e8f5"
  },
  "persona5": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#ffffff",
    cursor: "#e60012",
    cursorAccent: "#080808",
    selectionBackground: "rgba(230, 0, 18, 0.3)",
    black: "#000000",
    red: "#ff0000",
    green: "#39ff14",
    yellow: "#ff4d5a",
    blue: "#ffffff",
    magenta: "#e60012",
    cyan: "#999999",
    white: "#ffffff"
  },
  "sailor-moon": {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#f5f0ff",
    cursor: "#ffd700",
    cursorAccent: "#0a0814",
    selectionBackground: "rgba(255, 215, 0, 0.3)",
    black: "#000000",
    red: "#ff3131",
    green: "#39ff14",
    yellow: "#ffd700",
    blue: "#1e3a5f",
    magenta: "#ff69b4",
    cyan: "#ffe766",
    white: "#f5f0ff"
  }
};

function getActiveTheme(): string {
  return localStorage.getItem("nekossh-theme") || "nekossh";
}

function applyTheme(themeName: string): void {
  document.documentElement.dataset.theme = themeName;
  localStorage.setItem("nekossh-theme", themeName);

  // Sincronizar colores de todos los terminales xterm.js abiertos
  const termColors = THEME_TERMINAL_COLORS[themeName] || THEME_TERMINAL_COLORS["nekossh"];
  shellPanes.forEach((pane) => {
    pane.term.options.theme = { ...termColors };
  });

  // Logo de marca alineado al tema (fallback interno a nekossh)
  const brandLogo = document.querySelector<HTMLImageElement>(".brand-logo");
  if (brandLogo) {
    brandLogo.src = resolveBrandLogoUrl(themeName, BRAND_LOGO_URLS);
  }

  // Wallpaper de terminal scoped al tema (async IPC)
  void applyThemeWallpaper(themeName);

  // Actualizar indicador visual del selector
  document.querySelectorAll(".theme-item").forEach(item => {
    item.classList.toggle("is-active", (item as HTMLElement).dataset.themeId === themeName);
  });
}

// --- Interfaces ---
interface ConnectionFolder {
  id?: number;
  name: string;
  sort_order: number;
}

interface ConnectionProfile {
  id?: number;
  folder_id?: number;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_type: 'password' | 'key';
  password?: string;
  /** Contenido PEM de la llave (no ruta). */
  private_key?: string;
  passphrase?: string;
  keepalive: number;
  tunnel_type: 'none' | 'local' | 'dynamic';
  tunnel_local_port?: number;
  tunnel_dest?: string;
}

type ShellRole = "parent" | "child";

/** Un shell dentro del contexto de pestaña: su propio terminal_id y Session SSH. */
interface ShellPane {
  terminalId: string;
  contextId: string;
  role: ShellRole;
  label: string;
  term: Terminal;
  fitAddon: FitAddon;
  cellEl: HTMLElement;
  isConnected: boolean;
  isReconnecting: boolean;
}

/**
 * Contexto de pestaña: shell padre (ancla SFTP) + hasta MAX_CHILD_SHELLS hijos.
 * `id` es el terminal_id del padre, por lo que el explorador sigue ligado al padre.
 */
interface ActiveTerminal {
  id: string;
  profileName: string;
  /** Snapshot / profile used to open (and reconnect) this tab */
  profile: ConnectionProfile;
  /** panes[0] es siempre el shell padre */
  panes: ShellPane[];
  focusedTerminalId: string;
  gridEl: HTMLElement;
  panelEl: HTMLElement;
  tabEl: HTMLElement;
  addShellBtn: HTMLButtonElement | null;
  childSeq: number;
  /** Alias al shell padre (compatibilidad con el resto del frontend) */
  readonly term: Terminal;
  readonly fitAddon: FitAddon;
  isConnected: boolean;
  isReconnecting: boolean;
  explorerCwd?: string;
  explorerRoot?: ExplorerNodeState | null;
}

interface SshEventPayload {
  terminal_id: string;
  data: string;
}

interface SshClosedPayload {
  terminal_id: string;
  error?: string;
}

interface ExternalEditProbe {
  size: number;
  too_large: boolean;
  looks_binary: boolean;
  basename: string;
}

interface EditSessionChangedPayload {
  edit_id: string;
  terminal_id: string;
  remote_path: string;
  reason: string;
}

interface EditSessionDisconnectedPayload {
  terminal_id: string;
  edit_ids: string[];
  message: string;
}

/** edit_id con dialog A1 de subida abierto (coalesce en frontend). */
const editUploadConfirmOpen = new Set<string>();

// --- State Management ---
let currentFolders: ConnectionFolder[] = [];
let currentProfiles: ConnectionProfile[] = [];
let activeProfileId: number | null = null;
/** Folder context for "new connection" and highlight */
let activeFolderId: number | null = null;
const expandedFolderIds = new Set<number>();
let renamingFolderId: number | null = null;
let renamingProfileId: number | null = null;
let foldersExpandSeeded = false;

const activeTerminals = new Map<string, ActiveTerminal>();
let currentActiveTerminalId: string | null = null;

/** Índice global de shells por terminal_id, para enrutar eventos SSH. */
const shellPanes = new Map<string, ShellPane>();

function getContextForTerminal(terminalId: string): ActiveTerminal | undefined {
  const pane = shellPanes.get(terminalId);
  return pane ? activeTerminals.get(pane.contextId) : undefined;
}

// --- DOM Elements ---
let configBgUrlInput: HTMLInputElement | null = null;
let configBgOpacityInput: HTMLInputElement | null = null;
let opacityValLabel: HTMLElement | null = null;
let btnApplyBg: HTMLButtonElement | null = null;
let configEditorPathInput: HTMLInputElement | null = null;
let btnSaveEditorPref: HTMLButtonElement | null = null;

// Modal Elements
let profileModal: HTMLElement | null = null;
let profileForm: HTMLFormElement | null = null;
let modalTitle: HTMLElement | null = null;
let profileIdInput: HTMLInputElement | null = null;

let profNameInput: HTMLInputElement | null = null;
let profHostInput: HTMLInputElement | null = null;
let profPortInput: HTMLInputElement | null = null;
let profUsernameInput: HTMLInputElement | null = null;
let profAuthTypeSelect: HTMLSelectElement | null = null;
let profPasswordInput: HTMLInputElement | null = null;
let profKeyStatusEl: HTMLElement | null = null;
let profPassphraseInput: HTMLInputElement | null = null;
let profKeepaliveInput: HTMLInputElement | null = null;

let tunTypeSelect: HTMLSelectElement | null = null;
let tunLocalPortInput: HTMLInputElement | null = null;
let tunDestInput: HTMLInputElement | null = null;

let btnNewProfile: HTMLButtonElement | null = null;
let btnNewFolder: HTMLButtonElement | null = null;
let btnCancelProfile: HTMLButtonElement | null = null;
let profileListContainer: HTMLElement | null = null;
let profileFolderIdInput: HTMLInputElement | null = null;

/** PEM seleccionado en el formulario (no se muestra en UI). */
let draftPrivateKeyContent: string | null = null;
/** Material ya persistido al abrir el modal de edición. */
let existingPrivateKeyContent: string | null = null;

// Tabs sidebar
let tabBtnServers: HTMLButtonElement | null = null;
let tabBtnFiles: HTMLButtonElement | null = null;
let tabBtnMonitor: HTMLButtonElement | null = null;
let panelServers: HTMLElement | null = null;
let panelFiles: HTMLElement | null = null;
let panelMonitor: HTMLElement | null = null;

// Monitor elements
let monitorEmpty: HTMLElement | null = null;
let monitorContent: HTMLElement | null = null;
let monitorServerNameText: HTMLElement | null = null;
let monitorCpuValue: HTMLElement | null = null;
let monitorCpuLoad: HTMLElement | null = null;
let monitorCpuCores: HTMLElement | null = null;
let monitorRamValue: HTMLElement | null = null;
let monitorRamDetail: HTMLElement | null = null;
let monitorRamFree: HTMLElement | null = null;
let monitorDiskValue: HTMLElement | null = null;
let monitorDiskFill: HTMLElement | null = null;
let monitorDiskDetail: HTMLElement | null = null;
let monitorIntervalSelect: HTMLSelectElement | null = null;
let btnMonitorPause: HTMLButtonElement | null = null;
let monitorOsText: HTMLElement | null = null;
let monitorUptimeText: HTMLElement | null = null;
let monitorNetDown: HTMLElement | null = null;
let monitorNetUp: HTMLElement | null = null;
let monitorProcessesList: HTMLElement | null = null;
let monitorBtnPauseText: HTMLElement | null = null;

// Network speed deltas
let lastNetRecv: number = 0;
let lastNetSent: number = 0;
let lastNetTime: number = 0;

// Sparkline states
let cpuHistory: number[] = Array(30).fill(0);
let ramHistory: number[] = Array(30).fill(0);
let prevCpuActive: number = 0;
let prevCpuTotal: number = 0;
let monitorTimerId: any = null;
let isMonitorPaused: boolean = false;

// File explorer (Fase 2)
let filesEmpty: HTMLElement | null = null;
let filesToolbar: HTMLElement | null = null;
let filesCwdInput: HTMLInputElement | null = null;
let filesStatus: HTMLElement | null = null;
let filesTree: HTMLElement | null = null;
let filesContextMenu: HTMLElement | null = null;
let btnFilesUp: HTMLButtonElement | null = null;
let btnFilesGo: HTMLButtonElement | null = null;
let btnFilesRefresh: HTMLButtonElement | null = null;

// Remote history (Fase 5)
let historyModal: HTMLElement | null = null;
let btnCloseHistory: HTMLButtonElement | null = null;
let historySearchInput: HTMLInputElement | null = null;
let historyListTable: HTMLElement | null = null;
let btnHistoryPrev: HTMLButtonElement | null = null;
let btnHistoryNext: HTMLButtonElement | null = null;

interface HistoryCommandItem {
  date: string;
  command: string;
}
let historyItems: HistoryCommandItem[] = [];
let historyOffset = 0;
const historyLimit = 100;
let historySelectedRowIndex = -1;


interface SftpDirEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
}

interface ExplorerNodeState {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
  expanded: boolean;
  loaded: boolean;
  children: ExplorerNodeState[];
}

let explorerRoot: ExplorerNodeState | null = null;
let explorerCwd = "";
let explorerBoundTerminalId: string | null = null;
let contextMenuPath: string | null = null;
let explorerLoading = false;
let scpClipboard: {
  terminalId: string;
  path: string;
  name: string;
} | null = null;
let statusDismissTimer: ReturnType<typeof setTimeout> | null = null;

// Terminal layout elements
let mainDisplayArea: HTMLElement | null = null;
let terminalTabsList: HTMLElement | null = null;
let btnCloseAllTerminals: HTMLButtonElement | null = null;

/**
 * URL real que se pinta como fondo (path absoluto bajo app data, http(s) o vacío).
 * El input de preferencias muestra solo la etiqueta del archivo.
 */
let bgImageUrl = "";

type ThemeWallpaperDto = {
  theme_id: string;
  label: string;
  opacity: number;
  source_kind: "file" | "url" | "none";
  display_url: string;
};

function dtoToEntry(dto: ThemeWallpaperDto): ThemeWallpaper {
  return {
    url: dto.display_url || "",
    label: dto.label || "",
    opacity: clampAndFormatOpacity(dto.opacity).numeric,
  };
}

async function fetchThemeWallpaper(themeId: string): Promise<ThemeWallpaper> {
  try {
    const dto = await invoke<ThemeWallpaperDto>("get_theme_wallpaper_cmd", { themeId });
    return dtoToEntry(dto);
  } catch (err) {
    console.error("No se pudo cargar wallpaper del tema:", err);
    return {
      url: "",
      label: "",
      opacity: DEFAULT_WALLPAPER_OPACITY,
    };
  }
}

function clearLegacyWallpaperKeys(): void {
  localStorage.removeItem(LEGACY_BG_URL_KEY);
  localStorage.removeItem(LEGACY_BG_LABEL_KEY);
  localStorage.removeItem(LEGACY_BG_OPACITY_KEY);
  localStorage.removeItem(BG_BY_THEME_KEY);
}

/**
 * Migración one-shot: localStorage (mapa por tema + legacy globales) → SQLite + disco.
 */
async function migrateWallpapersFromLocalStorageIfNeeded(): Promise<void> {
  const mapRaw = localStorage.getItem(BG_BY_THEME_KEY);
  const legacyUrl = localStorage.getItem(LEGACY_BG_URL_KEY);
  const legacyLabel = localStorage.getItem(LEGACY_BG_LABEL_KEY);
  const legacyOpacity = localStorage.getItem(LEGACY_BG_OPACITY_KEY);
  const hasLegacy =
    (legacyUrl != null && legacyUrl !== "") ||
    (legacyLabel != null && legacyLabel !== "") ||
    legacyOpacity != null;

  if (!mapRaw && !hasLegacy) return;

  const map = parseThemeWallpaperMap(mapRaw);
  if (hasLegacy && !Object.prototype.hasOwnProperty.call(map, getActiveTheme())) {
    const opacity = clampAndFormatOpacity(
      legacyOpacity != null ? parseFloat(legacyOpacity) : DEFAULT_WALLPAPER_OPACITY,
    ).numeric;
    const url = legacyUrl ?? "";
    map[getActiveTheme()] = {
      url,
      label: legacyLabel || (url && !url.startsWith("data:") ? url : ""),
      opacity,
    };
  }

  for (const [themeId, entry] of Object.entries(map)) {
    if (!entry.url) {
      try {
        await invoke("set_theme_wallpaper_opacity_cmd", {
          themeId,
          opacity: entry.opacity,
        });
      } catch (err) {
        console.error(`Migración opacity ${themeId}:`, err);
      }
      continue;
    }
    try {
      if (entry.url.startsWith("data:")) {
        await invoke("set_theme_wallpaper_data_url_cmd", {
          themeId,
          dataUrl: entry.url,
          label: entry.label || "",
          opacity: entry.opacity,
        });
      } else if (entry.url.startsWith("http://") || entry.url.startsWith("https://")) {
        await invoke("set_theme_wallpaper_url_cmd", {
          themeId,
          url: entry.url,
          label: entry.label || entry.url,
          opacity: entry.opacity,
        });
      } else {
        // Path de disco legacy: intentar copiar
        await invoke("set_theme_wallpaper_file_cmd", {
          themeId,
          sourcePath: entry.url,
          label: entry.label || entry.url,
          opacity: entry.opacity,
        });
      }
    } catch (err) {
      console.error(`No se pudo migrar wallpaper de ${themeId}:`, err);
    }
  }

  clearLegacyWallpaperKeys();
}

function syncWallpaperControls(entry: ThemeWallpaper): void {
  bgImageUrl = entry.url;
  const label =
    entry.label || (entry.url && !entry.url.startsWith("data:") ? entry.url : "");
  const { numeric, formatted } = clampAndFormatOpacity(entry.opacity);
  if (configBgUrlInput) configBgUrlInput.value = label;
  if (configBgOpacityInput) configBgOpacityInput.value = numeric.toString();
  if (opacityValLabel) opacityValLabel.textContent = formatted;
}

async function applyThemeWallpaper(themeId: string): Promise<void> {
  const entry = await fetchThemeWallpaper(themeId);
  syncWallpaperControls(entry);
  applyBackgroundSettings(entry.url, entry.opacity);
}

function currentBgOpacity(): number {
  const raw = configBgOpacityInput?.value;
  if (raw != null && raw !== "") {
    return clampAndFormatOpacity(parseFloat(raw)).numeric;
  }
  return DEFAULT_WALLPAPER_OPACITY;
}

function applyDtoToUi(dto: ThemeWallpaperDto): void {
  const entry = dtoToEntry(dto);
  syncWallpaperControls(entry);
  applyBackgroundSettings(entry.url, entry.opacity);
}

/** Guarda la imagen del tema activo y la pinta. */
async function persistBackgroundFromFile(sourcePath: string, label: string): Promise<void> {
  const themeId = getActiveTheme();
  const opacity = currentBgOpacity();
  try {
    const dto = await invoke<ThemeWallpaperDto>("set_theme_wallpaper_file_cmd", {
      themeId,
      sourcePath,
      label,
      opacity,
    });
    applyDtoToUi(dto);
  } catch (err) {
    console.error("No se pudo guardar el fondo:", err);
    void alertDialog({
      title: "No se pudo guardar el fondo",
      message: String(err),
    });
  }
}

async function persistBackgroundFromBytes(
  bytes: Uint8Array,
  ext: string,
  label: string,
): Promise<void> {
  const themeId = getActiveTheme();
  const opacity = currentBgOpacity();
  try {
    const dto = await invoke<ThemeWallpaperDto>("set_theme_wallpaper_bytes_cmd", {
      themeId,
      bytes: Array.from(bytes),
      ext,
      label,
      opacity,
    });
    applyDtoToUi(dto);
  } catch (err) {
    console.error("No se pudo guardar el fondo:", err);
    void alertDialog({
      title: "No se pudo guardar el fondo",
      message: String(err),
    });
  }
}

async function persistBackgroundUrl(url: string, label: string): Promise<void> {
  const themeId = getActiveTheme();
  const opacity = currentBgOpacity();
  try {
    if (!url) {
      await invoke("clear_theme_wallpaper_cmd", { themeId });
      syncWallpaperControls({ url: "", label: "", opacity });
      applyBackgroundSettings("", opacity);
      return;
    }
    const dto = await invoke<ThemeWallpaperDto>("set_theme_wallpaper_url_cmd", {
      themeId,
      url,
      label,
      opacity,
    });
    applyDtoToUi(dto);
  } catch (err) {
    console.error("No se pudo guardar el fondo:", err);
    void alertDialog({
      title: "No se pudo guardar el fondo",
      message: String(err),
    });
  }
}

async function persistBackgroundOpacity(opacity: number): Promise<void> {
  const themeId = getActiveTheme();
  const { numeric, formatted } = clampAndFormatOpacity(opacity);
  if (opacityValLabel) opacityValLabel.textContent = formatted;
  applyBackgroundSettings(bgImageUrl, numeric);

  try {
    const dto = await invoke<ThemeWallpaperDto>("set_theme_wallpaper_opacity_cmd", {
      themeId,
      opacity: numeric,
    });
    bgImageUrl = dto.display_url || bgImageUrl;
  } catch (err) {
    console.error("No se pudo guardar la opacidad del fondo:", err);
  }
}

// --- Initialize App Settings (Background & Opacity + editor externo) ---
function initSettings() {
  configBgUrlInput = document.getElementById("config-bg-url") as HTMLInputElement;
  configBgOpacityInput = document.getElementById("config-bg-opacity") as HTMLInputElement;
  opacityValLabel = document.getElementById("opacity-val");
  btnApplyBg = document.getElementById("btn-apply-bg") as HTMLButtonElement;
  configEditorPathInput = document.getElementById("config-editor-path") as HTMLInputElement;
  btnSaveEditorPref = document.getElementById("btn-save-editor-pref") as HTMLButtonElement;

  const btnBrowseEditor = document.getElementById("btn-browse-editor") as HTMLButtonElement | null;
  const fileInputEditor = document.getElementById("file-input-editor") as HTMLInputElement | null;
  const btnBrowseBg = document.getElementById("btn-browse-bg") as HTMLButtonElement | null;
  const fileInputBg = document.getElementById("file-input-bg") as HTMLInputElement | null;
  const btnClearBg = document.getElementById("btn-clear-bg") as HTMLButtonElement | null;

  // Iconografía normalizada de botones de preferencias
  if (btnBrowseEditor) setButtonIcon(btnBrowseEditor, AppIcons.folder, { size: 14, className: "icon--sm" });
  if (btnSaveEditorPref) setButtonIcon(btnSaveEditorPref, AppIcons.check, { size: 14, className: "icon--sm" });
  if (btnBrowseBg) setButtonIcon(btnBrowseBg, AppIcons.folderPlus, { size: 14, className: "icon--sm" });
  if (btnApplyBg) setButtonIcon(btnApplyBg, AppIcons.check, { size: 14, className: "icon--sm" });
  if (btnClearBg) setButtonIcon(btnClearBg, AppIcons.trash2, { size: 14, className: "icon--sm" });

  // Sincronizar controles del popover con el wallpaper del tema (refs DOM ya listas)
  void applyThemeWallpaper(getActiveTheme());

  // --- Exploración y acciones de Editor Preferido ---
  btnBrowseEditor?.addEventListener("click", () => {
    fileInputEditor?.click();
  });

  fileInputEditor?.addEventListener("change", () => {
    if (fileInputEditor.files && fileInputEditor.files.length > 0) {
      const selectedFile = fileInputEditor.files[0] as File & { path?: string };
      const fullPath = selectedFile.path || selectedFile.name;
      if (configEditorPathInput) configEditorPathInput.value = fullPath;
    }
  });

  btnSaveEditorPref?.addEventListener("click", () => {
    void savePreferredEditorFromUi();
  });

  // --- Exploración y acciones de Fondo de Pantalla ---
  btnBrowseBg?.addEventListener("click", () => {
    fileInputBg?.click();
  });

  fileInputBg?.addEventListener("change", () => {
    if (fileInputBg.files && fileInputBg.files.length > 0) {
      const selectedFile = fileInputBg.files[0] as File & { path?: string };
      const label = selectedFile.name;
      if (configBgUrlInput) configBgUrlInput.value = label;

      const diskPath = selectedFile.path;
      if (diskPath) {
        void persistBackgroundFromFile(diskPath, label);
        return;
      }

      // Fallback: bytes vía IPC (sin data URL en storage)
      void selectedFile.arrayBuffer().then((buf) => {
        const ext = label.includes(".") ? label.split(".").pop() || "png" : "png";
        void persistBackgroundFromBytes(new Uint8Array(buf), ext, label);
      }).catch((err) => {
        console.error("No se pudo leer la imagen:", err);
        void alertDialog({
          title: "No se pudo leer la imagen",
          message: "El archivo seleccionado no se pudo abrir. Intenta con otra imagen.",
        });
      });
    }
  });

  btnApplyBg?.addEventListener("click", () => {
    const result = resolveBackgroundApply(configBgUrlInput?.value || "", bgImageUrl);
    switch (result.action) {
      case "clear":
        void persistBackgroundUrl("", "");
        break;
      case "set":
        void persistBackgroundUrl(result.url, result.url);
        break;
      case "keep":
        applyBackgroundSettings(bgImageUrl, currentBgOpacity());
        break;
      case "unsupported":
        void alertDialog({
          title: "Ruta no soportada",
          message:
            "Escribe una URL http(s) o elige el archivo con el botón de explorar. Una ruta de disco escrita a mano no se puede cargar.",
        });
        break;
    }
  });

  btnClearBg?.addEventListener("click", () => {
    if (configBgUrlInput) configBgUrlInput.value = "";
    void persistBackgroundUrl("", "");
  });

  configBgOpacityInput?.addEventListener("input", (e) => {
    const target = e.target as HTMLInputElement;
    void persistBackgroundOpacity(parseFloat(target.value));
  });

  void loadPreferredEditorIntoUi();
  initFooterPrefsPopover();
}

/** Engrane del footer: abre/cierra popover de prefs (misma capacidad que el strip previo). */
function initFooterPrefsPopover() {
  const gear = document.getElementById("btn-footer-gear");
  const pop = document.getElementById("prefs-popover");
  if (!gear || !pop) return;

  setButtonIcon(gear, AppIcons.settings, { size: 18, className: "icon--md" });

  gear.addEventListener("click", (e) => {
    e.stopPropagation();
    const open = !pop.classList.contains("is-open");
    pop.classList.toggle("is-open", open);
    gear.setAttribute("aria-expanded", open ? "true" : "false");
  });

  document.addEventListener("click", (e) => {
    if (!pop.classList.contains("is-open")) return;
    const target = e.target as Node;
    if (pop.contains(target) || gear.contains(target)) return;
    pop.classList.remove("is-open");
    gear.setAttribute("aria-expanded", "false");
  });

  document.addEventListener("keydown", (e) => {
    if (e.key !== "Escape" || !pop.classList.contains("is-open")) return;
    pop.classList.remove("is-open");
    gear.setAttribute("aria-expanded", "false");
  });

  const themeList = document.getElementById("theme-list");
  if (themeList) {
    themeList.addEventListener("click", (e) => {
      const item = (e.target as HTMLElement).closest(".theme-item") as HTMLElement | null;
      if (item?.dataset.themeId) {
        applyTheme(item.dataset.themeId);
      }
    });
  }
}

async function loadPreferredEditorIntoUi() {
  try {
    const path = await invoke<string>("get_preferred_external_editor_cmd");
    if (configEditorPathInput) configEditorPathInput.value = path || "";
  } catch (err) {
    console.error("No se pudo cargar editor preferido:", err);
  }
}

async function savePreferredEditorFromUi() {
  const path = configEditorPathInput?.value.trim() || "";
  try {
    await invoke("set_preferred_external_editor_cmd", { path });
    setExplorerStatus("Editor preferido guardado.");
  } catch (err) {
    await alertDialog({
      title: "No se pudo guardar",
      message: String(err),
    });
  }
}

function applyBackgroundSettings(url: string, opacity: number) {
  const targetUrl = url || bgImageUrl;
  const resolvedUrl = resolveBackgroundUrl(targetUrl, convertFileSrc);
  const overlayOpacity = calculateTerminalOverlayOpacity(opacity);

  const terminalPanels = document.querySelectorAll<HTMLElement>(".terminal-panel");
  terminalPanels.forEach((panel) => {
    if (resolvedUrl) {
      panel.style.backgroundImage = `url("${resolvedUrl}")`;
      panel.style.setProperty("--terminal-overlay-opacity", overlayOpacity.toString());
    } else {
      panel.style.backgroundImage = "";
      panel.style.setProperty("--terminal-overlay-opacity", "0.95");
    }
  });
}

// --- Initialize Navigation Tabs ---
function initTabs() {
  tabBtnServers = document.getElementById("tab-btn-servers") as HTMLButtonElement;
  tabBtnFiles = document.getElementById("tab-btn-files") as HTMLButtonElement;
  tabBtnMonitor = document.getElementById("tab-btn-monitor") as HTMLButtonElement;
  panelServers = document.getElementById("panel-servers");
  panelFiles = document.getElementById("panel-files");
  panelMonitor = document.getElementById("panel-monitor");

  // Monitor DOM
  monitorEmpty = document.getElementById("monitor-empty");
  monitorContent = document.getElementById("monitor-content");
  monitorServerNameText = document.getElementById("monitor-server-name-text");
  monitorCpuValue = document.getElementById("monitor-cpu-value");
  monitorCpuLoad = document.getElementById("monitor-cpu-load");
  monitorCpuCores = document.getElementById("monitor-cpu-cores");
  monitorRamValue = document.getElementById("monitor-ram-value");
  monitorRamDetail = document.getElementById("monitor-ram-detail");
  monitorRamFree = document.getElementById("monitor-ram-free");
  monitorDiskValue = document.getElementById("monitor-disk-value");
  monitorDiskFill = document.getElementById("monitor-disk-fill");
  monitorDiskDetail = document.getElementById("monitor-disk-detail");
  monitorIntervalSelect = document.getElementById("monitor-interval-select") as HTMLSelectElement;
  btnMonitorPause = document.getElementById("btn-monitor-pause") as HTMLButtonElement;
  monitorOsText = document.getElementById("monitor-os-text");
  monitorUptimeText = document.getElementById("monitor-uptime-text");
  monitorNetDown = document.getElementById("monitor-net-down");
  monitorNetUp = document.getElementById("monitor-net-up");
  monitorProcessesList = document.getElementById("monitor-processes-list");
  monitorBtnPauseText = document.getElementById("btn-monitor-pause-text");

  filesEmpty = document.getElementById("files-empty");
  filesToolbar = document.getElementById("files-toolbar");
  filesCwdInput = document.getElementById("files-cwd-input") as HTMLInputElement;
  filesStatus = document.getElementById("files-status");
  filesTree = document.getElementById("files-tree");
  filesContextMenu = document.getElementById("files-context-menu");
  btnFilesUp = document.getElementById("btn-files-up") as HTMLButtonElement;
  btnFilesGo = document.getElementById("btn-files-go") as HTMLButtonElement;
  btnFilesRefresh = document.getElementById("btn-files-refresh") as HTMLButtonElement;

  if (btnFilesUp) {
    setButtonIcon(btnFilesUp, AppIcons.arrowUp);
    btnFilesUp.setAttribute("aria-label", "Subir");
  }
  if (btnFilesGo) {
    setButtonIcon(btnFilesGo, AppIcons.arrowRight);
    btnFilesGo.setAttribute("aria-label", "Ir");
  }
  if (btnFilesRefresh) {
    setButtonIcon(btnFilesRefresh, AppIcons.refreshCw);
    btnFilesRefresh.setAttribute("aria-label", "Actualizar");
  }

  tabBtnServers?.addEventListener("click", () => {
    tabBtnServers?.classList.add("active");
    tabBtnFiles?.classList.remove("active");
    tabBtnMonitor?.classList.remove("active");
    panelServers?.classList.add("active");
    panelFiles?.classList.remove("active");
    panelMonitor?.classList.remove("active");
    stopMonitorInterval();
  });

  tabBtnFiles?.addEventListener("click", () => {
    tabBtnFiles?.classList.add("active");
    tabBtnServers?.classList.remove("active");
    tabBtnMonitor?.classList.remove("active");
    panelFiles?.classList.add("active");
    panelServers?.classList.remove("active");
    panelMonitor?.classList.remove("active");
    stopMonitorInterval();
    void refreshExplorerForActiveTerminal();
  });

  tabBtnMonitor?.addEventListener("click", () => {
    tabBtnMonitor?.classList.add("active");
    tabBtnServers?.classList.remove("active");
    tabBtnFiles?.classList.remove("active");
    panelMonitor?.classList.add("active");
    panelServers?.classList.remove("active");
    panelFiles?.classList.remove("active");
    
    initMonitorTab();
  });

  btnFilesUp?.addEventListener("click", () => {
    void goExplorerUp();
  });

  btnFilesGo?.addEventListener("click", () => {
    void goExplorerToInputPath();
  });

  filesCwdInput?.addEventListener("keydown", (ev) => {
    if (ev.key === "Enter") {
      ev.preventDefault();
      void goExplorerToInputPath();
    }
  });

  btnFilesRefresh?.addEventListener("click", () => {
    void refreshExplorerAtCurrentPath();
  });

  filesContextMenu?.querySelectorAll("li").forEach((li) => {
    li.addEventListener("click", () => {
      const action = (li as HTMLElement).dataset.action;
      if (action === "open-terminal" && contextMenuPath) {
        void openPathInTerminal(contextMenuPath);
      }
      hideContextMenu();
    });
  });

  document.addEventListener("click", () => hideContextMenu());

  filesTree?.addEventListener("contextmenu", async (ev) => {
    ev.preventDefault();
    if (ev.target !== filesTree) return; // solo fondo
    if (!currentActiveTerminalId) return;
    hideContextMenu();
    const items = [];
    if (scpClipboard && scpClipboard.terminalId !== currentActiveTerminalId) {
      items.push({ id: "paste-scp", label: "Pegar scp", icon: AppIcons.clipboard });
    }
    if (items.length > 0) {
      const action = await showContextMenu(ev.clientX, ev.clientY, items);
      if (action === "paste-scp") {
        void handlePasteScp(explorerCwd);
      }
    }
  });
}

async function handlePasteScp(targetDir: string) {
  if (!scpClipboard || !currentActiveTerminalId) return;
  const targetPath = (targetDir.endsWith("/") ? targetDir : targetDir + "/") + scpClipboard.name;
  const ok = await confirmDialog({
    title: "Pegar scp",
    message: `¿Copiar "${scpClipboard.name}" a "${targetPath}"?`,
    confirmLabel: "Copiar",
  });
  if (!ok) return;

  setExplorerStatus(`Copiando ${scpClipboard.name}…`);
  try {
    await invoke("sftp_copy_between_sessions", {
      sourceTerminalId: scpClipboard.terminalId,
      sourcePath: scpClipboard.path,
      targetTerminalId: currentActiveTerminalId,
      targetPath,
    });
    setExplorerStatus(`Copia exitosa: ${scpClipboard.name}`);
    await refreshExplorerForActiveTerminal(true);
  } catch (err) {
    console.error("Error al copiar scp:", err);
    setExplorerStatus(`Error al copiar: ${err}`, true);
  }
}

function hideContextMenu() {
  if (filesContextMenu) filesContextMenu.style.display = "none";
  contextMenuPath = null;
}

function normalizeRemotePath(path: string): string {
  if (!path || path === ".") return path || ".";
  if (path === "/") return "/";
  return path.replace(/\/+$/, "") || "/";
}

function pathsEqual(a: string, b: string): boolean {
  return normalizeRemotePath(a) === normalizeRemotePath(b);
}

function parentRemotePath(path: string): string | null {
  const n = normalizeRemotePath(path);
  if (!n || n === "/" || n === ".") return null;
  const parts = n.split("/").filter(Boolean);
  if (parts.length === 0) return null;
  parts.pop();
  return parts.length === 0 ? "/" : `/${parts.join("/")}`;
}

function setExplorerStatus(message: string, isError = false) {
  if (!filesStatus) return;
  if (statusDismissTimer) {
    clearTimeout(statusDismissTimer);
    statusDismissTimer = null;
  }
  if (!message) {
    filesStatus.classList.remove("is-visible", "error");
    filesStatus.textContent = "";
    filesStatus.setAttribute("title", "");
    return;
  }
  filesStatus.textContent = message;
  filesStatus.setAttribute("title", message);
  filesStatus.classList.toggle("error", isError);
  filesStatus.classList.add("is-visible");
  if (!isError) {
    statusDismissTimer = setTimeout(() => {
      filesStatus!.classList.remove("is-visible");
      statusDismissTimer = null;
    }, 3000);
  }
}

function updateUpButton() {
  if (!btnFilesUp) return;
  btnFilesUp.disabled = parentRemotePath(explorerCwd || filesCwdInput?.value || "") === null;
}

function showExplorerEmpty(message: string) {
  if (filesEmpty) {
    filesEmpty.style.display = "block";
    filesEmpty.textContent = message;
  }
  if (filesToolbar) filesToolbar.style.display = "none";
  setExplorerStatus("");
  if (filesTree) {
    filesTree.style.display = "none";
    filesTree.innerHTML = "";
  }
}

function showExplorerReady() {
  if (filesEmpty) filesEmpty.style.display = "none";
  if (filesToolbar) filesToolbar.style.display = "flex";
  if (filesTree) filesTree.style.display = "flex";
}

function setExplorerPathDisplay(path: string) {
  explorerCwd = normalizeRemotePath(path);
  if (filesCwdInput) {
    filesCwdInput.value = explorerCwd;
    filesCwdInput.setAttribute("title", explorerCwd);
  }
  updateUpButton();
}

async function goExplorerToInputPath() {
  if (!currentActiveTerminalId) return;
  const path = (filesCwdInput?.value || "").trim();
  if (!path) return;
  try {
    showExplorerReady();
    setExplorerStatus("");
    setExplorerPathDisplay(path);
    await loadExplorerAt(normalizeRemotePath(path), true);
  } catch (err) {
    console.error("Error al Ir a ruta:", err);
    setExplorerStatus(`No se pudo listar: ${path}`, true);
  }
}

async function goExplorerUp() {
  const parent = parentRemotePath(explorerCwd || filesCwdInput?.value || "");
  if (!parent) return;
  setExplorerPathDisplay(parent);
  try {
    setExplorerStatus("");
    await loadExplorerAt(parent, true);
  } catch (err) {
    console.error("Error al subir:", err);
    setExplorerStatus("No se pudo subir al directorio padre", true);
  }
}

async function refreshExplorerAtCurrentPath() {
  const path = normalizeRemotePath((filesCwdInput?.value || explorerCwd || ".").trim() || ".");
  try {
    setExplorerPathDisplay(path);
    setExplorerStatus("");
    await loadExplorerAt(path, true);
  } catch (err) {
    console.error("Error al actualizar explorador:", err);
    setExplorerStatus("Error al actualizar", true);
  }
}

async function refreshExplorerForActiveTerminal(forceReload = false) {
  const termId = currentActiveTerminalId;
  const term = termId ? activeTerminals.get(termId) : undefined;
  if (!termId || !term?.isConnected) {
    showExplorerEmpty("Conecta un servidor para explorar archivos remotos.");
    explorerBoundTerminalId = null;
    explorerRoot = null;
    return;
  }

  showExplorerReady();
  explorerBoundTerminalId = termId;

  const path = normalizeRemotePath(explorerCwd || ".");
  setExplorerPathDisplay(path === "." ? filesCwdInput?.value || "." : path);
  try {
    setExplorerStatus("");
    await loadExplorerAt(path, forceReload || !explorerRoot || !pathsEqual(explorerRoot.path, path));
  } catch (err) {
    console.error("Error al refrescar explorador:", err);
    showExplorerEmpty("No se pudo listar el filesystem remoto.");
  }
}

async function openExplorerFolder(path: string) {
  setExplorerPathDisplay(path);
  setExplorerStatus("");
  await loadExplorerAt(normalizeRemotePath(path), true);
}

async function loadExplorerAt(path: string, force = false) {
  const normalized = normalizeRemotePath(path);
  if (!force && explorerRoot && pathsEqual(explorerRoot.path, normalized) && explorerRoot.loaded) {
    renderExplorerTree();
    return;
  }
  explorerLoading = true;
  setExplorerStatus("Cargando…");
  renderExplorerTree();
  try {
    const entries = await invoke<SftpDirEntry[]>("sftp_list_dir", {
      terminalId: currentActiveTerminalId,
      path: normalized
    });
    explorerRoot = {
      path: normalized,
      name: normalized === "/" || normalized === "." ? normalized : normalized.split("/").filter(Boolean).pop() || normalized,
      isDir: true,
      size: 0,
      expanded: true,
      loaded: true,
      children: entries.map((e) => ({
        path: e.path,
        name: e.name,
        isDir: e.is_dir,
        size: e.size ?? 0,
        expanded: false,
        loaded: false,
        children: []
      }))
    };
    setExplorerStatus("");
  } catch (err) {
    explorerRoot = null;
    setExplorerStatus(`Error: ${String(err)}`, true);
    throw err;
  } finally {
    explorerLoading = false;
    renderExplorerTree();
    updateUpButton();
  }
}

async function toggleExplorerNode(node: ExplorerNodeState) {
  if (!node.isDir || !currentActiveTerminalId) return;
  if (!node.expanded) {
    if (!node.loaded) {
      setExplorerStatus("Cargando…");
      try {
        const entries = await invoke<SftpDirEntry[]>("sftp_list_dir", {
          terminalId: currentActiveTerminalId,
          path: node.path
        });
        node.children = entries.map((e) => ({
          path: e.path,
          name: e.name,
          isDir: e.is_dir,
          size: e.size ?? 0,
          expanded: false,
          loaded: false,
          children: []
        }));
        node.loaded = true;
        setExplorerStatus("");
      } catch (err) {
        setExplorerStatus(`Error al expandir: ${node.name}`, true);
        console.error(err);
        return;
      }
    }
    node.expanded = true;
  } else {
    node.expanded = false;
  }
  renderExplorerTree();
}

function renderExplorerTree() {
  if (!filesTree) return;
  filesTree.innerHTML = "";
  if (explorerLoading && !explorerRoot) {
    const el = document.createElement("div");
    el.className = "files-tree-empty";
    el.textContent = "Cargando…";
    filesTree.appendChild(el);
    return;
  }
  if (!explorerRoot) return;
  if (explorerRoot.children.length === 0) {
    const el = document.createElement("div");
    el.className = "files-tree-empty";
    el.textContent = "(vacío)";
    filesTree.appendChild(el);
    return;
  }
  explorerRoot.children.forEach((child) => {
    filesTree!.appendChild(buildExplorerNodeEl(child));
  });
}

function buildExplorerNodeEl(node: ExplorerNodeState): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "files-node";

  const row = document.createElement("div");
  row.className = "files-node-row";
  if (pathsEqual(node.path, explorerCwd)) row.classList.add("active");

  const toggle = document.createElement("span");
  toggle.className = "files-node-toggle";
  if (node.isDir) {
    toggle.replaceChildren(
      icon(node.expanded ? AppIcons.chevronDown : AppIcons.chevronRight, {
        size: 14,
        className: "icon--sm",
      }),
    );
    toggle.addEventListener("click", (ev) => {
      ev.stopPropagation();
      void toggleExplorerNode(node);
    });
  } else {
    toggle.classList.add("is-spacer");
  }
  row.appendChild(toggle);

  const iconEl = document.createElement("span");
  iconEl.className = "files-node-icon";
  iconEl.replaceChildren(
    icon(node.isDir ? AppIcons.folder : AppIcons.file, {
      size: 14,
      className: "icon--sm",
    }),
  );
  row.appendChild(iconEl);

  const label = document.createElement("span");
  label.className = "files-node-label";
  label.textContent = node.name;
  row.appendChild(label);

  row.addEventListener("click", (ev) => {
    ev.stopPropagation();
    if (node.isDir) {
      void openExplorerFolder(node.path).catch((err) => {
        console.error(err);
        setExplorerStatus(`No se pudo abrir: ${node.name}`, true);
      });
    }
  });

  row.addEventListener("dblclick", (ev) => {
    ev.preventDefault();
    ev.stopPropagation();
    if (node.isDir) {
      // Carpeta: navegación ya cubierta por click; no iniciar edición
      return;
    }
    void beginExternalEdit(node);
  });

  row.addEventListener("contextmenu", async (ev) => {
    ev.preventDefault();
    ev.stopPropagation();
    hideContextMenu();
    if (node.isDir) {
      const items = [{ id: "open-terminal", label: "Abrir en Terminal", icon: AppIcons.terminal }];
      if (scpClipboard && scpClipboard.terminalId !== currentActiveTerminalId) {
        items.push({ id: "paste-scp", label: "Pegar scp", icon: AppIcons.clipboard });
      }
      const action = await showContextMenu(ev.clientX, ev.clientY, items);
      if (action === "open-terminal") {
        void openPathInTerminal(node.path);
      } else if (action === "paste-scp") {
        void handlePasteScp(node.path);
      }
      return;
    }
    const action = await showContextMenu(ev.clientX, ev.clientY, [
      { id: "edit", label: "Editar", icon: AppIcons.pencil },
      { id: "copy-scp", label: "Copiar scp", icon: AppIcons.clipboard },
    ]);
    if (action === "edit") {
      void beginExternalEdit(node);
    } else if (action === "copy-scp") {
      if (currentActiveTerminalId) {
        scpClipboard = { terminalId: currentActiveTerminalId, path: node.path, name: node.name };
        setExplorerStatus(`Copiado al portapapeles scp: ${node.name}`);
      }
    }
  });

  wrap.appendChild(row);

  if (node.isDir && node.expanded) {
    const kids = document.createElement("div");
    kids.className = "files-node-children";
    if (node.children.length === 0) {
      const empty = document.createElement("div");
      empty.className = "files-tree-empty";
      empty.textContent = "(vacío)";
      kids.appendChild(empty);
    } else {
      node.children.forEach((child) => kids.appendChild(buildExplorerNodeEl(child)));
    }
    wrap.appendChild(kids);
  }

  return wrap;
}

async function openPathInTerminal(path: string) {
  if (!currentActiveTerminalId) return;
  try {
    const cwd = await invoke<string>("ssh_cd", {
      terminalId: currentActiveTerminalId,
      path
    });
    setExplorerPathDisplay(cwd || path);
    await loadExplorerAt(explorerCwd, true);
  } catch (err) {
    console.error("Error en Abrir en Terminal:", err);
  }
}

/** Flujo FileZilla: probe → (aviso binario) → download temp → editor → watch → A1 subir. */
async function beginExternalEdit(node: ExplorerNodeState) {
  if (node.isDir || !currentActiveTerminalId) return;
  const terminalId = currentActiveTerminalId;
  setExplorerStatus(`Preparando edición: ${node.name}…`);
  try {
    const probe = await invoke<ExternalEditProbe>("probe_external_edit", {
      terminalId,
      remotePath: node.path,
    });
    if (probe.too_large) {
      await alertDialog({
        title: "Archivo demasiado grande",
        message:
          "El archivo supera el límite de 10 MiB para edición externa en esta versión.",
      });
      setExplorerStatus("");
      return;
    }
    if (probe.looks_binary) {
      const ok = await confirmDialog({
        title: "Archivo posiblemente binario",
        message: "El archivo parece binario. ¿Abrir de todos modos?",
        confirmLabel: "Abrir",
        cancelLabel: "Cancelar",
      });
      if (!ok) {
        setExplorerStatus("");
        return;
      }
    }
    await invoke("start_external_edit", {
      terminalId,
      remotePath: node.path,
    });
    setExplorerStatus(`Editando: ${probe.basename}`);
  } catch (err) {
    console.error("Error al iniciar edición externa:", err);
    await alertDialog({
      title: "No se pudo editar",
      message: String(err),
    });
    setExplorerStatus(`Error al editar: ${node.name}`, true);
  }
}

type EditUploadErrorPayload = {
  kind: string;
  message: string;
  elevatable: boolean;
};

/** Normaliza el reject de invoke (objeto Tauri o JSON string). */
function asEditUploadError(err: unknown): EditUploadErrorPayload | null {
  if (err && typeof err === "object" && "elevatable" in err && "kind" in err && "message" in err) {
    const o = err as Record<string, unknown>;
    return {
      kind: String(o.kind),
      message: String(o.message),
      elevatable: Boolean(o.elevatable),
    };
  }
  if (typeof err === "string") {
    try {
      const parsed = JSON.parse(err) as unknown;
      return asEditUploadError(parsed);
    } catch {
      return null;
    }
  }
  return null;
}

function uploadErrorMessage(err: unknown): string {
  return asEditUploadError(err)?.message ?? String(err);
}

async function handleEditSessionChanged(payload: EditSessionChangedPayload) {
  if (payload.reason !== "content_changed") return;
  if (editUploadConfirmOpen.has(payload.edit_id)) return;
  editUploadConfirmOpen.add(payload.edit_id);
  try {
    const remotePath = payload.remote_path;
    const slash = remotePath.lastIndexOf("/");
    const detailFilename =
      slash >= 0 ? remotePath.slice(slash + 1) || remotePath : remotePath;
    const ok = await confirmDialog({
      title: "Subir cambios",
      message: "¿Subir al servidor?",
      detailFilename,
      detailFullPath: remotePath,
      confirmLabel: "Subir",
      cancelLabel: "Cancelar",
    });
    if (!ok) {
      await invoke("dismiss_edit_change", { editId: payload.edit_id });
      return;
    }
    try {
      await invoke("confirm_edit_upload", { editId: payload.edit_id });
      setExplorerStatus("Archivo subido al servidor.");
    } catch (err) {
      const structured = asEditUploadError(err);
      if (structured?.elevatable) {
        const withSudo = await confirmDialog({
          title: "Error al subir",
          message:
            "No hay permiso para escribir en el servidor. ¿Subir con sudo? (sin contraseña en la app; solo si el host lo permite sin prompt)",
          detailFilename,
          detailFullPath: remotePath,
          confirmLabel: "Subir con sudo",
          cancelLabel: "Cancelar",
          danger: false,
        });
        if (!withSudo) {
          setExplorerStatus("Subida cancelada; cambios locales conservados.", true);
          return;
        }
        try {
          await invoke("edit_session_upload_with_sudo", {
            editId: payload.edit_id,
          });
          setExplorerStatus("Archivo subido al servidor (sudo).");
        } catch (sudoErr) {
          const sudoMsg = uploadErrorMessage(sudoErr);
          await alertDialog({
            title: "Error al subir con sudo",
            message: sudoMsg,
          });
          setExplorerStatus("Error al subir con sudo; cambios locales conservados.", true);
        }
        return;
      }
      await alertDialog({
        title: "Error al subir",
        message: uploadErrorMessage(err),
      });
      setExplorerStatus("Error al subir cambios", true);
    }
  } finally {
    editUploadConfirmOpen.delete(payload.edit_id);
  }
}

function initExternalEditListeners() {
  void listen<EditSessionChangedPayload>("edit-session-changed", (event) => {
    void handleEditSessionChanged(event.payload);
  });
  void listen<EditSessionDisconnectedPayload>("edit-session-disconnected", (event) => {
    const hadUploadConfirm = event.payload.edit_ids.some((id) =>
      editUploadConfirmOpen.has(id),
    );
    for (const id of event.payload.edit_ids) {
      editUploadConfirmOpen.delete(id);
    }
    setExplorerStatus(event.payload.message, true);
    // No apilar A1 sobre el confirm de subida abierto (mismo root de chrome).
    if (!hadUploadConfirm) {
      void alertDialog({
        title: "Sesión desconectada",
        message: event.payload.message,
      });
    }
  });
}

// --- DOM Loaded Listener ---
window.addEventListener("DOMContentLoaded", () => {
  // Migrar localStorage → SQLite/disco, luego aplicar tema (incluye wallpaper).
  void (async () => {
    await migrateWallpapersFromLocalStorageIfNeeded();
    applyTheme(getActiveTheme());
  })();
  // Inhabilitar menú contextual nativo del navegador en todo el documento
  document.addEventListener("contextmenu", (e) => e.preventDefault());

  initSettings();
  initSnippetsUi();
  initTabs();
  initExternalEditListeners();
  initHistoryUi();
  
  // Elementos del Terminal Layout
  mainDisplayArea = document.getElementById("main-display-area");
  terminalTabsList = document.getElementById("terminal-tabs-list");
  btnCloseAllTerminals = document.getElementById("btn-close-all-terminals") as HTMLButtonElement;

  btnCloseAllTerminals?.addEventListener("click", () => {
    closeAllTerminals();
  });

  // Elementos del Modal
  profileModal = document.getElementById("profile-modal");
  profileForm = document.getElementById("profile-form") as HTMLFormElement;
  modalTitle = document.getElementById("modal-title");
  profileIdInput = document.getElementById("profile-id") as HTMLInputElement;

  profNameInput = document.getElementById("prof-name") as HTMLInputElement;
  profHostInput = document.getElementById("prof-host") as HTMLInputElement;
  profPortInput = document.getElementById("prof-port") as HTMLInputElement;
  profUsernameInput = document.getElementById("prof-username") as HTMLInputElement;
  profAuthTypeSelect = document.getElementById("prof-auth-type") as HTMLSelectElement;
  profPasswordInput = document.getElementById("prof-password") as HTMLInputElement;
  profKeyStatusEl = document.getElementById("prof-key-status");
  profPassphraseInput = document.getElementById("prof-passphrase") as HTMLInputElement;
  profKeepaliveInput = document.getElementById("prof-keepalive") as HTMLInputElement;

  tunTypeSelect = document.getElementById("tun-type") as HTMLSelectElement;
  tunLocalPortInput = document.getElementById("tun-local-port") as HTMLInputElement;
  tunDestInput = document.getElementById("tun-dest") as HTMLInputElement;

  btnNewProfile = document.getElementById("btn-new-profile") as HTMLButtonElement;
  btnNewFolder = document.getElementById("btn-new-folder") as HTMLButtonElement;
  btnCancelProfile = document.getElementById("btn-cancel-profile") as HTMLButtonElement;
  profileListContainer = document.getElementById("profile-list");
  profileFolderIdInput = document.getElementById("profile-folder-id") as HTMLInputElement;

  if (btnNewProfile) {
    setButtonIcon(btnNewProfile, AppIcons.plus, { size: 18, className: "icon--md" });
  }

  if (btnNewFolder) {
    setButtonIcon(btnNewFolder, AppIcons.folderPlus, { size: 18, className: "icon--md" });
  }

  // Mostrar modal de creación (usa carpeta activa o General)
  btnNewProfile?.addEventListener("click", () => {
    openProfileModal(undefined, activeFolderId ?? undefined);
  });

  btnNewFolder?.addEventListener("click", () => {
    void createNewFolder();
  });

  // Cancelar modal
  btnCancelProfile?.addEventListener("click", () => {
    closeProfileModal();
  });

  // Cambio de método de autenticación en formulario
  profAuthTypeSelect?.addEventListener("change", (e) => {
    const target = e.target as HTMLSelectElement;
    toggleAuthFields(target.value as 'password' | 'key');
  });

  // Cambio de tipo de túnel
  tunTypeSelect?.addEventListener("change", (e) => {
    const target = e.target as HTMLSelectElement;
    toggleTunnelFields(target.value);
  });

  // Selector nativo de archivo de llave privada (lee contenido, no ruta)
  const btnBrowseKey = document.getElementById("btn-browse-key") as HTMLButtonElement | null;
  const fileInputKey = document.getElementById("file-input-key") as HTMLInputElement | null;

  btnBrowseKey?.addEventListener("click", () => {
    fileInputKey?.click();
  });

  fileInputKey?.addEventListener("change", () => {
    if (!fileInputKey.files || fileInputKey.files.length === 0) return;
    const selectedFile = fileInputKey.files[0];
    const reader = new FileReader();
    reader.onload = () => {
      const text = typeof reader.result === "string" ? reader.result : "";
      if (!text.trim()) {
        void alertDialog({
          title: "Llave inválida",
          message: "No se pudo leer el contenido de la llave privada seleccionada.",
        });
        return;
      }
      draftPrivateKeyContent = text;
      updateKeyStatusUi(true);
    };
    reader.onerror = () => {
      void alertDialog({
        title: "Error",
        message: "Error al leer el archivo de llave privada.",
      });
    };
    reader.readAsText(selectedFile);
    fileInputKey.value = "";
  });

  // Submit del formulario
  profileForm?.addEventListener("submit", (e) => {
    e.preventDefault();
    saveProfile();
  });
  
  // Registrar listeners de eventos SSH de Tauri
  setupSshEventListeners();

  // Cargar perfiles iniciales de la DB
  loadProfiles();
});

// Resizing de terminales globales
window.addEventListener("resize", () => {
  shellPanes.forEach((pane) => {
    if (pane.isConnected) {
      setTimeout(() => {
        pane.fitAddon.fit();
        invoke("resize_ssh_pty", {
          terminalId: pane.terminalId,
          cols: pane.term.cols,
          rows: pane.term.rows
        }).catch(err => console.error("Error al redimensionar PTY:", err));
      }, 50);
    }
  });
});

// --- Setup SSH Event Listeners ---
function setTerminalConnectionStatus(
  activeTerm: ActiveTerminal,
  state: "connecting" | "connected" | "disconnected" | "error",
  text?: string,
) {
  const statusIndicator = activeTerm.panelEl.querySelector(".status-dot");
  const statusText = activeTerm.panelEl.querySelector(".terminal-status-text");
  if (statusIndicator) {
    statusIndicator.className = `status-dot ${state}`;
  }
  if (statusText) {
    const defaults: Record<typeof state, string> = {
      connecting: "Conectando...",
      connected: "Conectado al servidor remoto",
      disconnected: "Desconectado — Ctrl+R",
      error: "Error de Conexión — Ctrl+R",
    };
    statusText.textContent = text ?? defaults[state];
  }
}

/** Padre caído: el contexto queda offline y no se dejan hijos huérfanos sin SFTP. */
function handleParentShellDown(ctx: ActiveTerminal) {
  void closeChildShellsOf(ctx);
  if (ctx.id === currentActiveTerminalId || ctx.id === explorerBoundTerminalId) {
    showExplorerEmpty("Conecta un servidor para explorar archivos remotos.");
    explorerRoot = null;
    explorerBoundTerminalId = null;
  }
}

function setupSshEventListeners() {
  // Escuchar por conexión establecida
  listen<SshEventPayload>("ssh-connected", (event) => {
    const termId = event.payload.terminal_id;
    const pane = shellPanes.get(termId);
    const activeTerm = getContextForTerminal(termId);
    if (pane && activeTerm) {
      pane.isConnected = true;
      pane.isReconnecting = false;
      setPaneStatus(pane, "connected");
      if (pane.role === "parent") {
        setTerminalConnectionStatus(activeTerm, "connected");
      }

      // Limpiar terminal de mensajes previos
      pane.term.clear();
      if (activeTerm.focusedTerminalId === termId) {
        pane.term.focus();
      }

      // Ajustar dimensiones PTY
      setTimeout(() => {
        pane.fitAddon.fit();
        invoke("resize_ssh_pty", {
          terminalId: termId,
          cols: pane.term.cols,
          rows: pane.term.rows
        }).catch(err => console.error("Error al redimensionar PTY inicial:", err));
      }, 100);

      // No auto-listar SFTP al conectar: compite con el PTY y puede tumbar el transport.
      // El usuario abre Archivos / Actualizar cuando quiera.
    }
  });

  // Escuchar por salida de la consola (stdout)
  listen<SshEventPayload>("ssh-stdout", (event) => {
    const pane = shellPanes.get(event.payload.terminal_id);
    if (pane) {
      pane.term.write(event.payload.data);
    }
  });

  // Escuchar por error en la conexión
  listen<SshClosedPayload>("ssh-error", (event) => {
    const termId = event.payload.terminal_id;
    const errorMsg = event.payload.error || "Error desconocido";
    const pane = shellPanes.get(termId);
    const activeTerm = getContextForTerminal(termId);
    if (pane && activeTerm) {
      pane.isReconnecting = false;
      pane.term.write(
        `\r\n\x1b[31;1m[ERROR] ${errorMsg}\x1b[0m\r\n\x1b[33mCtrl+R para reconectar\x1b[0m\r\n`,
      );
      setPaneStatus(pane, "error");
      pane.isConnected = false;
      if (pane.role === "parent") {
        setTerminalConnectionStatus(activeTerm, "error");
        handleParentShellDown(activeTerm);
      }
    }
  });

  // Escuchar por sesión cerrada
  listen<SshClosedPayload>("ssh-closed", (event) => {
    const termId = event.payload.terminal_id;
    const pane = shellPanes.get(termId);
    const activeTerm = getContextForTerminal(termId);
    if (pane && activeTerm) {
      pane.isReconnecting = false;
      if (pane.isConnected) {
        const detail = event.payload.error ? ` (${event.payload.error})` : "";
        pane.term.write(
          `\r\n\x1b[33;1m[Conexión cerrada]${detail}\x1b[0m\r\n\x1b[33mCtrl+R para reconectar\x1b[0m\r\n`,
        );
      }
      setPaneStatus(pane, "disconnected");
      pane.isConnected = false;
      if (pane.role === "parent") {
        setTerminalConnectionStatus(activeTerm, "disconnected");
        handleParentShellDown(activeTerm);
      }
    }
  });
}

// --- Modal Helper Functions ---
function updateKeyStatusUi(configured: boolean) {
  if (!profKeyStatusEl) return;
  if (configured) {
    profKeyStatusEl.textContent = "Llave privada configurada";
    profKeyStatusEl.classList.add("is-configured");
  } else {
    profKeyStatusEl.textContent = "Sin llave configurada";
    profKeyStatusEl.classList.remove("is-configured");
  }
}

function openProfileModal(profile?: ConnectionProfile, folderId?: number) {
  if (!profileModal || !profileForm || !modalTitle) return;

  profileForm.reset();
  draftPrivateKeyContent = null;
  existingPrivateKeyContent = null;

  if (profile) {
    modalTitle.textContent = "Editar conexión";
    if (profileIdInput) profileIdInput.value = profile.id?.toString() || "";
    if (profileFolderIdInput) {
      profileFolderIdInput.value = profile.folder_id?.toString() || "";
    }
    if (profNameInput) profNameInput.value = profile.name;
    if (profHostInput) profHostInput.value = profile.host;
    if (profPortInput) profPortInput.value = profile.port.toString();
    if (profUsernameInput) profUsernameInput.value = profile.username;
    if (profAuthTypeSelect) profAuthTypeSelect.value = profile.auth_type;
    if (profPasswordInput) profPasswordInput.value = profile.password || "";
    existingPrivateKeyContent = profile.private_key?.trim() ? profile.private_key : null;
    updateKeyStatusUi(!!existingPrivateKeyContent);
    if (profPassphraseInput) profPassphraseInput.value = profile.passphrase || "";
    if (profKeepaliveInput) profKeepaliveInput.value = profile.keepalive.toString();
    
    if (tunTypeSelect) tunTypeSelect.value = profile.tunnel_type;
    if (tunLocalPortInput) tunLocalPortInput.value = profile.tunnel_local_port?.toString() || "";
    if (tunDestInput) tunDestInput.value = profile.tunnel_dest || "";

    toggleAuthFields(profile.auth_type);
    toggleTunnelFields(profile.tunnel_type);
  } else {
    modalTitle.textContent = "Nueva conexión";
    if (profileIdInput) profileIdInput.value = "";
    const targetFolder =
      folderId ??
      activeFolderId ??
      currentFolders[0]?.id ??
      null;
    if (profileFolderIdInput) {
      profileFolderIdInput.value = targetFolder?.toString() || "";
    }
    updateKeyStatusUi(false);
    toggleAuthFields('password');
    toggleTunnelFields('none');
  }

  profileModal.classList.add("active");
}

function closeProfileModal() {
  profileModal?.classList.remove("active");
  draftPrivateKeyContent = null;
  existingPrivateKeyContent = null;
}

function toggleAuthFields(authType: 'password' | 'key') {
  const pwdGroup = document.getElementById("auth-password-group");
  const keyGroup = document.getElementById("auth-key-group");

  if (authType === 'password') {
    if (pwdGroup) pwdGroup.style.display = "flex";
    if (keyGroup) keyGroup.style.display = "none";
  } else {
    if (pwdGroup) pwdGroup.style.display = "none";
    if (keyGroup) keyGroup.style.display = "flex";
  }
}

function toggleTunnelFields(tunnelType: string) {
  const destGroup = document.getElementById("tunnel-dest-group");
  const localPortInput = document.getElementById("tun-local-port") as HTMLInputElement;

  if (tunnelType === 'none') {
    if (destGroup) destGroup.style.display = "none";
    if (localPortInput) localPortInput.required = false;
  } else if (tunnelType === 'local') {
    if (destGroup) destGroup.style.display = "flex";
    if (localPortInput) localPortInput.required = true;
    if (tunDestInput) tunDestInput.required = true;
  } else if (tunnelType === 'dynamic') {
    if (destGroup) destGroup.style.display = "none";
    if (localPortInput) localPortInput.required = true;
  }
}

// --- CRUD Database Operations ---
async function loadProfiles() {
  try {
    const [folders, profiles] = await Promise.all([
      invoke<ConnectionFolder[]>("list_folders"),
      invoke<ConnectionProfile[]>("get_profiles"),
    ]);
    currentFolders = folders;
    currentProfiles = profiles;
    if (!foldersExpandSeeded) {
      foldersExpandSeeded = true;
    }
    // Keep expanded set in sync for newly created folders
    for (const f of currentFolders) {
      if (f.id !== undefined && !expandedFolderIds.has(f.id) && renamingFolderId === f.id) {
        expandedFolderIds.add(f.id);
      }
    }
    if (activeFolderId === null && currentFolders[0]?.id !== undefined) {
      activeFolderId = currentFolders[0].id;
    }
    renderProfileList();
  } catch (err) {
    console.error("Error al cargar perfiles:", err);
    renderProfileList();
  }
}

async function createNewFolder() {
  try {
    const id = await invoke<number>("create_folder", {
      name: "Nueva carpeta",
      sort_order: currentFolders.length,
    });
    expandedFolderIds.add(id);
    activeFolderId = id;
    renamingFolderId = id;
    await loadProfiles();
  } catch (err) {
    console.error("Error al crear carpeta:", err);
    void alertDialog({
      title: "Error",
      message: "Error al crear la carpeta: " + err,
    });
  }
}

async function saveProfile() {
  const idStr = profileIdInput?.value;
  const folderIdStr = profileFolderIdInput?.value;
  const profile: ConnectionProfile = {
    name: profNameInput?.value || "",
    host: profHostInput?.value || "",
    port: parseInt(profPortInput?.value || "22"),
    username: profUsernameInput?.value || "",
    auth_type: (profAuthTypeSelect?.value as 'password' | 'key') || 'password',
    keepalive: parseInt(profKeepaliveInput?.value || "60"),
    tunnel_type: (tunTypeSelect?.value as 'none' | 'local' | 'dynamic') || 'none',
    folder_id: folderIdStr ? parseInt(folderIdStr) : undefined,
  };

  if (idStr) {
    profile.id = parseInt(idStr);
  }

  if (profile.auth_type === 'password') {
    profile.password = profPasswordInput?.value || "";
  } else {
    const keyMaterial = draftPrivateKeyContent ?? existingPrivateKeyContent;
    if (!keyMaterial?.trim()) {
      void alertDialog({
        title: "Llave requerida",
        message: "Selecciona un archivo de llave privada con Examinar... antes de guardar.",
      });
      return;
    }
    profile.private_key = keyMaterial;
    profile.passphrase = profPassphraseInput?.value || "";
  }

  if (profile.tunnel_type !== 'none') {
    profile.tunnel_local_port = parseInt(tunLocalPortInput?.value || "8080");
  }
  if (profile.tunnel_type === 'local') {
    profile.tunnel_dest = tunDestInput?.value || "";
  }

  try {
    if (profile.id) {
      await invoke("update_profile", { profile });
    } else {
      await invoke("create_profile", { profile });
    }
    closeProfileModal();
    await loadProfiles();
  } catch (err) {
    console.error("Error al guardar perfil:", err);
    void alertDialog({
      title: "Error",
      message: "Error al guardar el perfil: " + err,
    });
  }
}

async function deleteProfile(id: number) {
  const ok = await confirmDialog({
    title: "¿Eliminar conexión?",
    message: "Se eliminará esta conexión guardada. No se puede deshacer.",
    confirmLabel: "Eliminar",
    danger: true,
  });
  if (!ok) return;

  try {
    await invoke("delete_profile", { id });
    await loadProfiles();
  } catch (err) {
    console.error("Error al eliminar perfil:", err);
    await alertDialog({
      title: "Error",
      message: "Error al eliminar el perfil: " + err,
    });
  }
}

async function deleteFolder(folder: ConnectionFolder) {
  if (folder.id === undefined) return;
  let count = 0;
  try {
    count = await invoke<number>("get_folder_connection_count", { id: folder.id });
  } catch {
    count = currentProfiles.filter((p) => p.folder_id === folder.id).length;
  }
  const impact =
    count === 0
      ? folder.name
      : `${folder.name} · ${count} conexión${count === 1 ? "" : "es"}`;
  const ok = await confirmDialog({
    title: "¿Eliminar carpeta?",
    message:
      count === 0
        ? "Se eliminará la carpeta vacía. No se puede deshacer."
        : "Se borrarán también las conexiones dentro. No se puede deshacer.",
    impact,
    confirmLabel: "Eliminar",
    danger: true,
  });
  if (!ok) return;

  try {
    await invoke("delete_folder", { id: folder.id });
    expandedFolderIds.delete(folder.id);
    if (activeFolderId === folder.id) {
      activeFolderId = currentFolders.find((f) => f.id !== folder.id)?.id ?? null;
    }
    await loadProfiles();
  } catch (err) {
    console.error("Error al eliminar carpeta:", err);
    await alertDialog({
      title: "Error",
      message: "Error al eliminar la carpeta: " + err,
    });
  }
}

// --- Render Helper ---
function renderProfileList() {
  const container = profileListContainer;
  if (!container) return;

  if (currentFolders.length === 0) {
    container.innerHTML = `<div class="profile-list-empty">No hay carpetas. Agrega una carpeta para organizar conexiones.</div>`;
    return;
  }

  container.innerHTML = "";

  for (const folder of currentFolders) {
    if (folder.id === undefined) continue;
    const folderId = folder.id;
    const expanded = expandedFolderIds.has(folderId);
    const children = currentProfiles.filter((p) => p.folder_id === folderId);

    // Omitir si es la carpeta General y está vacía
    if (folder.name === "General" && children.length === 0) {
      continue;
    }

    const block = document.createElement("div");
    block.className = "folder-block";
    block.dataset.folderId = String(folderId);

    const row = document.createElement("div");
    row.className = "folder-row";
    row.title = expanded ? "Clic para colapsar" : "Clic para expandir";
    if (activeFolderId === folderId) row.classList.add("is-active-context");

    const toggleFolderRow = () => {
      activeFolderId = folderId;
      if (expandedFolderIds.has(folderId)) {
        expandedFolderIds.delete(folderId);
      } else {
        expandedFolderIds.add(folderId);
      }
      renderProfileList();
    };

    const chevronBtn = document.createElement("button");
    chevronBtn.type = "button";
    chevronBtn.className = "folder-chevron";
    chevronBtn.title = expanded ? "Colapsar" : "Expandir";
    chevronBtn.setAttribute("aria-expanded", String(expanded));
    chevronBtn.appendChild(
      icon(expanded ? AppIcons.chevronDown : AppIcons.chevronRight, {
        size: 14,
        className: "icon--sm",
      }),
    );
    // Click bubbles to .folder-row (whole-row toggle)

    const folderIconEl = icon(AppIcons.folder, { size: 16, className: "folder-icon icon--md" });

    const actions = document.createElement("div");
    actions.className = "folder-actions";

    const addConnBtn = document.createElement("button");
    addConnBtn.type = "button";
    addConnBtn.className = "btn-icon";
    addConnBtn.title = "Nueva conexión en esta carpeta";
    addConnBtn.setAttribute("aria-label", "Nueva conexión en esta carpeta");
    setButtonIcon(addConnBtn, AppIcons.plus, { size: 14, className: "icon--sm" });
    addConnBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      activeFolderId = folderId;
      openProfileModal(undefined, folderId);
    });

    actions.append(addConnBtn);
    actions.addEventListener("click", (e) => e.stopPropagation());

    row.append(chevronBtn, folderIconEl);
    row.addEventListener("click", toggleFolderRow);
    row.addEventListener("contextmenu", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      void (async () => {
        const action = await showContextMenu(ev.clientX, ev.clientY, [
          { id: "rename", label: "Renombrar", icon: AppIcons.type },
          {
            id: "delete",
            label: "Eliminar",
            icon: AppIcons.trash2,
            danger: true,
            separatorBefore: true,
          },
        ]);
        if (action === "rename") {
          renamingFolderId = folderId;
          renderProfileList();
        } else if (action === "delete") {
          void deleteFolder(folder);
        }
      })();
    });

    if (renamingFolderId === folderId) {
      const input = document.createElement("input");
      input.type = "text";
      input.className = "folder-name-input";
      input.value = folder.name;
      input.addEventListener("click", (e) => e.stopPropagation());
      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          void commitFolderRename(folderId, input.value);
        } else if (e.key === "Escape") {
          e.preventDefault();
          renamingFolderId = null;
          renderProfileList();
        }
      });
      input.addEventListener("blur", () => {
        if (renamingFolderId === folderId) {
          void commitFolderRename(folderId, input.value);
        }
      });
      row.append(input, actions);
      block.appendChild(row);
      container.appendChild(block);
      queueMicrotask(() => {
        input.focus();
        input.select();
      });
    } else {
      const nameEl = document.createElement("span");
      nameEl.className = "folder-name";
      nameEl.textContent = folder.name;
      row.append(nameEl, actions);
      block.appendChild(row);
    }

    if (expanded) {
      const childrenEl = document.createElement("div");
      childrenEl.className = "folder-children";
      if (children.length === 0) {
        const empty = document.createElement("div");
        empty.className = "folder-empty";
        empty.textContent = "Sin conexiones";
        childrenEl.appendChild(empty);
      } else {
        for (const prof of children) {
          childrenEl.appendChild(buildProfileItem(prof));
        }
      }
      block.appendChild(childrenEl);
    }

    container.appendChild(block);
  }
}

async function commitFolderRename(folderId: number, name: string) {
  const trimmed = name.trim();
  renamingFolderId = null;
  if (!trimmed) {
    renderProfileList();
    return;
  }
  const current = currentFolders.find((f) => f.id === folderId);
  if (current && current.name === trimmed) {
    renderProfileList();
    return;
  }
  try {
    await invoke("update_folder", { id: folderId, name: trimmed });
    await loadProfiles();
  } catch (err) {
    console.error("Error al renombrar carpeta:", err);
    await alertDialog({
      title: "Error",
      message: "Error al renombrar: " + err,
    });
    await loadProfiles();
  }
}

async function commitProfileRename(profile: ConnectionProfile, name: string) {
  const trimmed = name.trim();
  renamingProfileId = null;
  if (!trimmed || profile.id === undefined) {
    renderProfileList();
    return;
  }
  if (profile.name === trimmed) {
    renderProfileList();
    return;
  }
  try {
    const updated: ConnectionProfile = { ...profile, name: trimmed };
    await invoke("update_profile", { profile: updated });
    await loadProfiles();
  } catch (err) {
    console.error("Error al renombrar conexión:", err);
    await alertDialog({
      title: "Error",
      message: "Error al renombrar: " + err,
    });
    await loadProfiles();
  }
}

function buildProfileItem(prof: ConnectionProfile): HTMLElement {
  const item = document.createElement("div");
  item.className = "profile-item";
  if (prof.id === activeProfileId) item.classList.add("active");

  const userAtHost = `${prof.username}@${prof.host}`;
  const hostDisplay = `${userAtHost}:${prof.port}`;
  const isRenaming = prof.id !== undefined && renamingProfileId === prof.id;

  const header = document.createElement("div");
  header.className = "profile-item-header";

  if (isRenaming) {
    const input = document.createElement("input");
    input.type = "text";
    input.className = "profile-name-input";
    input.value = prof.name;
    input.addEventListener("click", (e) => e.stopPropagation());
    input.addEventListener("dblclick", (e) => e.stopPropagation());
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        void commitProfileRename(prof, input.value);
      } else if (e.key === "Escape") {
        e.preventDefault();
        renamingProfileId = null;
        renderProfileList();
      }
    });
    input.addEventListener("blur", () => {
      if (renamingProfileId === prof.id) {
        void commitProfileRename(prof, input.value);
      }
    });
    header.appendChild(input);
    queueMicrotask(() => {
      input.focus();
      input.select();
    });
  } else {
    const nameEl = document.createElement("span");
    nameEl.className = "profile-item-name";
    nameEl.textContent = prof.name;
    header.appendChild(nameEl);
  }

  const hostRow = document.createElement("div");
  hostRow.className = "profile-item-host-row";

  const hostEl = document.createElement("span");
  hostEl.className = "profile-item-host";
  hostEl.textContent = hostDisplay;

  const copyBtn = document.createElement("button");
  copyBtn.type = "button";
  copyBtn.className = "btn-icon btn-copy-endpoint";
  copyBtn.title = "Copiar user@host";
  copyBtn.setAttribute("aria-label", "Copiar user@host");
  setButtonIcon(copyBtn, AppIcons.copy, { size: 14, className: "icon--sm" });
  copyBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    void copyUserAtHost(userAtHost, copyBtn);
  });

  hostRow.append(hostEl, copyBtn);
  item.append(header, hostRow);

  item.addEventListener("click", () => {
    highlightProfile(prof.id ?? null);
  });

  item.addEventListener("dblclick", () => {
    if (isRenaming || prof.id === undefined) return;
    activeProfileId = prof.id;
    startNewSshConnection(prof);
  });

  item.addEventListener("contextmenu", (ev) => {
    ev.preventDefault();
    ev.stopPropagation();
    if (prof.id === undefined) return;
    void (async () => {
      const action = await showContextMenu(ev.clientX, ev.clientY, [
        { id: "edit", label: "Editar", icon: AppIcons.pencil },
        { id: "rename", label: "Renombrar", icon: AppIcons.type },
        {
          id: "delete",
          label: "Eliminar",
          icon: AppIcons.trash2,
          danger: true,
          separatorBefore: true,
        },
      ]);
      if (action === "edit") {
        openProfileModal(prof);
      } else if (action === "rename") {
        renamingProfileId = prof.id!;
        renderProfileList();
      } else if (action === "delete") {
        void deleteProfile(prof.id!);
      }
    })();
  });

  return item;
}

async function copyUserAtHost(text: string, btn: HTMLButtonElement) {
  try {
    await navigator.clipboard.writeText(text);
    btn.title = "Copiado";
    setTimeout(() => {
      btn.title = "Copiar user@host";
    }, 1200);
  } catch (err) {
    console.error("No se pudo copiar al portapapeles:", err);
    await alertDialog({
      title: "Error",
      message: "No se pudo copiar: " + err,
    });
  }
}

function highlightProfile(id: number | null) {
  if (activeProfileId === id) return;
  activeProfileId = id;
  renderProfileList();
}

// --- SSH Connection Execution ---
/** Densidad del grid del contexto (1..4 celdas) y refit de cada shell. */
function applyGridDensity(ctx: ActiveTerminal) {
  ctx.gridEl.className = gridDensityClass(ctx.panes.length);
  if (ctx.addShellBtn) {
    const canAdd = canAddChildShell(ctx.panes.length);
    ctx.addShellBtn.disabled = !canAdd;
    ctx.addShellBtn.title = canAdd
      ? "Nuevo shell en este servidor"
      : `Máximo ${MAX_CHILD_SHELLS} shells adicionales`;
  }
  setTimeout(() => {
    ctx.panes.forEach((pane) => {
      pane.fitAddon.fit();
      if (pane.isConnected) {
        invoke("resize_ssh_pty", {
          terminalId: pane.terminalId,
          cols: pane.term.cols,
          rows: pane.term.rows,
        }).catch((err) => console.error("Error al redimensionar PTY:", err));
      }
    });
  }, 50);
}

function focusShellPane(ctx: ActiveTerminal, terminalId: string) {
  const pane = ctx.panes.find((p) => p.terminalId === terminalId);
  if (!pane) return;
  ctx.focusedTerminalId = terminalId;
  ctx.panes.forEach((p) => p.cellEl.classList.toggle("focused", p === pane));
  pane.term.focus();
}

function setPaneStatus(
  pane: ShellPane,
  state: "connecting" | "connected" | "disconnected" | "error",
) {
  const dot = pane.cellEl.querySelector(".status-dot");
  if (dot) dot.className = `status-dot ${state}`;
}

function createShellPane(
  ctx: ActiveTerminal,
  terminalId: string,
  role: ShellRole,
  label: string,
): ShellPane {
  const cellEl = document.createElement("div");
  cellEl.className = "term-cell";
  cellEl.id = `cell-${terminalId}`;
  cellEl.innerHTML = `
    <div class="term-cell-header">
      <span class="status-dot connecting"></span>
      <span class="term-cell-label">${escapeHtml(label)}</span>
    </div>
    <div class="terminal-canvas-container" id="canvas-${terminalId}"></div>
  `;

  if (role === "child") {
    const cellClose = document.createElement("button");
    cellClose.type = "button";
    cellClose.className = "term-cell-close";
    cellClose.title = "Cerrar este shell";
    cellClose.setAttribute("aria-label", "Cerrar este shell");
    setButtonIcon(cellClose, AppIcons.x, { size: 12, className: "icon--sm" });
    cellClose.addEventListener("click", (ev) => {
      ev.stopPropagation();
      void closeChildShell(ctx, terminalId);
    });
    cellEl.querySelector(".term-cell-header")?.appendChild(cellClose);
  }

  ctx.gridEl.appendChild(cellEl);

  const canvasContainer = cellEl.querySelector(".terminal-canvas-container") as HTMLElement;
  const monoFontFamily =
    getComputedStyle(document.documentElement).getPropertyValue("--font-mono").trim() ||
    "monospace";
  const term = new Terminal({
    allowTransparency: true,
    cursorBlink: true,
    cursorStyle: "block",
    theme: { ...(THEME_TERMINAL_COLORS[getActiveTheme()] || THEME_TERMINAL_COLORS["nekossh"]) },
    fontFamily: monoFontFamily,
    fontSize: 14,
  });

  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(canvasContainer);
  fitAddon.fit();

  term.write("\x1b[35;1m[Iniciando sesión SSH en NekoSSH...]\x1b[0m\r\n");

  // Evidencia: writes de 1 byte (onData por tecla) provocan "transport read" en libssh2
  // con lector PTY concurrente. Coalescer ~16ms agrupa tecleo en un solo invoke/write.
  const writeBuffer = { data: "" };
  let writeTimer: ReturnType<typeof setTimeout> | null = null;
  const flushWriteBuffer = () => {
    writeTimer = null;
    const payload = writeBuffer.data;
    writeBuffer.data = "";
    if (!payload) return;
    invoke("write_ssh_input", { terminalId, data: payload })
      .catch(err => console.error("Error al escribir input SSH:", err));
  };
  const enqueuePtyInput = (data: string) => {
    if (!data) return;
    writeBuffer.data += data;
    if (writeTimer == null) {
      writeTimer = setTimeout(flushWriteBuffer, 16);
    }
  };

  term.onData((data) => {
    enqueuePtyInput(data);
  });

  // Moba-style: seleccionar → copiar; clic derecho → pegar (sin Enter final).
  // Clipboard nativo Tauri (sin prompt de permiso del WebView).
  term.onSelectionChange(() => {
    const selected = term.getSelection();
    if (!selected) return;
    void writeText(selected).catch((err) => {
      console.error("Error al copiar selección de terminal:", err);
    });
  });

  canvasContainer.addEventListener("contextmenu", (ev) => {
    ev.preventDefault();
    void (async () => {
      try {
        const raw = await readText();
        const sanitized = stripTrailingPasteNoise(raw ?? "");
        enqueuePtyInput(sanitized);
      } catch (err) {
        console.error("Error al pegar en terminal:", err);
      }
    })();
  });

  cellEl.addEventListener("mousedown", () => {
    focusShellPane(ctx, terminalId);
  });

  // Ctrl+R: reconectar solo si este shell está desconectado (estilo Terminus/Moba).
  term.attachCustomKeyEventHandler((ev) => {
    if (ev.type !== "keydown") return true;
    if (!(ev.ctrlKey && !ev.altKey && !ev.metaKey && (ev.key === "r" || ev.key === "R"))) {
      return true;
    }
    const pane = shellPanes.get(terminalId);
    if (!pane || pane.isConnected || pane.isReconnecting) {
      return true; // dejar pasar al remoto si hay sesión viva
    }
    ev.preventDefault();
    void reconnectTerminalSession(terminalId);
    return false;
  });

  // Registrar resize del emulador hacia el backend
  term.onResize((size) => {
    invoke("resize_ssh_pty", {
      terminalId,
      cols: size.cols,
      rows: size.rows
    }).catch(err => console.error("Error al redimensionar PTY:", err));
  });

  const pane: ShellPane = {
    terminalId,
    contextId: ctx.id,
    role,
    label,
    term,
    fitAddon,
    cellEl,
    isConnected: false,
    isReconnecting: false,
  };

  ctx.panes.push(pane);
  shellPanes.set(terminalId, pane);
  return pane;
}

/** Abre un shell hijo en el mismo contexto (mismo perfil, login independiente). */
async function addChildShell(ctx: ActiveTerminal) {
  if (!canAddChildShell(ctx.panes.length)) return;

  const childId = `${ctx.id}-s${++ctx.childSeq}`;
  const pane = createShellPane(ctx, childId, "child", childShellLabel(ctx.panes.length));
  applyGridDensity(ctx);
  focusShellPane(ctx, childId);

  try {
    await invokeStartSshSession(childId, ctx.profile);
  } catch (err) {
    console.error("Error al iniciar shell hijo:", err);
    pane.isConnected = false;
    setPaneStatus(pane, "error");
    pane.term.write(
      `\r\n\x1b[31;1m[ERROR] No se pudo abrir el shell: ${err}\x1b[0m\r\n\x1b[33mCtrl+R para reintentar\x1b[0m\r\n`,
    );
  }
}

/** Cierra un shell hijo sin tocar el padre ni el SFTP del contexto. */
async function closeChildShell(ctx: ActiveTerminal, childId: string) {
  const index = ctx.panes.findIndex((p) => p.terminalId === childId);
  if (index <= 0) return; // 0 es el padre: no se cierra solo

  const pane = ctx.panes[index];
  pane.isConnected = false;

  try {
    await invoke("close_ssh_session", { terminalId: childId });
  } catch (err) {
    console.error("Error al cerrar shell hijo:", err);
  }

  pane.term.dispose();
  pane.cellEl.remove();
  ctx.panes.splice(index, 1);
  shellPanes.delete(childId);

  applyGridDensity(ctx);
  const fallback = ctx.panes[focusIndexAfterClose(index, ctx.panes.length)] ?? ctx.panes[0];
  if (fallback) focusShellPane(ctx, fallback.terminalId);
}

/** Cierra todos los hijos del contexto (padre caído o cierre de pestaña). */
async function closeChildShellsOf(ctx: ActiveTerminal) {
  const childIds = ctx.panes.filter((p) => p.role === "child").map((p) => p.terminalId);
  for (const id of childIds) {
    await closeChildShell(ctx, id);
  }
}

function startNewSshConnection(profile: ConnectionProfile) {
  const terminalId = `term-${Date.now()}`;
  
  // Ocultar pantalla de bienvenida
  const welcomeScreen = mainDisplayArea?.querySelector(".welcome-screen");
  if (welcomeScreen) {
    (welcomeScreen as HTMLElement).style.display = "none";
  }

  // Mostrar botón Cerrar Todo si hay terminales
  if (btnCloseAllTerminals) {
    btnCloseAllTerminals.style.display = "block";
  }

  // 1. Crear Pestaña de Terminal
  const tabEl = document.createElement("div");
  tabEl.className = "term-tab";
  tabEl.id = `tab-${terminalId}`;

  const titleSpan = document.createElement("span");
  titleSpan.className = "term-tab-title";
  titleSpan.textContent = profile.name;

  const closeBtn = document.createElement("button");
  closeBtn.type = "button";
  closeBtn.className = "term-tab-close";
  closeBtn.title = "Cerrar Terminal";
  closeBtn.setAttribute("aria-label", "Cerrar Terminal");
  setButtonIcon(closeBtn, AppIcons.x, { size: 12, className: "icon--sm" });

  tabEl.appendChild(titleSpan);
  tabEl.appendChild(closeBtn);

  tabEl.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (target.closest(".term-tab-close")) {
      void closeTerminalSession(terminalId);
    } else {
      switchActiveTerminal(terminalId);
    }
  });

  terminalTabsList?.appendChild(tabEl);

  // 2. Crear Contenedor del Emulador
  const panelEl = document.createElement("div");
  panelEl.className = "terminal-panel";
  panelEl.id = `panel-${terminalId}`;
  panelEl.innerHTML = `
    <div class="terminal-panel-header">
      <div class="terminal-status-indicator">
        <div class="status-dot connecting"></div>
        <span class="terminal-status-text">Conectando...</span>
      </div>
      <div class="terminal-panel-header-right">
        <div class="terminal-info-text">${escapeHtml(profile.username)}@${escapeHtml(profile.host)}:${profile.port}</div>
      </div>
    </div>
    <div class="term-grid cells-1" id="grid-${terminalId}"></div>
  `;

  const addShellBtn = document.createElement("button");
  addShellBtn.type = "button";
  addShellBtn.className = "term-add-shell";
  addShellBtn.title = "Nuevo shell en este servidor";
  addShellBtn.setAttribute("aria-label", "Nuevo shell en este servidor");
  setButtonIcon(addShellBtn, AppIcons.plus, { size: 12, className: "icon--sm" });
  panelEl.querySelector(".terminal-panel-header-right")?.prepend(addShellBtn);

  mainDisplayArea?.appendChild(panelEl);

  applyBackgroundSettings(bgImageUrl, currentBgOpacity());

  // 3. Crear el contexto de pestaña y su shell padre (ancla del SFTP)
  const activeTerm: ActiveTerminal = {
    id: terminalId,
    profileName: profile.name,
    profile: { ...profile },
    panes: [],
    focusedTerminalId: terminalId,
    gridEl: panelEl.querySelector(".term-grid") as HTMLElement,
    panelEl,
    tabEl,
    addShellBtn,
    childSeq: 0,
    get term() {
      return this.panes[0].term;
    },
    get fitAddon() {
      return this.panes[0].fitAddon;
    },
    get isConnected() {
      return this.panes[0]?.isConnected ?? false;
    },
    set isConnected(value: boolean) {
      if (this.panes[0]) this.panes[0].isConnected = value;
    },
    get isReconnecting() {
      return this.panes[0]?.isReconnecting ?? false;
    },
    set isReconnecting(value: boolean) {
      if (this.panes[0]) this.panes[0].isReconnecting = value;
    },
  };

  activeTerminals.set(terminalId, activeTerm);
  const parentPane = createShellPane(activeTerm, terminalId, "parent", "Principal");
  applyGridDensity(activeTerm);
  focusShellPane(activeTerm, terminalId);

  addShellBtn.addEventListener("click", (ev) => {
    ev.stopPropagation();
    void addChildShell(activeTerm);
  });

  // Seleccionar la terminal recién creada
  switchActiveTerminal(terminalId);

  // 4. Iniciar Conexión SSH en backend Rust
  void invokeStartSshSession(terminalId, profile).catch((err) => {
    console.error("Error al iniciar sesión SSH:", err);
    parentPane.term.write(`\r\n\x1b[31;1m[ERROR] No se pudo invocar el backend: ${err}\x1b[0m\r\n\x1b[33mCtrl+R para reconectar\x1b[0m\r\n`);
    parentPane.isConnected = false;
    setPaneStatus(parentPane, "error");
    setTerminalConnectionStatus(activeTerm, "error");
  });
}

function resolveProfileForReconnect(snapshot: ConnectionProfile): ConnectionProfile {
  if (snapshot.id !== undefined) {
    const fresh = currentProfiles.find((p) => p.id === snapshot.id);
    if (fresh) return fresh;
  }
  return snapshot;
}

async function invokeStartSshSession(terminalId: string, profile: ConnectionProfile) {
  await invoke("start_ssh_session", {
    terminalId,
    host: profile.host,
    port: profile.port,
    username: profile.username,
    authType: profile.auth_type,
    password: profile.password || null,
    privateKey: profile.private_key || null,
    passphrase: profile.passphrase || null,
    keepalive: profile.keepalive || 60,
  });
}

async function reconnectTerminalSession(terminalId: string) {
  const pane = shellPanes.get(terminalId);
  const activeTerm = getContextForTerminal(terminalId);
  if (!pane || !activeTerm || pane.isConnected || pane.isReconnecting) return;

  pane.isReconnecting = true;
  setPaneStatus(pane, "connecting");
  if (pane.role === "parent") {
    setTerminalConnectionStatus(activeTerm, "connecting", "Reconectando...");
  }
  pane.term.write("\r\n\x1b[35;1m[Reconectando sesión SSH...]\x1b[0m\r\n");

  // Cleanup idempotente por si quedó entrada stale en el backend.
  try {
    await invoke("close_ssh_session", { terminalId });
  } catch {
    // ignore
  }

  const profile = resolveProfileForReconnect(activeTerm.profile);
  if (pane.role === "parent") {
    activeTerm.profile = { ...profile };
    activeTerm.profileName = profile.name;
  }

  try {
    await invokeStartSshSession(terminalId, profile);
  } catch (err) {
    console.error("Error al reconectar SSH:", err);
    pane.isReconnecting = false;
    pane.isConnected = false;
    setPaneStatus(pane, "error");
    if (pane.role === "parent") {
      setTerminalConnectionStatus(activeTerm, "error");
    }
    pane.term.write(
      `\r\n\x1b[31;1m[ERROR] No se pudo reconectar: ${err}\x1b[0m\r\n\x1b[33mCtrl+R para reintentar\x1b[0m\r\n`,
    );
  }
}

function switchActiveTerminal(terminalId: string) {
  if (currentActiveTerminalId) {
    const outgoing = activeTerminals.get(currentActiveTerminalId);
    if (outgoing) {
      outgoing.explorerCwd = explorerCwd;
      outgoing.explorerRoot = explorerRoot;
    }
  }

  currentActiveTerminalId = terminalId;
  console.log("Terminal activa cambiada a:", currentActiveTerminalId);

  activeTerminals.forEach((term, id) => {
    if (id === terminalId) {
      term.tabEl.classList.add("active");
      term.panelEl.classList.add("active");
      
      explorerCwd = term.explorerCwd ?? "";
      explorerRoot = term.explorerRoot ?? null;
      if (filesCwdInput) {
        filesCwdInput.value = explorerCwd;
        filesCwdInput.setAttribute("title", explorerCwd);
      }

      // Pequeño delay para asegurar render correcto y foco del DOM
      setTimeout(() => {
        term.panes.forEach((pane) => pane.fitAddon.fit());
        const focused =
          term.panes.find((pane) => pane.terminalId === term.focusedTerminalId) ?? term.panes[0];
        focused?.term.focus();
      }, 50);
    } else {
      term.tabEl.classList.remove("active");
      term.panelEl.classList.remove("active");
    }
  });

  // Solo SFTP si la pestaña Archivos está visible. Llamar sftp_list_dir en cada
  // cambio de terminal compite con el PTY (mismo Session) y puede tumbar el transport.
  if (panelFiles?.classList.contains("active")) {
    void refreshExplorerForActiveTerminal();
  }

  if (panelMonitor?.classList.contains("active")) {
    checkMonitorSessionState();
  }
}

async function closeTerminalSession(terminalId: string, skipConfirm = false) {
  const activeTerm = activeTerminals.get(terminalId);
  if (!activeTerm) return;

  const hasLiveSession = activeTerm.panes.some((pane) => pane.isConnected);
  if (hasLiveSession && !skipConfirm) {
    const profileName = activeTerm.profile.name || `${activeTerm.profile.username}@${activeTerm.profile.host}`;
    const ok = await confirmDialog({
      title: "¿Cerrar sesión SSH activa?",
      message: `La conexión a "${profileName}" está activa. ¿Deseas desconectarte y cerrar la pestaña?`,
      confirmLabel: "Desconectar",
      danger: true,
    });
    if (!ok) return;
  }

  // Cerrar primero los shells hijos: el contexto se va completo con la pestaña.
  await closeChildShellsOf(activeTerm);

  // Marcar desconectado antes del invoke para que un ssh-closed tardío no pinte banner.
  activeTerm.isConnected = false;

  try {
    await invoke("close_ssh_session", { terminalId });
  } catch (err) {
    console.error("Error al cerrar sesión SSH en backend:", err);
  }

  // Limpiar explorador si estaba ligado a esta terminal
  if (explorerBoundTerminalId === terminalId) {
    showExplorerEmpty("Conecta un servidor para explorar archivos remotos.");
    explorerRoot = null;
    explorerBoundTerminalId = null;
  }

  activeTerm.term.dispose();
  shellPanes.delete(terminalId);
  activeTerm.tabEl.remove();
  activeTerm.panelEl.remove();
  activeTerminals.delete(terminalId);

  if (currentActiveTerminalId === terminalId) {
    currentActiveTerminalId = null;
  }

  if (activeTerminals.size > 0) {
    const nextKey = activeTerminals.keys().next().value;
    if (nextKey) switchActiveTerminal(nextKey);
  } else {
    currentActiveTerminalId = null;
    if (btnCloseAllTerminals) btnCloseAllTerminals.style.display = "none";

    const welcomeScreen = mainDisplayArea?.querySelector(".welcome-screen");
    if (welcomeScreen) {
      (welcomeScreen as HTMLElement).style.display = "flex";
    }
  }

  if (panelMonitor?.classList.contains("active")) {
    checkMonitorSessionState();
  }
}

async function closeAllTerminals() {
  if (activeTerminals.size === 0) return;

  const connectedCount = Array.from(activeTerminals.values()).filter((t) =>
    t.panes.some((pane) => pane.isConnected),
  ).length;
  if (connectedCount > 0) {
    const ok = await confirmDialog({
      title: "¿Cerrar todas las terminales?",
      message: `Hay ${connectedCount} sesión${connectedCount === 1 ? "" : "es"} SSH activa${connectedCount === 1 ? "" : "s"}. ¿Seguro que deseas cerrar todas las pestañas?`,
      confirmLabel: "Cerrar todas",
      danger: true,
    });
    if (!ok) return;
  }

  const ids = Array.from(activeTerminals.keys());
  // Secuencial: cada close apaga la Session en backend antes de la siguiente.
  for (const id of ids) {
    await closeTerminalSession(id, true);
  }
}

// --- Utils ---
function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.innerText = str;
  return div.innerHTML;
}

// --- Remote History (Fase 5) ---
function initHistoryUi() {
  historyModal = document.getElementById("history-modal");
  btnCloseHistory = document.getElementById("btn-close-history") as HTMLButtonElement;
  historySearchInput = document.getElementById("history-search") as HTMLInputElement;
  historyListTable = document.getElementById("history-list");
  btnHistoryPrev = document.getElementById("btn-history-prev") as HTMLButtonElement;
  btnHistoryNext = document.getElementById("btn-history-next") as HTMLButtonElement;

  if (btnCloseHistory) {
    setButtonIcon(btnCloseHistory, AppIcons.x, { size: 16, className: "icon--md" });
    btnCloseHistory.addEventListener("click", () => closeHistoryModal());
  }

  historyModal?.addEventListener("click", (e) => {
    if (e.target === historyModal) closeHistoryModal();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && historyModal?.classList.contains("active")) {
      closeHistoryModal();
      e.preventDefault();
      return;
    }

    // Ctrl+Shift+H (o Ctrl+Alt+H) para abrir el modal flotante
    if (e.ctrlKey && (e.shiftKey || e.altKey) && e.key.toLowerCase() === "h") {
      if (currentActiveTerminalId) {
        e.preventDefault();
        void openHistoryModal();
      }
    }
  });

  historySearchInput?.addEventListener("input", () => {
    historySelectedRowIndex = 0;
    renderHistoryList();
  });

  historySearchInput?.addEventListener("keydown", (e) => {
    const listRows = historyListTable?.querySelectorAll(".snippets-row");
    if (!listRows || listRows.length === 0 || listRows[0].classList.contains("snippets-empty")) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      historySelectedRowIndex = (historySelectedRowIndex + 1) % listRows.length;
      updateRowSelection(listRows as NodeListOf<HTMLElement>);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      historySelectedRowIndex = (historySelectedRowIndex - 1 + listRows.length) % listRows.length;
      updateRowSelection(listRows as NodeListOf<HTMLElement>);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const query = (historySearchInput?.value ?? "").trim().toLowerCase();
      const filtered = historyItems.filter((item) => item.command.toLowerCase().includes(query));
      if (filtered[historySelectedRowIndex]) {
        injectHistoryCommand(filtered[historySelectedRowIndex].command, e.shiftKey);
      }
    }
  });

  btnHistoryPrev?.addEventListener("click", () => {
    historyOffset += historyLimit;
    void loadRemoteHistory();
  });

  btnHistoryNext?.addEventListener("click", () => {
    if (historyOffset >= historyLimit) {
      historyOffset -= historyLimit;
      void loadRemoteHistory();
    }
  });
}

function updateRowSelection(rows: NodeListOf<HTMLElement>) {
  rows.forEach((row, idx) => {
    const isSel = idx === historySelectedRowIndex;
    row.classList.toggle("is-selected", isSel);
    if (isSel) {
      row.scrollIntoView({ block: "nearest" });
    }
  });
}

async function openHistoryModal() {
  if (!historyModal || !currentActiveTerminalId) return;
  historyModal.classList.add("active");
  historyOffset = 0;
  historySelectedRowIndex = 0;
  if (historySearchInput) historySearchInput.value = "";
  if (historyListTable) {
    historyListTable.innerHTML = `<div class="snippets-empty">Cargando historial remoto...</div>`;
  }
  await loadRemoteHistory();
  queueMicrotask(() => historySearchInput?.focus());
}

function closeHistoryModal() {
  historyModal?.classList.remove("active");
  if (currentActiveTerminalId) {
    const ctx = activeTerminals.get(currentActiveTerminalId);
    if (ctx) focusShellPane(ctx, ctx.focusedTerminalId);
  }
}

async function loadRemoteHistory() {
  if (!currentActiveTerminalId) return;
  try {
    const rawLines = await invoke<string[]>("sftp_read_remote_history_paged", {
      terminalId: currentActiveTerminalId,
      offset: historyOffset,
      limit: historyLimit,
    });

    historyItems = parseRemoteHistoryLines(rawLines);
    historyItems.reverse();
    historySelectedRowIndex = historyItems.length > 0 ? 0 : -1;
    renderHistoryList();
  } catch (err) {
    console.error("Error al leer historial:", err);
    if (historyListTable) {
      historyListTable.innerHTML = `<div class="snippets-empty" style="color: var(--color-error-neon);">Error al leer historial: ${escapeHtml(String(err))}</div>`;
    }
  }
}

function renderHistoryList() {
  if (!historyListTable) return;
  historyListTable.innerHTML = "";

  const query = (historySearchInput?.value ?? "").trim().toLowerCase();
  const filtered = historyItems.filter((item) => {
    return item.command.toLowerCase().includes(query);
  });

  if (filtered.length === 0) {
    historyListTable.innerHTML = `<div class="snippets-empty">No se encontraron comandos</div>`;
    return;
  }

  filtered.forEach((item, index) => {
    const row = document.createElement("div");
    row.className = "snippets-row";
    if (index === historySelectedRowIndex) {
      row.classList.add("is-selected");
    }

    // Clic en la fila: SOLO SELECCIONA ("no baja con un click")
    row.addEventListener("click", () => {
      historySelectedRowIndex = index;
      const listRows = historyListTable!.querySelectorAll(".snippets-row");
      updateRowSelection(listRows as NodeListOf<HTMLElement>);
    });

    const text = document.createElement("div");
    text.className = "snippets-row-text";

    const title = document.createElement("div");
    title.className = "snippets-row-title";
    title.style.color = "var(--color-text-muted)";
    title.style.fontSize = "0.75rem";
    title.textContent = item.date;

    const cmd = document.createElement("div");
    cmd.className = "snippets-row-cmd";
    cmd.textContent = item.command;

    text.append(title, cmd);

    const actions = document.createElement("div");
    actions.className = "snippets-row-actions";

    const injectBtn = document.createElement("button");
    injectBtn.type = "button";
    injectBtn.className = "btn-icon";
    injectBtn.title = "Pegar en terminal";
    setButtonIcon(injectBtn, AppIcons.copy, { size: 14, className: "icon--sm" }); // Reutilizar AppIcons.copy exacto de snippets

    // Clic en el botón: SOLO COPIA AL PORTAPAPELES
    injectBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      void copyCommandToClipboard(item.command);
    });

    actions.appendChild(injectBtn);
    row.append(text, actions);
    historyListTable!.appendChild(row);
  });

  if (btnHistoryNext) {
    btnHistoryNext.disabled = historyOffset === 0;
  }
}

async function injectHistoryCommand(command: string, execute: boolean) {
  closeHistoryModal();
  if (!currentActiveTerminalId) return;
  const term = activeTerminals.get(currentActiveTerminalId);
  if (!term) return;

  const data = execute ? `${command}\r` : command;
  void invoke("write_ssh_input", {
    terminalId: term.focusedTerminalId,
    data,
  });
}

// --- Resource Monitor Logic ---
function initMonitorIcons() {
  if (tabBtnMonitor && !tabBtnMonitor.querySelector("svg")) {
    tabBtnMonitor.prepend(icon(AppIcons.activity, { size: 14, className: "icon--sm" }));
  }

  const osSlot = document.getElementById("monitor-icon-os");
  if (osSlot && !osSlot.hasChildNodes()) osSlot.appendChild(icon(AppIcons.server, { size: 12 }));

  const uptimeSlot = document.getElementById("monitor-icon-uptime");
  if (uptimeSlot && !uptimeSlot.hasChildNodes()) uptimeSlot.appendChild(icon(AppIcons.clock, { size: 12 }));

  const cpuSlot = document.getElementById("monitor-icon-cpu");
  if (cpuSlot && !cpuSlot.hasChildNodes()) cpuSlot.appendChild(icon(AppIcons.cpu, { size: 14 }));

  const ramSlot = document.getElementById("monitor-icon-ram");
  if (ramSlot && !ramSlot.hasChildNodes()) ramSlot.appendChild(icon(AppIcons.database, { size: 14 }));

  const diskSlot = document.getElementById("monitor-icon-disk");
  if (diskSlot && !diskSlot.hasChildNodes()) diskSlot.appendChild(icon(AppIcons.hardDrive, { size: 14 }));

  const netSlot = document.getElementById("monitor-icon-net");
  if (netSlot && !netSlot.hasChildNodes()) netSlot.appendChild(icon(AppIcons.network, { size: 14 }));

  const netDownSlot = document.getElementById("monitor-icon-net-down");
  if (netDownSlot && !netDownSlot.hasChildNodes()) {
    netDownSlot.appendChild(icon(AppIcons.arrowUp, { size: 10, className: "icon-rotate-180" }));
  }

  const netUpSlot = document.getElementById("monitor-icon-net-up");
  if (netUpSlot && !netUpSlot.hasChildNodes()) netUpSlot.appendChild(icon(AppIcons.arrowUp, { size: 10 }));

  const procSlot = document.getElementById("monitor-icon-processes");
  if (procSlot && !procSlot.hasChildNodes()) procSlot.appendChild(icon(AppIcons.crown, { size: 14 }));

  updatePauseButtonVisuals();
}

function updatePauseButtonVisuals() {
  if (monitorBtnPauseText) {
    monitorBtnPauseText.textContent = isMonitorPaused ? "Reanudar" : "Pausar";
  }
  const pauseSlot = document.getElementById("monitor-icon-pause");
  if (pauseSlot) {
    pauseSlot.replaceChildren(icon(isMonitorPaused ? AppIcons.play : AppIcons.pause, { size: 12 }));
  }
  if (btnMonitorPause) {
    if (isMonitorPaused) {
      btnMonitorPause.style.background = "rgba(57, 255, 20, 0.1)";
      btnMonitorPause.style.borderColor = "var(--color-success)";
      btnMonitorPause.style.color = "var(--color-success)";
    } else {
      btnMonitorPause.style.background = "";
      btnMonitorPause.style.borderColor = "";
      btnMonitorPause.style.color = "";
    }
  }
}

function initMonitorTab() {
  initMonitorIcons();

  if (monitorIntervalSelect && !monitorIntervalSelect.dataset.listenerBound) {
    monitorIntervalSelect.addEventListener("change", () => {
      startMonitorInterval();
    });
    monitorIntervalSelect.dataset.listenerBound = "true";
  }

  if (btnMonitorPause && !btnMonitorPause.dataset.listenerBound) {
    btnMonitorPause.addEventListener("click", () => {
      isMonitorPaused = !isMonitorPaused;
      updatePauseButtonVisuals();
    });
    btnMonitorPause.dataset.listenerBound = "true";
  }

  checkMonitorSessionState();
}

function checkMonitorSessionState() {
  const termId = currentActiveTerminalId;
  const activeTerm = termId ? activeTerminals.get(termId) : undefined;

  if (!activeTerm) {
    if (monitorEmpty) monitorEmpty.style.display = "block";
    if (monitorContent) monitorContent.style.display = "none";
    stopMonitorInterval();
  } else {
    if (monitorEmpty) monitorEmpty.style.display = "none";
    if (monitorContent) monitorContent.style.display = "block";
    if (monitorServerNameText) {
      monitorServerNameText.textContent = activeTerm.profileName || activeTerm.profile.name || "Servidor remoto";
    }
    if (monitorTimerId === null) {
      cpuHistory = Array(30).fill(0);
      ramHistory = Array(30).fill(0);
      prevCpuActive = 0;
      prevCpuTotal = 0;
      lastNetRecv = 0;
      lastNetSent = 0;
      lastNetTime = 0;
      void updateMonitor();
      startMonitorInterval();
    }
  }
}

function startMonitorInterval() {
  stopMonitorInterval();
  const ms = monitorIntervalSelect ? parseInt(monitorIntervalSelect.value) : 5000;
  monitorTimerId = setInterval(() => {
    void updateMonitor();
  }, ms);
}

function stopMonitorInterval() {
  if (monitorTimerId !== null) {
    clearInterval(monitorTimerId);
    monitorTimerId = null;
  }
}

async function updateMonitor() {
  if (isMonitorPaused) return;

  const termId = currentActiveTerminalId;
  const activeTerm = termId ? activeTerminals.get(termId) : undefined;
  if (!activeTerm) {
    stopMonitorInterval();
    return;
  }

  try {
    const rawData = await invoke<string>("get_remote_sys_info", { terminalId: activeTerm.id });
    parseAndUpdateMonitorData(rawData);
  } catch (err) {
    console.error("Error al obtener información de monitor:", err);
    if (monitorCpuValue) monitorCpuValue.textContent = "N/A";
    if (monitorRamValue) monitorRamValue.textContent = "N/A";
    if (monitorDiskValue) monitorDiskValue.textContent = "N/A";
  }
}

function formatUptime(uptimeSec: number): string {
  const d = Math.floor(uptimeSec / (3600 * 24));
  const h = Math.floor((uptimeSec % (3600 * 24)) / 3600);
  const m = Math.floor((uptimeSec % 3600) / 60);
  if (d > 0) {
    return `${d}d ${h}h ${m}m`;
  } else if (h > 0) {
    return `${h}h ${m}m`;
  } else {
    return `${m}m`;
  }
}

function formatNetSpeed(bytesPerSec: number): string {
  if (bytesPerSec < 0) bytesPerSec = 0;
  if (bytesPerSec >= 1024 * 1024) {
    return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
  } else if (bytesPerSec >= 1024) {
    return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  } else {
    return `${Math.round(bytesPerSec)} B/s`;
  }
}

function parseAndUpdateMonitorData(raw: string) {
  const lines = raw.split("\n").map(l => l.trim()).filter(Boolean);

  let cpuLine = "";
  let ramLine = "";
  let diskLine = "";
  let uptimeLine = "";
  const netDevLines: string[] = [];
  const psLines: string[] = [];
  let readingPs = false;

  for (const line of lines) {
    // Determine which section we are in
    if (line.startsWith("cpu ")) {
      cpuLine = line;
    } else if (line.startsWith("Mem:")) {
      ramLine = line;
    } else if (line.includes(" /") && (line.startsWith("/") || line.match(/^[a-zA-Z0-9]/)) && !line.includes("%CPU")) {
      diskLine = line;
    } else if (line.match(/^\d+\.\d+\s+\d+\.\d+$/)) {
      uptimeLine = line;
    } else if (line.includes(":")) {
      netDevLines.push(line);
    } else if (line.includes("%CPU") || readingPs) {
      readingPs = true;
      psLines.push(line);
    }
  }

  // Backup uptime parsing if it didn't match the regex perfectly
  if (!uptimeLine) {
    const firstLine = lines[0];
    if (firstLine && firstLine.match(/^\d+(\.\d+)?\s+\d+/)) {
      uptimeLine = firstLine;
    }
  }

  // 1. Process CPU
  if (cpuLine) {
    const parts = cpuLine.split(/\s+/).slice(1).map(Number);
    if (parts.length >= 7) {
      const active = parts[0] + parts[1] + parts[2] + parts[5] + parts[6];
      const total = active + parts[3] + parts[4];

      if (prevCpuTotal > 0) {
        const activeDelta = active - prevCpuActive;
        const totalDelta = total - prevCpuTotal;
        const cpuPercent = totalDelta > 0 ? Math.round((activeDelta * 100) / totalDelta) : 0;
        const clampedCpu = Math.max(0, Math.min(100, cpuPercent));

        cpuHistory.shift();
        cpuHistory.push(clampedCpu);

        if (monitorCpuValue) monitorCpuValue.textContent = `${clampedCpu}%`;
        if (monitorCpuLoad) monitorCpuLoad.textContent = (clampedCpu / 100).toFixed(2);
      }
      prevCpuActive = active;
      prevCpuTotal = total;
    }
  }

  const cores = lines.filter(l => l.match(/^cpu\d+\s+/)).length;
  if (monitorCpuCores) monitorCpuCores.textContent = cores > 0 ? String(cores) : "1";

  // 2. Process RAM
  if (ramLine) {
    const parts = ramLine.split(/\s+/);
    if (parts.length >= 3) {
      const totalBytes = Number(parts[1]);
      const usedBytes = Number(parts[2]);
      
      const ramPercent = totalBytes > 0 ? Math.round((usedBytes * 100) / totalBytes) : 0;
      const clampedRam = Math.max(0, Math.min(100, ramPercent));

      ramHistory.shift();
      ramHistory.push(clampedRam);

      const totalGB = (totalBytes / (1024 * 1024 * 1024)).toFixed(1);
      const usedGB = (usedBytes / (1024 * 1024 * 1024)).toFixed(1);
      const freeGB = ((totalBytes - usedBytes) / (1024 * 1024 * 1024)).toFixed(1);

      if (monitorRamValue) monitorRamValue.textContent = `${clampedRam}%`;
      if (monitorRamDetail) monitorRamDetail.textContent = `${usedGB} GB / ${totalGB} GB`;
      if (monitorRamFree) monitorRamFree.textContent = `${freeGB} GB`;
    }
  }

  // 3. Process Disk
  if (diskLine) {
    const parts = diskLine.split(/\s+/);
    if (parts.length >= 3) {
      const totalBytes = Number(parts[1]);
      const usedBytes = Number(parts[2]);

      const diskPercent = totalBytes > 0 ? Math.round((usedBytes * 100) / totalBytes) : 0;
      const clampedDisk = Math.max(0, Math.min(100, diskPercent));

      const totalGB = (totalBytes / (1024 * 1024 * 1024)).toFixed(1);
      const usedGB = (usedBytes / (1024 * 1024 * 1024)).toFixed(1);

      if (monitorDiskValue) monitorDiskValue.textContent = `${clampedDisk}%`;
      if (monitorDiskFill) monitorDiskFill.style.width = `${clampedDisk}%`;
      if (monitorDiskDetail) monitorDiskDetail.textContent = `${usedGB} GB / ${totalGB} GB`;
    }
  }

  // 4. Process OS & Uptime
  if (monitorOsText) monitorOsText.textContent = "Linux";
  if (uptimeLine) {
    const sec = parseFloat(uptimeLine.split(/\s+/)[0]);
    if (!isNaN(sec) && monitorUptimeText) {
      monitorUptimeText.textContent = formatUptime(sec);
    }
  }

  // 5. Process Network
  let totalRecv = 0;
  let totalSent = 0;
  for (const netLine of netDevLines) {
    const parts = netLine.split(":");
    if (parts.length >= 2) {
      const iface = parts[0].trim();
      if (iface !== "lo") {
        const stats = parts[1].trim().split(/\s+/).map(Number);
        if (stats.length >= 9) {
          totalRecv += stats[0] || 0;
          totalSent += stats[8] || 0;
        }
      }
    }
  }

  const now = Date.now();
  if (lastNetTime > 0) {
    const timeDeltaSec = (now - lastNetTime) / 1000;
    if (timeDeltaSec > 0) {
      const recvSpeed = (totalRecv - lastNetRecv) / timeDeltaSec;
      const sentSpeed = (totalSent - lastNetSent) / timeDeltaSec;

      if (monitorNetDown) monitorNetDown.textContent = formatNetSpeed(recvSpeed);
      if (monitorNetUp) monitorNetUp.textContent = formatNetSpeed(sentSpeed);
    }
  }
  lastNetRecv = totalRecv;
  lastNetSent = totalSent;
  lastNetTime = now;

  // 6. Process Top Processes
  if (monitorProcessesList && psLines.length > 1) {
    monitorProcessesList.innerHTML = "";
    // Skip header line
    const rows = psLines.slice(1).map(r => r.trim()).filter(Boolean);
    let count = 0;
    for (const r of rows) {
      const cols = r.split(/\s+/);
      if (cols.length >= 3) {
        const cpuVal = cols[0];
        const memVal = cols[1];
        const cmdVal = cols.slice(2).join(" ");
        
        const rowEl = document.createElement("div");
        rowEl.className = "proc-row";

        const nameEl = document.createElement("span");
        nameEl.className = "proc-name";
        nameEl.textContent = cmdVal;
        nameEl.setAttribute("title", cmdVal);

        const badgesEl = document.createElement("div");
        badgesEl.className = "proc-stat-badges";

        const cpuBadge = document.createElement("span");
        cpuBadge.className = "proc-badge";
        cpuBadge.textContent = `${cpuVal}% C`;

        const memBadge = document.createElement("span");
        memBadge.className = "proc-badge cyan";
        memBadge.textContent = `${memVal}% M`;

        badgesEl.appendChild(cpuBadge);
        badgesEl.appendChild(memBadge);
        rowEl.appendChild(nameEl);
        rowEl.appendChild(badgesEl);

        monitorProcessesList.appendChild(rowEl);
        count++;
        if (count >= 3) break; // Only top 3
      }
    }
    if (count === 0) {
      monitorProcessesList.innerHTML = '<div class="profile-list-empty">No hay procesos activos.</div>';
    }
  }

  drawMonitorSparkline("canvas-cpu", cpuHistory, "rgba(255, 105, 180, 1)");
  drawMonitorSparkline("canvas-ram", ramHistory, "rgba(0, 255, 255, 1)");
}

function drawMonitorSparkline(canvasId: string, history: number[], color: string) {
  const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
  if (!canvas) return;

  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * window.devicePixelRatio;
  canvas.height = rect.height * window.devicePixelRatio;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
  ctx.clearRect(0, 0, rect.width, rect.height);

  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  ctx.shadowColor = color;
  ctx.shadowBlur = 4;

  const gradient = ctx.createLinearGradient(0, 0, 0, rect.height);
  const colorRgb = color.replace("1)", "0.15)");
  gradient.addColorStop(0, colorRgb);
  gradient.addColorStop(1, "rgba(0,0,0,0)");

  ctx.beginPath();
  const step = rect.width / (history.length - 1);
  history.forEach((val, idx) => {
    const x = idx * step;
    const y = rect.height - (val / 100) * (rect.height - 10) - 5;
    if (idx === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  });
  ctx.stroke();

  ctx.shadowBlur = 0;

  ctx.lineTo(rect.width, rect.height);
  ctx.lineTo(0, rect.height);
  ctx.closePath();
  ctx.fillStyle = gradient;
  ctx.fill();
}

