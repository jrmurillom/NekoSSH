# tab-context-multi-shell

## Purpose

Modelo de pestaña como contexto de conexión: un shell padre (ancla del SFTP) más hasta tres shells hijos del mismo perfil, con logins SSH independientes y cuadrícula interna en el panel unificado.

## Requirements

### Requirement: Pestaña como contexto padre/hijos
El sistema SHALL modelar cada pestaña de terminal como un contexto con un shell padre (ancla) y hasta tres shells hijos del mismo perfil de conexión. Cada shell (padre o hijo) SHALL tener su propio `terminal_id` y su propio login SSH independiente.

#### Scenario: Abrir contexto al conectar
- **WHEN** el usuario conecta un perfil y se crea una pestaña de terminal
- **THEN** el sistema crea un contexto con un único shell padre conectado y layout de una celda

#### Scenario: Límite de tres hijos
- **WHEN** el contexto ya tiene tres shells hijos
- **THEN** la acción de nuevo shell MUST estar deshabilitada o rechazarse sin crear un cuarto hijo

### Requirement: Nuevo shell hijo en el mismo contexto
El sistema SHALL permitir al usuario abrir un shell hijo adicional en el contexto activo, autenticando de nuevo con el mismo perfil, sin alterar la Session SFTP del padre.

#### Scenario: Abrir hijo con éxito
- **WHEN** el usuario activa “nuevo shell” y hay menos de tres hijos
- **THEN** el sistema inicia una Session SSH distinta, añade una celda al grid y el SFTP del padre permanece usable

#### Scenario: Hijo solo del mismo perfil
- **WHEN** se crea un shell hijo
- **THEN** MUST usar el perfil del contexto padre (no otro servidor)

### Requirement: Ciclo de vida padre e hijos
Cerrar la pestaña SHALL cerrar el padre y todos los hijos. Cerrar un hijo SHALL liberar solo esa Session. El padre MUST NOT ofrecer cierre de celda independiente: solo se elimina al cerrar la pestaña (o al reconectar/limpiar el contexto tras caída del padre).

#### Scenario: Cerrar pestaña con hijos
- **WHEN** el usuario confirma el cierre de la pestaña del contexto
- **THEN** el sistema cierra las Sessions del padre y de todos los hijos y remueve la pestaña

#### Scenario: Cerrar un hijo
- **WHEN** el usuario cierra la celda de un shell hijo
- **THEN** solo esa Session se libera, el grid se reacomoda y el SFTP del padre sigue activo

#### Scenario: Caída del padre
- **WHEN** la Session del padre termina de forma no solicitada
- **THEN** el sistema marca el contexto como desconectado, cierra los shells hijos vivos y ofrece reconexión del contexto (sin dejar hijos huérfanos)

### Requirement: Foco de celda e I/O
El sistema SHALL dirigir la entrada de teclado y el resize del PTY al shell cuyo `terminal_id` esté enfocado dentro del contexto activo. La salida PTY de cada shell SHALL renderizarse solo en su celda.

#### Scenario: Click enfoca celda
- **WHEN** el usuario hace clic en una celda del grid
- **THEN** esa celda recibe el foco visual y las pulsaciones siguientes van a su PTY

#### Scenario: Stdout aislado por celda
- **WHEN** llega output para un `terminal_id` del contexto
- **THEN** se escribe únicamente en el xterm de esa celda

### Requirement: Layout de cuadrícula hasta cuatro celdas
El sistema SHALL mostrar los shells del contexto activo dentro del `.terminal-panel` en una cuadrícula progresiva: 1 celda; 2 columnas; 3 en forma T (padre e hijo1 arriba, hijo2 abajo a ancho completo); 4 en 2×2 con el padre arriba a la izquierda. El fondo, la opacidad y el glow del panel SHALL permanecer en el contenedor de pestaña; las celdas MUST NOT aplicar un fondo de wallpaper propio.

#### Scenario: Cuatro shells visibles
- **WHEN** el contexto tiene padre y tres hijos conectados o en curso
- **THEN** el panel muestra una cuadrícula 2×2 con el padre en la esquina superior izquierda

#### Scenario: Fondo unificado
- **WHEN** el usuario tiene configurado un fondo con opacidad
- **THEN** la imagen/opacidad se aplica al `.terminal-panel` del contexto y se ve a través de las celdas transparentes
