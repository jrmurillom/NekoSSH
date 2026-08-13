## Context

El explorador SFTP del sidebar (`#files-tree`) pinta un árbol lazy desde `explorerRoot` en `main.ts`. Hoy no hay filtro: el usuario debe escaneo visual. Ya existe un patrón similar de búsqueda por texto en snippets (`#snippets-search`).

## Goals / Non-Goals

**Goals:**

- Filtrar en cliente por nombre (substring, case-insensitive) todas las filas ya renderizables del árbol (nivel actual + nodos hijos de carpetas expandidas y cargadas).
- Conservar jerarquía: si un hijo visible coincide y el padre no, el padre sigue mostrándose.
- Campo de filtro + botón × para limpiar y mostrar todo.
- Estado vacío cuando no hay coincidencias.

**Non-Goals:**

- Búsqueda recursiva remota o listados SFTP adicionales.
- Regex / opciones de case-sensitive.
- Cambios de backend, API Tauri o schema.
- Atajo de teclado global (opcional futuro).

## Decisions

### 1. Filtro solo en cliente sobre el modelo en memoria

- **Elección:** aplicar el filtro al pintar (`renderExplorerTree` / construcción de nodos), sin mutar `explorerRoot` ni invalidar caché SFTP.
- **Por qué:** cero costo de red; el estado expandido/cargado se preserva.
- **Alternativa:** ocultar con CSS (`display:none`) — más frágil con nodos anidados y empty states.

### 2. Alcance = nodos ya pintables (expandidos + cargados)

- **Elección:** recorrer el subárbol que hoy se dibujaría; nodos colapsados o no cargados no entran (no se listan hijos ocultos).
- **Por qué:** alineado con “solo lo presentado”; evita trabajo y sorpresas.

### 3. Padres ancla de coincidencias

- **Elección:** un nodo se muestra si su `name` coincide **o** algún descendiente visible (bajo expansión actual) coincide.
- **Por qué:** no romper la ruta visual hacia el archivo encontrado.

### 4. UI del filtro

- **Elección:** input de tipo search/text bajo o dentro de `#files-toolbar`, más botón × visible cuando hay texto.
- **Por qué:** patrón familiar; × cumple el requisito de limpiar rápido.
- **Estilo:** tokens existentes del panel Archivos / snippets; sin cards nuevas.

### 5. Persistencia del query al navegar

- **Elección:** conservar el texto del filtro al cambiar cwd / Actualizar / expandir; se reaplica al nuevo árbol pintado. × o borrar el texto es la forma explícita de “mostrar todo”.
- **Por qué:** el usuario puede seguir filtrando al entrar a otra carpeta; limpia es intencional.
- **Alternativa considerada:** limpiar al cambiar cwd — descartada por preferencia de control explícito vía ×.

## Risks / Trade-offs

- [Carpetas enormes ya listadas] → El filtro es O(n) en nodos en memoria; aceptable para listados tipicos de un directorio. Sin debounce obligatorio si n es pequeño; debounce ligero (~100–150 ms) si el input se siente pesado.
- [Expectativa de “buscar en todo el servidor”] → Mitigar con placeholder/copy claro (“Filtrar…” / “Filtrar nombre…”) y sin messaging de búsqueda remota.
- [Padres visibles sin match] → Puede parecer “ruido”; es el trade-off consciente de jerarquía.

## Migration Plan

- Cambio solo frontend; sin migración de datos.
- Rollback: quitar input/filtro y volver al render actual.

## Open Questions

- Ninguna bloqueante. (Atajo Ctrl+Shift+F en Archivos queda fuera de este change.)
