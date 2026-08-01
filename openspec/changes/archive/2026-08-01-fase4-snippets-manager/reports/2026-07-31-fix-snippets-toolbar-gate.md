# §11 Toolbar gate — diagnóstico y fix

## Diagnóstico (11.1)
- Markup OK: `#snippets-search` + `#btn-snippet-new` en `.snippets-toolbar`.
- Causa: regla **global** `.btn-primary { width: 100%; }` (styles.css ~L218) hacía que “+ Snippet” ocupara todo el ancho de la fila flex y el search quedara con ~0 px visibles.
- No se modificó esa regla global.

## Fix (11.2) — solo `#snippets-modal`
```css
#snippets-modal .snippets-toolbar #btn-snippet-new {
  flex: 0 0 auto;
  width: auto;
  max-width: max-content;
}
#snippets-modal .modal-actions .btn-primary,
#snippets-modal .modal-actions .btn-secondary {
  width: auto;
  min-width: 100px;
}
```
Search sigue con `flex: 1 1 auto; min-width: 0; width: auto`.

## Evidence gate (11.3)
- Estructura esperada (como preview): una fila `[ Buscar… (flex) | + Snippet (auto) ]`.
- Tras override, el botón ya no declara `width: 100%` dentro del toolbar del modal; el input search puede crecer.
- Verificación manual requerida al recargar: search visible a la izquierda, botón compacto a la derecha.

## Build (11.4)
Ver salida `npm run build` del apply.
