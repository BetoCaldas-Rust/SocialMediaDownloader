@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  Social Media Downloader - Starter
echo ========================================
echo.

REM Verifica se os arquivos de setup existem
if not exist "setup.bat" (
    echo [ERRO] setup.bat nao encontrado! Certifique-se de estar na raiz do projeto.
    pause
    exit /b 1
)

:MENU
echo Escolha como deseja iniciar o projeto:
echo.
echo [1] Rodar Backend Localmente + Frontend
echo [2] Rodar Backend via Docker + Frontend
echo [3] Apenas Setup (Instalar dependencias)
echo [4] Sair
echo.

set /p choice="Opcao: "

if "%choice%"=="1" goto LOCAL
if "%choice%"=="2" goto DOCKER
if "%choice%"=="3" goto SETUP
if "%choice%"=="4" exit /b 0

echo Opcao invalida. tente novamente.
goto MENU

:SETUP
echo Executando setup...
call setup.bat
goto MENU

:LOCAL
echo Verificando ambiente...
call setup.bat
if %errorlevel% neq 0 (
    echo [ERRO] Falha no setup. Resolva os erros acima primeiro.
    pause
    goto MENU
)

echo.
echo Iniciando Backend em uma nova janela...
start "SMD Backend" cmd /k "cd backend && venv\Scripts\activate && python main.py"

echo Aguardando inicializacao do backend (5s)...
timeout /t 5 /nobreak >nul

echo.
echo Iniciando Frontend Rust...
cargo run

exit /b 0

:DOCKER
echo Verificando Docker...
docker-compose --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERRO] Docker Compose nao encontrado!
    pause
    goto MENU
)

echo Iniciando Backend via Docker...
docker-compose up -d

echo.
echo Iniciando Frontend Rust...
cargo run

exit /b 0
