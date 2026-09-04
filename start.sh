#!/bin/bash

echo "========================================"
echo " Social Media Downloader - Starter"
echo "========================================"
echo ""
echo " Single Rust binary (no Python backend, no Docker)."
echo " See GitHub issues #1-#10 for the migration plan."
echo ""

# Verifica se o setup.sh existe
if [ ! -f "setup.sh" ]; then
    echo "[ERRO] setup.sh não encontrado! Certifique-se de estar na raiz do projeto."
    exit 1
fi

show_menu() {
    echo "Escolha como deseja iniciar o projeto:"
    echo ""
    echo "[1] Rodar o app (cargo run)"
    echo "[2] Apenas Setup (Instalar dependências)"
    echo "[3] Sair"
    echo ""
}

run_app() {
    echo "Verificando ambiente..."
    ./setup.sh
    if [ $? -ne 0 ]; then
        echo "[ERRO] Falha no setup."
        return 1
    fi

    echo ""
    if [ ! -f "resources/bin/yt-dlp" ]; then
        echo "[AVISO] sidecar yt-dlp ausente em resources/bin. Downloads reais exigem:"
        echo "  ./tools/fetch-ytdlp.sh"
        echo ""
    fi
    echo "Iniciando app (processo único)..."
    cargo run
}

while true; do
    show_menu
    read -p "Opção: " choice
    case $choice in
        1) run_app; break ;;
        2) ./setup.sh ;;
        3) exit 0 ;;
        *) echo "Opção inválida." ;;
    esac
done
