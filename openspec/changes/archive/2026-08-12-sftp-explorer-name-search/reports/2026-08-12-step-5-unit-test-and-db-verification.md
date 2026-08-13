# Paso 5 — Verificación de unit tests y estado de datos

**Change:** sftp-explorer-name-search
**Fecha:** 2026-08-12
**Rama:** feature/sftp-explorer-name-search
**Surface types:** desktop-ui

## Comandos ejecutados

### Type-check

```
npx tsc --noEmit
# Exit code: 0 (sin errores)
```

### Suite de unit tests

```
npm test   (vitest run)
```

Resultado:

```
Test Files  9 passed (9)
     Tests  61 passed (61)
```

Incluye el nuevo archivo `src/modules/explorer-name-filter.test.ts` (11 tests) que cubre:

- `isFilterActive`: vacío / solo espacios / con texto.
- `nodeNameMatches`: substring case-insensitive, no-match, consulta vacía = todo.
- `computeVisibleNodes`:
  - Consulta vacía deja todo lo pintado visible.
  - Filtrado del nivel actual (case-insensitive).
  - Padre ancla (padre sin match pero hijo coincidente → padre visible).
  - Hijos que no coinciden se ocultan aunque el padre esté expandido.
  - Carpetas colapsadas: sus hijos no se inspeccionan.
  - Sin coincidencias → conjunto vacío.

### Build de producción

```
npm run build   (tsc && vite build)
# Exit code: 0 — built in ~3.6s
```

## Estado de datos (DB)

**N/A — sin persistencia en este change.** El filtro es 100% cliente sobre el
modelo en memoria (`explorerRoot`); no toca SQLite, backend Rust ni comandos
SFTP. No se realizaron mutaciones de datos, por lo que no hay baseline que
restaurar.

## Conclusión

Lógica de filtro verificada por unit tests (todos los escenarios del spec) y
compilación/type-check sin errores. La verificación interactiva de UI se
documenta en el reporte del paso 6.
