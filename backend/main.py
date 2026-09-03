import sys
import asyncio
from fastapi import FastAPI, HTTPException

# Console do Windows usa cp1252 por padrão e quebra com emojis.
# Força UTF-8 para o backend nunca morrer no startup por causa de print.
for _stream in (sys.stdout, sys.stderr):
    if _stream is not None:
        try:
            _stream.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager
from typing import Dict

from models import (
    DownloadRequest, 
    DownloadResponse, 
    DownloadStatusResponse,
    Config,
    ConfigUpdateRequest,
    DownloadStatus
)
from downloader import DownloadManager
from config_manager import ConfigManager
from validators import detect_platform, is_supported_url
from updater import apply_update_path, update_yt_dlp

# Aplica paths de update antes de qualquer outro import que possa carregar yt_dlp
apply_update_path()


# Variáveis globais para managers
config_manager: ConfigManager = None
download_manager: DownloadManager = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle events"""
    global config_manager, download_manager
    
    # Startup
    config_manager = ConfigManager()
    download_manager = DownloadManager(config_manager)
    
    # Removemos o update automático do startup para evitar downloads desnecessários.
    # O update agora é estritamente reativo (em caso de erro) ou manual via endpoint.
    
    print("🚀 Backend iniciado!", flush=True)
    print(f"📂 Pasta de downloads: {config_manager.get_config().default_path}", flush=True)
    print("✨ Servidor pronto no http://localhost:8000", flush=True)
    
    yield
    
    # Shutdown
    print("👋 Backend finalizado!")


# Cria app FastAPI
app = FastAPI(
    title="Social Media Downloader API",
    description="API para download de vídeos de redes sociais",
    version="1.0.15",
    lifespan=lifespan
)


# Configura CORS para permitir requests do frontend Rust
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Em produção, especificar domínios
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
async def root():
    """Health check"""
    return {
        "status": "online",
        "service": "Social Media Downloader API",
        "version": "1.0.15"
    }


@app.post("/download", response_model=DownloadResponse)
async def initiate_download(request: DownloadRequest):
    """
    Inicia download de um vídeo
    """
    if not is_supported_url(request.url):
        raise HTTPException(
            status_code=400,
            detail="URL inválida ou não suportada"
        )
    
    try:
        download_id = await download_manager.download_video(
            request.url,
            request.custom_path
        )
        
        return DownloadResponse(
            id=download_id,
            status=DownloadStatus.QUEUED,
            message="Download iniciado com sucesso"
        )
    
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao iniciar download: {str(e)}"
        )


@app.get("/status/{download_id}", response_model=DownloadStatusResponse)
async def get_download_status(download_id: str):
    """
    Retorna status de um download
    """
    status = download_manager.get_status(download_id)
    if not status:
        raise HTTPException(
            status_code=404,
            detail="Download não encontrado"
        )
    return status


@app.get("/downloads", response_model=Dict[str, DownloadStatusResponse])
async def get_all_downloads():
    """
    Retorna todos os downloads
    """
    return download_manager.get_all_downloads()


@app.get("/config", response_model=Config)
async def get_config():
    """
    Retorna configurações atuais
    """
    return config_manager.get_config()


@app.post("/config", response_model=Config)
async def update_config(request: ConfigUpdateRequest):
    """
    Atualiza configurações
    """
    try:
        updated_config = config_manager.update_config(
            default_path=request.default_path,
            platform_paths=request.platform_paths,
            temporary=request.temporary,
            cookies_from_browser=request.cookies_from_browser
        )
        return updated_config
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao atualizar configurações: {str(e)}"
        )


@app.get("/platform")
async def detect_platform_from_url(url: str):
    """
    Detecta plataforma de uma URL
    """
    platform = detect_platform(url)
    if not platform:
        raise HTTPException(
            status_code=400,
            detail="URL inválida"
        )
    return platform


@app.post("/yt-dlp/update")
async def trigger_yt_dlp_update():
    """
    Aciona a atualização manual do yt-dlp
    """
    success = update_yt_dlp()
    if success:
        return {"message": "yt-dlp atualizado com sucesso. Reinicie o app para aplicar."}
    else:
        raise HTTPException(
            status_code=500,
            detail="Falha ao atualizar yt-dlp. Verifique os logs do console."
        )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000
    )
