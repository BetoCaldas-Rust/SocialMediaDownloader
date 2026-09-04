#!/bin/bash

echo "========================================"
echo " Social Media Downloader - Setup"
echo "========================================"
echo ""
echo " No Python is required. This app is a single Rust binary."
echo " See GitHub issues #1-#10 for the migration plan."
echo ""

# Verifica se Rust esta instalado
if ! command -v cargo &> /dev/null; then
    echo "[ERRO] Rust/Cargo nao encontrado!"
    echo ""
    echo "Para instalar o Rust, execute:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "Apos instalar o Rust, execute este script novamente."
    exit 1
fi

echo "[OK] Rust/Cargo encontrado"
cargo --version
echo ""

echo "========================================"
echo " Verificando app Rust..."
echo "========================================"

echo "Verificando codigo Rust..."
cargo check

if [ $? -ne 0 ]; then
    echo "[ERRO] Falha ao verificar codigo Rust"
    exit 1
fi

echo ""
echo "[OK] App Rust verificado com sucesso!"
echo ""

echo "========================================"
echo " SETUP COMPLETO!"
echo "========================================"
echo ""
echo "Para iniciar o aplicativo (processo único):"
echo ""
echo "   cargo run"
echo ""
echo "Ou use o menu em ./start.sh"
echo ""
