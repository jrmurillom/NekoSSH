/**
 * Lógica pura del copiloto conversacional NekoAI (4 Fases Progresivas).
 * Desacoplada del DOM/Tauri para cobertura unitaria con Vitest.
 */

export type ExecutionMode = "strict" | "flexible";
export type RiskLevel = "read_only" | "config_change" | "service_impact" | "destructive";
export type ConversationalPhaseTag =
  | "diagnosis"
  | "proposal"
  | "execution"
  | "verification"
  | "direct_answer";

export const MAX_AUTO_CHAIN_STEPS = 6;

export interface ActiveShellInfo {
  terminal_id: string;
  context_id: string;
  profile_name: string;
  label: string;
  is_connected: boolean;
  is_listening: boolean;
  captured_lines?: number;
  current_cwd?: string | null;
}

export interface ActiveShellsSnapshot {
  shells: ActiveShellInfo[];
  focused_terminal_id: string | null;
}

export interface AgentTaskProposal {
  id: number;
  title: string;
  why: string;
  command: string;
  risk_level: RiskLevel;
  backup_strategy: string;
  rollback_cmd: string;
  is_done?: boolean;
  is_rolled_back?: boolean;
}

export type SentinelPurpose =
  | { kind: "read_only_recon"; command: string }
  | { kind: "authorized_command"; command: string; purpose: string }
  | { kind: "plan_step"; step_index: number; command: string }
  | { kind: "step_rollback"; step_index: number; rollback_cmd: string };

export type HarnessPhase =
  | { type: "idle" }
  | { type: "reconnaissance"; objective: string }
  | { type: "plan_and_debate"; objective: string; plan: AgentTaskProposal[] }
  | {
      type: "guided_execution";
      objective: string;
      tasks: AgentTaskProposal[];
      current_step_index: number;
    }
  | {
      type: "halted_on_error";
      objective: string;
      tasks: AgentTaskProposal[];
      step_index: number;
      failed_command: string;
      exit_code: number;
      stderr: string;
    }
  | { type: "paused_on_disconnect"; previous_phase: HarnessPhase }
  | { type: "completed"; objective: string; completed_tasks_count: number };

export type AgentResponseKind =
  | {
      kind: "propose_step";
      reasoning: string;
      purpose?: string | null;
      command: string;
      risk_level: RiskLevel;
      backup_strategy?: string | null;
      rollback_cmd?: string | null;
      audit_warning?: string | null;
      phase_tag?: ConversationalPhaseTag | null;
    }
  | {
      kind: "propose_plan";
      reasoning: string;
      tasks: AgentTaskProposal[];
    }
  | {
      kind: "reply_only";
      message: string;
      phase_tag?: ConversationalPhaseTag | null;
    }
  | {
      kind: "goal_reached";
      summary: string;
    };

export interface AiModelDescriptor {
  id: string;
  label: string;
}

export interface DriverHealthStatus {
  driver_id: string;
  available: boolean;
  version: string | null;
  message: string;
}

export interface AiPreferences {
  execution_mode: ExecutionMode;
  operator_persona: string;
  operator_infra_rules: string;
  active_driver: string;
  selected_model: string;
  context_max_lines: number;
  driver_options: Record<string, string>;
}

export interface ShellSelectionState {
  targetTerminalId: string | null;
  pinned: boolean;
}

export interface RollbackContext {
  sourceCommand: string;
  rollbackCmd: string;
  label: string;
}

export interface PendingActionState {
  purpose: string;
  originalCommand: string;
  draftCommand: string;
  isEditing: boolean;
  riskLevel: RiskLevel;
  auditWarning: string | null;
  backupStrategy: string | null;
  rollbackCmd: string | null;
}

export interface FailedContextState {
  command: string;
  exitCode: number;
  output: string;
}

export interface RollbackDraftState {
  sourceCommand: string;
  draftCmd: string;
}

