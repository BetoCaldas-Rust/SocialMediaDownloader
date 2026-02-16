@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  SMD - Gerador de Instalador Automático
echo ========================================

REM 1. Ler versão atual
if not exist version.cfg (
    echo [ERRO] version.cfg não encontrado! Criando com 1.0.6...
    echo 1.0.6 > version.cfg
)
set /p VERSION=<version.cfg
echo Versão Atual: !VERSION!

REM 2. Incrementar Versão (Patch)
for /f "tokens=1,2,3 delims=." %%a in ("!VERSION!") do (
    set MAJOR=%%a
    set MINOR=%%b
    set PATCH=%%c
)
set /a NEW_PATCH=!PATCH! + 1
set NEW_VERSION=!MAJOR!.!MINOR!.!NEW_PATCH!
echo Nova Versão: !NEW_VERSION!

REM 3. Atualizar Arquivos
echo [1/4] Atualizando Cargo.toml...
powershell -Command "(gc Cargo.toml) -replace 'version = \"!VERSION!\"', 'version = \"!NEW_VERSION!\"' | Out-File -Encoding UTF8 Cargo.toml"

echo [2/4] Atualizando backend/main.py...
powershell -Command "(gc backend/main.py) -replace 'version=\"!VERSION!\"', 'version=\"!NEW_VERSION!\"' | Out-File -Encoding UTF8 backend/main.py"

REM 4. Build Backend
echo [3/4] Compilando Backend Python (PyInstaller)...
cd backend
uv run pyinstaller --onefile --name smd-backend main.py
if %errorlevel% neq 0 (
    echo [ERRO] Falha ao compilar backend!
    pause
    exit /b 1
)
cd ..

REM 5. Build Installer
echo [4/4] Gerando Instalador com cargo packager...
cargo packager --release
if %errorlevel% neq 0 (
    echo [ERRO] Falha ao gerar instalador!
    pause
    exit /b 1
)

REM 6. Persistir Nova Versão
echo !NEW_VERSION! > version.cfg

echo.
echo ========================================
echo  SUCESSO! Versão !NEW_VERSION! gerada.
echo ========================================
echo Local: dist\social-media-downloader_!NEW_VERSION!_x64-setup.exe
echo.
pause
