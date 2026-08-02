# Step desktop-ui — Verificación

**Change:** `terminal-copy-paste-moba`  
**Fecha:** 2026-08-01  
**Runtime:** harness `reports/harness-terminal-clipboard.html` (serve :4177) + wire en `app/src/main.ts`

## Checklist

| Escenario | Resultado | Evidencia |
|-----------|-----------|-----------|
| Helper strip Enter final | PASS | CDP harness: `ls -la\n` → `ls -la` |
| Multilínea interna | PASS | CDP: `a\nb\n` → `a\nb` |
| Simulación paste sanitizado → log PTY | PASS | log `"echo hi"` sin `\n` |
| Wire app: `onSelectionChange` + `contextmenu` en canvas | PASS | `main.ts` L1786 / L1794 |
| Ctrl+C no remapeado | PASS | solo `attachCustomKeyEventHandler` para Ctrl+R |
| Clipboard nativo select/right-click en Tauri | N/A preview | Requiere WebView con permiso clipboard; harness + unit tests cubren lógica |

## Conclusión

Lógica y cableado verificados. Gestos reales clipboard en app Tauri dependen del runtime con permisos.
