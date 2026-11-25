// Controlador de la ventana nativa independiente neko-ai (Copiloto Conversacional de 4 Fases)
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writeText as tauriWriteClipboard } from "@tauri-apps/plugin-clipboard-manager";
import { AppIcons, icon, setButtonIcon } from "../icons";
import { renderSafeMarkdown } from "./markdown-lite";
import {
  buildCommandQuestionText,
  buildEmptyHeroState,
  createEmptySessionUiState,
  escapeHtmlText,
  extractReadableStreamPreview,
  formatListenChipLabel,
  formatPlanSummaryMarkdown,
  formatQuotedSelectionPrompt,
  formatRiskLabel,
  formatStageBadgeText,
  getOrCreateSessionUiState,
  planFirstTaskToPendingAction,
  resolveTargetShell,
  sanitizeEditedCommand,
  shouldAutoRunReadOnlyStep,
  stepPromptHistory,
  type ActiveShellsSnapshot,
  type AgentResponseKind,
  type AiModelDescriptor,
  type AiPreferences,
  type ConversationalPhaseTag,
  type DriverHealthStatus,
  type HarnessPhase,
  type RiskLevel,
  type SentinelPurpose,
  type SessionUiState,
  type ShellSelectionState,
} from "./copilot-state";

interface TurnExecutionResult {
  phase: HarnessPhase;
  outcome: {
    response: AgentResponseKind;
    provider_conversation_id: string | null;
    raw_model_used: string | null;
  };
  execution_mode: "strict" | "flexible";
  auto_approved_read_only: boolean;
}

interface SentinelCompletion {
  terminal_id: string;
  nonce: string;
  exit_code: number;
  captured_output: string;
  purpose: SentinelPurpose;
  updated_phase?: HarnessPhase | null;
}

interface RiskAuditResultDto {
  computed_risk: RiskLevel;
  is_mutating: boolean;
}

interface FeedEntry {
  kind: "user" | "assistant" | "error" | "console";
  title: string;
  body: string;
  code?: string;
  exitCode?: number;
  expanded?: boolean;
  phaseLabel?: string;
}

interface AiFeedEntryDto {
  kind: "user" | "assistant" | "error" | "console";
  title: string;
  body: string;
  code: string | null;
  exit_code: number | null;
  expanded: boolean | null;
  phase_label: string | null;
}

interface AiPendingActionDto {
  purpose: string;
  original_command: string;
  draft_command: string;
  is_editing: boolean;
  risk_level: RiskLevel;
  audit_warning: string | null;
  backup_strategy: string | null;
  rollback_cmd: string | null;
}

interface AiFailedContextDto {
  command: string;
  exit_code: number;
  output: string;
}

interface AiRollbackDto {
  source_command: string;
  rollback_cmd: string;
  label: string;
}

interface AiUiSessionStateDto {
  pending_action: AiPendingActionDto | null;
  last_failed: AiFailedContextDto | null;
  last_rollback: AiRollbackDto | null;
}

let shellSelection: ShellSelectionState = { targetTerminalId: null, pinned: false };
let shellsSnapshot: ActiveShellsSnapshot = { shells: [], focused_terminal_id: null };
let currentPrefs: AiPreferences | null = null;
let currentPhase: HarnessPhase = { type: "idle" };
let currentPhaseTag: ConversationalPhaseTag | null = null;
let alwaysOnTop = false;
let isBusyTurn = false;
let autoStepsUsed = 0;

const sessionUiByTerminal = new Map<string, SessionUiState>();
const feedBySession = new Map<string, FeedEntry[]>();
const promptHistoryBySession = new Map<string, string[]>();
const promptHistoryCursorBySession = new Map<string, number>();
const promptDraftBySession = new Map<string, string>();
let rawProgressStreamBuffer = "";

let awaitingSentinel: {
  terminalId: string;
  nonce: string;
  command: string;
  kind: "auto_read" | "authorized" | "rollback";
  rollbackCmd: string | null;
  waitingHint: string | null;
} | null = null;

let riskAuditTimer: ReturnType<typeof setTimeout> | null = null;
let waitingHintTimer: ReturnType<typeof setTimeout> | null = null;

function clearWaitingHintTimer() {
  if (waitingHintTimer) {
    clearTimeout(waitingHintTimer);
    waitingHintTimer = null;
  }
}

function armWaitingHintFallback(terminalId: string) {
  clearWaitingHintTimer();
  waitingHintTimer = setTimeout(() => {
    if (awaitingSentinel && awaitingSentinel.terminalId === terminalId && !awaitingSentinel.waitingHint) {
      awaitingSentinel.waitingHint =
        "El comando sigue en ejecución. Si solicita contraseña, confirmación [y/N] o usas shell fish, respóndelo en la terminal o pulsa Ctrl+C.";
      renderFeed();
    }
  }, 10_000);
}

function syncSessionUiToBackend(terminalId: string) {
  const s = getOrCreateSessionUiState(sessionUiByTerminal, terminalId);
  const dto: AiUiSessionStateDto = {
    pending_action: s.pendingAction
      ? {
          purpose: s.pendingAction.purpose,
          original_command: s.pendingAction.originalCommand,
          draft_command: s.pendingAction.draftCommand,
          is_editing: s.pendingAction.isEditing,
          risk_level: s.pendingAction.riskLevel,
          audit_warning: s.pendingAction.auditWarning,
          backup_strategy: s.pendingAction.backupStrategy,
          rollback_cmd: s.pendingAction.rollbackCmd,
        }
      : null,
    last_failed: s.lastFailedContext
      ? {
          command: s.lastFailedContext.command,
          exit_code: s.lastFailedContext.exitCode,
          output: s.lastFailedContext.output,
        }
      : null,
    last_rollback: s.lastRollback
      ? {
          source_command: s.lastRollback.sourceCommand,
          rollback_cmd: s.lastRollback.rollbackCmd,
          label: s.lastRollback.label,
        }
      : null,
  };
  void invoke("save_ai_session_ui_state_cmd", {
    sessionId: terminalId,
    uiState: dto,
  }).catch(() => {});
}

// Elementos DOM
const shellSelect = document.getElementById("copilot-shell-select") as HTMLSelectElement;
const modelSelect = document.getElementById("copilot-model-select") as HTMLSelectElement;
const btnPinShell = document.getElementById("btn-pin-shell") as HTMLButtonElement;
const btnClearChat = document.getElementById("btn-clear-chat") as HTMLButtonElement;
const btnAlwaysOnTop = document.getElementById("btn-always-on-top") as HTMLButtonElement;
const btnToggleSettings = document.getElementById("btn-toggle-settings") as HTMLButtonElement;
const btnToggleListen = document.getElementById("btn-toggle-listen") as HTMLButtonElement;
const listenLabel = document.getElementById("listen-label") as HTMLSpanElement;
const btnToggleMode = document.getElementById("btn-toggle-mode") as HTMLButtonElement;
const modeLabel = document.getElementById("mode-label") as HTMLSpanElement;
const btnRefreshModels = document.getElementById("btn-refresh-models") as HTMLButtonElement;
const feedEl = document.getElementById("copilot-feed") as HTMLElement;
const thinkingBox = document.getElementById("copilot-thinking") as HTMLElement;
const thinkingDriverName = document.getElementById("thinking-driver-name") as HTMLElement;
const progressStreamEl = document.getElementById("copilot-progress-stream") as HTMLElement;
const btnCancelTurn = document.getElementById("btn-cancel-turn") as HTMLButtonElement;
const chatForm = document.getElementById("copilot-chat-form") as HTMLFormElement;
const promptInput = document.getElementById("copilot-prompt-input") as HTMLTextAreaElement;
const btnSendPrompt = document.getElementById("btn-send-prompt") as HTMLButtonElement;
const workspaceEl = document.getElementById("copilot-workspace") as HTMLElement;
const settingsViewEl = document.getElementById("copilot-settings-view") as HTMLElement;

