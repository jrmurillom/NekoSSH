**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/increase-terminal-scrollback` previa autorización del usuario
- [x] 0.2 Verificar rama activa en el repositorio

## 1. Helper Modular de Opciones de Terminal (TDD)

- [x] 1.1 Crear suite de pruebas exhaustiva en `app/src/modules/terminal-options-helper.test.ts` cubriendo:
  - Nivel 1: Contrato y opciones puras (`scrollback: 10000`, cursor, fuentes, transparencia y tema).
  - Nivel 2: Comportamiento real del buffer en `@xterm/xterm` inyectando 2,000 líneas (`tail -n2000`) y comprobando retención completa sin truncamiento a 1,000.
  - Nivel 3: Descarte circular y tope del buffer inyectando 12,000 líneas y comprobando que no excede 10,000 líneas de scrollback.
- [x] 1.2 Implementar `app/src/modules/terminal-options-helper.ts` exportando `DEFAULT_TERMINAL_SCROLLBACK = 10000` y la función pura `buildTerminalOptions`.
- [x] 1.3 Ejecutar pruebas unitarias para confirmar resultado en verde al 100%.

## 2. Integración en el Controlador de Terminal

- [x] 2.1 Integrar `buildTerminalOptions` en `createShellPane` dentro de `app/src/main.ts` para que todas las terminales (padres e hijos) usen el helper.
- [x] 2.2 Verificar tipado y compilación del frontend con `npm run build`.

## 3. Review and Update Existing Unit Tests (MANDATORY)

- [x] 3.1 Revisar y ejecutar suite completa de pruebas unitarias existentes en `app/src/**/*.test.ts` para asegurar cero regresiones.

## 4. Run Unit Tests and Verify Local State (MANDATORY)

- [x] 4.1 Ejecutar suite completa de tests (`npm run test`) y capturar resultados.
- [x] 4.2 Documentar reporte de verificación de pruebas en `openspec/changes/increase-terminal-scrollback/reports/2026-09-28-step-4-unit-test-and-db-verification.md`.

## 5. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Verificar la inicialización del scrollback de 10,000 líneas en el emulador en ejecución de escritorio.
- [x] 5.2 Generar reporte de verificación de UI en `openspec/changes/increase-terminal-scrollback/reports/2026-09-28-step-5-desktop-ui-verification.md`.

## 6. Update Technical Documentation (MANDATORY)

- [x] 6.1 Actualizar documentación permanente del proyecto reflejando la retención de 10,000 líneas en xterm.js.
