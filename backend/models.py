from pydantic import BaseModel, HttpUrl
from typing import Optional, Dict
from enum import Enum


class DownloadStatus(str, Enum):
    """Status do download"""
    QUEUED = "queued"
    DOWNLOADING = "downloading"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"


class DownloadRequest(BaseModel):
    """Request para iniciar download"""
    url: str
    custom_path: Optional[str] = None
    
    class Config:
        json_schema_extra = {
            "example": {
                "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                "custom_path": None
            }
        }


class DownloadResponse(BaseModel):
    """Response ao iniciar download"""
    id: str
    status: DownloadStatus
    message: str = "Download iniciado"


class DownloadStatusResponse(BaseModel):
    """Status detalhado do download"""
    id: str
    status: DownloadStatus
    progress: float = 0.0  # 0-100
    speed: Optional[str] = None  # "2.3 MB/s"
    eta: Optional[str] = None  # "00:12"
    filename: Optional[str] = None
    platform: Optional[str] = None
    error: Optional[str] = None


class Config(BaseModel):
    """Configurações do usuário"""
    default_path: str
    platform_paths: Dict[str, str] = {}
    temporary: bool = False


class ConfigUpdateRequest(BaseModel):
    """Request para atualizar configurações"""
    default_path: Optional[str] = None
    platform_paths: Optional[Dict[str, str]] = None
    temporary: bool = False


class PlatformInfo(BaseModel):
    """Informações sobre plataforma detectada"""
    name: str
    url: str
    supported: bool
    folder_name: str
