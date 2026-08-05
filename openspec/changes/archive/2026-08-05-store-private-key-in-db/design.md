## Context

Hoy `auth_credentials.key_path` guarda una ruta de archivo. En Tauri WebView el `<input type="file">` suele no exponer path absoluto; además Windows + `userauth_pubkey_file` de libssh2 producen fallos opacos. El change `fix-ssh-keyfile` intentó parches de ruta y quedó incompleto. Este diseño reemplaza ese enfoque: el material de la llave vive en SQLite y la autenticación usa memoria.

## Goals / Non-Goals

**Goals:**
- Persistir el contenido de la llave privada en columna `private_key`.
- Leer el archivo solo en el momento de "Examinar..." (blob/`FileReader`), no en cada conexión desde disco.
- Autenticar usando el material de `private_key` (temp efímero + libssh2), sin rutas de usuario.
- UI sin revelar el PEM: solo estado "configurado por llave" + acción para reemplazar.
- Renombrar el campo en modelo, SQL e IPC a `private_key`.

**Non-Goals:**
- Migrar perfiles existentes con `key_path` (rutas).
- Cifrado at-rest adicional del material de llave (mismo nivel que `password` actual).
- Soporte de formatos propietarios (p. ej. `.ppk` binario) más allá de lo que ya acepte `ssh2`/`libssh2` en memoria.
- Nuevas dependencias SSH.

## Decisions

1. **Contenido en BD, no ruta**  
   Al seleccionar archivo, el frontend lee texto (PEM/OpenSSH) y lo envía en el payload de save/update. En connect, Rust usa ese string.  
   *Alternativa descartada:* normalizar rutas + `userauth_pubkey_file` (sigue dependiente del SO y del path WebView).

2. **Renombrar `key_path` → `private_key`**  
   Schema SQLite: recrear/alterar la columna en init o migración de esquema sin preservar datos de rutas. Identificadores TS/Rust alineados.  
   *Alternativa descartada:* reutilizar el nombre `key_path` con semántica de contenido (confuso).

3. **Sin migración de perfiles viejos**  
   Producto en desarrollo; perfiles con ruta se invalidan o se recrean a mano. No hay script de import desde disco.

4. **UI: ocultar PEM**  
   Sin input de texto con el material. Mostrar indicador del tipo "Llave privada configurada" cuando `private_key` no esté vacío; "Examinar..." carga/reemplaza; al editar sin cambiar llave, reenviar el material ya guardado (o no sobrescribir con vacío).  
   *Alternativa descartada:* mostrar path o PEM en un input editable.

5. **Auth desde material persistido (sin ruta de usuario)**  
   En Windows, `userauth_pubkey_memory` del crate `ssh2` exige OpenSSL (`vendored-openssl` / `openssl-on-win32`), lo cual introduce builds frágiles (Perl/OpenSSL). Decisión: el backend escribe el contenido de `private_key` a un **archivo temporal efímero**, autentica con `userauth_pubkey_file`, y elimina el archivo al terminar el intento. La fuente de verdad sigue siendo SQLite; el usuario no gestiona ni ve rutas.  
   *Alternativa descartada:* `userauth_pubkey_memory` + OpenSSL vendored (falla de build en el entorno Windows actual).  
   *Alternativa descartada:* depender de `key_path` del usuario.

6. **Descartar trabajo a medias de rutas**  
   Revertir o reemplazar en apply los cambios de path-normalization / `userauth_pubkey_file` del intento previo en la misma área de código.

## Risks / Trade-offs

- **[Riesgo] Material sensible en SQLite en claro** → Misma postura que contraseñas actuales; documentar; no ampliar superficie (no loguear PEM).
- **[Riesgo] Usuario edita perfil y guarda sin re-seleccionar llave** → El save MUST conservar `private_key` existente si no hubo nuevo archivo; no borrar por campo vacío en UI.
- **[Riesgo] Archivo no-PEM / binario** → Validar/fallar con mensaje claro al leer o al autenticar; no inventar convertidores.
- **[Trade-off] Duplicar llave si el usuario tiene el archivo en disco** → Aceptable: la app es fuente de verdad para la conexión.

## Migration Plan

1. Actualizar `CREATE TABLE` / init de `auth_credentials` a `private_key`.
2. Sin backfill desde `key_path`.
3. Rollback: restaurar columna `key_path` y auth por archivo (solo si se abandona el change antes de archive).

## Open Questions

- Ninguna bloqueante: decisiones de producto (rename, sin migración, PEM oculto) ya confirmadas.
