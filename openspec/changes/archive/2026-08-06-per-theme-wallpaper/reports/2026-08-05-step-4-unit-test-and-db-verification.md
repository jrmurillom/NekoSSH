# Verificación unit tests + datos — per-theme-wallpaper

**Change:** per-theme-wallpaper  
**Fecha:** 2026-08-05  
**Rama:** `feature/per-theme-wallpaper`

## Suite ejecutada

```
cd app
npx tsc --noEmit
npx vitest run
```

**Resultado:** `tsc` OK; Vitest **8 files / 49 tests passed** (incluye `theme-wallpaper-helper.test.ts`: CRUD por tema, aislamiento, migración one-shot, sin legacy).

## Estado de datos

N/A BD — este change solo usa `localStorage` (`nekossh-bg-by-theme` + migración de claves globales). No se tocó `nekossh.db`.