// Ajustes DOM
const btnCloseSettings = document.getElementById("btn-close-settings") as HTMLButtonElement;
const btnProbeHealth = document.getElementById("btn-probe-health") as HTMLButtonElement;
const settingsDriverId = document.getElementById("settings-driver-id") as HTMLElement;
const settingsHealthMsg = document.getElementById("settings-health-msg") as HTMLElement;
const settingsForm = document.getElementById("copilot-settings-form") as HTMLFormElement;
const cfgAgyPath = document.getElementById("cfg-agy-path") as HTMLInputElement;
const cfgContextLines = document.getElementById("cfg-context-lines") as HTMLInputElement;
const cfgOperatorPersona = document.getElementById("cfg-operator-persona") as HTMLTextAreaElement;
const cfgInfraRules = document.getElementById("cfg-infra-rules") as HTMLTextAreaElement;
const btnResetSettings = document.getElementById("btn-reset-settings") as HTMLButtonElement;
const settingsSaveStatus = document.getElementById("settings-save-status") as HTMLElement;

async function copyToClipboard(text: string): Promise<void> {
  try {
    await tauriWriteClipboard(text);
    return;
  } catch {
    // Fallback WebView estándar
  }
  await navigator.clipboard?.writeText(text).catch(() => {});
}

function applyThemeFromStorage(themeOverride?: string) {
  const themeName = themeOverride || localStorage.getItem("nekossh-theme") || "nekossh";
  document.documentElement.dataset.theme = themeName;
}

function getSessionFeed(sid: string): FeedEntry[] {
  let list = feedBySession.get(sid);
  if (!list) {
    list = [];
    feedBySession.set(sid, list);
  }
  return list;
}

function pushFeed(sid: string, entry: FeedEntry) {
  getSessionFeed(sid).push(entry);
  const dto: AiFeedEntryDto = {
    kind: entry.kind,
    title: entry.title,
    body: entry.body,
    code: entry.code ?? null,
    exit_code: entry.exitCode ?? null,
    expanded: entry.expanded ?? null,
    phase_label: entry.phaseLabel ?? null,
  };
  void invoke("append_ai_feed_cmd", { sessionId: sid, entry: dto }).catch(() => {});
  renderFeed();
}

function getActiveTargetShell() {
  if (!shellSelection.targetTerminalId) return null;
  return (
    shellsSnapshot.shells.find((s) => s.terminal_id === shellSelection.targetTerminalId) ?? null
  );
}

function initStaticIcons() {
  const brandIcon = document.getElementById("copilot-brand-icon");
  if (brandIcon) brandIcon.replaceChildren(icon(AppIcons.sparkles, { size: 15 }));

  const sendIcon = document.getElementById("send-icon");
  if (sendIcon) sendIcon.replaceChildren(icon(AppIcons.send, { size: 13 }));

  const resetIcon = document.getElementById("reset-settings-icon");
  if (resetIcon) resetIcon.replaceChildren(icon(AppIcons.rotateCcw, { size: 14 }));

  setButtonIcon(btnPinShell, AppIcons.pin, { size: 13 });
  setButtonIcon(btnClearChat, AppIcons.trash2, { size: 14 });
  setButtonIcon(btnAlwaysOnTop, AppIcons.pin, { size: 14 });
  setButtonIcon(btnToggleSettings, AppIcons.settings, { size: 14 });
  setButtonIcon(btnRefreshModels, AppIcons.refreshCw, { size: 13 });
}

function renderToolbar() {
  shellSelect.replaceChildren();
  if (shellsSnapshot.shells.length === 0) {
    const opt = document.createElement("option");
    opt.value = "";
    opt.textContent = "Sin sesiones SSH activas";
    shellSelect.appendChild(opt);
    shellSelect.disabled = true;
  } else {
    shellSelect.disabled = Boolean(awaitingSentinel);
    for (const sh of shellsSnapshot.shells) {
      const opt = document.createElement("option");
      opt.value = sh.terminal_id;
      const statusTag = sh.is_connected ? "" : " (desconectado)";
      opt.textContent = `${sh.profile_name} · ${sh.label}${statusTag}`;
      opt.selected = sh.terminal_id === shellSelection.targetTerminalId;
      shellSelect.appendChild(opt);
    }
  }

  btnPinShell.disabled = Boolean(awaitingSentinel);
  btnPinShell.classList.toggle("is-pinned", shellSelection.pinned);
  btnPinShell.setAttribute("aria-pressed", shellSelection.pinned ? "true" : "false");
  btnPinShell.title = shellSelection.pinned
    ? "Shell fijado (clic para volver a seguir foco)"
    : "Seguir foco actual (clic para fijar este shell)";
  btnPinShell.setAttribute("aria-label", btnPinShell.title);

  const target = getActiveTargetShell();
  shellSelect.title = target?.current_cwd ? `Directorio actual (OSC 7): ${target.current_cwd}` : "";
  btnClearChat.disabled = !target || isBusyTurn || Boolean(awaitingSentinel);

  const isListening = Boolean(target?.is_listening);
  btnToggleListen.disabled = !target;
  btnToggleListen.classList.toggle("is-listening", isListening);
  btnToggleListen.setAttribute("aria-pressed", isListening ? "true" : "false");
  listenLabel.textContent = formatListenChipLabel(isListening, target?.captured_lines ?? 0);
  const listenIconEl = document.getElementById("listen-icon");
  if (listenIconEl) {
    listenIconEl.replaceChildren(
      icon(isListening ? AppIcons.radio : AppIcons.lock, { size: 12 }),
    );
  }

  const mode = currentPrefs?.execution_mode ?? "strict";
  const isFlex = mode === "flexible";
  btnToggleMode.classList.toggle("is-flexible", isFlex);
  modeLabel.textContent = isFlex ? "Flexible" : "Estricto";
  const modeIconEl = document.getElementById("mode-icon");
  if (modeIconEl) {
    modeIconEl.replaceChildren(icon(isFlex ? AppIcons.zap : AppIcons.shield, { size: 12 }));
  }

  if (currentPrefs) {
    thinkingDriverName.textContent = currentPrefs.active_driver || "agy";
  }

  const canSubmit = Boolean(target && target.is_connected && !isBusyTurn && !awaitingSentinel);
  promptInput.disabled = !canSubmit;
  btnSendPrompt.disabled = !canSubmit;
}

function scheduleLiveRiskAudit(
  terminalId: string,
  commandText: string,
  badgeEl: HTMLElement,
) {
  if (riskAuditTimer) clearTimeout(riskAuditTimer);
  riskAuditTimer = setTimeout(async () => {
    const clean = sanitizeEditedCommand(commandText);
    if (!clean) return;
    try {
      const res = await invoke<RiskAuditResultDto>("audit_command_risk_cmd", {
        command: clean,
      });
      const sess = getOrCreateSessionUiState(sessionUiByTerminal, terminalId);
      if (sess.pendingAction) {
        sess.pendingAction.riskLevel = res.computed_risk;
        syncSessionUiToBackend(terminalId);
      }
      const info = formatRiskLabel(res.computed_risk);
      badgeEl.className = `copilot-risk-badge ${info.cssModifier}`;
      badgeEl.textContent = info.text;
    } catch {
      // Ignorar errores transitorios mientras el usuario edita
    }
  }, 130);
}

