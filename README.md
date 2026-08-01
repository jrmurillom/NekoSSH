# NekoSSH

Cliente SSH de escritorio con estética **Cyber-Sakura**: carpetas y conexiones locales, terminal interactiva, explorador SFTP y backend nativo en Rust (Tauri v2).

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
- `lucide` — iconos de contorno (chrome UI; color vía tema / `currentColor`)

Dependencias relevantes del backend (`app/src-tauri`):

- `ssh2` — una Session SSH por terminal (PTY + SFTP por canales; sin 2º login)
- `rusqlite` (bundled) — CRUD de carpetas (`connection_folders`) y conexiones (`profiles` + `folder_id`) en SQLite local
- Sidebar **Servidores**: árbol carpetas → conexiones; icono carpeta para agregar grupo; rename inline (Enter/Escape)
- `tauri-plugin-sql` — migraciones / SQL plugin

Tras conectar un perfil, la pestaña **Archivos** lista el filesystem remoto (**SFTP** en la misma conexión, canal aparte). Navegación con iconos Lucide: expandir/colapsar, abrir carpeta, Subir, Ir, Actualizar. Clic derecho → **Abrir en Terminal**. El árbol no sigue automáticamente el `cd` tipado.

Cerrar una pestaña (o “cerrar todas”) libera la Session SSH de ese terminal. Cerrar la ventana de NekoSSH cierra **todas** las Sessions activas.
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
