**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/direct-p2p-scp-transfer` previa autorización del usuario
- [x] 0.2 Verificar rama activa en el repositorio

## 1. Módulo Backend de Seguridad y Gestión de Credenciales Efímeras (Rust)

- [x] 1.1 Implementar guardia RAII `P2PCredentialGuard` en Rust con trait `Drop` para destrucción atómica (`shred -u` o `rm -f`) de credenciales temporales en el servidor origen
- [x] 1.2 Implementar función para transferir e instalar de forma segura la clave privada del servidor destino en `/dev/shm` o `tmpfs` con permisos restrictivos `0600`
- [x] 1.3 Crear pruebas unitarias en Rust validando el ciclo de vida, eliminación en caso de pánico y fallback de directorios temporales

## 2. Orquestador de Transferencia Directa P2P y Telemetría en Tiempo Real (Rust)

- [x] 2.1 Implementar sondeo de conectividad de red TCP no bloqueante desde el servidor origen hacia el destino (`bash /dev/tcp` o `nc`)
- [x] 2.2 Implementar orquestación de comando `rsync` con flags `--inplace`, `-avP` y fallback transparente a `scp` nativo en canal `exec`
- [x] 2.3 Implementar parser en streaming del `stdout` remoto para extraer porcentaje, velocidad (MB/s o KB/s) y tiempo restante estimado (ETA)
- [x] 2.4 Conectar el flujo de telemetría al canal IPC `TransferProgressPayload` de Tauri
- [x] 2.5 Implementar soporte de cancelación cooperativa que envíe señal `SIGTERM`/`SIGKILL` al proceso remoto y purgue credenciales

## 3. Fallback de Tubería Streaming de Alta Velocidad (Rust)

- [x] 3.1 Implementar transporte de streaming por tubería de sockets (`cat <SRC> | cat > <DEST>`) entre las dos sesiones SSH para servidores sin visibilidad de red directa
- [x] 3.2 Crear pruebas unitarias deterministas simulando fallo de conectividad directa y activación limpia del fallback de tubería

## 4. Integración en Frontend: UI, Diálogo y Telemetría

- [x] 4.1 Actualizar menú contextual del explorador SFTP y diálogo de confirmación A1 en `app/src/main.ts` indicando modo de transferencia inter-servidor
- [x] 4.2 Conectar la telemetría recibida al indicador visual `#files-status` mostrando velocidad real en MB/s y ETA
- [x] 4.3 Actualizar helpers y pruebas unitarias de frontend (`transfer-progress-helper.test.ts`) para soportar los nuevos campos de telemetría P2P

## 5. Matriz de Robustez y Casos Negativos en Backend (Rust / Tolerancia Cero a Fallos)

- [x] 5.1 Implementar pruebas unitarias para el parser de `stdout` de `rsync` y `scp` cubriendo variaciones de retorno de carro (`\r`), saltos de velocidad y formato de ETA
- [x] 5.2 Implementar prueba unitaria de corte determinista por timeout en sondeo TCP evitando congelamiento del runtime de Tauri
- [x] 5.3 Implementar prueba de seguridad RAII en `P2PCredentialGuard` asegurando la ejecución de `shred -u` ante aborto o pánico del hilo
- [x] 5.4 Implementar prueba de consumo constante de memoria $O(1)$ (< 30 MB) durante la simulación de transferencia masiva de 10 GB

## 6. Suite de Integración E2E Frontend / WebView (Vitest)

- [x] 6.1 Implementar `app/src/modules/direct-p2p-transfer.e2e.test.ts` simulando el ciclo completo de copia inter-sesión, diálogo A1 y actualización reactiva de `#files-status`
- [x] 6.2 Implementar prueba E2E de cancelación cooperativa a mitad de transferencia con confirmación en modal A1 y desregistro de guardia
- [x] 6.3 Implementar prueba E2E de conmutación automática visual al modo de "Túnel de streaming" ante indisponibilidad de red directa

## 7. Review and Update Existing Unit Tests (MANDATORY)

- [x] 7.1 Revisar y ajustar suite completa de pruebas unitarias existentes en frontend (`npm run test`) y backend (`cargo test`) asegurando cero regresiones

## 8. Run Unit Tests and Verify Local State (MANDATORY)

- [x] 8.1 Ejecutar suite completa de tests de frontend y backend capturando resultados (100% aprobadas)
- [x] 8.2 Documentar reporte de pruebas en `openspec/changes/direct-p2p-scp-transfer/reports/2026-09-29-step-8-unit-test-and-db-verification.md`

## 9. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 9.1 Invocar y verificar los comandos y eventos de transferencia directa P2P y fallback en el runtime de escritorio
- [x] 9.2 Generar reporte de verificación de comandos en `openspec/changes/direct-p2p-scp-transfer/reports/2026-09-29-step-9-desktop-commands-verification.md`

## 10. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 10.1 Verificar el comportamiento visual en el WebView del diálogo de confirmación, la indicación de velocidad en MB/s y el progreso en tiempo real en `#files-status`
- [x] 10.2 Generar reporte de verificación de UI en `openspec/changes/direct-p2p-scp-transfer/reports/2026-09-29-step-10-desktop-ui-verification.md`

## 11. Update Technical Documentation (MANDATORY)

- [x] 11.1 Actualizar documentación permanente del proyecto en `docs/` reflejando la nueva arquitectura P2P directa inter-servidor y el fallback de tubería
