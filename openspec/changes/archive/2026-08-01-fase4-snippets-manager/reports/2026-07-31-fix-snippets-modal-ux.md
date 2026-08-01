# Fix UX modal snippets (§9)

Alineado a `docs/design/preview-snippets-modal.html`:

- Search + campos con `.snippets-field` (borde sakura); “+ Snippet” = `btn-primary`
- Lista con borde + dividers entre filas
- Fila = título + comando (sin categoría redundante)
- Nueva categoría = panel in-modal (sin `window.prompt`)
- CSS solo `#snippets-modal` / `.snippets-*`

`npm run build` — ver salida del apply.
