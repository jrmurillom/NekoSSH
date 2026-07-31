@echo off
setlocal EnableExtensions

REM zenit-spec-driven.bat — Init/update del kit Spec-Driven
REM Uso: zenit-spec-driven.bat [init^|update]
REM Destino: raiz del proyecto. Rama: main. Sin symlinks.

set MODE=%~1
set REPO_URL=https://github.com/zenitprogramacion/zenit-spec-driven.git
set BRANCH=main
set TEMP_DIR=_temp_spec_driven

REM Lista blanca Core (editable):
REM   scripts, ai-specs, .cursor, .claude
REM   AGENTS.md, CLAUDE.md, codex.md, GEMINI.md
REM Negocio solo en init: docs, openspec

if "%MODE%"=="" goto usage
if "%MODE%"=="init" goto begin
if "%MODE%"=="update" goto begin
if "%MODE%"=="setup" goto do_setup
goto usage

:usage
echo Uso: zenit-spec-driven.bat [init^|update^|setup]
exit /b 1

:do_setup
where node >nul 2>&1
if errorlevel 1 (
    echo Error: se requiere Node.js en el PATH para ejecutar la configuracion.
    exit /b 1
)
if not exist "scripts\setup-env.mjs" (
    echo Error: no se encuentra scripts\setup-env.mjs en este proyecto. Ejecuta 'init' primero.
    exit /b 1
)
node scripts\setup-env.mjs --force
exit /b %errorlevel%

:begin
set "PREV_VERSION=ninguna"
where node >nul 2>&1
if not errorlevel 1 (
    if exist "scripts\core-version.json" (
        for /f "delims=" %%i in ('node -e "try { console.log(require('./scripts/core-version.json').version); } catch { console.log('ninguna'); }" 2^>nul') do set PREV_VERSION=%%i
    )
)

where git >nul 2>&1
if errorlevel 1 (
    echo Error: se requiere git en el PATH.
    exit /b 1
)

if exist "%TEMP_DIR%" (
    echo Error: ya existe %TEMP_DIR%. Eliminalo y vuelve a intentar.
    exit /b 1
)

echo Descargando base de zenit-spec-driven (rama %BRANCH%)...
git clone --depth 1 --branch %BRANCH% %REPO_URL% %TEMP_DIR%
if errorlevel 1 (
    echo Error: fallo el git clone.
    exit /b 1
)

if "%MODE%"=="init" goto do_init
if "%MODE%"=="update" goto do_update
goto cleanup_fail

:do_init
echo Inicializando Core + plantillas de Negocio en la raiz...
call :copy_dir scripts
if errorlevel 1 goto cleanup_fail
call :copy_dir ai-specs
if errorlevel 1 goto cleanup_fail
call :copy_dir .cursor
if errorlevel 1 goto cleanup_fail
call :copy_dir .claude
if errorlevel 1 goto cleanup_fail
call :copy_file AGENTS.md
if errorlevel 1 goto cleanup_fail
call :copy_file CLAUDE.md
if errorlevel 1 goto cleanup_fail
call :copy_file codex.md
if errorlevel 1 goto cleanup_fail
call :copy_file GEMINI.md
if errorlevel 1 goto cleanup_fail
call :copy_dir docs
if errorlevel 1 goto cleanup_fail
call :copy_dir openspec
if errorlevel 1 goto cleanup_fail
goto cleanup_ok

:do_update
echo Actualizando solo Core (docs y openspec protegidos)...
call :copy_dir scripts
if errorlevel 1 goto cleanup_fail
call :copy_dir ai-specs
if errorlevel 1 goto cleanup_fail
call :copy_dir .cursor
if errorlevel 1 goto cleanup_fail
call :copy_dir .claude
if errorlevel 1 goto cleanup_fail
call :copy_file AGENTS.md
if errorlevel 1 goto cleanup_fail
call :copy_file CLAUDE.md
if errorlevel 1 goto cleanup_fail
call :copy_file codex.md
if errorlevel 1 goto cleanup_fail
call :copy_file GEMINI.md
if errorlevel 1 goto cleanup_fail
goto cleanup_ok

:copy_dir
set "NAME=%~1"
if not exist "%TEMP_DIR%\%NAME%" (
    echo Aviso: no existe %NAME% en el origen; se omite.
    exit /b 0
)
if exist "%NAME%" rmdir /S /Q "%NAME%"
xcopy /E /I /Y /H /K "%TEMP_DIR%\%NAME%" "%NAME%\" >nul
if errorlevel 1 (
    echo Error copiando %NAME%\
    exit /b 1
)
echo   OK  %NAME%\
exit /b 0

:copy_file
set "NAME=%~1"
if not exist "%TEMP_DIR%\%NAME%" (
    echo Aviso: no existe %NAME% en el origen; se omite.
    exit /b 0
)
copy /Y "%TEMP_DIR%\%NAME%" "%NAME%" >nul
if errorlevel 1 (
    echo Error copiando %NAME%
    exit /b 1
)
echo   OK  %NAME%
exit /b 0

:cleanup_ok
where node >nul 2>&1
if not errorlevel 1 (
    node scripts\setup-env.mjs --prev-version %PREV_VERSION%
) else (
    echo Aviso: se requiere Node.js en el PATH para la configuracion interactiva del entorno.
)
:: Copiar los scripts de instalacion y limpiar la carpeta temporal en una sola linea para evitar que cmd.exe falle al modificarse el .bat en ejecucion
copy /Y "%TEMP_DIR%\zenit-spec-driven.bat" "zenit-spec-driven.bat" >nul & copy /Y "%TEMP_DIR%\zenit-spec-driven.sh" "zenit-spec-driven.sh" >nul & if exist "%TEMP_DIR%" rmdir /S /Q "%TEMP_DIR%" & exit /b 0

:cleanup_fail
if exist "%TEMP_DIR%" rmdir /S /Q "%TEMP_DIR%"
exit /b 1