function renderActionCard(terminalId: string): HTMLElement | null {
  if (awaitingSentinel && awaitingSentinel.terminalId === terminalId) {
    const runningCard = document.createElement("div");
    runningCard.className = "copilot-action-card";
    runningCard.innerHTML = `
      <div style="display:flex; justify-content:space-between; align-items:center; gap:8px;">
        <strong>Ejecutando comando en terminal…</strong>
        <button type="button" id="btn-interrupt-sentinel" class="copilot-btn-danger-sm">Interrumpir (Ctrl+C)</button>
      </div>
      ${
        awaitingSentinel.waitingHint
          ? `<div class="copilot-waiting-hint"><span>${escapeHtmlText(awaitingSentinel.waitingHint)}</span></div>`
          : ""
      }
      <pre class="copilot-code-block">${escapeHtmlText(awaitingSentinel.command)}</pre>
    `;
    const btnCtrlC = runningCard.querySelector<HTMLButtonElement>("#btn-interrupt-sentinel");
    btnCtrlC?.addEventListener("click", () => {
      void interruptPtyStep(terminalId);
    });
    return runningCard;
  }

  const sess = getOrCreateSessionUiState(sessionUiByTerminal, terminalId);
  if (!sess.pendingAction) return null;
  const action = sess.pendingAction;
  const riskInfo = formatRiskLabel(action.riskLevel);

  const card = document.createElement("div");
  card.className = "copilot-action-card";

  const head = document.createElement("div");
  head.style.display = "flex";
  head.style.justifyContent = "space-between";
  head.style.alignItems = "center";
  head.style.gap = "8px";

  const qTitle = document.createElement("strong");
  qTitle.textContent = buildCommandQuestionText(action.purpose, action.riskLevel);

  const riskBadge = document.createElement("span");
  riskBadge.className = `copilot-risk-badge ${riskInfo.cssModifier}`;
  riskBadge.textContent = riskInfo.text;

  head.appendChild(qTitle);
  head.appendChild(riskBadge);
  card.appendChild(head);

  if (action.auditWarning) {
    const alertBox = document.createElement("div");
    alertBox.className = "copilot-audit-alert";
    alertBox.appendChild(icon(AppIcons.alertTriangle, { size: 15 }));
    const msgSpan = document.createElement("span");
    msgSpan.textContent = action.auditWarning;
    alertBox.appendChild(msgSpan);
    card.appendChild(alertBox);
  }

  if (action.isEditing) {
    const editor = document.createElement("textarea");
    editor.className = "copilot-inline-editor";
    editor.rows = 3;
    editor.autocomplete = "off";
    editor.spellcheck = false;
    editor.setAttribute("aria-label", "Editar comando antes de autorizar");
    editor.value = action.draftCommand;
    editor.addEventListener("input", () => {
      if (sess.pendingAction) {
        sess.pendingAction.draftCommand = editor.value;
        syncSessionUiToBackend(terminalId);
      }
      scheduleLiveRiskAudit(terminalId, editor.value, riskBadge);
    });
    card.appendChild(editor);
  } else {
    const pre = document.createElement("pre");
    pre.className = "copilot-code-block";
    pre.textContent = action.draftCommand;
    card.appendChild(pre);
  }

  if (action.backupStrategy) {
    const bk = document.createElement("div");
    bk.className = "copilot-plan-meta";
    bk.innerHTML = `<strong>Respaldo previsto:</strong> ${escapeHtmlText(action.backupStrategy)}`;
    card.appendChild(bk);
  }

  if (action.rollbackCmd) {
    const rb = document.createElement("div");
    rb.className = "copilot-plan-meta";
    rb.innerHTML = `<strong>Rollback preparado:</strong> <code>${escapeHtmlText(action.rollbackCmd)}</code>`;
    card.appendChild(rb);
  }

  const actions = document.createElement("div");
  actions.className = "copilot-card-actions";

  const btnAuthorize = document.createElement("button");
  btnAuthorize.type = "button";
  btnAuthorize.className = "copilot-btn-primary";
  btnAuthorize.appendChild(icon(AppIcons.play, { size: 14 }));
  const authLabel = document.createElement("span");
  authLabel.textContent =
    action.riskLevel === "read_only" ? "Autorizar Lectura" : "Autorizar y Ejecutar";
  btnAuthorize.appendChild(authLabel);
  btnAuthorize.addEventListener("click", () => {
    const cmdToRun = sanitizeEditedCommand(action.draftCommand);
    if (!cmdToRun) return;
    void executeAuthorizedCommand(
      terminalId,
      cmdToRun,
      action.purpose,
      action.rollbackCmd,
      false,
    );
  });

  const btnEdit = document.createElement("button");
  btnEdit.type = "button";
  btnEdit.className = "copilot-btn-ghost";
  btnEdit.appendChild(icon(AppIcons.pencil, { size: 13 }));
  const editTxt = document.createElement("span");
  editTxt.textContent = action.isEditing ? "Listo edición" : "Editar";
  btnEdit.appendChild(editTxt);
  btnEdit.addEventListener("click", () => {
    if (!sess.pendingAction) return;
    sess.pendingAction.isEditing = !sess.pendingAction.isEditing;
    syncSessionUiToBackend(terminalId);
    renderFeed();
  });

  const btnInsertTerminal = document.createElement("button");
  btnInsertTerminal.type = "button";
  btnInsertTerminal.className = "copilot-btn-ghost";
  btnInsertTerminal.title = "Inserta el comando en el prompt de la terminal sin presionar Enter";
  btnInsertTerminal.appendChild(icon(AppIcons.terminal, { size: 13 }));
  const insertTxt = document.createElement("span");
  insertTxt.textContent = "Insertar en terminal";
  btnInsertTerminal.appendChild(insertTxt);
  btnInsertTerminal.addEventListener("click", () => {
    const clean = sanitizeEditedCommand(action.draftCommand);
    if (!clean) return;
    void (async () => {
      try {
        await invoke("insert_ai_command_to_pty_cmd", {
          terminalId,
          command: clean,
        });
        insertTxt.textContent = "Insertado";
        setTimeout(() => {
          insertTxt.textContent = "Insertar en terminal";
        }, 1200);
      } catch (err) {
        pushFeed(terminalId, {
          kind: "error",
          title: "No se pudo insertar en terminal",
          body: String(err),
        });
      }
    })();
  });

  const btnCopy = document.createElement("button");
  btnCopy.type = "button";
  btnCopy.className = "copilot-btn-ghost";
  btnCopy.appendChild(icon(AppIcons.copy, { size: 13 }));
  const copyTxt = document.createElement("span");
  copyTxt.textContent = "Copiar";
  btnCopy.appendChild(copyTxt);
  btnCopy.addEventListener("click", () => {
    void copyToClipboard(action.draftCommand);
    copyTxt.textContent = "Copiado";
    setTimeout(() => {
      copyTxt.textContent = "Copiar";
    }, 1200);
  });

  const btnSkip = document.createElement("button");
  btnSkip.type = "button";
  btnSkip.className = "copilot-btn-ghost";
  btnSkip.textContent = "Omitir";
  btnSkip.addEventListener("click", () => {
    sess.pendingAction = null;
    syncSessionUiToBackend(terminalId);
    renderToolbar();
    renderFeed();
  });

  actions.appendChild(btnAuthorize);
  actions.appendChild(btnEdit);
  actions.appendChild(btnInsertTerminal);
  actions.appendChild(btnCopy);
  actions.appendChild(btnSkip);
  card.appendChild(actions);

  return card;
}

