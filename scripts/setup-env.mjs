#!/usr/bin/env node
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Códigos de escape ANSI para colores y estilos en consola
const colors = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m'
};

const printHeader = (title) => {
  console.log(`\n${colors.cyan}────────────────────────────────────────────────────────────${colors.reset}`);
  console.log(`${colors.bold}${colors.cyan}🚀  ${title}${colors.reset}`);
  console.log(`${colors.cyan}────────────────────────────────────────────────────────────${colors.reset}`);
};

const printDivider = () => {
  console.log(`${colors.cyan}────────────────────────────────────────────────────────────${colors.reset}`);
};

// Interface de Readline para interactividad
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

const askQuestion = (query) => new Promise((resolve) => rl.question(query, resolve));

// Obtener argumentos
const args = process.argv.slice(2);
const forceIndex = args.indexOf('--force');
const isForce = forceIndex >= 0;
const prevVersionIndex = args.indexOf('--prev-version');
let prevVersion = prevVersionIndex >= 0 ? args[prevVersionIndex + 1] : null;

// Cargar la versión actual
let currentVersion = 'unknown';
try {
  const versionPath = path.join(__dirname, 'core-version.json');
  if (fs.existsSync(versionPath)) {
    const versionData = JSON.parse(fs.readFileSync(versionPath, 'utf8'));
    currentVersion = versionData.version || 'unknown';
  }
} catch (err) {
  // Ignorar errores al leer la versión actual
}

// 1. Validar la instalación de OpenSpec CLI
async function checkOpenSpec() {
  console.log(`\n${colors.bold}🔍  Paso 1: Comprobando requisitos del sistema...${colors.reset}`);
  
  // Validar Node.js
  console.log(`    ${colors.green}✔${colors.reset} Node.js detectado (${process.version})`);

  let openSpecInstalled = false;
  let openSpecVersion = '';
  try {
    const versionOutput = execSync('openspec --version', { stdio: 'pipe' }).toString().trim();
    openSpecInstalled = true;
    openSpecVersion = versionOutput;
    console.log(`    ${colors.green}✔${colors.reset} OpenSpec CLI detectado (v${openSpecVersion})`);
  } catch (err) {
    console.log(`    ${colors.red}✘${colors.reset} OpenSpec CLI no detectado en el PATH`);
  }

  if (!openSpecInstalled) {
    console.log(`\n${colors.yellow}⚠️   OpenSpec CLI es necesario para ejecutar los comandos /opsx de automatización.${colors.reset}`);
    const answer = await askQuestion(`    ¿Deseas instalarlo de forma global ahora? (S/n): `);
    
    if (answer.trim().toLowerCase() === 's' || answer.trim() === '') {
      console.log(`\n    ⚙️   Instalando @fission-ai/openspec@latest globalmente...`);
      try {
        execSync('npm install -g @fission-ai/openspec@latest', { stdio: 'inherit' });
        console.log(`    ${colors.green}✔${colors.reset} ¡Instalación completada con éxito!`);
        openSpecInstalled = true;
      } catch (installErr) {
        console.log(`    ${colors.red}✘${colors.reset} Error al instalar OpenSpec globalmente. Por favor, instálalo manualmente corriendo:`);
        console.log(`        ${colors.bold}npm install -g @fission-ai/openspec${colors.reset}`);
      }
    } else {
      console.log(`    ${colors.yellow}ℹ${colors.reset} Instalación omitida. Recuerda instalarlo de forma manual.`);
    }
  }
  
  return openSpecInstalled;
}

