---
description: Estándares de desarrollo de Frontend en TypeScript, Vite y CSS Vanilla
alwaysApply: true
---

# Estándares de Frontend (TypeScript & CSS Vanilla)

Este documento establece las reglas técnicas y lineamientos de diseño para el frontend de **NekoSSH** en el directorio `src/`.

---

## 1. Diseño Visual y Estilo Cyberpunk-Anime

SSOT visual: `docs/design/DESIGN.md`. SSOT de layout: `docs/design/ui-layout-contract.md`.

El frontend debe cumplir estrictamente con los siguientes requisitos visuales y de diseño:

- **Tokens de Diseño**: Toda definición de color, espaciado, tipografía, bordes y sombras debe utilizar las CSS Custom Properties definidas en `DESIGN.md` (p. ej. `--color-sakura-neon`, `--bg-dark-card`, `--glow-sakura`), no hex sueltos ni nombres de token inventados.
- **Fondos Personalizables**: El fondo principal de la aplicación debe soportar la carga de rutas de imágenes locales o colores planos, y respetar un valor de opacidad dinámico para proveer un efecto translúcido premium.
- **Terminal Estilizada**: La terminal emulada contará con:
  - Cursor de tipo personalizable (`block`, `underline`, `bar`).
  - Animación de parpadeo ("blink blink").
  - Efecto de resplandor (neon glow) del color activo de la terminal configurado mediante sombras CSS.

---

## 2. Emulación de Terminal (xterm.js)

- **Ajuste Dinámico**: Se utilizará `xterm.js` junto a `xterm-addon-fit` para ajustar el número de filas y columnas de la terminal de manera proporcional al redimensionar la ventana de la aplicación.
- **Comunicación**: El paso de datos con el backend ( Rust) se estructurará mediante eventos y Tauri commands asíncronos para garantizar latencia mínima en la emulación.

---

## 3. Editor Monaco Embebido

- **Configuración del Editor**: Monaco Editor se integrará para la visualización y edición remota de archivos.
- **Características Obligatorias**: Debe contar con números de línea activos, resaltado de sintaxis multilinguaje y atajos funcionales de buscar (`Ctrl + F`) y reemplazar (`Ctrl + H`).
- **Edición Remota Silenciosa**:
  - Al hacer doble clic sobre un archivo en el explorador SFTP, se descargará temporalmente y se abrirá en Monaco.
  - Al presionar `Ctrl + S`, se disparará un comando Tauri para re-subir silenciosamente las modificaciones al servidor remoto en segundo plano.

---

## 4. Convenciones de Código e Idioma

- **Código Fuente**: Las variables, clases, interfaces, funciones, componentes e identificadores en TypeScript se escribirán en **inglés** usando `camelCase`.
- **Textos de Interfaz**: Toda etiqueta, botón, diálogo de alerta, modal y menú contextual que sea visible para el usuario final se redactará estrictamente en **español latino**.
- **Pruebas de Componentes**: Los nombres de los bloques de test y la documentación de las suites de prueba en el frontend deben estar escritos en **español latino**.

Ejemplo:
```typescript
// Componente interno en inglés
export function TermTerminal({ isBlinking, neonColor }: TermTerminalProps) {
  // Las propiedades y estilos utilizan variables y lógica en inglés.
  const cursorStyle = isBlinking ? 'blink' : 'static';

  return (
    <div 
      className="terminal-container" 
      style={{ '--glow-color': neonColor } as React.CSSProperties}
    >
      {/* Las etiquetas textuales del usuario van en español latino */}
      <span className="terminal-status">Conectado al servidor remoto</span>
      <div id="terminal-canvas" data-cursor={cursorStyle} />
    </div>
  );
}
```
