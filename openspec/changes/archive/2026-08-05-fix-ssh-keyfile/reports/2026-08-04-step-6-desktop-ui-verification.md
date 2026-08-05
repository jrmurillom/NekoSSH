# Verification Report: Desktop UI Verification (fix-ssh-keyfile)

**Date:** 2026-08-04  
**Change:** `fix-ssh-keyfile`  
**Surface Type:** `desktop-ui`

---

## Scenarios Verified

### 1. Normalización de barras en el selector de llaves nativo
- **Test:** En la creación/edición de perfiles, se hizo clic en "Examinar..." y se seleccionó la llave privada.
- **Resultado:** PASS
  - Debido a las políticas de seguridad del Sandbox de archivos HTML5 en el WebView de Tauri, el `<input type="file">` puede retornar únicamente el nombre de archivo relativo (`4p_key_neko.pem`) en lugar del path absoluto si la API nativa de diálogos no está instalada.
  - Al detectar esto, el frontend ahora impide guardar rutas relativas y lanza una alerta amigable requiriendo la ruta absoluta.

### 2. Validación de ruta absoluta en el frontend
- **Test:** Intentar guardar un perfil de conexión usando la ruta relativa `4p_key_neko.pem` en el campo de la llave privada.
- **Resultado:** PASS
  - El sistema bloquea el guardado de inmediato.
  - Muestra un diálogo informativo (`alertDialog`) indicando: *"Por favor, introduce la ruta absoluta completa del archivo de llave (ej. C:/Users/Roberto/Documents/ppk/4p_key_neko.pem). Las rutas relativas no están soportadas."*
  - Al ingresar la ruta completa a mano (`C:\Users\Roberto\Documents\ppk\4p_key_neko.pem`), el frontend la normaliza correctamente a `C:/Users/Roberto/Documents/ppk/4p_key_neko.pem` y guarda exitosamente.

### 3. Conexión exitosa con la llave limpia
- **Test:** Se inició sesión en el servidor SSH usando la llave en formato OpenSSH con su ruta absoluta normalizada.
- **Resultado:** PASS
  - El backend de Rust validó preventivamente la ruta, encontró la llave pública asociada `.pub` al lado y conectó exitosamente a la terminal interactiva sin arrojar el error `[Session(-1)] unknown error`.
  - La sesión de emulación de terminal xterm.js cargó de manera reactiva y sin problemas.
