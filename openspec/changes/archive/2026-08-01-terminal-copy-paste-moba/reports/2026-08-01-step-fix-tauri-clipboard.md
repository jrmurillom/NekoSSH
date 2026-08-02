# Addendum — Fix clipboard nativo Tauri

**Change:** `terminal-copy-paste-moba`  
**Fecha:** 2026-08-01

## Cambios

| Pieza | Resultado |
|-------|-----------|
| npm `@tauri-apps/plugin-clipboard-manager` | PASS |
| Cargo `tauri-plugin-clipboard-manager` | PASS |
| `lib.rs` `.plugin(...::init())` | PASS |
| Capabilities `allow-read-text` / `allow-write-text` | PASS |
| Terminal gestos usan `writeText`/`readText` del plugin | PASS — sin `navigator.clipboard` en copy/paste terminal |
| `npm test` | PASS — 6/6 strip helper |
| `npm run build` | PASS |
| `cargo check` | PASS |

## Nota

Snippets / copy `user@host` siguen en `navigator.clipboard` (fuera de alcance del fix). Gestos de terminal ya no dependen del prompt del WebView.
