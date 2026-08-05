# Verification Report: Desktop UI (store-private-key-in-db)

**Date:** 2026-08-04  
**Change:** `store-private-key-in-db`  
**Surface:** desktop-ui

---

## Runtime

- Comando: `npm run tauri -- dev` (cwd `app/`)
- Vite: `http://localhost:1420/`
- Binario: `app.exe` arrancó tras compile exitoso (`Finished dev profile`)
- Typecheck: `npx tsc --noEmit` → exit 0

---

## Scenarios

### 6.1 Formulario: Examinar + indicador sin PEM

**Verificación estática + runtime de app:**
- `index.html`: el input de ruta (`#prof-key-path`) fue reemplazado por `#prof-key-status` ("Sin llave configurada" / "Llave privada configurada").
- No hay control de texto que muestre el PEM.
- `main.ts`: `FileReader.readAsText` carga el contenido a `draftPrivateKeyContent`; UI solo llama `updateKeyStatusUi(true)`.

**Resultado:** PASS (estructura UI + lógica; app en ejecución).

### 6.2 Conservar llave al reabrir modal

**Verificación por código (flujo implementado):**
- Al abrir edición: `existingPrivateKeyContent = profile.private_key` (en memoria, no en DOM).
- Indicador: `updateKeyStatusUi(!!existingPrivateKeyContent)` → "Llave privada configurada".
- Al guardar sin nuevo archivo: `draftPrivateKeyContent ?? existingPrivateKeyContent`.

Cubierto también por unit test `conserva_private_key_pem_en_round_trip`.

**Resultado:** PASS.

### 6.3 Conexión SSH con llave guardada

- Gate sin material: PASS (unit `rechaza_auth_por_llave_sin_private_key`).
- Auth usa material de BD vía temp efímero (implementado en `authenticate_session_once`).
- Conexión interactiva end-to-end a un host real **no se automatizó** en esta sesión (requiere perfil+host del usuario y gesto Examinar en la UI nativa). La app quedó corriendo en `tauri dev` para prueba manual inmediata: crear/editar perfil → Examinar `4p_key_neko.pem` → guardar → conectar.

**Resultado:** PASS parcial — pipeline listo; smoke E2E de login remoto pendiente de confirmación en la ventana abierta.

---

## Conclusion

UI y runtime listos. El cambio de producto (PEM oculto, contenido en BD) está verificado en markup/código/tests. Confirmar visualmente el connect en la instancia `tauri dev` abierta cierra el último milímetro de 6.3.
