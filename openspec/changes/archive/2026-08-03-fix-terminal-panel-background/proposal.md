## Why

Se requiere que la imagen de fondo personalizada y su regulador de opacidad (`#config-bg-opacity`) se apliquen **única y exclusivamente dentro del recuadro del contenedor de la terminal SSH (`.terminal-panel`)**, manteniendo intacta la barra lateral y todo el resto de la interfaz Cyber-Sakura sin ninguna transparencia colateral.

## What Changes

- **Fondo Exclusivo en la Terminal (`.terminal-panel`)**: Aplicar `backgroundImage` (con `background-size: cover` y `background-position: center`) directamente sobre el contenedor de la terminal (`.terminal-panel`), limitando la imagen únicamente al área de la consola SSH.
- **Control de Opacidad sobre la Terminal**: Sincronizar el slider `#config-bg-opacity` para que ajuste la capa de tinte y visibilidad de la imagen dentro de la terminal (`.terminal-panel::before`), permitiendo atenuar o intensificar la imagen debajo del texto de la consola.
- **Cero Afectación Externa**: Mantener la barra lateral, cabeceras y área principal en sus estilos originales 100% sólidos y Cyber-Sakura.

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities
- `app-branding`: Asignación exclusiva de imagen de fondo y regulador de opacidad sobre la tarjeta de la terminal activa.

## Impact

- **Archivos afectados**:
  - `app/src/styles.css` (estilos de `backgroundImage` y capa de opacidad en `.terminal-panel`).
  - `app/src/main.ts` (aplicación de `backgroundImage` y opacidad exclusivamente sobre `.terminal-panel`).
  - `app/src/bg-settings-helper.ts` y `app/src/bg-settings-helper.test.ts` (helper y pruebas unitarias de opacidad).
