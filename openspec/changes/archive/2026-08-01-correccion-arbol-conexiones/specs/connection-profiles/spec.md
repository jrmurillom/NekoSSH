## RENAMED Requirements

- FROM: `Tarjeta de conexión compacta en el árbol`
- TO: `Fila densa de conexión en el árbol`

## MODIFIED Requirements

### Requirement: Fila densa de conexión en el árbol
En el sidebar, cada conexión SHALL mostrarse como **fila densa de lista** (no tarjeta decorativa con fondo/borde permanentes en reposo): nombre + línea `user@host:port` con acento cyan (sin label de tipo auth/túnel). La fila NO MUST mostrar botones inline de editar (lápiz) ni eliminar (basurero). Esas acciones MUST estar en el menú contextual. Un control de copiar junto a la línea de endpoint SHALL copiar `user@host` al portapapeles sin abrir sesión. La fila SHALL abrir la sesión SSH con **doble clic** (fuera del control de copiar y del menú). Renombrar el nombre visible MUST ser inline iniciado desde el menú contextual, no por doble clic. “Editar” en el menú SHALL abrir el formulario/modal de perfil existente. La presentación visual MUST alinearse a la densidad del SSOT visual del change (`docs/design/preview-connection-tree-dense.html`).

#### Scenario: Sin label de método de auth
- **WHEN** el usuario ve una conexión en el árbol
- **THEN** no aparece el texto `SSH (Contraseña)` ni equivalente de llave/túnel en la fila

#### Scenario: Sin lápiz ni basurero en la fila
- **WHEN** el usuario ve una conexión en el árbol
- **THEN** no hay controles inline de editar ni eliminar en la fila; sí puede haber el icono de copiar endpoint

#### Scenario: Copiar user@host
- **WHEN** el usuario activa el icono de copiar en la línea de endpoint
- **THEN** el sistema copia `username@host` al clipboard y no inicia una sesión SSH

#### Scenario: Conectar con doble clic
- **WHEN** el usuario hace doble clic en la fila de conexión (fuera del control de copiar)
- **THEN** el sistema inicia la sesión SSH hacia ese perfil

#### Scenario: Acciones desde menú contextual
- **WHEN** el usuario abre el menú contextual de una conexión
- **THEN** puede elegir al menos editar (modal), renombrar (inline) y eliminar (con confirmación glass)

#### Scenario: Renombrar conexión desde menú
- **WHEN** el usuario elige renombrar en el menú contextual de la conexión
- **THEN** el nombre de la fila entra en modo input inline (Enter guarda, Escape cancela)

#### Scenario: Densidad de lista sin tarjeta decorativa
- **WHEN** el usuario ve conexiones en el árbol en estado de reposo (sin hover ni selección)
- **THEN** las filas no se leen como tarjetas decorativas (sin bloque visual grueso con fondo/borde permanente dominante); se perciben como filas compactas de lista alineadas al preview denso
