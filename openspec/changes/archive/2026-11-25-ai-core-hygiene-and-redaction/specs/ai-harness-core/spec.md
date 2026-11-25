## MODIFIED Requirements

### Requirement: Sanitizador Preventivo de Secretos (SecretRedactor)
El sistema SHALL sanitizar todo fragmento de texto o buffer capturado antes de persistirlo o enviarlo fuera del runtime de la terminal, enmascarando llaves privadas PEM, asignaciones `KEY=VALUE` simples o compuestas (`*_PASSWORD=`, `*_SECRET=`, `*_TOKEN=`, `*_KEY=`), pares JSON/YAML sensibles (`"password": "..."`, `token: ...`), encabezados `Bearer`, credenciales en URLs y prefijos reconocidos de tokens cloud (`sk-`, `ghp_`, `gho_`, `github_pat_`, `glpat-`, `AKIA...`) mediante búsqueda ASCII *case-insensitive* por bytes segura frente a caracteres UTF-8 multibyte.

#### Scenario: Enmascaramiento de llaves privadas PEM
- **WHEN** el texto capturado contiene bloques delimitados por `-----BEGIN [A-Z ]*PRIVATE KEY-----` y `-----END [A-Z ]*PRIVATE KEY-----`
- **THEN** el sanitizador SHALL reemplazar todo el bloque por el marcador `[REDACTED_PRIVATE_KEY]`

#### Scenario: Enmascaramiento de tokens y variables de credenciales
- **WHEN** el texto contiene asignaciones del tipo `PASSWORD=...`, `TOKEN=...`, `SECRET=...`, `AWS_SECRET_ACCESS_KEY=...`, claves JSON/YAML (`"api_key": "..."`, `token: ...`), prefijos (`sk-...`, `ghp_...`, `glpat-...`, `AKIA...`), encabezados `Authorization: Bearer ...` o caracteres Unicode multibyte cuyo tamaño en bytes varía al cambiar de caja (`ß`, `İ`)
- **THEN** el sanitizador SHALL sustituir los valores sensibles por `[REDACTED_SECRET]` o `[REDACTED_TOKEN]` preservando el nombre de la variable/clave sin sufrir `panic!` UTF-8
