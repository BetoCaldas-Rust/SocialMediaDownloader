#!/bin/bash

echo "========================================"
echo " Social Media Downloader - Setup"
echo "========================================"
echo ""

# Verifica se Python esta instalado
if ! command -v python3 &> /dev/null; then
    echo "[ERRO] Python3 nao encontrado!"
    echo "Por favor, instale Python 3.11+ do seu gerenciador de pacotes"
    exit 1
fi

echo "[OK] Python3 encontrado"
echo ""

# Verifica se Rust esta instalado
if ! command -v cargo &> /dev/null; then
    echo "[AVISO] Rust/Cargo nao encontrado!"
    echo ""
    echo "Para instalar o Rust, execute:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "Apos instalar o Rust, execute este script novamente."
    exit 1
fi

echo "[OK] Rust/Cargo encontrado"
echo ""

echo "========================================"
echo " Configurando Backend Python..."
echo "========================================"
cd backend

# Verifica se UV esta instalado
if ! command -v uv &> /dev/null; then
    echo "[AVISO] UV nao encontrado. Instalando UV..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    if [ $? -ne 0 ]; then
        echo "[ERRO] Falha ao instalar UV"
        exit 1
    fi
    # Adiciona UV ao PATH da sessao atual
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "[OK] UV encontrado"
echo ""

# Cria ambiente virtual se nao existir
if [ ! -d "venv" ]; then
    echo "Criando ambiente virtual Python com UV..."
    uv venv venv
fi

echo "Ativando ambiente virtual..."
source venv/bin/activate

echo "Instalando dependencias Python com UV..."
uv pip install -r requirements.txt

if [ $? -ne 0 ]; then
    echo "[ERRO] Falha ao instalar dependencias Python"
    exit 1
fi

echo ""
echo "[OK] Backend Python configurado com sucesso!"
echo ""
cd ..

echo "========================================"
echo " Verificando Frontend Rust..."
echo "========================================"

echo "Verificando codigo Rust..."
cargo check

if [ $? -ne 0 ]; then
    echo "[ERRO] Falha ao verificar codigo Rust"
    exit 1
fi

echo ""
echo "[OK] Frontend Rust verificado com sucesso!"
echo ""

echo "========================================"
echo " SETUP COMPLETO!"
echo "========================================"
echo ""
echo "Para iniciar o aplicativo:"
echo ""
echo "1. Terminal 1 - Backend:"
echo "   cd backend"
echo "   source venv/bin/activate"
echo "   python main.py"
echo ""
echo "2. Terminal 2 - Frontend:"
echo "   cargo run"
echo ""
echo "Ou use Docker:"
echo "   docker-compose up -d"
echo "   cargo run"
echo ""
