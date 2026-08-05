<p align="center">
  <img src="docs/design/logo3.png" alt="NekoSSH" width="180" />
</p>

<h1 align="center">NekoSSH</h1>

<p align="center">
  Una terminal SSH simple, con estética anime.<br />
</p>

---

## Qué es

Cliente SSH de escritorio (Tauri) para conectar, ver y trabajar en el remoto sin la vibra aburrida de siempre.

- Edición de archivos en remoto (sync con tu editor)
- Snippets a un toque
- Historial de comandos a mano
- Varias pestañas de terminal
- Temas anime y fondo a tu gusto

Una sesión por pestaña. Archivos SFTP en la misma conexión.

---

## Stack

| Capa | Tecnología |
|------|------------|
| Shell de escritorio | **Tauri v2** |
| Backend | **Rust** (`ssh2`, SQLite / `rusqlite`) |
| Frontend | **TypeScript**, Vite, HTML/CSS (tokens de diseño) |
| Terminal | **xterm.js** + Fit addon |
| UI chrome | **Lucide** (outline) |

Código de la app en `app/` (frontend + `src-tauri`).

---

## Requisitos

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) stable
- Windows: WebView2 (incluido en 10/11 recientes)

---

## Comandos

Desde `app/`:

```bash
npm install
npm run tauri dev      # desarrollo
npm run tauri build    # instalable / release
npm run test           # tests frontend (Vitest)
```

Tests Rust:

```bash
cargo test --manifest-path app/src-tauri/Cargo.toml
```

Solo Vite (sin shell nativo):

```bash
cd app && npm run dev
```

---

<p align="center">
  <sub>NekoSSH — SSH con onda anime, sin perder el ritmo.</sub>
</p>
