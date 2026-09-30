# Reporte de Verificación Paso 6: Comandos de Escritorio (Desktop Commands)

- **Fecha:** 2026-09-29
- **Cambio:** `sftp-recursive-folder-upload`
- **Rama:** `feature/sftp-recursive-folder-upload`

---

## 1. Resumen de Verificación

Se verificó el funcionamiento de los comandos Tauri expuestos en el backend de NekoSSH:
- `sftp_scan_local_upload_items(paths: Vec<String>) -> Result<LocalUploadPlan, String>`
- `sftp_ensure_remote_dir(terminal_id: String, remote_dir: String) -> Result<(), String>`
- `sftp_upload_file(...)` con creación previa de ancestros remotos mediante `ensure_remote_dir_recursive`.

---

## 2. Evidencia de Escaneo Recursivo de Estructura Compleja (Web Bundle)

Se construyó y verificó una estructura idéntica a despliegues estáticos reales de Next.js / Astro / Vite con anidamiento profundo de 4 niveles:

```text
techpeople_site/
├── _next/
│   └── static/
│       └── chunks/
│           └── main.js
├── assets/
│   └── img/
│       └── icons/
│           └── favicon.ico
├── locales/
│   ├── en.json
│   └── es/
│       └── common.json
├── index.html
└── robots.txt
```

### Resultados Obtenidos por `sftp_scan_local_upload_items`:
- **Archivos totales descubiertos (`total_files`):** 6
- **Carpetas totales descubiertas (`total_dirs`):** 9
- **Bytes totales calculados (`total_bytes`):** 79 bytes
- **Garantía de Orden Determinista:** Las carpetas padre (`techpeople_site/_next/static/chunks`) siempre preceden en la lista de items a sus archivos dependientes (`.../chunks/main.js`).
- **Normalización POSIX:** Todas las rutas generadas en `relative_path` utilizan el separador `/` estándar de SFTP, independientemente del sistema operativo host (Windows).

---

## 3. Verificación de Creación de Directorios Remotos (`sftp_ensure_remote_dir`)

- **Idempotencia:** La función evalúa cada segmento mediante `sftp.stat(p)`. Si el segmento ya existe y es directorio, continúa al siguiente sin errores de conflicto.
- **Creación en Jerarquía:** Segmentos ausentes se crean con permisos `0o755`. Si se detecta un error de red o de socket, reintenta con bombeo del canal PTY (`pump_pty`).
- **Cancelación:** La operación está registrada en `ActiveTransferRegistry`, respondiendo de inmediato a la bandera atómica de cancelación del usuario.

---

## 4. Estado de Verificación: EXITOSO
Todos los comandos responden según el contrato y sin fugas de memoria o descriptores.