export interface SessionUiState {
  pendingAction: PendingActionState | null;
  lastFailedContext: FailedContextState | null;
  lastRollback: RollbackContext | null;
  rollbackDraftOpen: RollbackDraftState | null;
}

export function createEmptySessionUiState(): SessionUiState {
  return {
    pendingAction: null,
    lastFailedContext: null,
    lastRollback: null,
    rollbackDraftOpen: null,
  };
}

export function getOrCreateSessionUiState(
  store: Map<string, SessionUiState>,
  terminalId: string,
): SessionUiState {
  let state = store.get(terminalId);
  if (!state) {
    state = createEmptySessionUiState();
    store.set(terminalId, state);
  }
  return state;
}

/** Formatea una respuesta `ProposePlan` en Markdown claro enumerando pasos y comandos. */
export function formatPlanSummaryMarkdown(
  reasoning: string,
  tasks: AgentTaskProposal[],
): string {
  const lines: string[] = [];
  if (reasoning.trim().length > 0) {
    lines.push(reasoning.trim(), "");
  }
  tasks.forEach((task, idx) => {
    const whyTxt = task.why?.trim() ? ` — ${task.why.trim()}` : "";
    lines.push(`${idx + 1}. **${task.title.trim() || "Paso"}**${whyTxt}`);
    if (task.command?.trim()) {
      lines.push(`   \`${task.command.trim()}\``);
    }
  });
  return lines.join("\n");
}

/** Promueve el primer paso pendiente de un plan a tarjeta HITL accionable. */
export function planFirstTaskToPendingAction(
  tasks: AgentTaskProposal[],
): PendingActionState | null {
  const nextTask = tasks.find((t) => !t.is_done && !t.is_rolled_back) ?? tasks[0];
  if (!nextTask || !nextTask.command.trim()) {
    return null;
  }
  return {
    purpose: nextTask.title?.trim() || nextTask.why?.trim() || "ejecutar paso propuesto",
    originalCommand: nextTask.command.trim(),
    draftCommand: nextTask.command.trim(),
    isEditing: false,
    riskLevel: nextTask.risk_level,
    auditWarning: null,
    backupStrategy: nextTask.backup_strategy?.trim() || null,
    rollbackCmd: nextTask.rollback_cmd?.trim() || null,
  };
}

/**
 * Resuelve el terminal objetivo respetando Pin Shell cuando sigue vivo.
 * Si el shell fijado se cerró, libera el pin y sigue al foco actual (o al primero disponible).
 */
export function resolveTargetShell(
  current: ShellSelectionState,
  snapshot: ActiveShellsSnapshot,
): ShellSelectionState {
  const exists = (id: string | null): boolean =>
    Boolean(id && snapshot.shells.some((s) => s.terminal_id === id));

  if (current.pinned && exists(current.targetTerminalId)) {
    return current;
  }

  if (exists(snapshot.focused_terminal_id)) {
    return {
      targetTerminalId: snapshot.focused_terminal_id,
      pinned: false,
    };
  }

  return {
    targetTerminalId: snapshot.shells[0]?.terminal_id ?? null,
    pinned: false,
  };
}

/**
 * Determina si una propuesta de comando puede auto-ejecutarse sin clic humano.
 * Solo ocurre en Modo Flexible cuando backend verificó lectura inocua (`autoApprovedByBackend`)
 * sin advertencias de auditoría y dentro de `MAX_AUTO_CHAIN_STEPS`.
 */
export function shouldAutoRunReadOnlyStep(params: {
  mode: ExecutionMode;
  autoApprovedByBackend: boolean;
  response: AgentResponseKind;
  autoStepsUsed: number;
}): boolean {
  if (params.mode !== "flexible") return false;
  if (!params.autoApprovedByBackend) return false;
  if (params.autoStepsUsed >= MAX_AUTO_CHAIN_STEPS) return false;
  if (params.response.kind !== "propose_step") return false;
  return params.response.risk_level === "read_only" && !params.response.audit_warning;
}

