# Verificación desktop-commands — contexto multi-shell

**Change:** tab-context-multi-shell
**Fecha:** 2026-08-05
**Rama:** `feature/tab-context-multi-shell`

## Objetivo

Comprobar contra un servidor SSH real que el modelo “pestaña = contexto con padre + hasta 3 hijos” funciona con **logins independientes por `terminal_id`**, que el eco está aislado por shell, que el resize es por sesión, que cerrar un hijo no afecta a los demás, y que una credencial inválida se rechaza.

## Harness

`app/src-tauri/examples/smoke_multi_shell_context.rs` — abre 4 Sessions al perfil id=1 (mismo host/usuario), cada una con canal PTY, replicando lo que el frontend hace con 4 `terminal_id` distintos (`start_ssh_session`, `write_ssh_input`, `resize_ssh_pty`, `close_ssh_session`).

```
cd app/src-tauri
cargo run --example smoke_multi_shell_context
```

## Resultado (ejecución real)

```
contexto multi-shell: 1 padre + 3 hijos (4 logins independientes)
  abrir padre   ... PASS
  abrir hijo-1  ... PASS
  abrir hijo-2  ... PASS
  abrir hijo-3  ... PASS
  eco padre     ... PASS
  eco hijo-1    ... PASS
  eco hijo-2    ... PASS
  eco hijo-3    ... PASS
  resize padre  ... PASS
  resize hijo-1 ... PASS
  resize hijo-2 ... PASS
  resize hijo-3 ... PASS
  cerrar hijo-1  ... PASS
  vivo padre    ... PASS
  vivo hijo-2   ... PASS
  vivo hijo-3   ... PASS
  cerrar contexto ... PASS (3 sesiones)
  auth inválida  ... PASS (rechazada: auth pass)
```

## Cobertura contra los escenarios de spec

| Escenario | Evidencia |
|---|---|
| Abrir hijo con éxito (mismo perfil) | `abrir hijo-1..3 PASS` (4 logins concurrentes al mismo host) |
| Límite de tres hijos | Cubierto en unit test `canAddChildShell` (frontend) |
| Stdout aislado por celda | `eco <shell> PASS` con token distinto por sesión |
| Resize por shell | `resize <shell> PASS` |
| Cerrar un hijo | `cerrar hijo-1 PASS` + `vivo padre/hijo-2/hijo-3 PASS` |
| Cerrar pestaña libera todas las Sessions | `cerrar contexto PASS (3 sesiones)` |
| Error de autenticación | `auth inválida PASS (rechazada)` |

## Estado de datos

Solo lecturas sobre `nekossh.db` (perfil id=1). Sin mutaciones; hash pre/post idéntico (ver report del Step 6). No se imprimieron secretos.