function renderFailureRescueCard(terminalId: string): HTMLElement | null {
  const sess = getOrCreateSessionUiState(sessionUiByTerminal, terminalId);
  if (!sess.lastFailedContext || awaitingSentinel || isBusyTurn) return null;

  const rescue = document.createElement("div");
  rescue.className = "copilot-rollback-panel";
  rescue.innerHTML = `
    <strong>Intervención detenida por fallo (exit=${sess.lastFailedContext.exitCode})</strong>
    <p class="copilot-plan-meta" style="margin-top:3px;">Puedes analizar el fallo real con el copiloto para recibir la corrección o revertir de inmediato.</p>
  `;

  const actions = document.createElement("div");
  actions.className = "copilot-card-actions";

  const btnDiagnose = document.createElement("button");
  btnDiagnose.type = "button";
  btnDiagnose.className = "copilot-btn-primary";
  btnDiagnose.appendChild(icon(AppIcons.sparkles, { size: 14 }));
  const diagTxt = document.createElement("span");
  diagTxt.textContent = "Analizar fallo con IA";
  btnDiagnose.appendChild(diagTxt);
  btnDiagnose.addEventListener("click", () => {
    const failed = sess.lastFailedContext;
    sess.lastFailedContext = null;
    syncSessionUiToBackend(terminalId);
    if (!failed) return;
    void runAgentTurn(
      `El comando \`${failed.command}\` falló con exit=${failed.exitCode}. Analiza el error capturado, explica la causa y propón la corrección.`,
      failed.output,
    );
  });
  actions.appendChild(btnDiagnose);

  if (sess.lastRollback) {
    const rbCtx = sess.lastRollback;
    const btnRb = document.createElement("button");
    btnRb.type = "button";
    btnRb.className = "copilot-btn-ghost";
    btnRb.appendChild(icon(AppIcons.rotateCcw, { size: 14 }));
    const rbTxt = document.createElement("span");
    rbTxt.textContent = "Preparar Rollback";
    btnRb.appendChild(rbTxt);
    btnRb.addEventListener("click", () => {
      sess.rollbackDraftOpen = {
        sourceCommand: rbCtx.sourceCommand,
        draftCmd: rbCtx.rollbackCmd,
      };
      renderFeed();
    });
    actions.appendChild(btnRb);
  }

  const btnDismiss = document.createElement("button");
  btnDismiss.type = "button";
  btnDismiss.className = "copilot-btn-ghost";
  btnDismiss.textContent = "Descartar";
  btnDismiss.addEventListener("click", () => {
    sess.lastFailedContext = null;
    sess.rollbackDraftOpen = null;
    syncSessionUiToBackend(terminalId);
    renderFeed();
  });
  actions.appendChild(btnDismiss);
  rescue.appendChild(actions);

  if (sess.rollbackDraftOpen) {
    const draft = sess.rollbackDraftOpen;
    const editor = document.createElement("textarea");
    editor.className = "copilot-inline-editor";
    editor.rows = 2;
    editor.autocomplete = "off";
    editor.spellcheck = false;
    editor.setAttribute("aria-label", "Comando de rollback editable");
    editor.value = draft.draftCmd;
    editor.addEventListener("input", () => {
      if (sess.rollbackDraftOpen) sess.rollbackDraftOpen.draftCmd = editor.value;
    });
    rescue.appendChild(editor);

    const rbActions = document.createElement("div");
    rbActions.className = "copilot-card-actions";
    const btnRunRb = document.createElement("button");
    btnRunRb.type = "button";
    btnRunRb.className = "copilot-btn-danger-sm";
    btnRunRb.textContent = "Confirmar y Ejecutar Rollback";
    btnRunRb.addEventListener("click", () => {
      const clean = sanitizeEditedCommand(editor.value);
      if (!clean) return;
      sess.rollbackDraftOpen = null;
      sess.lastFailedContext = null;
      syncSessionUiToBackend(terminalId);
      void executeRollbackCommand(terminalId, clean);
    });
    rbActions.appendChild(btnRunRb);
    rescue.appendChild(rbActions);
  }

  return rescue;
}