/** Construye la pregunta explicativa natural para la tarjeta de autorización humana. */
export function buildCommandQuestionText(
  purpose: string | null | undefined,
  risk: RiskLevel,
): string {
  const cleanPurpose = (purpose ?? "").trim().replace(/\.+$/, "");
  if (cleanPurpose.length > 0) {
    const normalized = cleanPurpose.charAt(0).toLowerCase() + cleanPurpose.slice(1);
    return risk === "read_only"
      ? `¿Autorizas ejecutar este comando de lectura para ${normalized}?`
      : `¿Autorizas ejecutar este comando para ${normalized}?`;
  }
  return risk === "read_only"
    ? "¿Autorizas ejecutar este comando de inspección?"
    : "¿Autorizas ejecutar este comando en el servidor?";
}

/** Valida un comando editado inline antes de enviarlo al backend. */
export function sanitizeEditedCommand(raw: string): string | null {
  const trimmed = raw.trim();
  return trimmed.length > 0 ? trimmed : null;
}

/** Formatea etiqueta legible y autoexplicativa en español para los 4 niveles de riesgo. */
export function formatRiskLabel(risk: RiskLevel): {
  text: string;
  cssModifier: string;
} {
  switch (risk) {
    case "read_only":
      return { text: "Solo lectura · No modifica", cssModifier: "is-readonly" };
    case "config_change":
      return { text: "Modifica archivos / config", cssModifier: "is-config" };
    case "service_impact":
      return { text: "Impacto en servicio", cssModifier: "is-service" };
    case "destructive":
      return { text: "Crítico · Destructivo", cssModifier: "is-destructive" };
  }
}

