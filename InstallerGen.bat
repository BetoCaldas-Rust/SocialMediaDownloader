@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  SMD - Gerador de Instalador Automático
echo ========================================

REM 1. Ler versão atual do Cargo.toml
set POWERSHELL_CMD="(gc Cargo.toml | Select-String -Pattern '^version\s*=\s*\"(.*?)\"' | select -First 1).Matches.Groups[1].Value"
for /f "usebackq tokens=*" %%v in (`powershell -Command !POWERSHELL_CMD!`) do set VERSION=%%v

if "!VERSION!"=="" (
    echo [ERRO] Não foi possível encontrar a versão no Cargo.toml!
    pause
    exit /b 1
)
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
powershell -Command "$v = '!NEW_VERSION!'; $c = Get-Content Cargo.toml -Raw; $c = $c -replace '(?s)(\[package\].*?version\s*=\s*\").*?(\")', ('${1}' + $v + '${2}'); $c = $c -replace '(?s)(\[package\.metadata\.packager\].*?version\s*=\s*\").*?(\")', ('${1}' + $v + '${2}'); [System.IO.File]::WriteAllText('Cargo.toml', $c, (New-Object System.Text.UTF8Encoding($false)))"

echo [2/4] Atualizando backend/main.py...
powershell -Command "$v = '!NEW_VERSION!'; $c = Get-Content backend/main.py -Raw; $c = $c -replace '(?s)(FastAPI\(.*?version\s*=\s*\").*?(\")', ('${1}' + $v + '${2}'); $c = $c -replace '(?s)(\"version\":\s*\").*?(\")', ('${1}' + $v + '${2}'); [System.IO.File]::WriteAllText('backend/main.py', $c, (New-Object System.Text.UTF8Encoding($false)))"

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

REM 6. Fim
echo.
echo ========================================
echo  SUCESSO! Versão !NEW_VERSION! gerada.
echo ========================================
echo Local: dist\social-media-downloader_!NEW_VERSION!_x64-setup.exe
echo.
pause
