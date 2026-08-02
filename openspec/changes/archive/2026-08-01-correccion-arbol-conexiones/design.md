## Context

Hoy el panel Servidores usa `.panel-actions--split` con CTA `#btn-new-profile` (“Nueva conexión”) + `#btn-new-folder`. Las specs (`connection-folders`, `connection-profiles`) ya piden carpeta en densidad de lista sin chrome de tarjeta, pero `.folder-row` aún aplica borde/radius y los hace visibles en hover/activo — se lee como caja. El footer (Snippets + engrane) no entra en este change.

Referencia visual externa: solo la **composición** de un header de zona con label + iconos de crear conexión/carpeta. No se copia look, tipografía ni densidad de esa app.

## Goals / Non-Goals

**Goals:**

- Header de zona encima del árbol: label **Connections** + icon-button crear conexión + icon-button crear carpeta.
- Misma lógica de negocio que hoy (`createNewFolder`, flujo de nueva conexión / modal); solo cambia el control y el lugar.
- Fila de carpeta visualmente plana (sin caja); cajita solo en `.profile-item`.
- Actualizar contrato de layout para el patrón del header de zona.

**Non-Goals:**

- Copiar estética de la captura de referencia (más allá del patrón header + iconos).
- Mover FolderPlus al footer o meter acciones de árbol en `sidebar-footer`.
- CTA “Nueva conexión” full-width como botón primario dominante.
- Cambios IPC/SQLite, menús contextuales, `+` por carpeta, o pestaña Archivos.

## Decisions

1. **Header de zona en `#panel-servers`, no en el shell global**
   - **Decisión:** Reemplazar `.panel-actions--split` por un bloque (p. ej. `.connections-zone-header`) solo visible en el panel Servidores.
   - **Motivo:** Crear conexión/carpeta son acciones del árbol; no deben aparecer en Archivos.
   - **Alternativa descartada:** Franja encima del footer — mezcla roles con Snippets/prefs.

2. **Label “Connections” + dos icon-buttons**
   - **Decisión:** Label a la izquierda; a la derecha icono crear conexión (p. ej. Plus / ServerPlus) e icono crear carpeta (FolderPlus), ambos `btn-icon-action` o equivalente Lucide outline.
   - **Motivo:** Pedido explícito del explore: composición tipo Connections; sin CTA rosa de texto largo.
   - **Alternativa descartada:** Mantener CTA “Nueva conexión” full width + carpeta aparte.

3. **Wire: mismos handlers, nuevos ids si hace falta**
   - **Decisión:** Reutilizar listeners actuales (`btn-new-profile` / `btn-new-folder` o renombrar ids en HTML + `main.ts` de forma mínima). Nueva conexión desde el header usa el mismo flujo que el botón actual (carpeta destino = contexto activo / reglas existentes). El `+` en cada `.folder-row` se conserva.
   - **Motivo:** Cero cambio de dominio; solo chrome.

4. **Carpeta sin caja (CSS)**
   - **Decisión:** `.folder-row` sin `border` de tarjeta (ni transparente que se pinte en hover/activo como caja). Hover/activo = tint de fondo plano opcional, sin `border-color` que dibuje rectángulo. Conservar chevron + icono + nombre + `+`.
   - **Motivo:** Alinear implementación con el requisito ya escrito; la percepción de “caja” viene del borde.
   - **Alternativa descartada:** Quitar también el tint de hover — empeora affordance de click en toda la fila.

5. **Hijos = única cajita**
   - **Decisión:** No tocar el modelo de `.profile-item` como tarjeta; solo verificar contraste visual carpeta plana vs hijo con caja.
   - **Motivo:** Spec de cajita ya es SSOT.

6. **Docs**
   - **Decisión:** Actualizar `ui-layout-contract.md` § árbol (header de zona + iconos; carpeta sin caja). `DESIGN.md` solo si el header introduce un patrón de componente reutilizable no documentado.

## Risks / Trade-offs

- **[Riesgo] Label en inglés “Connections” vs regla UI en español latino** → Mitigación: pedido explícito del explore; si en apply se prefiere `Conexiones`, es un cambio de copy de una línea sin tocar estructura.
- **[Riesgo] Nueva conexión desde header sin carpeta de contexto clara** → Mitigación: reutilizar la regla actual del CTA (carpeta activa / default); no inventar flujo nuevo.
- **[Riesgo] Regresión de densidad / click en fila** → Mitigación: checklist desktop-ui (expand/collapse, `+` con stopPropagation, crear desde header).
- **[Trade-off] Menos énfasis sakura en “crear conexión”** → Aceptado: iconos en header priorizan jerarquía del árbol sobre CTA marketing.

## Migration Plan

- Cambio solo frontend chrome; sin migración de datos.
- Rollback: restaurar `panel-actions--split` + estilos previos de `.folder-row`.

## Open Questions

- ~~Copy final del label: Connections vs Conexiones~~ → **Resuelto en Corrección de Ruta (Fix):** label UI = **Conexiones**.

### Corrección de Ruta (Fix)

**Problema:** El apply dejó el label visible del header de zona como **Connections** (inglés). Eso viola la SSOT de idioma: UI visible al usuario MUST estar en **español latino** (`docs/base-standards.md`). La captura de referencia solo aportaba la *composición* (label de zona + iconos), no el copy en inglés.

**Estrategia:**
1. Copy visible del header: **Conexiones** (no “Connections”).
2. Identificadores de código (clases CSS `.connections-zone-header`, ids) permanecen en inglés — eso sí es correcto por la regla bilingüe.
3. Actualizar delta specs, contrato de layout y `DESIGN.md` para que el label documentado sea **Conexiones**.
4. Re-verificar desktop-ui con el copy corregido.

**Zombie:** El string `Connections` en `app/index.html` (y docs/specs que lo repiten) queda invalidado; las tareas §5 lo corrigen.

### Corrección de Ruta (Fix) — color primario en Snippets

**Problema / pedido:** Captura de **referencia** (no copiar layout ni copy). Tomar solo el **color de relleno primario del tema** usado por el CTA histórico “Nueva conexión” (`.btn-primary` → `linear-gradient(135deg, var(--color-sakura-neon), #d82b7d)`, texto blanco) y aplicarlo al botón **Snippets** del `sidebar-footer`. Hoy Snippets usa fondo oscuro semitransparente + borde sakura (ghost/outline), no el fill primario sólido.

**Estrategia:**
1. Estilos de `.sidebar-footer .snippets-footer-btn`: mismo fill/gradiente y color de texto que `.btn-primary` (tokens del tema). Hover puede reutilizar brightness del primario.
2. **No** restaurar el CTA “Nueva conexión” en el árbol; **no** copiar la captura (solo el color).
3. Engrane (gear) sin cambio de rol/estilo primario.
4. Actualizar look en `DESIGN.md` / contrato si documentan el botón Snippets; delta `snippets-manager`.

**Zombie:** Estilos “ghost sakura” de `.snippets-footer-btn` quedan sustituidos por fill primario; las tareas §6 lo implementan.
