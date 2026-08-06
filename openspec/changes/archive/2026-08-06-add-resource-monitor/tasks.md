## 1. Control de Versiones (Git)

- [x] 1.1 Crear la rama de Git `feature/add-resource-monitor`.

## 2. Iconos de Lucide

- [ ] 2.1 Registrar `Cpu`, `Database`, `HardDrive`, `Network`, `Clock`, `Server`, `Activity`, `Crown`, `Play`, `Pause` en [`app/src/icons.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/icons.ts).

## 3. Backend (Tauri / Rust)

- [ ] 3.1 Actualizar el comando `get_remote_sys_info` en [`app/src-tauri/src/lib.rs`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/lib.rs) para incluir en la cadena de comandos `cat /proc/uptime && cat /proc/net/dev && (ps -eo %cpu,%mem,comm --sort=-%cpu | head -n 4 || true)`.

## 4. Frontend Layout & Estilos (HTML / CSS)

- [ ] 4.1 Reemplazar emojis de texto en [`app/index.html`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html) con nodos SVG de Lucide usando el atributo `data-lucide` para la pestaña Monitor y las cabeceras de métricas.
- [ ] 4.2 Añadir los apartados de **Uptime / OS**, **Tráfico de Red** y **Top Procesos** en [`app/index.html`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html).
- [ ] 4.3 Implementar los estilos CSS de Red y Top Procesos en [`app/src/styles.css`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css) respetando el tema de NekoSSH.

## 5. Frontend Logic (TypeScript)

- [ ] 5.1 Instanciar y renderizar dinámicamente los iconos Lucide en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts).
- [ ] 5.2 Implementar el parseador para leer y procesar Uptime, OS, velocidades de Red (calculando el delta entre lecturas) y Top Procesos.
- [ ] 5.3 Ligar el cambio de icono de pausa/reanudar a los iconos de Lucide correspondientes.

## 6. Compilación y Validación

- [ ] 6.1 Compilar el proyecto con `npm run build` en el directorio [`app/`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app).
- [ ] 6.2 Validar que el backend Rust compila exitosamente con `cargo check`.
