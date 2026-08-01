## Why

Tras confirmar «¿Subir al servidor?» en el flujo de edición externa (FileZilla-style), el upload SFTP normal falla a menudo por permisos del path remoto (p. ej. archivos de sistema o owned por root). El usuario necesita un **reintento opcional y explícito con sudo**, sin convertirlo en el camino por defecto ni construir UI de contraseña.

## What Changes

- **Dependencia**: change hermano `fase3-external-edit-sync` (activo / aún no archivado). Este change **no** reescribe el ciclo editor externo + confirm A1; solo añade un **delta** post-fallo de upload.
- Tras confirm A1 de subida, el producto intenta primero el **upload SFTP normal** (comportamiento fase3).
- Si ese upload **falla** (en especial por permisos / write denegado), mostrar dialog A1 ofreciendo **«Subir con sudo»** (aceptar / cancelar).
- Si el usuario acepta → un único reintento por ruta elevada mínima (p. ej. subir a temp remoto + `sudo cp` / equivalente simple acordado en design).
- Si sudo pide password, no hay TTY, o el comando falla → **fallar y parar**: alert de error; temp local dirty se conserva; sin reintentos mágicos.
- **Non-goals (explícitos)**: sudo siempre activo; UI de prompt de password; Monaco; reescribir fase3; mutar el host SSH de pruebas compartido durante verificación de agente (tests/verificación **solo mock/local**).

**Constraints (verificación / lab SSH):**
- **MANDATORIO**: implementación, tests y verificación por agente MUST NOT escribir/sobrescribir/borrar en el host SSH de pruebas compartido.
- Evidencia del camino sudo vía **mocks / fixtures locales / fake SFTP / harness de exec**; cero mutaciones al lab.
- El producto **sí** puede usar sudo en uso real cuando el usuario acepta; la restricción aplica solo a desarrollo/verificación contra el lab.

## Capabilities

### New Capabilities

<!-- Ninguna: es delta sobre edición externa ya propuesta en fase3. -->

### Modified Capabilities

- `external-file-edit`: Tras fallo del upload SFTP post-confirm, ofrecer reintento elevado «Subir con sudo»; ejecutar path elevado mínimo; fallar limpio si sudo requiere password / no TTY / falla; conservar temp local dirty. (Capability base aún en change activo `fase3-external-edit-sync`, no archivada en `openspec/specs/`.)

## Impact

- **Backend (`app/src-tauri`)**: comando o extensión del upload de edit-session para path elevado (temp remoto + exec sudo no interactivo); clasificación de fallo de upload que dispara la oferta; sin UI de password.
- **Frontend (`app/src`)**: tras error de upload en el flujo fase3, segundo dialog A1 «Subir con sudo»; alert de error si el path elevado falla; copy en español latino.
- **Docs**: mención breve en alcance/flujo de edición externa si el SSOT de producto lo requiere; sin redesign de layout.
- **Specs**: delta bajo `external-file-edit` en este change (hermano de fase3).
- **Dependencias de change**: aplicar **después** (o sobre) el código de `fase3-external-edit-sync`; no archivar ni modificar artefactos de fase3 en este propose.
- **Lab safety**: mismos mocks/fake SFTP que fase3; path sudo cubierto sin SSH live de escritura.
