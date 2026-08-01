# Fix §10 — toolbar snippets vs preview

- Ajuste **solo** bajo `#snippets-modal`:
  - `#snippets-search`: `flex: 1 1 auto; min-width: 0; width: auto` (sin `width: 100%`)
  - `#btn-snippet-new`: `flex: 0 0 auto`
  - Campos de form in-modal siguen con `width: 100%` solo dentro de `.snippet-form-panel`
- **Sin** cambios a selectores globales (`input[type=…]`, `.btn-primary` global, footer).
- Referencia: `docs/design/preview-snippets-modal.html`