function renderFeed() {
  feedEl.replaceChildren();
  const stack = document.createElement("div");
  stack.className = "copilot-feed-stack";
  feedEl.appendChild(stack);

  const target = getActiveTargetShell();
  if (!target) {
    const empty = document.createElement("div");
    empty.className = "copilot-empty-hero";
    empty.innerHTML = `<h2 class="copilot-empty-title">Sin sesión SSH activa</h2><p class="copilot-empty-subtitle">Conecta un servidor en la ventana principal de NekoSSH para habilitar el copiloto.</p>`;
    stack.appendChild(empty);
    return;
  }

  const entries = getSessionFeed(target.terminal_id);
  const sess = getOrCreateSessionUiState(sessionUiByTerminal, target.terminal_id);

  // Estado vacío consolidado en un único hero minimalista centrado
  if (entries.length === 0 && !sess.pendingAction && !awaitingSentinel) {
    const heroState = buildEmptyHeroState(target.profile_name, target.is_listening);
    const hero = document.createElement("div");
    hero.className = "copilot-empty-hero";
    hero.innerHTML = `<h2 class="copilot-empty-title">${escapeHtmlText(heroState.title)}</h2><p class="copilot-empty-subtitle">${escapeHtmlText(heroState.subtitle)}</p>`;

    if (heroState.showActivateListenBtn) {
      const actBtn = document.createElement("button");
      actBtn.type = "button";
      actBtn.className = "copilot-btn-primary";
      actBtn.textContent = "Activar Escucha ON";
      actBtn.addEventListener("click", () => {
        void toggleListening(true);
      });
      hero.appendChild(actBtn);
    }

    const chips = document.createElement("div");
    chips.className = "copilot-quick-chips";

    const quickPrompts = [
      {
        label: "Radiografía del servidor (OS, RAM, disco, puertos)",
        prompt:
          "Haz una radiografía rápida del servidor: sistema operativo, kernel, memoria, uso de disco y puertos en escucha.",
        needsListen: false,
      },
      {
        label: "Servicios fallidos y logs recientes",
        prompt:
          "Revisa si hay unidades de systemd en error o alertas recientes en journalctl.",
        needsListen: false,
      },
      {
        label: "Explicar lo que está en mi pantalla",
        prompt:
          "Analiza lo último visible en mi terminal, explícame qué ocurre y qué acción recomiendas.",
        needsListen: true,
      },
    ];

    for (const q of quickPrompts) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "copilot-quick-chip";
      btn.textContent = q.label;
      btn.addEventListener("click", async () => {
        if (!target.is_connected || isBusyTurn) return;
        if (q.needsListen && !target.is_listening) {
          await toggleListening(true);
          // Pequeña pausa para que main.ts siembre las líneas visibles de xterm.js
          await new Promise((r) => setTimeout(r, 120));
        }
        autoStepsUsed = 0;
        pushFeed(target.terminal_id, { kind: "user", title: "Operador", body: q.prompt });
        void runAgentTurn(q.prompt);
      });
      chips.appendChild(btn);
    }

    hero.appendChild(chips);
    stack.appendChild(hero);
  } else if (!target.is_listening) {
    const strip = document.createElement("div");
    strip.className = "copilot-privacy-strip";
    strip.innerHTML = `<span>Modo privacidad: escucha apagada en <code>${escapeHtmlText(target.profile_name)}</code></span>`;
    const onBtn = document.createElement("button");
    onBtn.type = "button";
    onBtn.className = "copilot-btn-xs";
    onBtn.textContent = "Activar Escucha";
    onBtn.addEventListener("click", () => {
      void toggleListening(true);
    });
    strip.appendChild(onBtn);
    stack.appendChild(strip);
  }

  for (const item of entries) {
    if (item.kind === "console") {
      const details = document.createElement("details");
      details.className = "copilot-console-details";
      details.open = Boolean(item.expanded || (item.exitCode !== undefined && item.exitCode !== 0));
      const isFail = item.exitCode !== undefined && item.exitCode !== 0;

      const summary = document.createElement("summary");
      const titleSpan = document.createElement("span");
      titleSpan.textContent = `${item.title} · ${item.body}`;
      const pill = document.createElement("span");
      pill.className = `copilot-exit-pill ${isFail ? "is-fail" : ""}`;
      pill.textContent = `exit=${item.exitCode ?? 0}`;
      summary.appendChild(titleSpan);
      summary.appendChild(pill);
      details.appendChild(summary);

      if (item.code) {
        const pre = document.createElement("pre");
        pre.className = "copilot-code-block";
        pre.textContent = item.code;
        details.appendChild(pre);
      }
      stack.appendChild(details);
      continue;
    }

    const card = document.createElement("div");
    card.className = `copilot-msg-card ${
      item.kind === "user"
        ? "is-user"
        : item.kind === "error"
          ? "is-error"
          : "is-assistant"
    }`;

    if (item.kind !== "user") {
      const meta = document.createElement("div");
      meta.className = "copilot-msg-meta";
      const titleEl = document.createElement("span");
      titleEl.textContent = item.title;
      meta.appendChild(titleEl);
      if (item.phaseLabel) {
        const pill = document.createElement("span");
        pill.className = "copilot-stage-pill";
        pill.textContent = item.phaseLabel;
        meta.appendChild(pill);
      }
      card.appendChild(meta);
    }

    if (item.kind === "assistant") {
      const mdWrap = document.createElement("div");
      mdWrap.className = "copilot-md-body";
      mdWrap.innerHTML = renderSafeMarkdown(item.body);
      mdWrap.querySelectorAll<HTMLButtonElement>(".copilot-copy-code-btn").forEach((btn) => {
        btn.addEventListener("click", () => {
          const targetId = btn.dataset.copyTarget;
          const codeEl = targetId ? mdWrap.querySelector<HTMLElement>(`#${targetId}`) : null;
          if (!codeEl) return;
          void copyToClipboard(codeEl.textContent ?? "");
          btn.textContent = "Copiado";
          setTimeout(() => {
            btn.textContent = "Copiar";
          }, 1200);
        });
      });
      card.appendChild(mdWrap);
    } else {
      const body = document.createElement("div");
      body.textContent = item.body;
      card.appendChild(body);
    }

    if (item.code) {
      const pre = document.createElement("pre");
      pre.className = "copilot-code-block";
      pre.textContent = item.code;
      card.appendChild(pre);
    }
    stack.appendChild(card);
  }

  const actionCard = renderActionCard(target.terminal_id);
  if (actionCard) {
    stack.appendChild(actionCard);
  }

  const rescueCard = renderFailureRescueCard(target.terminal_id);
  if (rescueCard) {
    stack.appendChild(rescueCard);
  }

  feedEl.scrollTop = feedEl.scrollHeight;
}

async function refreshPhaseForTarget() {
  const target = getActiveTargetShell();
  if (!target) {
    currentPhase = { type: "idle" };
    currentPhaseTag = null;
    renderToolbar();
    renderFeed();
    return;
  }
  try {
    currentPhase = await invoke<HarnessPhase>("get_harness_phase_cmd", {
      sessionId: target.terminal_id,
    });
    if (getSessionFeed(target.terminal_id).length === 0) {
      const stored = await invoke<AiFeedEntryDto[]>("get_ai_feed_cmd", {
        sessionId: target.terminal_id,
      });
      if (stored.length > 0) {
        feedBySession.set(
          target.terminal_id,
          stored.map((item) => ({
            kind: item.kind,
            title: item.title,
            body: item.body,
            code: item.code ?? undefined,
            exitCode: item.exit_code ?? undefined,
            expanded: item.expanded ?? undefined,
            phaseLabel: item.phase_label ?? undefined,
          })),
        );
      }
    }
    const existingUi = sessionUiByTerminal.get(target.terminal_id);
    if (!existingUi) {
      const remoteState = await invoke<AiUiSessionStateDto>("get_ai_session_ui_state_cmd", {
        sessionId: target.terminal_id,
      });
      const restored = createEmptySessionUiState();
      if (remoteState.pending_action) {
        restored.pendingAction = {
          purpose: remoteState.pending_action.purpose,
          originalCommand: remoteState.pending_action.original_command,
          draftCommand: remoteState.pending_action.draft_command,
          isEditing: remoteState.pending_action.is_editing,
          riskLevel: remoteState.pending_action.risk_level,
          auditWarning: remoteState.pending_action.audit_warning,
          backupStrategy: remoteState.pending_action.backup_strategy,
          rollbackCmd: remoteState.pending_action.rollback_cmd,
        };
      }
      if (remoteState.last_failed) {
        restored.lastFailedContext = {
          command: remoteState.last_failed.command,
          exitCode: remoteState.last_failed.exit_code,
          output: remoteState.last_failed.output,
        };
      }
      if (remoteState.last_rollback) {
        restored.lastRollback = {
          sourceCommand: remoteState.last_rollback.source_command,
          rollbackCmd: remoteState.last_rollback.rollback_cmd,
          label: remoteState.last_rollback.label,
        };
      }
      sessionUiByTerminal.set(target.terminal_id, restored);
    }
  } catch (err) {
    console.error("Error leyendo fase del arnés:", err);
  }
  renderToolbar();
  renderFeed();
}

async function toggleListening(explicit?: boolean) {
  const target = getActiveTargetShell();
  if (!target) return;
  const next = typeof explicit === "boolean" ? explicit : !target.is_listening;
  try {
    await invoke("set_pty_listening_cmd", {
      terminalId: target.terminal_id,
      enabled: next,
    });
  } catch (err) {
    console.error("Error cambiando escucha:", err);
  }
}

async function toggleExecutionMode() {
  if (!currentPrefs) return;
  const nextMode = currentPrefs.execution_mode === "strict" ? "flexible" : "strict";
  const updated: AiPreferences = { ...currentPrefs, execution_mode: nextMode };
  try {
    await invoke("set_ai_preferences_cmd", { prefs: updated });
    currentPrefs = updated;
    renderToolbar();
  } catch (err) {
    console.error("Error actualizando modo:", err);
  }
}

async function loadModelsCatalog() {
  try {
    const models = await invoke<AiModelDescriptor[]>("list_ai_models_cmd");
    const selected = currentPrefs?.selected_model ?? "";
    modelSelect.replaceChildren();
    const defOpt = document.createElement("option");
    defOpt.value = "";
    defOpt.textContent = "Predeterminado del proveedor";
    modelSelect.appendChild(defOpt);
    for (const m of models) {
      const opt = document.createElement("option");
      opt.value = m.id;
      opt.textContent = m.label;
      opt.selected = m.id === selected;
      modelSelect.appendChild(opt);
    }
    if (!selected) defOpt.selected = true;
  } catch (err) {
    console.warn("No se pudo obtener catálogo de modelos:", err);
  }
}

