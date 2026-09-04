# Social Media Downloader - Implementation Plan

> CONCLUÍDO (F0–F8, issues #1–#10 fechadas): este documento descreve a arquitetura histórica híbrida Rust+Python, substituída pelo binário Rust único (MVI + sidecar yt-dlp + i18n runtime). Preservado para referência — ver README.md.

> Direção atual (F0, issue #2): este plano descrevia uma arquitetura híbrida Rust + Python que foi **substituída por um binário Rust único** — o backend Python, a camada HTTP, o Docker e as janelas de terminal separadas foram removidos. O conteúdo abaixo é histórico; o trabalho ativo está nos GitHub issues #1–#10.

Aplicativo desktop para download de vídeos de redes sociais (plano histórico: ~~GUI nativa em Rust + Backend REST API em Python com suporte a Docker~~; direção atual: binário Rust único).

## User Review Required (histórico — superseded em F0)

> [!IMPORTANT]
> **Arquitetura Híbrida Rust + Python + Docker**:
> - **Frontend (Rust)**: GUI nativa multiplataforma, clipboard, hotkeys, performance nativa
> - **Backend (Python + FastAPI)**: REST API, yt-dlp para downloads de 1000+ sites
> - **Docker**: Backend containerizado para isolamento e deploy flexível
> - **Comunicação**: HTTP REST API (localhost durante desenvolvimento, pode ser remoto)

> [!NOTE]
> **Fluxo de Desenvolvimento** (histórico — superseded em F0; direção atual: processo único via `cargo run`):
> 1. ~~Desenvolvimento local: Backend Python roda localmente, frontend Rust conecta via HTTP~~
> 2. ~~Deploy produção: Backend em Docker, frontend continua nativo~~

## Proposed Changes

### Backend (histórico — removido em F0; mantido apenas como referência)

#### [REMOVED] backend/requirements.txt
(removido em F0: o backend Python foi excluído; dependências de servidor HTTP não se aplicam ao binário Rust único)

#### [REMOVED] backend/main.py
(historical: HTTP API servida pelo backend Python; removida em F0)

#### [NEW] [backend/models.py](file:///G:/Projetos/Tools/SocialMediaDownloader/backend/models.py)
Modelos Pydantic:
- `DownloadRequest`: URL, path customizado, opções
- `DownloadResponse`: ID do download, status inicial
- `DownloadStatus`: Progresso, velocidade, ETA, estado
- `Config`: Configurações de paths e preferências
- `Platform`: Info sobre plataforma detectada

#### [NEW] [backend/downloader.py](file:///G:/Projetos/Tools/SocialMediaDownloader/backend/downloader.py)
Serviço de download com yt-dlp:
- Detecção automática de plataforma via URL
- Progress hooks para atualização em tempo real
- Gerenciamento de múltiplos downloads simultâneos
- Criação dinâmica de subpastas por plataforma
- Tratamento de erros e retry logic

#### [NEW] [backend/config_manager.py](file:///G:/Projetos/Tools/SocialMediaDownloader/backend/config_manager.py)
Gerenciamento de configurações:
- Path padrão: `~/Downloads/SocialMediaDownloader/{platform}/`
- Persistência em JSON
- Paths temporários vs permanentes
- Validação de paths

#### [NEW] [backend/validators.py](file:///G:/Projetos/Tools/SocialMediaDownloader/backend/validators.py)
Validações de URL:
- Detecção de plataforma (YouTube, Instagram, TikTok, Twitter, etc.)
- Verificação se plataforma é suportada pelo yt-dlp
- Extração de metadados da URL

---

### Docker (histórico — removido em F0; mantido apenas como referência)

#### [REMOVED] backend/Dockerfile
(removido em F0: sem container; o app é um binário Rust único)

#### [REMOVED] container orchestration manifest
(removido em F0 junto com o Docker)

---

### Frontend (Rust + egui)

#### [NEW] [Cargo.toml](file:///G:/Projetos/Tools/SocialMediaDownloader/Cargo.toml)
```toml
[package]
name = "social-media-downloader"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.25"
egui = "0.25"
tracing = "0.1" (logs go to the integrated Console tab, never to an external terminal)
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
arboard = "3.3"
global-hotkey = "0.5"
rfd = "0.13"  # File dialogs
```

#### [NEW] [src/main.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/main.rs)
Entry point da aplicação Rust:
- Inicialização da GUI com egui
- Setup do runtime tokio para async
- Configuração de hotkeys globais
- Tema preto/amarelo customizado

#### [NEW] [src/ui/app.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/ui/app.rs)
GUI principal:
- Layout com tema preto (#1a1a1a) e amarelo (#ffd700)
- Campo de input para URL
- Botão de download com loading state
- Progress bar com percentual e velocidade
- Lista de downloads ativos/completos
- Botão de configurações

#### [NEW] [src/ui/theme.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/ui/theme.rs)
Tema visual customizado:
- Cores: Background preto, acentos amarelo/dourado
- Estilos para botões, inputs, progress bars
- Fontes personalizadas
- Ícones e animações

#### [NEW] [src/ui/settings.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/ui/settings.rs)
Painel de configurações:
- Seleção de pasta de download (rfd file dialog)
- URL do backend API (localhost:8000 padrão)
- Configuração de paths por plataforma
- Toggle temporário/permanente

#### [NEW] [src/api/client.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/api/client.rs)
Cliente HTTP para backend:
- POST `/download` - Iniciar download
- GET `/status/{id}` - Polling de progresso
- GET/POST `/config` - Gerenciar configurações
- Tratamento de erros de conexão

#### [NEW] [src/api/models.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/api/models.rs)
Structs Rust espelhando modelos do backend:
- `DownloadRequest`, `DownloadResponse`, `DownloadStatus`, `Config`
- Serialização/deserialização com serde

#### [NEW] [src/clipboard.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/clipboard.rs)
Gerenciamento de clipboard:
- Monitoramento via `arboard`
- Detecção de URLs
- Auto-população do campo de input

#### [NEW] [src/hotkeys.rs](file:///G:/Projetos/Tools/SocialMediaDownloader/src/hotkeys.rs)
Hotkeys globais multiplataforma:
- Win/Cmd + Shift + X via `global-hotkey` crate
- Trigger para capturar clipboard
- Funciona em Windows, Linux, macOS

---

### Documentation

#### [NEW] [README.md](file:///G:/Projetos/Tools/SocialMediaDownloader/README.md)
Documentação completa:
- Arquitetura híbrida
- Instalação e setup
- Desenvolvimento local vs Docker
- Como usar o app
- Plataformas suportadas
- Build e deployment

#### [NEW] [.gitignore](file:///G:/Projetos/Tools/SocialMediaDownloader/.gitignore)
```
# Rust
/target/
Cargo.lock

# Python
__pycache__/
*.pyc
venv/
.env

# App data
config.json
downloads/

# OS
.DS_Store
Thumbs.db
```

## API Specification

### Endpoints

**POST /download**
```json
Request: {"url": "https://youtube.com/...", "custom_path": null}
Response: {"id": "uuid", "status": "queued"}
```

**GET /status/{id}**
```json
Response: {
  "id": "uuid",
  "status": "downloading",
  "progress": 45.2,
  "speed": "2.3 MB/s",
  "eta": "00:12",
  "filename": "video.mp4"
}
```

**GET /config**
```json
Response: {
  "default_path": "C:/Users/.../Downloads/SocialMediaDownloader",
  "platform_paths": {"youtube": "C:/..."}
}
```

**POST /config**
```json
Request: {"default_path": "D:/Videos", "temporary": false}
Response: {"success": true}
```

## Verification Plan (histórico — fluxo de dois terminais e Docker removidos em F0)

### Processo único (direção atual)
```bash
# Terminal único: binário Rust
cargo run
```

### Testes Manuais
1. **Backend API**: Testar endpoints via Postman/curl
2. **GUI**: Verificar tema preto/amarelo, responsividade
3. **Download**: YouTube, Instagram, TikTok, Twitter
4. **Hotkey**: Win+Shift+X captura clipboard
5. **Paths**: Criação dinâmica de subpastas
6. **Docker**: Backend em container funcional
7. **Cross-platform**: Testar em Windows (Linux/Mac se disponível)
