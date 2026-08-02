# Step desktop-ui — Verificación

**Change:** `correccion-arbol-conexiones`  
**Fecha:** 2026-08-01  
**Runtime:** `npm run preview` → `http://127.0.0.1:4173/` (build de producción; agente vía browser MCP)

## Checklist

| Escenario | Resultado | Evidencia |
|-----------|-----------|-----------|
| Header zona **Connections** + 2 icon-buttons | PASS | Snapshot: label CONNECTIONS; botones aria “Nueva conexión” / “Agregar carpeta” |
| Sin toolbar CTA split “Nueva conexión” | PASS | CDP: `panel-actions--split` = null; ambos botones `btn-icon-action` |
| Carpeta sin caja | PASS | Mock `.folder-row`: `border: 0px none`, `border-radius: 0`; tint activo `rgba(255,105,180,0.1)` |
| Hijo con cajita | PASS | Mock `.profile-item`: borde sakura + `border-radius: 8px` + fondo tarjeta |
| Icono crear conexión abre modal | PASS | Click `#btn-new-profile` → heading “Nueva conexión” + form |
| `+` por carpeta presente | PASS | Tras mock: botón “Nueva conexión en esta carpeta” |
| Footer Snippets + engrane intacto | PASS | Visibles en screenshot; sin cambios de rol |
| Crear carpeta vía IPC | N/A en preview | Requiere Tauri/`create_folder`; wire UI intacto (`#btn-new-folder` → mismo listener) |

## Estilos medidos (CDP)

```
hasZoneHeader: true
hasSplitToolbar: false
labelText: "Connections"
folderBorder: 0px none
folderBorderRadius: 0px
itemBorder: ~0.76px solid rgba(255,105,180,0.1)
itemBorderRadius: 8px
```

## Conclusión

Chrome del change verificado en preview. Persistencia create_folder queda fuera del preview web (N/A documentado); no bloquea el gate de presentación.
