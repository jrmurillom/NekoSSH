## Context

El encabezado del sidebar actualmente está compuesto por:
```html
<div class="sidebar-header">
  <h1 class="brand-title">NekoSSH</h1>
  <span class="brand-subtitle">Estética Cyber-Sakura</span>
</div>
```
Se requiere incorporar el logo oficial de la aplicación al lado izquierdo del título, asegurando nitidez visual en diferentes densidades de pantalla y una alineación estética premium Cyberpunk-Sakura.

## Goals / Non-Goals

**Goals:**
- Integrar el logo al lado izquierdo del título "NekoSSH".
- Asegurar que el logo sea nítido en pantallas de alta resolución.
- Mantener la armonía del layout del sidebar sin desplazar elementos hacia abajo de forma tosca.
- Aplicar estilos CSS consistentes con el diseño Cyber-Sakura.

**Non-Goals:**
- Rediseñar el logo de la aplicación.
- Agregar interactividad o animaciones pesadas al logo en este slice.

## Decisions

### 1. Elección de la resolución del Icono
* **Decisión:** Utilizar el archivo de icono de **`128x128.png`** (ubicado en [app/src-tauri/icons/128x128.png](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/icons/128x128.png)) y copiarlo a la ruta de assets del frontend como `logo.png`.
* **Alternativas consideradas:**
  * *32x32.png:* Es el tamaño de renderizado aproximado en la pantalla, pero en pantallas Retina o 4K se vería borroso debido a la falta de píxeles lógicos adicionales.
  * *icon.png (512x512):* Excelente nitidez, pero el tamaño del archivo es innecesariamente grande (~113 KB) para el área tan pequeña que va a ocupar en la interfaz.
* **Justificación:** `128x128.png` (~26 KB) ofrece una excelente relación entre nitidez visual (2x o 3x sobre el tamaño del contenedor web de 24-32px) y peso de red.

### 2. Estructuración y Alineación en CSS
* **Decisión:** Usar una disposición Flexbox horizontal para alinear el logo y los textos:
  ```css
  .brand-container {
    display: flex;
    align-items: center;
    gap: var(--space-sm, 10px);
  }
  .brand-logo {
    width: 32px;
    height: 32px;
    object-fit: contain;
    filter: drop-shadow(0 0 4px var(--color-sakura-primary, #ff79c6));
  }
  ```
* **Justificación:** El uso de Flexbox simplifica la alineación vertical. El filtro `drop-shadow` de CSS le añade un brillo de neón sakura que resalta sobre el fondo oscuro y mantiene la estética Cyber-Sakura.

## Risks / Trade-offs

* **[Riesgo]** El logo podría desplazar la altura del header del sidebar reduciendo el espacio disponible para el árbol de conexiones.
  * *Mitigación:* Se limitará la altura del logo a un máximo de `32px` y se ajustarán los márgenes internos del `.sidebar-header` para mantener la altura total del bloque bajo el contrato visual existente.
