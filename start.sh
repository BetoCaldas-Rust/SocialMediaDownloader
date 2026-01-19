#!/bin/bash

echo "========================================"
echo " Social Media Downloader - Starter"
echo "========================================"
echo ""

# Verifica se o setup.sh existe
if [ ! -f "setup.sh" ]; then
    echo "[ERRO] setup.sh não encontrado! Certifique-se de estar na raiz do projeto."
    exit 1
fi

show_menu() {
    echo "Escolha como deseja iniciar o projeto:"
    echo ""
    echo "[1] Rodar Backend Localmente + Frontend"
    echo "[2] Rodar Backend via Docker + Frontend"
    echo "[3] Apenas Setup (Instalar dependências)"
    echo "[4] Sair"
    echo ""
}

run_local() {
    echo "Verificando ambiente..."
    ./setup.sh
    if [ $? -ne 0 ]; then
        echo "[ERRO] Falha no setup."
        return 1
    fi

    echo ""
    echo "Iniciando Backend em segundo plano..."
    cd backend
    source venv/bin/activate
    python3 main.py &
    BACKEND_PID=$!
    cd ..

    echo "Aguardando inicialização do backend (5s)..."
    sleep 5

    echo ""
    echo "Iniciando Frontend Rust..."
    cargo run

    # Kill backend when frontend exits
    kill $BACKEND_PID
}

run_docker() {
    echo "Verificando Docker..."
    if ! command -v docker-compose &> /dev/null; then
        echo "[ERRO] Docker Compose não encontrado!"
        return 1
    fi

    echo "Iniciando Backend via Docker..."
    docker-compose up -d

    echo ""
    echo "Iniciando Frontend Rust..."
    cargo run
}

while true; do
    show_menu
    read -p "Opção: " choice
    case $choice in
        1) run_local; break ;;
        2) run_docker; break ;;
        3) ./setup.sh ;;
        4) exit 0 ;;
        *) echo "Opção inválida." ;;
    esac
done
