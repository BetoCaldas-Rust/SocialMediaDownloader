@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  Social Media Downloader - Starter
echo ========================================
echo.
echo  Single Rust binary (no Python backend, no Docker).
echo  See GitHub issues #1-#10 for the migration plan.
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
echo [1] Rodar o app (cargo run)
echo [2] Apenas Setup (Instalar dependencias)
echo [3] Sair
echo.

set /p choice="Opcao: "

if "%choice%"=="1" goto RUN
if "%choice%"=="2" goto SETUP
if "%choice%"=="3" exit /b 0

echo Opcao invalida. tente novamente.
goto MENU

:SETUP
echo Executando setup...
call setup.bat
goto MENU

:RUN
echo Verificando ambiente...
call setup.bat
if %errorlevel% neq 0 (
    echo [ERRO] Falha no setup. Resolva os erros acima primeiro.
    pause
    goto MENU
)

echo.
if not exist "resources\bin\yt-dlp.exe" (
    echo [AVISO] yt-dlp.exe ausente em resources\bin. Downloads reais exigem o sidecar:
    echo   powershell -ExecutionPolicy Bypass -File tools\fetch-ytdlp.ps1
    echo.
)
echo Iniciando app (processo unico)...
cargo run

exit /b 0
