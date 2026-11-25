# Reporte de Verificación de Tests Unitarios y Base de Datos (Step 7)

- **Change:** `ai-harness-core`
- **Fecha:** 2026-11-24
- **Herramienta:** `cargo test`

## 1. Resumen de Ejecución de Pruebas

Se ejecutó la suite completa de pruebas unitarias y de integración de la aplicación con `cargo test` sobre `app/src-tauri`.

```text
running 112 tests
test ai::prompt::tests::ensambla_constitucion_y_directivas ... ok
test ai::prompt::tests::usa_defaults_si_directivas_estan_vacias ... ok
test ai::fsm::tests::rechaza_saltos_ilegales ... ok
test ai::fsm::tests::freno_automatico_por_error ... ok
test ai::fsm::tests::flujo_completo_feliz_de_tres_fases ... ok
test ai::risk_guard::tests::risk_guard_eleva_a_destructivo_pipes_a_interprete ... ok
test ai::commands::tests::registry_crea_fsm_independiente_por_sesion ... ok
test ai::schema::tests::tarea_solo_lectura_valida_sin_rollback ... ok
test ai::risk_guard::tests::risk_guard_eleva_read_only_con_redireccion_a_config_change ... ok
test ai::schema::tests::tarea_mutante_falla_sin_rollback_o_backup ... ok
test ai::redactor::tests::enmascara_credenciales_en_url ... ok
test ai::redactor::tests::enmascara_asignaciones_sensibles ... ok
test ai::risk_guard::tests::risk_guard_clasifica_servicios_correctamente ... ok
test ai::redactor::tests::enmascara_bloque_pem ... ok
test ai::redactor::tests::enmascara_bearer_token ... ok
test ai::risk_guard::tests::recon_rechaza_redirecciones_y_mutaciones ... ok
test ai::risk_guard::tests::recon_permite_comandos_diagnostico_puro ... ok
test ai::fsm::tests::congelamiento_por_desconexion_y_reanudacion ... ok
test ai::schema::tests::serializacion_deserializacion_roundtrip ... ok
test ai::preferences::tests::set_y_get_roundtrip ... ok
test ai::preferences::tests::reset_restaura_defaults ... ok
test ai::preferences::tests::get_retorna_defaults_en_bd_vacia ... ok
[... 90 tests restantes de SFTP, Terminal, Crypto ECDH, Preferences, etc. ...]

test result: ok. 112 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
```

## 2. Verificación de Persistencia en Base de Datos Local

- **Tabla involucrada:** `app_preferences` (SQLite).
- **Pruebas de persistencia ejecutadas en memoria:**
  - `ai::preferences::tests::get_retorna_defaults_en_bd_vacia`: Valida inicialización con valores por defecto cuando la tabla está vacía (`Strict`, directivas predeterminadas).
  - `ai::preferences::tests::set_y_get_roundtrip`: Valida inserción y actualización (`ON CONFLICT`) de modo de ejecución (`Flexible`) y directivas de Capa 2.
  - `ai::preferences::tests::reset_restaura_defaults`: Valida la reversión idempotente a valores de fábrica del operador.
- **Estado de datos:** Cero efectos secundarios en bases de datos de producción locales; pruebas ejecutadas con `Connection::open_in_memory()`.

## 3. Conclusión
La suite de pruebas pasó al 100% (112 de 112 pruebas aprobadas). El código implementado mantiene compatibilidad total con el motor SSH, SFTP y criptografía previos.
