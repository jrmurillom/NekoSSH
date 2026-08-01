# Fix: UX path en confirm de subida

**Fecha:** 2026-07-31  
**Change:** `fase3-external-edit-sync`  
**Lab SSH:** cero mutaciones (solo frontend/CSS/OpenSpec; sin upload remoto).

## Qué se corrigió

El A1 “Subir cambios” ya no pone el path remoto largo como `impact` inline (overflow). Por defecto muestra el **basename**; “ver ruta completa” revela un textarea readonly con wrap.

## Verificación

- `npm run build` en `app/`: **OK** (`tsc && vite build`).
- No se ejecutaron writes al host SSH de pruebas.