/** Escapa texto arbitrario antes de interpolar en HTML plano. */
export function escapeHtmlText(raw: string): string {
  return String(raw)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** Traduce la etapa actual de 4 fases (1..4) para la barra superior de fases. */
export function activeFourPhaseStage(
  phaseTag: ConversationalPhaseTag | null | undefined,
  harnessPhase: HarnessPhase,
): 1 | 2 | 3 | 4 {
  switch (phaseTag) {
    case "diagnosis":
      return 1;
    case "proposal":
      return 2;
    case "execution":
      return 3;
    case "verification":
    case "direct_answer":
      return 4;
  }

  switch (harnessPhase.type) {
    case "idle":
    case "reconnaissance":
      return 1;
    case "plan_and_debate":
      return 2;
    case "guided_execution":
    case "halted_on_error":
      return 3;
    case "completed":
      return 4;
    case "paused_on_disconnect":
      return activeFourPhaseStage(null, harnessPhase.previous_phase);
  }
}

export interface EmptyHeroState {
  title: string;
  subtitle: string;
  showActivateListenBtn: boolean;
}

/** Traduce la etapa actual en una etiqueta compacta para el encabezado de respuesta. */
export function formatStageBadgeText(
  phaseTag: ConversationalPhaseTag | null | undefined,
  harnessPhase: HarnessPhase,
): string {
  if (phaseTag === "direct_answer") {
    return "Respuesta directa";
  }
  switch (activeFourPhaseStage(phaseTag, harnessPhase)) {
    case 1:
      return "Fase 1 · Diagnóstico";
    case 2:
      return "Fase 2 · Propuesta";
    case 3:
      return "Fase 3 · Ejecución";
    case 4:
      return "Fase 4 · Verificación";
  }
}

/** Construye el estado unificado del hero inicial según si la escucha está encendida o apagada. */
export function buildEmptyHeroState(profileName: string, isListening: boolean): EmptyHeroState {
  return {
    title: `Listo en ${profileName}`,
    subtitle: isListening
      ? "Escucha activa: leo tu contexto limpio para diagnosticar sin suposiciones y pedir autorización antes de cada cambio."
      : "Modo privacidad activo (Escucha OFF): no capturo salida de tu terminal hasta que enciendas la escucha abajo.",
    showActivateListenBtn: !isListening,
  };
}

/** Formatea la micro-etiqueta viva del chip de escucha con el conteo real de líneas. */
export function formatListenChipLabel(isListening: boolean, capturedLines = 0): string {
  if (!isListening) return "Escucha OFF";
  const count = Math.max(0, Math.floor(capturedLines));
  const unit = count === 1 ? "línea" : "líneas";
  return `Escucha ON · ${count} ${unit}`;
}

function decodePartialJsonString(rawSlice: string): string {
  let out = "";
  for (let i = 0; i < rawSlice.length; i += 1) {
    const ch = rawSlice[i];
    if (ch === "\"") {
      break;
    }
    if (ch === "\\") {
      if (i + 1 >= rawSlice.length) {
        break;
      }
      const next = rawSlice[i + 1];
      i += 1;
      if (next === "n") out += "\n";
      else if (next === "r") out += "\r";
      else if (next === "t") out += "\t";
      else if (next === "\"") out += "\"";
      else if (next === "\\") out += "\\";
      else if (next === "/") out += "/";
      else out += next;
      continue;
    }
    out += ch;
  }
  return out;
}

/**
 * Limpia fragmentos JSON incrementales emitidos durante el streaming de `agy`,
 * extrayendo el texto legible de `"message"` / `"reasoning"` o mostrando estado limpio.
 */
export function extractReadableStreamPreview(rawAccumulated: string): string {
  const trimmedStart = rawAccumulated.trimStart();
  if (!trimmedStart) return "";
  if (!trimmedStart.startsWith("{")) {
    return rawAccumulated;
  }

  for (const field of ["message", "reasoning", "summary", "thought"]) {
    const keyRe = new RegExp(`"${field}"\\s*:\\s*"`);
    const match = keyRe.exec(trimmedStart);
    if (match && match.index !== undefined) {
      const valueStart = match.index + match[0].length;
      const decoded = decodePartialJsonString(trimmedStart.slice(valueStart)).trimStart();
      if (decoded.length > 0) {
        return decoded;
      }
    }
  }

  return "Preparando respuesta técnica…";
}

export interface PromptHistoryStepResult {
  nextIndex: number;
  value: string;
  savedDraft: string;
}

/** Navega por el historial de prompts enviados en la sesión preservando cualquier borrador activo. */
export function stepPromptHistory(
  history: readonly string[],
  currentIndex: number,
  currentInputValue: string,
  direction: "up" | "down",
  savedDraft = "",
): PromptHistoryStepResult {
  if (history.length === 0) {
    return { nextIndex: -1, value: currentInputValue, savedDraft };
  }

  if (direction === "up") {
    if (currentIndex === -1) {
      const nextIdx = history.length - 1;
      return {
        nextIndex: nextIdx,
        value: history[nextIdx] ?? "",
        savedDraft: currentInputValue,
      };
    }
    const nextIdx = Math.max(0, currentIndex - 1);
    return {
      nextIndex: nextIdx,
      value: history[nextIdx] ?? "",
      savedDraft,
    };
  }

  if (currentIndex === -1) {
    return { nextIndex: -1, value: currentInputValue, savedDraft };
  }
  if (currentIndex >= history.length - 1) {
    return {
      nextIndex: -1,
      value: savedDraft,
      savedDraft,
    };
  }
  const nextIdx = currentIndex + 1;
  return {
    nextIndex: nextIdx,
    value: history[nextIdx] ?? "",
    savedDraft,
  };
}

/** Formatea texto seleccionado de terminal como bloque citado listo para consultar a NekoAI. */
export function formatQuotedSelectionPrompt(selectionText: string): string {
  const normalizedLines = selectionText
    .replace(/\r\n/g, "\n")
    .trim()
    .split("\n")
    .slice(-160);
  const snippet = normalizedLines.join("\n").slice(0, 10000).trim();
  if (!snippet) return "";
  return `Explica qué ocurre en este fragmento de consola y dime cómo proceder:\n\`\`\`\n${snippet}\n\`\`\`\n`;
}
