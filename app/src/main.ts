// --- NekoSSH Frontend Controller (Cyber-Sakura Estética) ---
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

// --- Interfaces ---
interface ConnectionProfile {
  id?: number;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_type: 'password' | 'key';
  password?: string;
  key_path?: string;
  passphrase?: string;
  keepalive: number;
  tunnel_type: 'none' | 'local' | 'dynamic';
  tunnel_local_port?: number;
  tunnel_dest?: string;
}

interface ActiveTerminal {
  id: string;
  profileName: string;
  term: Terminal;
  fitAddon: FitAddon;
  panelEl: HTMLElement;
  tabEl: HTMLElement;
  isConnected: boolean;
}

interface SshEventPayload {
  terminal_id: string;
  data: string;
}

interface SshClosedPayload {
  terminal_id: string;
  error?: string;
}

// --- State Management ---
let currentProfiles: ConnectionProfile[] = [];
let activeProfileId: number | null = null;

const activeTerminals = new Map<string, ActiveTerminal>();
let currentActiveTerminalId: string | null = null;

// --- DOM Elements ---
let bgImageLayer: HTMLElement | null = null;
let configBgUrlInput: HTMLInputElement | null = null;
let configBgOpacityInput: HTMLInputElement | null = null;
let opacityValLabel: HTMLElement | null = null;
let btnApplyBg: HTMLButtonElement | null = null;

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
let profKeyPathInput: HTMLInputElement | null = null;
let profPassphraseInput: HTMLInputElement | null = null;
let profKeepaliveInput: HTMLInputElement | null = null;

let tunTypeSelect: HTMLSelectElement | null = null;
let tunLocalPortInput: HTMLInputElement | null = null;
let tunDestInput: HTMLInputElement | null = null;

let btnNewProfile: HTMLButtonElement | null = null;
let btnCancelProfile: HTMLButtonElement | null = null;

let profileListContainer: HTMLElement | null = null;

// Tabs sidebar
let tabBtnServers: HTMLButtonElement | null = null;
let tabBtnFiles: HTMLButtonElement | null = null;
let panelServers: HTMLElement | null = null;
let panelFiles: HTMLElement | null = null;

// Terminal layout elements
let mainDisplayArea: HTMLElement | null = null;
let terminalTabsList: HTMLElement | null = null;
let btnCloseAllTerminals: HTMLButtonElement | null = null;

// --- Initialize App Settings (Background & Opacity) ---
function initSettings() {
  bgImageLayer = document.getElementById("bg-image-layer");
  configBgUrlInput = document.getElementById("config-bg-url") as HTMLInputElement;
  configBgOpacityInput = document.getElementById("config-bg-opacity") as HTMLInputElement;
  opacityValLabel = document.getElementById("opacity-val");
  btnApplyBg = document.getElementById("btn-apply-bg") as HTMLButtonElement;

  // Cargar de localStorage
  const savedBgUrl = localStorage.getItem("nekossh-bg-url") || "";
  const savedOpacity = localStorage.getItem("nekossh-bg-opacity") || "0.30";

  if (configBgUrlInput) configBgUrlInput.value = savedBgUrl;
  if (configBgOpacityInput) configBgOpacityInput.value = savedOpacity;
  if (opacityValLabel) opacityValLabel.textContent = parseFloat(savedOpacity).toFixed(2);

  applyBackgroundSettings(savedBgUrl, parseFloat(savedOpacity));

  // Event Listeners para Fondo
  btnApplyBg?.addEventListener("click", () => {
    const url = configBgUrlInput?.value.trim() || "";
    const opacity = parseFloat(configBgOpacityInput?.value || "0.30");
    localStorage.setItem("nekossh-bg-url", url);
    applyBackgroundSettings(url, opacity);
  });

  configBgOpacityInput?.addEventListener("input", (e) => {
    const target = e.target as HTMLInputElement;
    const opacity = parseFloat(target.value);
    if (opacityValLabel) opacityValLabel.textContent = opacity.toFixed(2);
    localStorage.setItem("nekossh-bg-opacity", opacity.toString());
    applyBackgroundSettings(configBgUrlInput?.value.trim() || "", opacity);
  });
}

