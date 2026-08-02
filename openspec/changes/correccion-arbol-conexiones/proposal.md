## Why

El árbol de Servidores mezcla un CTA rosa “Nueva conexión” con el icono de carpeta en la misma fila, y la fila de carpeta se lee como caja (borde/fondo) cuando solo las conexiones hijas deberían llevar chrome de tarjeta. Hay que corregir la jerarquía visual y el punto de entrada de acciones sin copiar el look de referencias externas: solo la idea de un header de zona con iconos.

## What Changes

- Reemplazar la toolbar `panel-actions--split` (CTA “Nueva conexión” + icono carpeta) por un **header de zona** con label **Conexiones** + icono crear conexión + icono crear carpeta (composición tipo sección de lista; copy en español latino).
- Quitar el CTA primario de ancho compartido; crear conexión y crear carpeta viven como icon-buttons del header.
- Asegurar que la **fila de carpeta** sea plana (sin caja visible: sin borde/tarjeta); el chrome de caja queda **solo en hijos** (`.profile-item`).
- Mantener el `+` por carpeta (nueva conexión en esa carpeta), expand/collapse, menús contextuales; footer conserva Snippets + engrane (mismo rol).
- Botón **Snippets**: usar el **fill primario del tema** (mismo gradiente/color que `.btn-primary` / CTA sakura), tomado de captura solo como **referencia de color** — sin copiar layout ni copy de esa captura.
- Actualizar `ui-layout-contract.md` (y tokens/look en `DESIGN.md` solo si hace falta) para reflejar header de zona + carpeta plana + look Snippets primario.

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `connection-folders`: Punto de entrada de “agregar carpeta” en el header de zona del panel; fila de carpeta sin chrome de caja (solo densidad de lista); escenarios de creación alineados al nuevo control.
- `connection-profiles`: Punto de entrada de “nueva conexión” desde el icono del header de zona (además del `+` por carpeta); reafirmar que solo las cajitas de conexión llevan tarjeta, no las carpetas padre.
- `snippets-manager`: El botón de apertura en `sidebar-footer` MUST usar el fill primario sakura del tema (paridad visual con `.btn-primary`), no el estilo ghost/outline actual.

## Impact

- UI: `app/index.html` (estructura del panel Servidores), `app/src/styles.css` (header / `.folder-row` / `.snippets-footer-btn` fill primario), `app/src/main.ts` (wire de botones e iconos Lucide).
- Docs: `docs/design/ui-layout-contract.md`; `DESIGN.md` para look del botón Snippets.
- Sin cambios de IPC/SQLite; footer cambia solo el look de Snippets (no el engrane).
- Fuera de alcance: copiar layout/copy de capturas de referencia; restaurar CTA “Nueva conexión” full-width; mover FolderPlus al footer.
