# Addendum desktop-ui — Fix Conexiones + Snippets primario

**Change:** `correccion-arbol-conexiones`  
**Fecha:** 2026-08-01  
**Runtime:** `npm run preview` → `http://127.0.0.1:4173/`

## Checklist

| Escenario | Resultado | Evidencia |
|-----------|-----------|-----------|
| Label **Conexiones** (ES) | PASS | CDP `labelText: "Conexiones"`; screenshot CONEXIONES |
| Sin “Connections” en UI | PASS | CDP `hasConnectionsEn: false` |
| Snippets fill primario | PASS | `linear-gradient(135deg, rgb(255,105,180), rgb(216,43,125))`, texto blanco, `border: 0` |
| Engrane sin fill primario | PASS | gear `rgba(0,0,0,0.35)` + borde outline |
| Abrir modal Snippets | PASS | Click abre UI Snippets (error IPC `invoke` esperado sin Tauri) |

## Conclusión

Fixes §5 y §6 verificados en preview.
