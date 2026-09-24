export type TabContextMenuAction =
  | "close"
  | "close-others"
  | "close-left"
  | "close-right"
  | "close-all";

export interface TabContextMenuItemDescriptor {
  id: TabContextMenuAction;
  label: string;
  disabled: boolean;
  danger?: boolean;
  separatorBefore?: boolean;
}

export interface BulkTabCloseConfirmSpec {
  title: string;
  message: string;
  confirmLabel: string;
  danger: true;
}

/**
 * Construye los descriptores del menú contextual de pestañas estilo VS Code.
 * Las opciones sin pestañas aplicables permanecen visibles pero deshabilitadas (`disabled: true`).
 */
export function buildTabContextMenuState(
  orderedIds: readonly string[],
  targetId: string,
): TabContextMenuItemDescriptor[] {
  const index = orderedIds.indexOf(targetId);
  const exists = index >= 0;
  const hasLeft = exists && index > 0;
  const hasRight = exists && index < orderedIds.length - 1;
  const hasOthers = exists && orderedIds.length > 1;

  return [
    {
      id: "close",
      label: "Cerrar pestaña",
      disabled: !exists,
    },
    {
      id: "close-others",
      label: "Cerrar otras pestañas",
      disabled: !hasOthers,
    },
    {
      id: "close-left",
      label: "Cerrar pestañas a la izquierda",
      disabled: !hasLeft,
      separatorBefore: true,
    },
    {
      id: "close-right",
      label: "Cerrar pestañas a la derecha",
      disabled: !hasRight,
    },
    {
      id: "close-all",
      label: "Cerrar todas las pestañas",
      disabled: orderedIds.length === 0,
      danger: true,
      separatorBefore: true,
    },
  ];
}

/**
 * Calcula la lista ordenada de IDs de pestañas que deben cerrarse según la acción elegida.
 */
export function resolveTabsToClose(
  orderedIds: readonly string[],
  targetId: string,
  action: TabContextMenuAction,
): string[] {
  const index = orderedIds.indexOf(targetId);
  if (index < 0) {
    return action === "close-all" ? [...orderedIds] : [];
  }

  switch (action) {
    case "close":
      return [targetId];
    case "close-others":
      return orderedIds.filter((id) => id !== targetId);
    case "close-left":
      return orderedIds.slice(0, index);
    case "close-right":
      return orderedIds.slice(index + 1);
    case "close-all":
      return [...orderedIds];
  }
}

/**
 * Construye el texto del diálogo de confirmación consolidado cuando se cierran múltiples pestañas
 * que contienen al menos una sesión SSH conectada.
 */
export function buildBulkTabCloseConfirm(
  action: Exclude<TabContextMenuAction, "close">,
  totalToClose: number,
  connectedCount: number,
): BulkTabCloseConfirmSpec {
  const sessionNoun = connectedCount === 1 ? "sesión SSH activa" : "sesiones SSH activas";
  const tabNoun = totalToClose === 1 ? "pestaña" : "pestañas";

  switch (action) {
    case "close-left":
      return {
        title: "¿Cerrar pestañas a la izquierda?",
        message: `Hay ${connectedCount} ${sessionNoun} a la izquierda. ¿Seguro que deseas cerrar ${totalToClose} ${tabNoun}?`,
        confirmLabel: "Cerrar a la izquierda",
        danger: true,
      };
    case "close-right":
      return {
        title: "¿Cerrar pestañas a la derecha?",
        message: `Hay ${connectedCount} ${sessionNoun} a la derecha. ¿Seguro que deseas cerrar ${totalToClose} ${tabNoun}?`,
        confirmLabel: "Cerrar a la derecha",
        danger: true,
      };
    case "close-others":
      return {
        title: "¿Cerrar otras pestañas?",
        message: `Hay ${connectedCount} ${sessionNoun} en las otras pestañas. ¿Seguro que deseas cerrar ${totalToClose} ${tabNoun}?`,
        confirmLabel: "Cerrar otras",
        danger: true,
      };
    case "close-all":
      return {
        title: "¿Cerrar todas las terminales?",
        message: `Hay ${connectedCount} ${sessionNoun}. ¿Seguro que deseas cerrar todas las pestañas?`,
        confirmLabel: "Cerrar todas",
        danger: true,
      };
  }
}

/**
 * Determina qué pestaña debe quedar activa después de cerrar un subconjunto de pestañas.
 * Si la pestaña activa anterior formaba parte del subconjunto eliminado (y la pestaña clicada sobrevive),
 * el foco pasa automáticamente a la pestaña donde se abrió el menú contextual (`targetId`).
 */
export function resolveNextActiveTabAfterBulkClose(
  currentActiveId: string | null,
  targetId: string,
  closedIds: readonly string[],
): string | null {
  const closedSet = new Set(closedIds);
  if (closedSet.has(targetId)) {
    return null;
  }
  if (!currentActiveId || closedSet.has(currentActiveId)) {
    return targetId;
  }
  return currentActiveId;
}
