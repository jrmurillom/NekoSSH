import { describe, expect, it } from "vitest";
import {
  activeFourPhaseStage,
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
  MAX_AUTO_CHAIN_STEPS,
  planFirstTaskToPendingAction,
  resolveTargetShell,
  sanitizeEditedCommand,
  shouldAutoRunReadOnlyStep,
  stepPromptHistory,
  type ActiveShellsSnapshot,
  type SessionUiState,
} from "./copilot-state";

describe("copilot-state: resolución de shell y Pin", () => {
  const snapshot: ActiveShellsSnapshot = {
    shells: [
      {
        terminal_id: "term-1",
        context_id: "term-1",
        profile_name: "Prod",
        label: "Principal",
        is_connected: true,
        is_listening: true,
      },
      {
        terminal_id: "term-1-s1",
        context_id: "term-1",
        profile_name: "Prod",
        label: "Shell 2",
        is_connected: true,
        is_listening: false,
      },
    ],
    focused_terminal_id: "term-1-s1",
  };

  it("sigue al foco actual cuando Pin está desactivado", () => {
    const next = resolveTargetShell(
      { targetTerminalId: "term-1", pinned: false },
      snapshot,
    );
    expect(next).toEqual({ targetTerminalId: "term-1-s1", pinned: false });
  });

  it("mantiene el shell fijado aunque cambie el foco cuando Pin está activo", () => {
    const next = resolveTargetShell(
      { targetTerminalId: "term-1", pinned: true },
      snapshot,
    );
    expect(next).toEqual({ targetTerminalId: "term-1", pinned: true });
  });

  it("libera el Pin si el shell fijado fue cerrado", () => {
    const next = resolveTargetShell(
      { targetTerminalId: "term-cerrado", pinned: true },
      snapshot,
    );
    expect(next).toEqual({ targetTerminalId: "term-1-s1", pinned: false });
  });
});

describe("copilot-state: guardrails de auto-ejecución en 4 fases", () => {
  it("nunca auto-ejecuta en modo estricto", () => {
    expect(
      shouldAutoRunReadOnlyStep({
        mode: "strict",
        autoApprovedByBackend: true,
        response: {
          kind: "propose_step",
          reasoning: "Ver memoria",
          command: "free -h",
          risk_level: "read_only",
        },
        autoStepsUsed: 0,
      }),
    ).toBe(false);
  });

  it("auto-ejecuta lecturas puras verificadas en modo flexible sin advertencias", () => {
    expect(
      shouldAutoRunReadOnlyStep({
        mode: "flexible",
        autoApprovedByBackend: true,
        response: {
          kind: "propose_step",
          reasoning: "Ver OS",
          command: "cat /etc/os-release 2>/dev/null || uname -a",
          risk_level: "read_only",
          audit_warning: null,
        },
        autoStepsUsed: 0,
      }),
    ).toBe(true);

    expect(
      shouldAutoRunReadOnlyStep({
        mode: "flexible",
        autoApprovedByBackend: true,
        response: {
          kind: "propose_step",
          reasoning: "Tope alcanzado",
          command: "df -h",
          risk_level: "read_only",
        },
        autoStepsUsed: MAX_AUTO_CHAIN_STEPS,
      }),
    ).toBe(false);
  });

  it("frena y pide permiso si hay audit_warning o riesgo mutante aunque esté en modo flexible", () => {
    expect(
      shouldAutoRunReadOnlyStep({
        mode: "flexible",
        autoApprovedByBackend: false,
        response: {
          kind: "propose_step",
          reasoning: "Exportar reporte",
          command: "ss -tulpn > /tmp/ports.txt",
          risk_level: "config_change",
          audit_warning: "Redirección a archivo detectada",
        },
        autoStepsUsed: 0,
      }),
    ).toBe(false);
  });
});

