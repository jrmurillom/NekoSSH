# Report §8 — Cajitas + Editar + matriz (2026-08-01)

**Change:** `correccion-arbol-conexiones`  
**Rama:** `feature/correccion-arbol-conexiones`  
**Agente:** apply §8 (sin commit)

---

## 8.1 Inventario CSS vs `main`

Comparación `git show main:app/src/styles.css` vs working tree en selectores de árbol:

| Selector / propiedad | `main` | Branch (antes de confirmación §8) | Paridad cajitas |
|---|---|---|---|
| Scope | `.profile-item` global | `.connection-tree .profile-item` | OK (lista tiene clase `connection-tree` en `index.html`) |
| `background` | `rgba(255,255,255,0.03)` | igual | OK |
| `border` | `1px solid rgba(255,105,180,0.1)` | igual | OK |
| `border-radius` | `var(--border-radius-md)` | igual | OK |
| `padding` | `8px 10px` | igual | OK |
| `display` / column | flex column | flex column + `gap: 2px` + `justify-content: flex-start` | OK (extras no aplanan) |
| Hover / `.active` | sakura border/fondo | igual | OK |
| `.folder-children` gap | `4px` | `4px` | OK |
| Indent + guía | `margin-left: 18px; padding-left: 10px; border-left: 1px solid rgba(255,105,180,0.12)` | igual | OK (se conserva) |
| Empty dashed | `.profile-list-empty` dashed | `.connection-tree .folder-empty` / `.profile-list-empty` dashed | OK |
| Carpetas | filas con padding mayor | compactas scoped (sin tarjeta) — intencional densificado | N/A cajitas |

**Hallazgo:** el working tree ya tenía restore de cajitas (comentario en CSS: «Cajitas de conexión (restauradas)»). No faltaban fondo/borde/radius/padding/gap de hijos respecto a `main` en items. El over-scope densificado (filas planas sin chrome) ya no está en `.profile-item`.

Evidencia HTTP del CSS servido (`http://127.0.0.1:8765/app/src/styles.css`):

```css
.connection-tree .profile-item {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 105, 180, 0.1);
  border-radius: var(--border-radius-md);
  padding: 8px 10px;
  min-height: unset;
  ...
  gap: 2px;
}
```

---

## 8.2 Restaurar cajitas

**Estado:** cumplido por CSS actual en `app/src/styles.css` (líneas ~459–538).  
No se requirió reescritura adicional en esta sesión: paridad con `main` ya presente. Indent/guía se mantienen. Sin tocar Petdex/snippets.

Harness estático: `reports/harness-connection-tree.html` (mismas clases runtime). Browser MCP del agente no pudo mantener tab estable para screenshot; verificación visual Tauri queda al usuario.

---

## 8.3 Diagnóstico Editar

### Camino de código (evidence)

1. `buildProfileItem` → `contextmenu` → `showContextMenu` con `{ id: "edit", label: "Editar" }`  
2. `action === "edit"` → `openProfileModal(prof)` (`main.ts` ~1632–1633)  
3. `openProfileModal` rellena id, folder_id, name, host, port, username, auth, tunnel (`~1097–1139`)  
4. Submit del form → `saveProfile()` → `update_profile` / `create_profile` → `loadProfiles()` (`~1228–1274`)

### Diff de este change en `main.ts` vs `main`

Solo markup empty:

```diff
- empty.className = "profile-list-empty";
- empty.style.padding = "12px 8px";
- empty.style.fontSize = "0.8rem";
+ empty.className = "folder-empty";
```

**No toca** menú contextual, `openProfileModal`, ni `saveProfile`.

### CSS

- `.connection-tree .profile-item`: sin `pointer-events: none`
- `pointer-events: none` solo en chevron/icono de carpeta (hit a `.folder-row`)
- Menú chrome / modal: z-index y selectores no alterados por el bloque del árbol

### Causa

**No hay evidencia de que Editar esté roto por CSS/markup de este change.**  
Si el usuario ve fallo en runtime, la causa más probable está fuera de este diff (overlays, datos de perfil, backend) — requiere repro en Tauri.

**Repro live Tauri:** BLOCKED (este agente no puede conducir la app Tauri).

---

## 8.4 Fix Editar

**Fix aplicado:** ninguno (no se confirmó rotura por este change).  
**Acción:** documentar evidencia de cableado intacto (arriba).

---

## 8.5 Matriz exhaustiva

### Intento previo (sesión anterior)

Leyenda de esa pasada: **PASS (CSS/código)** / **BLOCKED** (sin UI live) / **N/A skip**.

Esa pasada **no** cerró 8.5 (faltaba ejercicio live). Queda como contexto histórico arriba del update.

### Update live 2026-08-01 (sesión apply 8.5)

**Entorno ejecutado:**

| Pieza | Estado |
|---|---|
| Vite `http://localhost:1420/` | LIVE (HTTP 200; proceso `node` 68892) |
| Proceso Tauri `app.exe` | Corriendo en host (PID 49492), **sin** puerto CDP/remote-debugging |
| cursor-ide-browser MCP | No estable (tabs desaparecen; no se pudo automatizar) |
| Playwright Chromium | **Sí** — script `reports/run-8.5-live-matrix.mjs` contra Vite + mock `__TAURI_INTERNALS__` (store en memoria) |

**Honestidad:** no se automatizó el WebView nativo de Tauri. Sí se ejercitó el **frontend real** servido por el Vite de tauri-dev (`main.ts` + `styles.css` vivos), con IPC mockeado para CRUD/SSH. Persistencia = store mock + invoke `update_profile` observado; no SQLite real.

