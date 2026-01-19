import re
from urllib.parse import urlparse
from typing import Optional
from models import PlatformInfo


# Mapeamento de plataformas para nomes de pastas
PLATFORM_MAPPING = {
    "youtube.com": ("YouTube", "youtube"),
    "youtu.be": ("YouTube", "youtube"),
    "instagram.com": ("Instagram", "instagram"),
    "tiktok.com": ("TikTok", "tiktok"),
    "twitter.com": ("Twitter", "twitter"),
    "x.com": ("Twitter", "twitter"),
    "facebook.com": ("Facebook", "facebook"),
    "fb.watch": ("Facebook", "facebook"),
    "reddit.com": ("Reddit", "reddit"),
    "twitch.tv": ("Twitch", "twitch"),
    "vimeo.com": ("Vimeo", "vimeo"),
    "dailymotion.com": ("Dailymotion", "dailymotion"),
}


def validate_url(url: str) -> bool:
    """Valida se a URL é válida"""
    try:
        result = urlparse(url)
        return all([result.scheme, result.netloc])
    except Exception:
        return False


def detect_platform(url: str) -> Optional[PlatformInfo]:
    """
    Detecta a plataforma a partir da URL
    
    Args:
        url: URL do vídeo
        
    Returns:
        PlatformInfo com informações da plataforma ou None se não detectada
    """
    if not validate_url(url):
        return None
    
    parsed = urlparse(url)
    domain = parsed.netloc.lower().replace("www.", "")
    
    # Procura por match exato ou subdomínio
    for platform_domain, (platform_name, folder_name) in PLATFORM_MAPPING.items():
        if domain == platform_domain or domain.endswith(f".{platform_domain}"):
            return PlatformInfo(
                name=platform_name,
                url=url,
                supported=True,
                folder_name=folder_name
            )
    
    # Plataforma não reconhecida, mas URL válida
    # yt-dlp suporta 1000+ sites, então tentamos mesmo assim
    return PlatformInfo(
        name="Unknown",
        url=url,
        supported=True,  # Deixamos yt-dlp decidir
        folder_name="outros"
    )


def is_supported_url(url: str) -> bool:
    """Verifica se a URL é suportada"""
    platform = detect_platform(url)
    return platform is not None and platform.supported