// 2. Configurar ZenTao MCP en mcp.json
async function configureZenTao() {
  console.log(`\n${colors.bold}🔍  Paso 2: Configuración de la integración con ZenTao...${colors.reset}`);
  
  const mcpDir = path.join(os.homedir(), '.cursor');
  const mcpPath = path.join(mcpDir, 'mcp.json');
  let mcpData = { mcpServers: {} };
  let fileExists = false;

  if (fs.existsSync(mcpPath)) {
    fileExists = true;
    console.log(`    ${colors.cyan}ℹ${colors.reset} Se detectó un archivo mcp.json existente en:`);
    console.log(`       ${mcpPath}`);
    
    try {
      mcpData = JSON.parse(fs.readFileSync(mcpPath, 'utf8'));
      if (!mcpData.mcpServers) {
        mcpData.mcpServers = {};
      }
    } catch (parseErr) {
      console.log(`\n${colors.red}✘  ERROR: El archivo mcp.json está malformado o no es un JSON válido.${colors.reset}`);
      console.log(`   Para proteger tus datos, se ha abortado la configuración automática.`);
      console.log(`   Por favor, corrige el formato de mcp.json y vuelve a ejecutar:`);
      console.log(`       ${colors.bold}zenit-spec-driven.bat setup${colors.reset}`);
      return false;
    }
  } else {
    console.log(`    ${colors.cyan}ℹ${colors.reset} No se encontró mcp.json local. Se creará una configuración nueva.`);
  }

  const zentaoConfig = mcpData.mcpServers.zentao;
  
  if (zentaoConfig && !isForce) {
    console.log(`    ${colors.green}✔${colors.reset} ZenTao ya está configurado en tu mcp.json.`);
    const answer = await askQuestion(`\n    ¿Deseas sobrescribir la configuración actual de ZenTao? (s/N): `);
    if (answer.trim().toLowerCase() !== 's') {
      console.log(`    ${colors.cyan}ℹ${colors.reset} Configuración de ZenTao conservada.`);
      return true;
    }
  }

  console.log(`\n    Introduce los datos para configurar el servidor MCP de ZenTao:`);
  
  // Preguntar URL
  let ztUrl = '';
  while (!ztUrl) {
    const inputUrl = await askQuestion(`    > URL de ZenTao (ej. http://localhost/zentao/www): `);
    ztUrl = inputUrl.trim();
    if (!ztUrl) {
      console.log(`    ${colors.red}✘ La URL no puede estar vacía.${colors.reset}`);
    }
  }

  // Preguntar Cuenta
  let ztAccount = '';
  while (!ztAccount) {
    const inputAccount = await askQuestion(`    > Usuario de ZenTao: `);
    ztAccount = inputAccount.trim();
    if (!ztAccount) {
      console.log(`    ${colors.red}✘ El usuario no puede estar vacío.${colors.reset}`);
    }
  }

  // Preguntar Contraseña
  let ztPassword = '';
  while (!ztPassword) {
    const inputPassword = await askQuestion(`    > Contraseña (se mostrará en texto plano): `);
    ztPassword = inputPassword.trim();
    if (!ztPassword) {
      console.log(`    ${colors.red}✘ La contraseña no puede estar vacía.${colors.reset}`);
    }
  }

  // Crear la configuración del servidor MCP
  mcpData.mcpServers.zentao = {
    command: 'npx',
    args: ['-y', '@tytt/zentao-mcp'],
    env: {
      ZENTAO_URL: ztUrl.replace(/\/$/, ''),
      ZENTAO_ACCOUNT: ztAccount,
      ZENTAO_PASSWORD: ztPassword,
      ZENTAO_SKIP_SSL: 'false'
    }
  };

  try {
    // Asegurar que exista la carpeta .cursor
    if (!fs.existsSync(mcpDir)) {
      fs.mkdirSync(mcpDir, { recursive: true });
    }
    
    // Guardar archivo
    fs.writeFileSync(mcpPath, JSON.stringify(mcpData, null, 2), 'utf8');
    console.log(`\n    ${colors.green}✔${colors.reset} Configuración de ZenTao guardada correctamente en mcp.json.`);
    console.log(`    ${colors.green}✔${colors.reset} Servidor MCP '@tytt/zentao-mcp' inyectado con éxito.`);
    return true;
  } catch (writeErr) {
    console.log(`\n${colors.red}✘  ERROR: No se pudo escribir en ${mcpPath}${colors.reset}`);
    console.log(`   Detalle: ${writeErr.message}`);
    return false;
  }
}

// 3. Imprimir el resumen y versión
function printSummary(success) {
  printDivider();
  if (success) {
    console.log(`${colors.bold}${colors.green}🎉  ¡Configuración finalizada! Todo listo para trabajar.${colors.reset}`);
    
    // Si hay salto de versión, reportarlo
    if (prevVersion && prevVersion !== 'ninguna' && prevVersion !== currentVersion) {
      console.log(`    📦  Core Actualizado: ${colors.bold}${prevVersion} ➔ ${currentVersion}${colors.reset}`);
    } else {
      console.log(`    📦  Core Versión: ${colors.bold}v${currentVersion}${colors.reset}`);
    }
  } else {
    console.log(`${colors.bold}${colors.red}✘  La configuración finalizó con algunas advertencias o errores.${colors.reset}`);
  }
  printDivider();
}

// Flujo Principal Async
async function main() {
  // Ajuste para CMD en Windows
  if (process.platform === 'win32') {
    // Forzar codificación UTF-8 en salida si es posible
  }

  const isSetupOnly = isForce && args.length === 1;
  const headerTitle = isSetupOnly 
    ? 'ZENIT SPEC-DRIVEN — CONFIGURACIÓN DE ZENTAO' 
    : 'ZENIT SPEC-DRIVEN — CONFIGURACIÓN DE ENTORNO';

  printHeader(headerTitle);

  try {
    let success = true;
    
    if (isSetupOnly) {
      // Si se llamó con setup y --force directamente, saltar verificación de openspec y centrarse en ZT
      success = await configureZenTao();
    } else {
      // Flujo completo: CLI + ZenTao
      await checkOpenSpec();
      success = await configureZenTao();
    }

    printSummary(success);
    process.exit(success ? 0 : 1);
  } catch (err) {
    console.error(`\n${colors.red}✘ Ocurrió un error inesperado durante la configuración:${colors.reset}`);
    console.error(err);
    process.exit(1);
  } finally {
    rl.close();
  }
}

main();
