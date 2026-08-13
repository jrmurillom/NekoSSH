/**
 * Filtro por nombre para el árbol del explorador SFTP.
 *
 * Opera solo sobre el árbol ya pintado (cliente): un nodo se considera visible
 * si su nombre coincide con la consulta (substring, sin distinguir mayúsculas)
 * o si algún descendiente bajo la expansión actual coincide. Los hijos de
 * carpetas colapsadas o no cargadas no se inspeccionan (no están pintados).
 */

export interface FilterableNode {
  name: string;
  isDir: boolean;
  expanded: boolean;
  children: FilterableNode[];
}

/** Indica si la consulta de filtro tiene contenido significativo. */
export function isFilterActive(query: string): boolean {
  return query.trim().length > 0;
}

/**
 * Coincidencia por nombre: substring sin distinguir mayúsculas.
 * Una consulta vacía coincide con todo.
 */
export function nodeNameMatches(name: string, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return name.toLowerCase().includes(q);
}

/**
 * Calcula el conjunto de nodos visibles (por referencia) dado un conjunto de
 * raíces pintadas y una consulta. Solo se recorre dentro de carpetas
 * expandidas, reflejando lo que realmente se dibuja en el árbol.
 *
 * - Consulta vacía: todos los nodos pintados quedan visibles.
 * - Nodo hoja: visible si su nombre coincide.
 * - Carpeta: visible si su nombre coincide o si algún descendiente visible
 *   coincide (para conservar la jerarquía hacia la coincidencia).
 */
export function computeVisibleNodes<T extends FilterableNode>(
  roots: T[],
  query: string,
): Set<T> {
  const visible = new Set<T>();
  const q = query.trim().toLowerCase();

  const walk = (node: T): boolean => {
    let anyChildVisible = false;
    if (node.isDir && node.expanded) {
      for (const child of node.children as T[]) {
        if (walk(child)) anyChildVisible = true;
      }
    }
    const selfMatch = !q || node.name.toLowerCase().includes(q);
    const nodeVisible = selfMatch || anyChildVisible;
    if (nodeVisible) visible.add(node);
    return nodeVisible;
  };

  for (const root of roots) walk(root);
  return visible;
}