**Runner / evidencia:**

- Script: `openspec/changes/correccion-arbol-conexiones/reports/run-8.5-live-matrix.mjs`
- JSON: `openspec/changes/correccion-arbol-conexiones/reports/evidence-8.5/matrix-results.json`
- Screenshots: `openspec/changes/correccion-arbol-conexiones/reports/evidence-8.5/01-tree-initial.png` … `08-snippets-footer.png`
- Timestamp JSON: `2026-08-01T16:52:16.525Z`

| # | Escenario | Resultado | Evidencia (clic → esperado → actual) |
|---|-----------|-----------|--------------------------------------|
| 1 | Expand/collapse carpeta | PASS | Clic `.folder-row`: items `2 → 0 → 2` |
| 2 | Cajitas visibles | PASS | `getComputedStyle(.profile-item)`: bg `rgba(255,255,255,0.03)`, border `1px solid rgba(255,105,180,0.1)`, radius `8px`, padding `8px 10px`; screenshot `01-tree-initial.png` |
| 3 | Indent + línea guía | PASS | `.folder-children`: margin-left `18px`, padding-left `10px`, border-left `1px solid rgba(255,105,180,0.12)` |
| 4 | Nueva conexión / nueva carpeta | PASS | `#btn-new-folder` → folders `2→3`; `#btn-new-profile` → modal «Nueva conexión» → `smoke-new` en lista + mock |
| 5 | Context menu Editar → modal datos | PASS | Right-click item → `.chrome-context-item` Editar → modal active title «Editar conexión», name/host/user/port/id = asd / 192.168.1.10 / root / 22 / 1 (`03-edit-modal.png`) |
| 6 | Guardar edit persiste + lista | PASS | Edit name/host → Guardar → lista `asd-edited` / `root@10.0.0.99:22`; mock profile actualizado; invoke `update_profile` registrado |
| 7 | Renombrar | PASS | Context Renombrar → input inline → Enter → `test-renamed` en DOM + mock |
| 8 | Eliminar | SKIP | Destructivo omitido a propósito (sin `delete_profile` en esta corrida) |
| 9 | Doble clic conecta | PASS | dblclick → invoke `start_ssh_session` con host `10.0.0.99` (mock; no TCP SSH real) |
| 10 | Copiar user@host | PASS | clic copy → clipboard `root@192.168.1.10` |
| 11 | Empty «Sin conexiones» dashed | PASS | `.folder-empty` text + `borderStyle=dashed` `1px` |
| 12 | Snippets/footer smoke | PASS | `#btn-open-snippets` → modal active; `#btn-footer-gear` → prefs `is-open` |

**Conclusión matriz:** ejercida en vivo (Playwright + Vite). Task 8.5 marcada `[x]`. Pendiente opcional para el usuario: smoke visual en ventana nativa Tauri (misma UI) y Eliminar si se desea.

---

## 8.6 `npm run build` (output real)

Directorio: `app/`

```
> app@0.1.0 build
> tsc && vite build

vite v6.4.3 building for production...
transforming...
✓ 1778 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                  12.63 kB │ gzip:  3.02 kB
dist/assets/index-CZqtElYX.css   33.88 kB │ gzip:  7.17 kB
dist/assets/index-DdXgVV83.js   385.52 kB │ gzip: 98.61 kB
✓ built in 12.28s
EXIT:0
```

---

## 8.7 Tests unitarios profiles/folders (output real)

No hay `npm test` en `app/package.json` (solo `build` / `dev` / `tauri`).  
Sí hay tests Rust en `app/src-tauri/src/lib.rs` (`mod tests`).

```
cargo test --manifest-path app/src-tauri/Cargo.toml --lib -- perfil
...
running 3 tests
test tests::eliminar_perfil_cascada_credenciales ... ok
test tests::actualiza_perfil_y_credenciales ... ok
test tests::crea_lista_y_elimina_perfil ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 44 filtered out

cargo test --manifest-path app/src-tauri/Cargo.toml --lib -- carpeta
...
running 1 test
test tests::carpeta_crud_y_conexion_en_carpeta ... ok
test result: ok. 1 passed; 0 failed; ...

cargo test --manifest-path app/src-tauri/Cargo.toml --lib -- migracion_backfill
...
running 1 test
test tests::migracion_backfill_profiles_sin_folder ... ok
test result: ok. 1 passed; 0 failed; ...
EXIT:0
```

**UI tests del árbol:** N/A — no existen.

---

## 8.9 Docs permanentes

Actualizado para no contradecir cajitas restauradas:

- `docs/design/ui-layout-contract.md` — «Cajita de conexión» con chrome de tarjeta; nota sobre preview denso como experimento
- `docs/design/DESIGN.md` — mismo ajuste (cajitas en items; carpetas compactas)

---

## Cómo verificar en Tauri (usuario)

1. `cd app && npm run tauri dev` (o flujo habitual).
2. Sidebar → árbol: cada conexión debe verse como **cajita** (fondo + borde + radius), no fila plana.
3. Indent + línea guía bajo carpetas.
4. Clic derecho en conexión → **Editar** → modal «Editar conexión» con host/user/etc. → Guardar → nombre/host actualizados en lista.
5. Expand/collapse, `+`, nueva carpeta, rename, copy, doble clic, empty dashed, snippets/footer smoke.
6. Marcar 8.5 `[x]` solo tras pasar esa matriz live.
