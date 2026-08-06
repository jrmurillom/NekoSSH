## Why

Hoy el fondo de terminal es global: un solo wallpaper/opacidad para todos los temas. Al cambiar de tema conceptual, el usuario pierde el “look” de fondo que había afinado para ese tema (o arrastra el fondo de otro). Hace falta que cada tema recuerde su propio fondo.

## What Changes

- El wallpaper de terminal (imagen + etiqueta + opacidad) pasa a guardarse **por id de tema**.
- Al seleccionar o restaurar un tema, se aplica el fondo guardado de ese tema (o vacío si nunca se configuró).
- Al subir, aplicar, ajustar opacidad o quitar el fondo, el cambio afecta **solo el tema activo**.
- Migración one-shot: si existen las claves globales actuales (`nekossh-bg-url` / label / opacity), se copian al tema activo (o `nekossh`) y dejan de usarse como fuente de verdad.
- **BREAKING** (comportamiento de producto): el fondo ya no es compartido entre temas.

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `app-branding`: personalización de fondo/opacidad scoped al tema conceptual activo; persistencia por tema.
- `conceptual-themes`: al cambiar/restaurar tema, sincronizar también el wallpaper del tema (además de CSS, xterm y logo).

## Impact

- Frontend: `app/src/main.ts` (persistencia, `applyTheme`, prefs de fondo), posiblemente helper puro en `bg-settings-helper` o módulo nuevo.
- `localStorage`: nuevas claves/estructura por tema; migración desde claves globales.
- Specs: deltas en `app-branding` y `conceptual-themes`.
- Docs: `DESIGN.md` / branding si documenta el fondo global.
- Sin cambios de backend Rust, IPC ni SQLite.