async function loadPreferencesAndHealth() {
  try {
    currentPrefs = await invoke<AiPreferences>("get_ai_preferences_cmd");
    fillSettingsForm(currentPrefs);
    renderToolbar();
  } catch (err) {
    console.error("Error cargando preferencias AI:", err);
  }
  void loadModelsCatalog();
  void probeDriverHealth();
}

function fillSettingsForm(prefs: AiPreferences) {
  settingsDriverId.textContent = prefs.active_driver || "agy";
  cfgAgyPath.value = prefs.driver_options?.agy_binary_path ?? "";
  cfgContextLines.value = String(prefs.context_max_lines ?? 200);
  cfgOperatorPersona.value = prefs.operator_persona ?? "";
  cfgInfraRules.value = prefs.operator_infra_rules ?? "";
}

async function probeDriverHealth() {
  settingsHealthMsg.textContent = "Verificando disponibilidad…";
  try {
    const status = await invoke<DriverHealthStatus>("probe_ai_driver_health_cmd");
    settingsHealthMsg.textContent = status.message;
    settingsHealthMsg.style.color = status.available
      ? "var(--color-accent-secondary)"
      : "var(--color-error)";
  } catch (err) {
    settingsHealthMsg.textContent = String(err);
    settingsHealthMsg.style.color = "var(--color-error)";
  }
}

async function runAgentTurn(promptText: string, explicitContext?: string) {
  const target = getActiveTargetShell();
  if (!target || isBusyTurn) return;

  const sess = getOrCreateSessionUiState(sessionUiByTerminal, target.terminal_id);
  isBusyTurn = true;
  sess.pendingAction = null;
  sess.lastFailedContext = null;
  sess.rollbackDraftOpen = null;
  syncSessionUiToBackend(target.terminal_id);
  rawProgressStreamBuffer = "";
  progressStreamEl.textContent = "";
  thinkingBox.hidden = false;
  renderToolbar();

  try {
    const res = await invoke<TurnExecutionResult>("run_ai_harness_turn_cmd", {
      sessionId: target.terminal_id,
      userPrompt: promptText,
      terminalContext: explicitContext ?? null,
    });

    currentPhase = res.phase;
    const resp = res.outcome.response;

    if (resp.kind === "propose_step") {
      currentPhaseTag = resp.phase_tag ?? "diagnosis";

      if (resp.rollback_cmd && resp.rollback_cmd.trim().length > 0) {
        sess.lastRollback = {
          sourceCommand: resp.command,
          rollbackCmd: resp.rollback_cmd,
          label: resp.purpose || "Último cambio",
        };
      }

      if (
        shouldAutoRunReadOnlyStep({
          mode: res.execution_mode,
          autoApprovedByBackend: res.auto_approved_read_only,
          response: resp,
          autoStepsUsed,
        })
      ) {
        autoStepsUsed += 1;
        if (resp.reasoning.trim().length > 0) {
          pushFeed(target.terminal_id, {
            kind: "assistant",
            title: "NekoAI",
            body: resp.reasoning,
            phaseLabel: "Diagnóstico automático",
          });
        }
        isBusyTurn = false;
        thinkingBox.hidden = true;
        await executeAuthorizedCommand(
          target.terminal_id,
          resp.command,
          resp.purpose || resp.reasoning,
          resp.rollback_cmd ?? null,
          true,
        );
        return;
      }

      if (resp.reasoning.trim().length > 0) {
        pushFeed(target.terminal_id, {
          kind: "assistant",
          title: "NekoAI",
          body: resp.reasoning,
          phaseLabel: formatStageBadgeText(currentPhaseTag, currentPhase),
        });
      }

      sess.pendingAction = {
        purpose: resp.purpose || resp.reasoning,
        originalCommand: resp.command,
        draftCommand: resp.command,
        isEditing: false,
        riskLevel: resp.risk_level,
        auditWarning: resp.audit_warning ?? null,
        backupStrategy: resp.backup_strategy ?? null,
        rollbackCmd: resp.rollback_cmd ?? null,
      };
      syncSessionUiToBackend(target.terminal_id);
    } else if (resp.kind === "reply_only") {
      currentPhaseTag = resp.phase_tag ?? "direct_answer";
      autoStepsUsed = 0;
      pushFeed(target.terminal_id, {
        kind: "assistant",
        title: "NekoAI",
        body: resp.message,
        phaseLabel: formatStageBadgeText(currentPhaseTag, currentPhase),
      });
    } else if (resp.kind === "goal_reached") {
      currentPhaseTag = "direct_answer";
      autoStepsUsed = 0;
      pushFeed(target.terminal_id, {
        kind: "assistant",
        title: "NekoAI",
        body: resp.summary,
        phaseLabel: "Conclusión",
      });
    } else if (resp.kind === "propose_plan") {
      currentPhaseTag = "proposal";
      pushFeed(target.terminal_id, {
        kind: "assistant",
        title: "NekoAI",
        body: formatPlanSummaryMarkdown(resp.reasoning, resp.tasks),
        phaseLabel: formatStageBadgeText(currentPhaseTag, currentPhase),
      });
      sess.pendingAction = planFirstTaskToPendingAction(resp.tasks);
      syncSessionUiToBackend(target.terminal_id);
    }
  } catch (err) {
    pushFeed(target.terminal_id, {
      kind: "error",
      title: "No se pudo completar la consulta",
      body: String(err),
    });
  } finally {
    isBusyTurn = false;
    thinkingBox.hidden = true;
    renderToolbar();
    renderFeed();
  }
}

async function executeAuthorizedCommand(
  terminalId: string,
  command: string,
  purpose: string,
  rollbackCmd: string | null,
  isAutoRead: boolean,
) {
  const sess = getOrCreateSessionUiState(sessionUiByTerminal, terminalId);
  sess.pendingAction = null;
  sess.lastFailedContext = null;
  sess.rollbackDraftOpen = null;

  if (rollbackCmd && rollbackCmd.trim().length > 0) {
    sess.lastRollback = {
      sourceCommand: command,
      rollbackCmd,
      label: purpose || "Último cambio",
    };
  }
  syncSessionUiToBackend(terminalId);

  try {
    const nonce = await invoke<string>("execute_ai_command_cmd", {
      terminalId,
      command,
      purpose,
    });
    awaitingSentinel = {
      terminalId,
      nonce,
      command,
      kind: isAutoRead ? "auto_read" : "authorized",
      rollbackCmd,
      waitingHint: null,
    };
    armWaitingHintFallback(terminalId);
    renderToolbar();
    renderFeed();
  } catch (err) {
    clearWaitingHintTimer();
    awaitingSentinel = null;
    pushFeed(terminalId, {
      kind: "error",
      title: "Error al enviar comando al PTY",
      body: String(err),
    });
    renderToolbar();
    renderFeed();
  }
}

async function executeRollbackCommand(terminalId: string, rollbackCmd: string) {
  try {
    const nonce = await invoke<string>("execute_ai_command_cmd", {
      terminalId,
      command: rollbackCmd,
      purpose: "ejecutar rollback de rescate",
    });
    awaitingSentinel = {
      terminalId,
      nonce,
      command: rollbackCmd,
      kind: "rollback",
      rollbackCmd: null,
      waitingHint: null,
    };
    armWaitingHintFallback(terminalId);
    renderToolbar();
    renderFeed();
  } catch (err) {
    clearWaitingHintTimer();
    awaitingSentinel = null;
    pushFeed(terminalId, {
      kind: "error",
      title: "No se pudo iniciar rollback",
      body: String(err),
    });
    renderToolbar();
    renderFeed();
  }
}

