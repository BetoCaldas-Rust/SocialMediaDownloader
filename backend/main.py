from fastapi import FastAPI, HTTPException
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


# VariÃ¡veis globais para managers
config_manager: ConfigManager = None
download_manager: DownloadManager = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle events"""
    global config_manager, download_manager
    
    # Startup
    config_manager = ConfigManager()
    download_manager = DownloadManager(config_manager)
    print("ðŸš€ Backend iniciado!", flush=True)
    print(f"ðŸ“ Pasta de downloads: {config_manager.get_config().default_path}", flush=True)
    print("âœ¨ Servidor pronto no http://localhost:8000", flush=True)
    
    yield
    
    # Shutdown
    print("ðŸ‘‹ Backend finalizado!")


# Cria app FastAPI
app = FastAPI(
    title="Social Media Downloader API",
    description="API para download de vÃ­deos de redes sociais",
    version="1.0.4",
    lifespan=lifespan
)


# Configura CORS para permitir requests do frontend Rust
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Em produÃ§Ã£o, especificar domÃ­nios
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
        "version": "1.0.4"
    }


@app.post("/download", response_model=DownloadResponse)
async def initiate_download(request: DownloadRequest):
    """
    Inicia download de um vÃ­deo
    
    Args:
        request: DownloadRequest com URL e path opcional
        
    Returns:
        DownloadResponse com ID do download
    """
    # Valida URL
    if not is_supported_url(request.url):
        raise HTTPException(
            status_code=400,
            detail="URL invÃ¡lida ou nÃ£o suportada"
        )
    
    try:
        # Inicia download
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
    
    Args:
        download_id: ID do download
        
    Returns:
        DownloadStatusResponse com informaÃ§Ãµes do download
    """
    status = download_manager.get_status(download_id)
    
    if not status:
        raise HTTPException(
            status_code=404,
            detail="Download nÃ£o encontrado"
        )
    
    return status


@app.get("/downloads", response_model=Dict[str, DownloadStatusResponse])
async def get_all_downloads():
    """
    Retorna todos os downloads
    
    Returns:
        DicionÃ¡rio com todos os downloads
    """
    return download_manager.get_all_downloads()


@app.get("/config", response_model=Config)
async def get_config():
    """
    Retorna configuraÃ§Ãµes atuais
    
    Returns:
        Config com configuraÃ§Ãµes
    """
    return config_manager.get_config()


@app.post("/config", response_model=Config)
async def update_config(request: ConfigUpdateRequest):
    """
    Atualiza configuraÃ§Ãµes
    
    Args:
        request: ConfigUpdateRequest com novas configuraÃ§Ãµes
        
    Returns:
        Config atualizado
    """
    try:
        updated_config = config_manager.update_config(
            default_path=request.default_path,
            platform_paths=request.platform_paths,
            temporary=request.temporary
        )
        return updated_config
    
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao atualizar configuraÃ§Ãµes: {str(e)}"
        )


@app.get("/platform")
async def detect_platform_from_url(url: str):
    """
    Detecta plataforma de uma URL
    
    Args:
        url: URL para detectar
        
    Returns:
        PlatformInfo com informaÃ§Ãµes da plataforma
    """
    platform = detect_platform(url)
    
    if not platform:
        raise HTTPException(
            status_code=400,
            detail="URL invÃ¡lida"
        )
    
    return platform


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000
    )