function applyBackgroundSettings(url: string, opacity: number) {
  if (bgImageLayer) {
    if (url) {
      bgImageLayer.style.backgroundImage = `url('${url}')`;
      bgImageLayer.style.opacity = opacity.toString();
      bgImageLayer.style.zIndex = "-1";
    } else {
      bgImageLayer.style.backgroundImage = "";
      bgImageLayer.style.opacity = "0";
    }
  }
}

// --- Initialize Navigation Tabs ---
function initTabs() {
  tabBtnServers = document.getElementById("tab-btn-servers") as HTMLButtonElement;
  tabBtnFiles = document.getElementById("tab-btn-files") as HTMLButtonElement;
  panelServers = document.getElementById("panel-servers");
  panelFiles = document.getElementById("panel-files");

  tabBtnServers?.addEventListener("click", () => {
    tabBtnServers?.classList.add("active");
    tabBtnFiles?.classList.remove("active");
    panelServers?.classList.add("active");
    panelFiles?.classList.remove("active");
  });

  tabBtnFiles?.addEventListener("click", () => {
    tabBtnFiles?.classList.add("active");
    tabBtnServers?.classList.remove("active");
    panelFiles?.classList.add("active");
    panelServers?.classList.remove("active");
  });
}

// --- DOM Loaded Listener ---
window.addEventListener("DOMContentLoaded", () => {
  initSettings();
  initTabs();
  
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
  profKeyPathInput = document.getElementById("prof-key-path") as HTMLInputElement;
  profPassphraseInput = document.getElementById("prof-passphrase") as HTMLInputElement;
  profKeepaliveInput = document.getElementById("prof-keepalive") as HTMLInputElement;

  tunTypeSelect = document.getElementById("tun-type") as HTMLSelectElement;
  tunLocalPortInput = document.getElementById("tun-local-port") as HTMLInputElement;
  tunDestInput = document.getElementById("tun-dest") as HTMLInputElement;

  btnNewProfile = document.getElementById("btn-new-profile") as HTMLButtonElement;
  btnCancelProfile = document.getElementById("btn-cancel-profile") as HTMLButtonElement;
  profileListContainer = document.getElementById("profile-list");

  // Mostrar modal de creación
  btnNewProfile?.addEventListener("click", () => {
    openProfileModal();
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
  activeTerminals.forEach(t => {
    if (t.isConnected) {
      setTimeout(() => {
        t.fitAddon.fit();
        invoke("resize_ssh_pty", {
          terminalId: t.id,
          cols: t.term.cols,
          rows: t.term.rows
        }).catch(err => console.error("Error al redimensionar PTY:", err));
      }, 50);
    }
  });
});

// --- Setup SSH Event Listeners ---
function setupSshEventListeners() {
  // Escuchar por conexión establecida
  listen<SshEventPayload>("ssh-connected", (event) => {
    const termId = event.payload.terminal_id;
    const activeTerm = activeTerminals.get(termId);
    if (activeTerm) {
      activeTerm.isConnected = true;
      const statusIndicator = activeTerm.panelEl.querySelector(".status-dot");
      const statusText = activeTerm.panelEl.querySelector(".terminal-status-text");
      
      if (statusIndicator) {
        statusIndicator.className = "status-dot connected";
      }
      if (statusText) {
        statusText.textContent = "Conectado al servidor remoto";
      }

      // Limpiar terminal de mensajes previos
      activeTerm.term.clear();
      activeTerm.term.focus();
      
      // Ajustar dimensiones PTY
      setTimeout(() => {
        activeTerm.fitAddon.fit();
        invoke("resize_ssh_pty", {
          terminalId: termId,
          cols: activeTerm.term.cols,
          rows: activeTerm.term.rows
        }).catch(err => console.error("Error al redimensionar PTY inicial:", err));
      }, 100);
    }
  });

  // Escuchar por salida de la consola (stdout)
  listen<SshEventPayload>("ssh-stdout", (event) => {
    const termId = event.payload.terminal_id;
    const data = event.payload.data;
    const activeTerm = activeTerminals.get(termId);
    if (activeTerm) {
      activeTerm.term.write(data);
    }
  });

  // Escuchar por error en la conexión
  listen<SshClosedPayload>("ssh-error", (event) => {
    const termId = event.payload.terminal_id;
    const errorMsg = event.payload.error || "Error desconocido";
    const activeTerm = activeTerminals.get(termId);
    if (activeTerm) {
      activeTerm.term.write(`\r\n\x1b[31;1m[ERROR] ${errorMsg}\x1b[0m\r\n`);
      const statusIndicator = activeTerm.panelEl.querySelector(".status-dot");
      const statusText = activeTerm.panelEl.querySelector(".terminal-status-text");
      
      if (statusIndicator) {
        statusIndicator.className = "status-dot error";
      }
      if (statusText) {
        statusText.textContent = "Error de Conexión";
      }
      activeTerm.isConnected = false;
    }
  });

  // Escuchar por sesión cerrada
  listen<SshClosedPayload>("ssh-closed", (event) => {
    const termId = event.payload.terminal_id;
    const activeTerm = activeTerminals.get(termId);
    if (activeTerm) {
      if (activeTerm.isConnected) {
        activeTerm.term.write(`\r\n\x1b[33;1m[Conexión Cerrada por el Servidor]\x1b[0m\r\n`);
      }
      const statusIndicator = activeTerm.panelEl.querySelector(".status-dot");
      const statusText = activeTerm.panelEl.querySelector(".terminal-status-text");
      
      if (statusIndicator) {
        statusIndicator.className = "status-dot";
      }
      if (statusText) {
        statusText.textContent = "Desconectado";
      }
      activeTerm.isConnected = false;
    }
  });
}

// --- Modal Helper Functions ---
function openProfileModal(profile?: ConnectionProfile) {
  if (!profileModal || !profileForm || !modalTitle) return;

  profileForm.reset();

  if (profile) {
    modalTitle.textContent = "Editar Perfil de Servidor";
    if (profileIdInput) profileIdInput.value = profile.id?.toString() || "";
    if (profNameInput) profNameInput.value = profile.name;
    if (profHostInput) profHostInput.value = profile.host;
    if (profPortInput) profPortInput.value = profile.port.toString();
    if (profUsernameInput) profUsernameInput.value = profile.username;
    if (profAuthTypeSelect) profAuthTypeSelect.value = profile.auth_type;
    if (profPasswordInput) profPasswordInput.value = profile.password || "";
    if (profKeyPathInput) profKeyPathInput.value = profile.key_path || "";
    if (profPassphraseInput) profPassphraseInput.value = profile.passphrase || "";
    if (profKeepaliveInput) profKeepaliveInput.value = profile.keepalive.toString();
    
    if (tunTypeSelect) tunTypeSelect.value = profile.tunnel_type;
    if (tunLocalPortInput) tunLocalPortInput.value = profile.tunnel_local_port?.toString() || "";
    if (tunDestInput) tunDestInput.value = profile.tunnel_dest || "";

    toggleAuthFields(profile.auth_type);
    toggleTunnelFields(profile.tunnel_type);
  } else {
    modalTitle.textContent = "Nuevo Perfil de Servidor";
    if (profileIdInput) profileIdInput.value = "";
    toggleAuthFields('password');
    toggleTunnelFields('none');
  }

  profileModal.classList.add("active");
}

function closeProfileModal() {
  profileModal?.classList.remove("active");
}

function toggleAuthFields(authType: 'password' | 'key') {
  const pwdGroup = document.getElementById("auth-password-group");
  const keyGroup = document.getElementById("auth-key-group");

  if (authType === 'password') {
    if (pwdGroup) pwdGroup.style.display = "flex";
    if (keyGroup) keyGroup.style.display = "none";
    if (profKeyPathInput) profKeyPathInput.required = false;
  } else {
    if (pwdGroup) pwdGroup.style.display = "none";
    if (keyGroup) keyGroup.style.display = "flex";
    if (profKeyPathInput) profKeyPathInput.required = true;
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
    currentProfiles = await invoke<ConnectionProfile[]>("get_profiles");
    renderProfileList();
  } catch (err) {
    console.error("Error al cargar perfiles:", err);
    renderProfileList();
  }
}

async function saveProfile() {
  const idStr = profileIdInput?.value;
  const profile: ConnectionProfile = {
    name: profNameInput?.value || "",
    host: profHostInput?.value || "",
    port: parseInt(profPortInput?.value || "22"),
    username: profUsernameInput?.value || "",
    auth_type: (profAuthTypeSelect?.value as 'password' | 'key') || 'password',
    keepalive: parseInt(profKeepaliveInput?.value || "60"),
    tunnel_type: (tunTypeSelect?.value as 'none' | 'local' | 'dynamic') || 'none'
  };

  if (idStr) {
    profile.id = parseInt(idStr);
  }

  if (profile.auth_type === 'password') {
    profile.password = profPasswordInput?.value || "";
  } else {
    profile.key_path = profKeyPathInput?.value || "";
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
    alert("Error al guardar el perfil: " + err);
  }
}

async function deleteProfile(id: number, event: Event) {
  event.stopPropagation();
  if (!confirm("¿Está seguro de que desea eliminar este perfil?")) return;

  try {
    await invoke("delete_profile", { id });
    await loadProfiles();
  } catch (err) {
    console.error("Error al eliminar perfil:", err);
    alert("Error al eliminar el perfil: " + err);
  }
}

// --- Render Helper ---
function renderProfileList() {
  const container = profileListContainer;
  if (!container) return;

  if (currentProfiles.length === 0) {
    container.innerHTML = `<div class="profile-list-empty">No hay perfiles de conexión.</div>`;
    return;
  }

  container.innerHTML = "";
  currentProfiles.forEach(prof => {
    const item = document.createElement("div");
    item.className = "profile-item";
    if (prof.id === activeProfileId) item.classList.add("active");

    const detailText = prof.tunnel_type !== 'none' 
      ? `SSH (Túnel: ${prof.tunnel_type})` 
      : `SSH (${prof.auth_type === 'password' ? 'Contraseña' : 'Llave'})`;

    item.innerHTML = `
      <div class="profile-item-header">
        <span class="profile-item-name">${escapeHtml(prof.name)}</span>
        <div class="profile-item-actions">
          <button class="btn-icon btn-edit" title="Editar">✏️</button>
          <button class="btn-icon btn-delete" title="Eliminar">🗑️</button>
        </div>
      </div>
      <div class="profile-item-host">${escapeHtml(prof.username)}@${escapeHtml(prof.host)}:${prof.port}</div>
      <div class="profile-item-details">${detailText}</div>
    `;

    // Click en item para conectar
    item.addEventListener("click", () => {
      selectProfile(prof.id || null);
    });

    // Editar
    item.querySelector(".btn-edit")?.addEventListener("click", (e) => {
      e.stopPropagation();
      openProfileModal(prof);
    });

    // Eliminar
    item.querySelector(".btn-delete")?.addEventListener("click", (e) => {
      if (prof.id !== undefined) deleteProfile(prof.id, e);
    });

    container.appendChild(item);
  });
}

function selectProfile(id: number | null) {
  activeProfileId = id;
  const container = profileListContainer;
  const items = container?.querySelectorAll(".profile-item");
  items?.forEach((item, index) => {
    const prof = currentProfiles[index];
    if (prof.id === id) {
      item.classList.add("active");
    } else {
      item.classList.remove("active");
    }
  });

  // Disparar conexión SSH
  if (id !== null) {
    const selectedProfile = currentProfiles.find(p => p.id === id);
    if (selectedProfile) {
      startNewSshConnection(selectedProfile);
    }
  }
}

// --- SSH Connection Execution ---
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
  tabEl.innerHTML = `
    <span class="term-tab-title">${escapeHtml(profile.name)}</span>
    <button class="term-tab-close" title="Cerrar Terminal">×</button>
  `;
  
  tabEl.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (target.classList.contains("term-tab-close")) {
      closeTerminalSession(terminalId);
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
        <div class="status-dot"></div>
        <span class="terminal-status-text">Conectando...</span>
      </div>
      <div class="terminal-info-text">${escapeHtml(profile.username)}@${escapeHtml(profile.host)}:${profile.port}</div>
    </div>
    <div class="terminal-canvas-container" id="canvas-${terminalId}"></div>
  `;

  mainDisplayArea?.appendChild(panelEl);

  // 3. Inicializar xterm.js (tipografía desde tokens CSS en runtime)
  const canvasContainer = panelEl.querySelector(`.terminal-canvas-container`) as HTMLElement;
  const monoFontFamily =
    getComputedStyle(document.documentElement).getPropertyValue("--font-mono").trim() ||
    "monospace";
  const term = new Terminal({
    cursorBlink: true,
    cursorStyle: "block",
    theme: {
      background: "#080409",
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
    fontFamily: monoFontFamily,
    fontSize: 14,
  });

  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  term.open(canvasContainer);
  fitAddon.fit();

  term.write("\x1b[35;1m[Iniciando sesión SSH en NekoSSH...]\x1b[0m\r\n");

  // Registrar input del usuario en xterm hacia el backend
  term.onData((data) => {
    invoke("write_ssh_input", { terminalId, data })
      .catch(err => console.error("Error al escribir input SSH:", err));
  });

  // Registrar resize del emulador hacia el backend
  term.onResize((size) => {
    invoke("resize_ssh_pty", {
      terminalId,
      cols: size.cols,
      rows: size.rows
    }).catch(err => console.error("Error al redimensionar PTY:", err));
  });

  // Guardar estado de la terminal activa
  const activeTerm: ActiveTerminal = {
    id: terminalId,
    profileName: profile.name,
    term,
    fitAddon,
    panelEl,
    tabEl,
    isConnected: false
  };

  activeTerminals.set(terminalId, activeTerm);

  // Seleccionar la terminal recién creada
  switchActiveTerminal(terminalId);

  // 4. Iniciar Conexión SSH en backend Rust
  invoke("start_ssh_session", {
    terminalId,
    host: profile.host,
    port: profile.port,
    username: profile.username,
    authType: profile.auth_type,
    password: profile.password || null,
    keyPath: profile.key_path || null,
    passphrase: profile.passphrase || null
  }).catch(err => {
    console.error("Error al iniciar sesión SSH:", err);
    term.write(`\r\n\x1b[31;1m[ERROR] No se pudo invocar el backend: ${err}\x1b[0m\r\n`);
  });
}

function switchActiveTerminal(terminalId: string) {
  currentActiveTerminalId = terminalId;
  console.log("Terminal activa cambiada a:", currentActiveTerminalId);

  activeTerminals.forEach((term, id) => {
    if (id === terminalId) {
      term.tabEl.classList.add("active");
      term.panelEl.classList.add("active");
      term.term.focus();
      // Pequeño delay para asegurar render correcto
      setTimeout(() => {
        term.fitAddon.fit();
      }, 50);
    } else {
      term.tabEl.classList.remove("active");
      term.panelEl.classList.remove("active");
    }
  });
}

async function closeTerminalSession(terminalId: string) {
  const activeTerm = activeTerminals.get(terminalId);
  if (!activeTerm) return;

  // Invocar cierre nativo en backend
  try {
    await invoke("close_ssh_session", { terminalId });
  } catch (err) {
    console.error("Error al cerrar sesión SSH en backend:", err);
  }

  // Destruir terminal y remover elementos DOM
  activeTerm.term.dispose();
  activeTerm.tabEl.remove();
  activeTerm.panelEl.remove();

  activeTerminals.delete(terminalId);

  // Seleccionar otra terminal activa si hay alguna
  if (activeTerminals.size > 0) {
    const nextKey = activeTerminals.keys().next().value;
    if (nextKey) switchActiveTerminal(nextKey);
  } else {
    // Volver a la pantalla de bienvenida
    currentActiveTerminalId = null;
    if (btnCloseAllTerminals) btnCloseAllTerminals.style.display = "none";
    
    const welcomeScreen = mainDisplayArea?.querySelector(".welcome-screen");
    if (welcomeScreen) {
      (welcomeScreen as HTMLElement).style.display = "flex";
    }
  }
}

function closeAllTerminals() {
  if (activeTerminals.size === 0) return;
  if (!confirm("¿Está seguro de que desea cerrar todas las terminales activas?")) return;

  const ids = Array.from(activeTerminals.keys());
  ids.forEach(id => {
    closeTerminalSession(id);
  });
}

// --- Utils ---
function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.innerText = str;
  return div.innerHTML;
}