async function interruptPtyStep(terminalId: string) {
  const ranCommand = awaitingSentinel?.command ?? "";
  try {
    currentPhase = await invoke<HarnessPhase>("interrupt_ai_pty_step_cmd", {
      terminalId,
    });
  } catch (err) {
    console.error("Error enviando Ctrl+C:", err);
  }
  clearWaitingHintTimer();
  awaitingSentinel = null;
  const sess = getOrCreateSessionUiState(sessionUiByTerminal, terminalId);
  sess.lastFailedContext = {
    command: ranCommand,
    exitCode: 130,
    output: "Ejecución interrumpida manualmente por el operador (Ctrl+C).",
  };
  syncSessionUiToBackend(terminalId);
  pushFeed(terminalId, {
    kind: "console",
    title: "Interrumpido (Ctrl+C)",
    body: ranCommand || "comando en ejecución",
    code: "Ejecución detenida manualmente con SIGINT.",
    exitCode: 130,
    expanded: true,
  });
  renderToolbar();
  renderFeed();
}

async function clearCurrentConversation() {
  const target = getActiveTargetShell();
  if (!target || isBusyTurn || awaitingSentinel) return;
  try {
    currentPhase = await invoke<HarnessPhase>("clear_ai_conversation_cmd", {
      sessionId: target.terminal_id,
    });
  } catch (err) {
    console.error("Error limpiando conversación:", err);
  }
  clearWaitingHintTimer();
  feedBySession.delete(target.terminal_id);
  sessionUiByTerminal.set(target.terminal_id, createEmptySessionUiState());
  currentPhaseTag = null;
  autoStepsUsed = 0;
  renderToolbar();
  renderFeed();
}

// --- Eventos de DOM ---
shellSelect.addEventListener("change", () => {
  if (!shellSelect.value || awaitingSentinel) return;
  shellSelection = { targetTerminalId: shellSelect.value, pinned: true };
  void refreshPhaseForTarget();
});

btnPinShell.addEventListener("click", () => {
  if (!shellSelection.targetTerminalId || awaitingSentinel) return;
  shellSelection = {
    ...shellSelection,
    pinned: !shellSelection.pinned,
  };
  if (!shellSelection.pinned) {
    shellSelection = resolveTargetShell(shellSelection, shellsSnapshot);
    void refreshPhaseForTarget();
  } else {
    renderToolbar();
  }
});

btnClearChat.addEventListener("click", () => {
  void clearCurrentConversation();
});

btnAlwaysOnTop.addEventListener("click", async () => {
  alwaysOnTop = !alwaysOnTop;
  btnAlwaysOnTop.classList.toggle("is-active", alwaysOnTop);
  btnAlwaysOnTop.setAttribute("aria-pressed", alwaysOnTop ? "true" : "false");
  try {
    await invoke("set_ai_window_always_on_top_cmd", { alwaysOnTop });
  } catch (err) {
    console.error("Error ajustando Always on Top:", err);
  }
});

function setSettingsOpen(open: boolean) {
  settingsViewEl.hidden = !open;
  workspaceEl.hidden = open;
  btnToggleSettings.classList.toggle("is-active", open);
  btnToggleSettings.setAttribute("aria-expanded", open ? "true" : "false");
  if (open) {
    void probeDriverHealth();
  }
}

btnToggleSettings.addEventListener("click", () => {
  setSettingsOpen(settingsViewEl.hidden);
});

btnCloseSettings.addEventListener("click", () => {
  setSettingsOpen(false);
});

btnToggleListen.addEventListener("click", () => {
  void toggleListening();
});

btnToggleMode.addEventListener("click", () => {
  void toggleExecutionMode();
});

btnRefreshModels.addEventListener("click", () => {
  void loadModelsCatalog();
});

modelSelect.addEventListener("change", async () => {
  if (!currentPrefs) return;
  const updated: AiPreferences = {
    ...currentPrefs,
    selected_model: modelSelect.value,
  };
  try {
    await invoke("set_ai_preferences_cmd", { prefs: updated });
    currentPrefs = updated;
  } catch (err) {
    console.error("Error guardando modelo seleccionado:", err);
  }
});

btnCancelTurn.addEventListener("click", async () => {
  const target = getActiveTargetShell();
  if (!target) return;
  await invoke("cancel_ai_turn_cmd", { sessionId: target.terminal_id }).catch(() => {});
});

function autoResizePromptInput() {
  promptInput.style.height = "auto";
  const nextHeight = Math.min(promptInput.scrollHeight, 140);
  promptInput.style.height = `${Math.max(nextHeight, 38)}px`;
}

function applyQuotedSelectionToComposer(payload: { terminal_id: string; selection_text: string }) {
  const formatted = formatQuotedSelectionPrompt(payload.selection_text);
  if (!formatted) return;
  if (
    payload.terminal_id &&
    shellsSnapshot.shells.some((s) => s.terminal_id === payload.terminal_id) &&
    shellSelection.targetTerminalId !== payload.terminal_id
  ) {
    shellSelection = { targetTerminalId: payload.terminal_id, pinned: true };
    void refreshPhaseForTarget();
  }
  const existing = promptInput.value.trim();
  promptInput.value = existing ? `${existing}\n\n${formatted}` : formatted;
  autoResizePromptInput();
  promptInput.focus();
}

promptInput.addEventListener("input", () => {
  autoResizePromptInput();
  const target = getActiveTargetShell();
  if (target) {
    promptHistoryCursorBySession.set(target.terminal_id, -1);
  }
});

promptInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    chatForm.requestSubmit();
    return;
  }

  const target = getActiveTargetShell();
  if (!target) return;

  if (e.key === "ArrowUp") {
    const atTop = promptInput.selectionStart === 0 && promptInput.selectionEnd === 0;
    const isSingleLine = !promptInput.value.includes("\n");
    if (!atTop && !isSingleLine && promptInput.value.length > 0) return;

    const history = promptHistoryBySession.get(target.terminal_id) ?? [];
    if (history.length === 0) return;
    e.preventDefault();
    const cur = promptHistoryCursorBySession.get(target.terminal_id) ?? -1;
    const draft = promptDraftBySession.get(target.terminal_id) ?? "";
    const step = stepPromptHistory(history, cur, promptInput.value, "up", draft);
    promptHistoryCursorBySession.set(target.terminal_id, step.nextIndex);
    promptDraftBySession.set(target.terminal_id, step.savedDraft);
    promptInput.value = step.value;
    autoResizePromptInput();
  } else if (e.key === "ArrowDown") {
    const cur = promptHistoryCursorBySession.get(target.terminal_id) ?? -1;
    if (cur < 0) return;
    const atEnd =
      promptInput.selectionStart === promptInput.value.length &&
      promptInput.selectionEnd === promptInput.value.length;
    if (!atEnd && promptInput.value.includes("\n")) return;

    e.preventDefault();
    const history = promptHistoryBySession.get(target.terminal_id) ?? [];
    const draft = promptDraftBySession.get(target.terminal_id) ?? "";
    const step = stepPromptHistory(history, cur, promptInput.value, "down", draft);
    promptHistoryCursorBySession.set(target.terminal_id, step.nextIndex);
    promptInput.value = step.value;
    autoResizePromptInput();
  }
});

