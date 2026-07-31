# NekoSSH

Cliente SSH de escritorio con estética **Cyber-Sakura**: perfiles locales, terminal interactiva y backend nativo en Rust (Tauri v2).

## Requisitos

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable) + toolchain del host
- En Windows: WebView2 (suele venir con Windows 10/11)

## Estructura

| Ruta | Contenido |
|------|-----------|
| `app/` | Código de la aplicación (frontend + `src-tauri`) |
| `docs/` | SSOT técnicos y flujo de trabajo |
| `openspec/` | Changes y specs |

## Instalación

```bash
cd app
npm install
```

Dependencias relevantes del frontend:

- `@xterm/xterm` + `@xterm/addon-fit` — emulador de terminal
- `@tauri-apps/api` / plugins — bridge con el runtime

Dependencias relevantes del backend (`app/src-tauri`):

- `ssh2` — sesiones SSH / PTY
- `rusqlite` (bundled) — CRUD de perfiles en SQLite local
- `tauri-plugin-sql` — migraciones / SQL plugin

## Desarrollo

Desde `app/`:

```bash
npm run tauri dev
```

Solo frontend (Vite):

```bash
npm run dev
```

## Pruebas

```bash
cargo test --manifest-path app/src-tauri/Cargo.toml
```

## Build de producción

```bash
cd app
npm run tauri build
```

## Documentación

- Alcance: `docs/project_scope.md`
- Estándares: `docs/base-standards.md`
- Diseño visual: `docs/design/DESIGN.md`
- Layout: `docs/design/ui-layout-contract.md`
- Ciclo OpenSpec: `docs/workflow-ciclo-end-to-end.md`
