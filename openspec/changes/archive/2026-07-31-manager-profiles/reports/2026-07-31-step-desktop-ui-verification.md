# Desktop UI verification — manager-profiles

- Date: 2026-07-31
- Change: manager-profiles
- Method: build producción + inspección estática de fuentes (árbol sidebar)

## Checklist

| Ítem | Resultado |
|------|-----------|
| Build Vite OK (`npm run build`) | PASS |
| `btn-new-folder` + `FolderPlus` Lucide | PASS (`index.html`, `icons.ts`, `main.ts`) |
| Árbol: chevron expand/collapse + icono folder + nombre | PASS (`renderProfileList`) |
| Conexiones anidadas bajo carpeta | PASS (`.folder-children` + `buildProfileItem`) |
| Nueva conexión en contexto de carpeta (`folder_id` hidden) | PASS |
| Rename inline: Enter / Escape / blur | PASS (`commitFolderRename`) |
| Delete carpeta con confirmación + conteo | PASS (`deleteFolder` + `get_folder_connection_count`) |
| Editar/eliminar/conectar conexión adaptado al árbol | PASS |
| Docs: `ui-layout-contract.md`, `DESIGN.md`, `README.md` | PASS |
| Runtime Tauri E2E visual interactivo | no corrido aquí — build + código verificados |

## Outcome

UI de árbol de carpetas/conexiones cableada. Validación visual en app viva: `npm run tauri dev`.
