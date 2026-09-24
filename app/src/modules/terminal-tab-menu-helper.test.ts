import { describe, expect, it } from "vitest";
import {
  buildBulkTabCloseConfirm,
  buildTabContextMenuState,
  resolveNextActiveTabAfterBulkClose,
  resolveTabsToClose,
  type TabContextMenuAction,
} from "./terminal-tab-menu-helper";

describe("terminal-tab-menu-helper", () => {
  const fourTabs = ["tab-auth", "tab-db", "tab-api", "tab-cdn"];

  it("habilita y deshabilita correctamente las opciones en los extremos (izquierda, centro, derecha) y con pestaña única", () => {
    // 1. Primera pestaña (extremo izquierdo): "close-left" deshabilitado, "close-right" y "close-others" habilitados
    const leftEdge = buildTabContextMenuState(fourTabs, "tab-auth");
    expect(leftEdge.map((i) => ({ id: i.id, disabled: i.disabled }))).toEqual([
      { id: "close", disabled: false },
      { id: "close-others", disabled: false },
      { id: "close-left", disabled: true },
      { id: "close-right", disabled: false },
      { id: "close-all", disabled: false },
    ]);

    // 2. Pestaña intermedia: todas las opciones habilitadas
    const middle = buildTabContextMenuState(fourTabs, "tab-db");
    expect(middle.every((i) => !i.disabled)).toBe(true);

    // 3. Última pestaña (extremo derecho): "close-right" deshabilitado, "close-left" y "close-others" habilitados
    const rightEdge = buildTabContextMenuState(fourTabs, "tab-cdn");
    expect(rightEdge.map((i) => ({ id: i.id, disabled: i.disabled }))).toEqual([
      { id: "close", disabled: false },
      { id: "close-others", disabled: false },
      { id: "close-left", disabled: false },
      { id: "close-right", disabled: true },
      { id: "close-all", disabled: false },
    ]);

    // 4. Pestaña única: "close-others", "close-left" y "close-right" deshabilitados
    const singleTab = buildTabContextMenuState(["tab-only"], "tab-only");
    expect(singleTab.map((i) => ({ id: i.id, disabled: i.disabled }))).toEqual([
      { id: "close", disabled: false },
      { id: "close-others", disabled: true },
      { id: "close-left", disabled: true },
      { id: "close-right", disabled: true },
      { id: "close-all", disabled: false },
    ]);
  });

  it("resuelve exactamente los subconjuntos de pestañas a cerrar para cada acción", () => {
    expect(resolveTabsToClose(fourTabs, "tab-api", "close")).toEqual(["tab-api"]);
    expect(resolveTabsToClose(fourTabs, "tab-api", "close-left")).toEqual([
      "tab-auth",
      "tab-db",
    ]);
    expect(resolveTabsToClose(fourTabs, "tab-api", "close-right")).toEqual([
      "tab-cdn",
    ]);
    expect(resolveTabsToClose(fourTabs, "tab-api", "close-others")).toEqual([
      "tab-auth",
      "tab-db",
      "tab-cdn",
    ]);
    expect(resolveTabsToClose(fourTabs, "tab-api", "close-all")).toEqual(fourTabs);

    // Extremos devuelven lista vacía para su dirección
    expect(resolveTabsToClose(fourTabs, "tab-auth", "close-left")).toEqual([]);
    expect(resolveTabsToClose(fourTabs, "tab-cdn", "close-right")).toEqual([]);
  });

  it("construye el diálogo de confirmación consolidado con pluralización precisa en español", () => {
    const rightConfirm = buildBulkTabCloseConfirm("close-right", 3, 2);
    expect(rightConfirm).toEqual({
      title: "¿Cerrar pestañas a la derecha?",
      message:
        "Hay 2 sesiones SSH activas a la derecha. ¿Seguro que deseas cerrar 3 pestañas?",
      confirmLabel: "Cerrar a la derecha",
      danger: true,
    });

    const leftSingular = buildBulkTabCloseConfirm("close-left", 1, 1);
    expect(leftSingular).toEqual({
      title: "¿Cerrar pestañas a la izquierda?",
      message:
        "Hay 1 sesión SSH activa a la izquierda. ¿Seguro que deseas cerrar 1 pestaña?",
      confirmLabel: "Cerrar a la izquierda",
      danger: true,
    });

    const othersConfirm = buildBulkTabCloseConfirm("close-others", 4, 3);
    expect(othersConfirm.title).toBe("¿Cerrar otras pestañas?");
    expect(othersConfirm.confirmLabel).toBe("Cerrar otras");
  });

  it("transfiere el foco a la pestaña clicada si la pestaña activa fue cerrada y lo preserva si no fue cerrada", () => {
    // Caso 1: La pestaña activa ("tab-cdn") estaba a la derecha de "tab-db" y se cierra -> foco pasa a "tab-db"
    expect(
      resolveNextActiveTabAfterBulkClose("tab-cdn", "tab-db", ["tab-api", "tab-cdn"]),
    ).toBe("tab-db");

    // Caso 2: La pestaña activa ("tab-auth") estaba a la izquierda de "tab-db" cuando se cierran las de la derecha -> se preserva "tab-auth"
    expect(
      resolveNextActiveTabAfterBulkClose("tab-auth", "tab-db", ["tab-api", "tab-cdn"]),
    ).toBe("tab-auth");

    // Caso 3: Se cierran todas -> retorna null
    expect(
      resolveNextActiveTabAfterBulkClose("tab-db", "tab-db", fourTabs),
    ).toBeNull();
  });

  it("ejecuta el flujo E2E de cierre masivo direccional con confirmación única, cancelación segura y transición de foco", async () => {
    const openTabs = new Map<string, { isConnected: boolean }>([
      ["tab-1", { isConnected: true }],
      ["tab-2", { isConnected: true }],
      ["tab-3", { isConnected: true }],
      ["tab-4", { isConnected: false }],
    ]);
    let currentActiveId: string | null = "tab-4";
    const confirmPrompts: string[] = [];

    async function executeContextMenuAction(
      targetId: string,
      action: Exclude<TabContextMenuAction, "close">,
      userConfirms: boolean,
    ) {
      const orderedIds = Array.from(openTabs.keys());
      const idsToClose = resolveTabsToClose(orderedIds, targetId, action);
      if (idsToClose.length === 0) return;

      const connectedCount = idsToClose.filter(
        (id) => openTabs.get(id)?.isConnected,
      ).length;

      if (connectedCount > 0) {
        const spec = buildBulkTabCloseConfirm(
          action,
          idsToClose.length,
          connectedCount,
        );
        confirmPrompts.push(spec.message);
        if (!userConfirms) {
          return;
        }
      }

      const nextActive = resolveNextActiveTabAfterBulkClose(
        currentActiveId,
        targetId,
        idsToClose,
      );

      for (const id of idsToClose) {
        openTabs.delete(id);
      }
      currentActiveId = nextActive;
    }

    // 1. Usuario hace clic derecho en "tab-2", elige "Cerrar pestañas a la derecha", pero CANCELA el confirmDialog
    await executeContextMenuAction("tab-2", "close-right", false);
    expect(confirmPrompts).toHaveLength(1);
    expect(confirmPrompts[0]).toContain("Hay 1 sesión SSH activa a la derecha");
    expect(Array.from(openTabs.keys())).toEqual(["tab-1", "tab-2", "tab-3", "tab-4"]);
    expect(currentActiveId).toBe("tab-4");

    // 2. Usuario repite "Cerrar pestañas a la derecha" en "tab-2" y CONFIRMA -> se cierran tab-3 y tab-4 con UN SOLO prompt, y el foco pasa a tab-2
    await executeContextMenuAction("tab-2", "close-right", true);
    expect(confirmPrompts).toHaveLength(2);
    expect(Array.from(openTabs.keys())).toEqual(["tab-1", "tab-2"]);
    expect(currentActiveId).toBe("tab-2");
  });
});