describe("copilot-state: tarjetas explicativas, etapas y utilidades", () => {
  it("redacta la pregunta explicativa natural a partir del propósito", () => {
    expect(
      buildCommandQuestionText("Identificar la distribución Linux y kernel", "read_only"),
    ).toBe(
      "¿Autorizas ejecutar este comando de lectura para identificar la distribución Linux y kernel?",
    );
    expect(
      buildCommandQuestionText("Crear respaldo fechado de nginx.conf", "config_change"),
    ).toBe("¿Autorizas ejecutar este comando para crear respaldo fechado de nginx.conf?");
  });

  it("mapea correctamente las 4 fases adaptativas", () => {
    expect(activeFourPhaseStage("diagnosis", { type: "idle" })).toBe(1);
    expect(activeFourPhaseStage("proposal", { type: "idle" })).toBe(2);
    expect(activeFourPhaseStage("execution", { type: "idle" })).toBe(3);
    expect(activeFourPhaseStage("verification", { type: "idle" })).toBe(4);
    expect(activeFourPhaseStage("direct_answer", { type: "idle" })).toBe(4);
  });

  it("escapa XSS y valida edición inline", () => {
    expect(escapeHtmlText('<script>alert("x")</script>')).toBe(
      "&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;",
    );
    expect(sanitizeEditedCommand("   ")).toBeNull();
    expect(sanitizeEditedCommand("  nginx -t  ")).toBe("nginx -t");
    expect(formatRiskLabel("read_only").text).toBe("Solo lectura · No modifica");
    expect(formatRiskLabel("config_change").text).toBe("Modifica archivos / config");
    expect(formatRiskLabel("service_impact").text).toBe("Impacto en servicio");
    expect(formatRiskLabel("destructive")).toEqual({
      text: "Crítico · Destructivo",
      cssModifier: "is-destructive",
    });
  });

  it("formatea etiquetas breves de etapa y unifica el estado vacío", () => {
    expect(formatStageBadgeText("direct_answer", { type: "idle" })).toBe("Respuesta directa");
    expect(formatStageBadgeText("diagnosis", { type: "idle" })).toBe("Fase 1 · Diagnóstico");
    expect(formatStageBadgeText("proposal", { type: "idle" })).toBe("Fase 2 · Propuesta");
    expect(formatStageBadgeText("execution", { type: "idle" })).toBe("Fase 3 · Ejecución");
    expect(formatStageBadgeText("verification", { type: "idle" })).toBe("Fase 4 · Verificación");

    const privHero = buildEmptyHeroState("Sekura TEST", false);
    expect(privHero.title).toBe("Listo en Sekura TEST");
    expect(privHero.showActivateListenBtn).toBe(true);

    const activeHero = buildEmptyHeroState("Sekura TEST", true);
    expect(activeHero.showActivateListenBtn).toBe(false);
  });

  it("aisla tarjetas pendientes y rollbacks por terminal_id sin sobrescribir entre pestañas", () => {
    const store = new Map<string, SessionUiState>();
    const s1 = getOrCreateSessionUiState(store, "term-1");
    s1.pendingAction = {
      purpose: "probar nginx",
      originalCommand: "nginx -t",
      draftCommand: "nginx -t",
      isEditing: false,
      riskLevel: "read_only",
      auditWarning: null,
      backupStrategy: null,
      rollbackCmd: null,
    };

    const s2 = getOrCreateSessionUiState(store, "term-2");
    expect(s2).toEqual(createEmptySessionUiState());
    expect(getOrCreateSessionUiState(store, "term-1").pendingAction?.originalCommand).toBe("nginx -t");
  });

  it("convierte respuestas ProposePlan en resumen Markdown y tarjeta pendiente accionable", () => {
    const tasks = [
      {
        id: 1,
        title: "Respaldar nginx.conf",
        why: "Conservar copia segura antes de editar",
        command: "cp /etc/nginx/nginx.conf /etc/nginx/nginx.conf.bak",
        risk_level: "config_change" as const,
        backup_strategy: "Copia local .bak",
        rollback_cmd: "rm -f /etc/nginx/nginx.conf.bak",
      },
      {
        id: 2,
        title: "Validar y recargar",
        why: "Evitar caída de servicio",
        command: "nginx -t && systemctl reload nginx",
        risk_level: "service_impact" as const,
        backup_strategy: "nginx -t previo",
        rollback_cmd: "cp /etc/nginx/nginx.conf.bak /etc/nginx/nginx.conf && systemctl reload nginx",
      },
    ];
    const md = formatPlanSummaryMarkdown("Plan seguro para actualizar Nginx", tasks);
    expect(md).toContain("Plan seguro para actualizar Nginx");
    expect(md).toContain("1. **Respaldar nginx.conf**");
    expect(md).toContain("`nginx -t && systemctl reload nginx`");

    const pending = planFirstTaskToPendingAction(tasks);
    expect(pending?.originalCommand).toBe("cp /etc/nginx/nginx.conf /etc/nginx/nginx.conf.bak");
    expect(pending?.rollbackCmd).toBe("rm -f /etc/nginx/nginx.conf.bak");
  });

  it("limpia streaming JSON parcial, formatea conteo de escucha, historial y selección citada", () => {
    expect(extractReadableStreamPreview('{"kind":"propose_step","phase_tag":"diag')).toBe(
      "Preparando respuesta técnica…",
    );
    expect(
      extractReadableStreamPreview(
        '{"kind":"propose_step","message":"Revisando logs de Nginx\\nen /var/log/nginx',
      ),
    ).toBe("Revisando logs de Nginx\nen /var/log/nginx");
    expect(
      extractReadableStreamPreview('{"kind":"propose_plan","reasoning":"Plan listo","tasks":[]}'),
    ).toBe("Plan listo");
    expect(extractReadableStreamPreview("Texto directo sin JSON")).toBe("Texto directo sin JSON");

    expect(formatListenChipLabel(false, 50)).toBe("Escucha OFF");
    expect(formatListenChipLabel(true, 0)).toBe("Escucha ON · 0 líneas");
    expect(formatListenChipLabel(true, 1)).toBe("Escucha ON · 1 línea");
    expect(formatListenChipLabel(true, 84)).toBe("Escucha ON · 84 líneas");

    const hist = ["uname -a", "systemctl status nginx"];
    const step1 = stepPromptHistory(hist, -1, "borrador", "up");
    expect(step1).toEqual({
      nextIndex: 1,
      value: "systemctl status nginx",
      savedDraft: "borrador",
    });
    const step2 = stepPromptHistory(hist, step1.nextIndex, step1.value, "up", step1.savedDraft);
    expect(step2.nextIndex).toBe(0);
    expect(step2.value).toBe("uname -a");
    const stepBack = stepPromptHistory(hist, step2.nextIndex, step2.value, "down", step2.savedDraft);
    expect(stepBack.value).toBe("systemctl status nginx");
    const stepDraft = stepPromptHistory(
      hist,
      stepBack.nextIndex,
      stepBack.value,
      "down",
      stepBack.savedDraft,
    );
    expect(stepDraft).toEqual({ nextIndex: -1, value: "borrador", savedDraft: "borrador" });

    const quoted = formatQuotedSelectionPrompt("  nginx: [emerg] bind() failed \n");
    expect(quoted).toContain("```\nnginx: [emerg] bind() failed\n```");
    expect(quoted).toContain("Explica");
  });
});
