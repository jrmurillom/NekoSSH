# terminal-search Specification

## Purpose
TBD - created by archiving change add-terminal-search. Update Purpose after archive.
## Requirements
### Requirement: Convención de Atajos de Teclado (Norma Global)
Todos los comandos y atajos de teclado personalizados en NekoSSH **SHALL** utilizar el patrón `Ctrl + Shift + <tecla>` (o `Cmd + Shift + <tecla>` en macOS) para no interferir con las secuencias de control nativas de terminales Unix/readline.

#### Scenario: Uso del atajo de búsqueda personalizado
- **WHEN** el usuario presiona `Ctrl+Shift+F`
- **THEN** la aplicación intercepta la tecla para la funcionalidad de búsqueda sin enviarla al shell remoto

### Requirement: Requerimientos Funcionales de Búsqueda
El sistema **SHALL** proveer un buscador de texto en la terminal activa que resalte las coincidencias encontradas.
1. **Atajo de Teclado:** Al presionar `Ctrl+Shift+F` (o `Cmd+Shift+F` en macOS) dentro del área del shell activo, se **MUST** mostrar el buscador.
2. **Navegación:** Debe permitir saltar entre el resultado anterior y el siguiente.
3. **Resaltado:** Debe iluminar en la terminal las palabras que coincidan con la búsqueda.
4. **Sensibilidad:** Botón para distinguir entre mayúsculas y minúsculas.
5. **Cierre:** Al presionar `Esc` o el botón "X", se oculta y limpia el resaltado de búsqueda.

#### Scenario: Buscar un texto existente
- **WHEN** el usuario busca una cadena existente en el buffer
- **THEN** las coincidencias son seleccionadas y resaltadas en la terminal

