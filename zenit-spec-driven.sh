#!/bin/bash
# zenit-spec-driven.sh — Init/update del kit Spec-Driven
# Uso: ./zenit-spec-driven.sh [init|update]
# Destino: raíz del proyecto. Rama: main. Sin symlinks.

set -euo pipefail

MODE="${1:-}"
REPO_URL="https://github.com/zenitprogramacion/zenit-spec-driven.git"
BRANCH="main"
TEMP_DIR="_temp_spec_driven"

# Lista blanca Core (editable)
CORE_DIRS=(scripts ai-specs .cursor .claude)
CORE_FILES=(AGENTS.md CLAUDE.md codex.md GEMINI.md zenit-spec-driven.bat zenit-spec-driven.sh)

# Negocio: solo en init (docs, openspec)

usage() {
    echo "Uso: ./zenit-spec-driven.sh [init|update|setup]"
    exit 1
}

copy_dir() {
    local src="$1"
    local name="$2"
    if [ ! -e "$src" ]; then
        echo "Aviso: no existe $name en el origen; se omite."
        return 0
    fi
    rm -rf "./$name"
    cp -a "$src" "./$name"
    echo "  OK  $name/"
}

copy_file() {
    local src="$1"
    local name="$2"
    if [ ! -e "$src" ]; then
        echo "Aviso: no existe $name en el origen; se omite."
        return 0
    fi
    cp -a "$src" "./$name"
    echo "  OK  $name"
}

if [ "$MODE" != "init" ] && [ "$MODE" != "update" ] && [ "$MODE" != "setup" ]; then
    usage
fi

if [ "$MODE" = "setup" ]; then
    if ! command -v node >/dev/null 2>&1; then
        echo "Error: se requiere Node.js en el PATH para ejecutar la configuración."
        exit 1
    fi
    if [ ! -f "scripts/setup-env.mjs" ]; then
        echo "Error: no se encuentra scripts/setup-env.mjs en este proyecto. Ejecuta 'init' primero."
        exit 1
    fi
    node scripts/setup-env.mjs --force
    exit $?
fi

if ! command -v git >/dev/null 2>&1; then
    echo "Error: se requiere git en el PATH."
    exit 1
fi

PREV_VERSION="ninguna"
if command -v node >/dev/null 2>&1; then
    if [ -f "scripts/core-version.json" ]; then
        PREV_VERSION=$(node -e "try { console.log(require('./scripts/core-version.json').version); } catch { console.log('ninguna'); }" 2>/dev/null || echo "ninguna")
    fi
fi

if [ -e "$TEMP_DIR" ]; then
    echo "Error: ya existe $TEMP_DIR. Elimínalo y vuelve a intentar."
    exit 1
fi

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "Descargando base de zenit-spec-driven (rama $BRANCH)..."
git clone --depth 1 --branch "$BRANCH" "$REPO_URL" "$TEMP_DIR"

if [ "$MODE" = "init" ]; then
    echo "Inicializando Core + plantillas de Negocio en la raíz..."
    for d in "${CORE_DIRS[@]}"; do
        copy_dir "$TEMP_DIR/$d" "$d"
    done
    for f in "${CORE_FILES[@]}"; do
        copy_file "$TEMP_DIR/$f" "$f"
    done
    copy_dir "$TEMP_DIR/docs" "docs"
    copy_dir "$TEMP_DIR/openspec" "openspec"
elif [ "$MODE" = "update" ]; then
    echo "Actualizando solo Core (docs/ y openspec/ protegidos)..."
    for d in "${CORE_DIRS[@]}"; do
        copy_dir "$TEMP_DIR/$d" "$d"
    done
    for f in "${CORE_FILES[@]}"; do
        copy_file "$TEMP_DIR/$f" "$f"
    done
fi

if command -v node >/dev/null 2>&1; then
    node scripts/setup-env.mjs --prev-version "$PREV_VERSION"
else
    echo "Aviso: se requiere Node.js en el PATH para la configuración interactiva del entorno."
fi
