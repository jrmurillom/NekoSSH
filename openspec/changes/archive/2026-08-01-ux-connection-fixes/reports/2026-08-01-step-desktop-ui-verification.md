# Reporte de Evidencia de Validación de Interfaz Desktop (UI Verification)

**Fecha:** 2026-08-01  
**Cambio:** `ux-connection-fixes`

## 1. Verificación de Compilación del Frontend (`npm run build`)

### Comando Ejecutado
`npm run build` en la carpeta `app`.

### Output
```text
> app@0.1.0 build
> tsc && vite build

vite v6.4.3 building for production...
transforming...
✓ 1781 modules transformed.
rendering chunks...
dist/index.html                  13.48 kB │ gzip:  3.16 kB
dist/assets/logo-Lk_-B2Z9.png    26.78 kB
dist/assets/index-DnUYwuZy.css   34.74 kB │ gzip:  7.35 kB
dist/assets/index-C_3uGSnj.js   386.69 kB │ gzip: 99.00 kB
✓ built in 1.77s
```
* **Estado:** Exitoso.

## 2. Checklist de Verificación de Interacción y UX

1. **Árbol Colapsado por Defecto:**
   - **Verificación:** Se removió la auto-expansión inicial en `loadProfiles()`. Al abrir la app, `expandedFolderIds` inicia como `Set` vacío y todas las carpetas se muestran colapsadas hasta que el usuario las expanda manualmente.
2. **Sin Tinte Persistente en Categorías Padres:**
   - **Verificación:** Se cambió `.connection-tree .folder-row.is-active-context` a `background: transparent;`. Al hacer clic en las filas de carpetas para desplegarlas/colapsarlas o activarlas como contexto, el fondo permanece transparente sin dejar ningún tintero o resplandor activo rosa retido.
3. **Diálogo de Confirmación para Conexiones Vivas:**
   - **Verificación:** `closeTerminalSession` evalúa `activeTerm.isConnected`. Si la conexión está viva, se despliega el `confirmDialog` glass antes de cerrar la pestaña. Si está desconectada, se cierra sin dialog. En `closeAllTerminals()`, si hay terminales vivas, se solicita confirmación global una sola vez.
4. **Inhabilitación Global del Menú Contextual del Navegador:**
   - **Verificación:** Se registró `document.addEventListener("contextmenu", (e) => e.preventDefault())`. Al hacer clic derecho en zonas neutras (sidebar header, background, workspace vacío), no aparece el menú nativo del browser (WebView2).
5. **Selector Nativo del SO para Claves Privadas (.ppk, .pem, etc.):**
   - **Verificación:** Se integró el botón `<button id="btn-browse-key">Examinar...</button>` en `#auth-key-group` junto con un `<input type="file" id="file-input-key">` oculto. Al activarlo, abre el explorador nativo de archivos del SO y escribe la ruta elegida directamente en `#prof-key-path`.
