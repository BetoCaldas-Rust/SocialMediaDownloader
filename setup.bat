@echo off
echo ========================================
echo  Social Media Downloader - Setup
echo ========================================
echo.

REM Verifica se Python esta instalado
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERRO] Python nao encontrado!
    echo Por favor, instale Python 3.11+ de https://www.python.org/
    pause
    exit /b 1
)

echo [OK] Python encontrado
echo.

REM Verifica se Rust esta instalado
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [AVISO] Rust/Cargo nao encontrado!
    echo.
    echo Para instalar o Rust, visite: https://rustup.rs/
    echo Ou execute: winget install Rustlang.Rustup
    echo.
    echo Apos instalar o Rust, execute este script novamente.
    pause
    exit /b 1
)

echo [OK] Rust/Cargo encontrado
echo.

echo ========================================
echo  Configurando Backend Python...
echo ========================================
cd backend

REM Verifica se UV esta instalado
uv --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [AVISO] UV nao encontrado. Instalando UV...
    pip install uv
    if %errorlevel% neq 0 (
        echo [ERRO] Falha ao instalar UV
        pause
        exit /b 1
    )
)

echo [OK] UV encontrado
echo.

REM Cria ambiente virtual se nao existir
if not exist "venv" (
    echo Criando ambiente virtual Python com UV...
    uv venv venv
)

echo Ativando ambiente virtual...
call venv\Scripts\activate.bat

echo Instalando dependencias Python com UV...
uv pip install -r requirements.txt

if %errorlevel% neq 0 (
    echo [ERRO] Falha ao instalar dependencias Python
    pause
    exit /b 1
)

echo.
echo [OK] Backend Python configurado com sucesso!
echo.
cd ..

echo ========================================
echo  Verificando Frontend Rust...
echo ========================================

echo Verificando codigo Rust...
cargo check

if %errorlevel% neq 0 (
    echo [ERRO] Falha ao verificar codigo Rust
    pause
    exit /b 1
)

echo.
echo [OK] Frontend Rust verificado com sucesso!
echo.

echo ========================================
echo  SETUP COMPLETO!
echo ========================================
echo.
echo Para iniciar o aplicativo:
echo.
echo 1. Terminal 1 - Backend:
echo    cd backend
echo    venv\Scripts\activate
echo    python main.py
echo.
echo 2. Terminal 2 - Frontend:
echo    cargo run
echo.
echo Ou use Docker:
echo    docker-compose up -d
echo    cargo run
echo.
pause
