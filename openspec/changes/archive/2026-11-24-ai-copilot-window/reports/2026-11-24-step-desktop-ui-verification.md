# Reporte de Verificación Desktop UI (Step 8)

- **Change:** `ai-copilot-window` (Hito 3)
- **Fecha:** 2026-11-24
- **Superficie:** `desktop-ui`
- **Entorno de Verificación:** Build productivo `tsc && vite build`, auditoría estática HTML/CSS, renderizado real en navegador headless (`msedge.exe --headless --dump-dom` sobre `vite preview --port 4179`) y compilación nativa `cargo check --bin app`.

## 1. Cobertura de Superficies UI

### 1.1 Ventana Principal (`app/index.html` + `app/src/main.ts`)
- Botón `#btn-footer-ai` agregado en `.footer-actions` con icono Lucide outline `AppIcons.sparkles` (`stroke="currentColor"`, `aria-label="NekoAI Copiloto"`, `title="NekoAI Copiloto (Ctrl+Shift+A)"`).
- Atajo `Ctrl+Shift+A` documentado en `#shortcuts-popover` y conectado en el listener global `keydown`.
- `applyTheme` emite `notify_ai_theme_changed_cmd`; `switchActiveTerminal`, `closeTerminalSession`, `closeChildShell`, `ssh-connected`, `ssh-error` y `ssh-closed` invocan `sync_active_shells_cmd`.
- **Verificación DOM en headless Edge (`http://localhost:4179/index.html`):** `MAIN_DOM_LEN=35419`, `MAIN_HAS_AI_BTN_SVG=True` (botón disparador hidratado con `<svg>` Lucide).

### 1.2 Ventana Independiente `"neko-ai"` (`app/ai-copilot.html` + `copilot.css` + `copilot-main.ts`)
- **Multi-entry Vite (`app/vite.config.ts`):** genera `dist/index.html` (`29.18 kB`) y `dist/ai-copilot.html` (`8.53 kB`) + `dist/assets/aiCopilot-*.css` / `.js`.
- **Controles e Iconografía (`openspec/specs/ui-icons`):** todos los controles (marca, Always on Top, Ajustes, Pin Shell, Escucha ON/OFF, Modo Estricto/Flexible, Refrescar modelos, Enviar, Restaurar) usan iconos Lucide outline con `currentColor` y atributos `aria-label` / `aria-pressed` / `aria-expanded` / `title`. Cero emojis en controles chrome.
- **Seguridad y Estética (`DESIGN.md`, `base-standards.md`):**
  - `autocomplete="off"` aplicado en los 5 inputs/textareas (`copilot-prompt-input`, `cfg-agy-path`, `cfg-context-lines`, `cfg-operator-persona`, `cfg-infra-rules`).
  - `copilot.css` reutiliza estrictamente las variables CSS de `styles.css` (compatible en vivo con los 8 temas mediante `data-theme`) y no introduce `box-shadow` tipo neon glow.
  - En `copilot-main.ts`, las propuestas de comandos de lectura, progreso streaming y tarjetas de Plan sanitizan texto mediante `escapeHtmlText` / `textContent`.
- **Verificación DOM en headless Edge (`http://localhost:4179/ai-copilot.html`):** `AI_DOM_LEN=12417`, `AI_HAS_SVG=True`, `AI_HAS_THEME_ATTR=True`.

## 2. Alcance y Observaciones de Entorno
- Nota de tooling: la skill `modern-web-guidance` requiere Node.js `>= 22`, mientras el entorno dispone de `v21.7.3` (`npx -y modern-web-guidance@latest` salió con código 1); el diseño siguió directamente los contratos SSOT locales (`DESIGN.md`, `ui-layout-contract.md`, `openspec/specs/ui-icons/spec.md`, `base-standards.md`).
- La verificación de arrastre/posicionamiento nativo multi-monitor y `Always on Top` sobre el sistema de ventanas Windows queda lista para comprobación visual interactiva al lanzar `tauri dev` / ejecutable compilado.

## 3. Conclusión
Las vistas `index.html` y `ai-copilot.html` compilan, hidratan sus iconos Lucide y estado inicial en navegador real sin errores TypeScript ni violaciones de tokens o seguridad.