chatForm.addEventListener("submit", (e) => {
  e.preventDefault();
  const target = getActiveTargetShell();
  const text = promptInput.value.trim();
  if (!target || !text || isBusyTurn || awaitingSentinel) return;
  const hist = promptHistoryBySession.get(target.terminal_id) ?? [];
  if (hist[hist.length - 1] !== text) {
    hist.push(text);
    if (hist.length > 40) hist.shift();
    promptHistoryBySession.set(target.terminal_id, hist);
  }
  promptHistoryCursorBySession.set(target.terminal_id, -1);
  promptDraftBySession.set(target.terminal_id, "");
  promptInput.value = "";
  autoResizePromptInput();
  autoStepsUsed = 0;
  pushFeed(target.terminal_id, {
    kind: "user",
    title: "Operador",
    body: text,
  });
  void runAgentTurn(text);
});

btnProbeHealth.addEventListener("click", () => {
  void probeDriverHealth();
});

settingsForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  if (!currentPrefs) return;
  const driverOpts = { ...(currentPrefs.driver_options ?? {}) };
  const customBin = cfgAgyPath.value.trim();
  if (customBin) {
    driverOpts.agy_binary_path = customBin;
  } else {
    delete driverOpts.agy_binary_path;
  }

  const parsedLines = Number.parseInt(cfgContextLines.value, 10);
  const updated: AiPreferences = {
    ...currentPrefs,
    context_max_lines: Number.isFinite(parsedLines) ? parsedLines : 200,
    operator_persona: cfgOperatorPersona.value,
    operator_infra_rules: cfgInfraRules.value,
    driver_options: driverOpts,
  };

  try {
    await invoke("set_ai_preferences_cmd", { prefs: updated });
    currentPrefs = await invoke<AiPreferences>("get_ai_preferences_cmd");
    fillSettingsForm(currentPrefs);
    renderToolbar();
    settingsSaveStatus.textContent = "Ajustes guardados correctamente.";
    void loadModelsCatalog();
    void probeDriverHealth();
  } catch (err) {
    settingsSaveStatus.textContent = `Error al guardar: ${String(err)}`;
  }
});

btnResetSettings.addEventListener("click", async () => {
  try {
    const defaults = await invoke<AiPreferences>("reset_ai_preferences_cmd");
    currentPrefs = defaults;
    fillSettingsForm(defaults);
    renderToolbar();
    settingsSaveStatus.textContent = "Valores predeterminados restaurados.";
    void loadModelsCatalog();
  } catch (err) {
    settingsSaveStatus.textContent = `Error al restaurar: ${String(err)}`;
  }
});

// --- Inicialización y Suscripciones ---
window.addEventListener("DOMContentLoaded", () => {
  applyThemeFromStorage();
  initStaticIcons();
  renderToolbar();
  renderFeed();

  void listen<string>("nekossh-theme-changed", (ev) => {
    applyThemeFromStorage(ev.payload);
  });

  void listen<ActiveShellsSnapshot>("ai-shells-updated", (ev) => {
    const prevTarget = shellSelection.targetTerminalId;
    shellsSnapshot = ev.payload;
    if (!awaitingSentinel) {
      shellSelection = resolveTargetShell(shellSelection, shellsSnapshot);
    }
    if (shellSelection.targetTerminalId !== prevTarget) {
      void refreshPhaseForTarget();
    } else {
      renderToolbar();
      renderFeed();
    }
  });

  void listen<{ session_id: string; phase: HarnessPhase }>(
    "ai-harness-phase-changed",
    (ev) => {
      const target = getActiveTargetShell();
      if (target && ev.payload.session_id === target.terminal_id) {
        currentPhase = ev.payload.phase;
        awaitingSentinel = null;
        renderToolbar();
        renderFeed();
      }
    },
  );

  void listen<{ session_id: string; chunk: string }>("ai-turn-progress", (ev) => {
    const target = getActiveTargetShell();
    if (target && ev.payload.session_id === target.terminal_id) {
      rawProgressStreamBuffer += ev.payload.chunk;
      progressStreamEl.textContent = extractReadableStreamPreview(rawProgressStreamBuffer);
      progressStreamEl.scrollTop = progressStreamEl.scrollHeight;
    }
  });

  void listen<{ terminal_id: string; selection_text: string }>("ai-composer-selection", (ev) => {
    applyQuotedSelectionToComposer(ev.payload);
  });

  void listen<{ terminal_id: string; hint: string }>("ai-pty-waiting-input", (ev) => {
    if (awaitingSentinel && awaitingSentinel.terminalId === ev.payload.terminal_id) {
      awaitingSentinel.waitingHint = ev.payload.hint;
      renderFeed();
    }
  });

  void listen<SentinelCompletion>("ai-pty-command-finished", (ev) => {
    const done = ev.payload;
    const target = getActiveTargetShell();
    const isCurrentTarget = Boolean(target && target.terminal_id === done.terminal_id);

    const ranCmd =
      awaitingSentinel?.terminalId === done.terminal_id && awaitingSentinel?.nonce === done.nonce
        ? awaitingSentinel.command
        : done.purpose.kind === "authorized_command" || done.purpose.kind === "read_only_recon"
          ? done.purpose.command
          : "";
    const wasRollback = awaitingSentinel?.kind === "rollback";

    if (
      awaitingSentinel &&
      awaitingSentinel.terminalId === done.terminal_id &&
      awaitingSentinel.nonce === done.nonce
    ) {
      clearWaitingHintTimer();
      awaitingSentinel = null;
    }

    if (isCurrentTarget && done.updated_phase) {
      currentPhase = done.updated_phase;
    }

    pushFeed(done.terminal_id, {
      kind: "console",
      title: wasRollback ? "Rollback ejecutado" : "Salida capturada",
      body: ranCmd || "comando",
      code: done.captured_output || "(sin salida)",
      exitCode: done.exit_code,
      expanded: done.exit_code !== 0,
    });

    if (wasRollback) {
      renderToolbar();
      renderFeed();
      return;
    }

    if (done.exit_code === 0) {
      if (isCurrentTarget) {
        const resultMsg = `Resultado del comando \`${ranCmd}\` (exit=0):\n${done.captured_output || "(sin salida)"}\n\nContinúa con el análisis, la siguiente acción progresiva de la fase que corresponda o la respuesta final si el objetivo ya quedó cubierto.`;
        void runAgentTurn(resultMsg, done.captured_output);
      } else {
        renderToolbar();
        renderFeed();
      }
    } else {
      const sess = getOrCreateSessionUiState(sessionUiByTerminal, done.terminal_id);
      sess.lastFailedContext = {
        command: ranCmd,
        exitCode: done.exit_code,
        output: done.captured_output,
      };
      syncSessionUiToBackend(done.terminal_id);
      renderToolbar();
      renderFeed();
    }
  });

  void (async () => {
    await loadPreferencesAndHealth();
    try {
      shellsSnapshot = await invoke<ActiveShellsSnapshot>("get_active_shells_cmd");
      shellSelection = resolveTargetShell(shellSelection, shellsSnapshot);
    } catch (err) {
      console.error("Error obteniendo shells iniciales:", err);
    }
    await refreshPhaseForTarget();
    try {
      const pendingSelection = await invoke<{ terminal_id: string; selection_text: string } | null>(
        "take_pending_ai_selection_cmd",
      );
      if (pendingSelection) {
        applyQuotedSelectionToComposer(pendingSelection);
      }
    } catch {
      // Ignorar si no hay selección en cola
    }
  })();
});
