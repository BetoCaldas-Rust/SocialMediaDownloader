@echo off
echo ========================================
echo  Social Media Downloader - Setup
echo ========================================
echo.
echo  No Python is required. This app is a single Rust binary.
echo  See GitHub issues #1-#10 for the migration plan.
echo.

REM Verifica se Rust esta instalado
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERRO] Rust/Cargo nao encontrado!
    echo.
    echo Para instalar o Rust, visite: https://rustup.rs/
    echo Ou execute: winget install Rustlang.Rustup
    echo.
    echo Apos instalar o Rust, execute este script novamente.
    pause
    exit /b 1
)

echo [OK] Rust/Cargo encontrado
cargo --version
echo.

echo ========================================
echo  Verificando app Rust...
echo ========================================

echo Verificando codigo Rust...
cargo check

if %errorlevel% neq 0 (
    echo [ERRO] Falha ao verificar codigo Rust
    pause
    exit /b 1
)

echo.
echo [OK] App Rust verificado com sucesso!
echo.

echo ========================================
echo  SETUP COMPLETO!
echo ========================================
echo.
echo Para iniciar o aplicativo (processo unico):
echo.
echo    cargo run
echo.
echo Ou use o menu em start.bat
echo.
pause
