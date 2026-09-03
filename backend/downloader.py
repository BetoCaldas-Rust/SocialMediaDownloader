import os
import uuid
import asyncio
from typing import Dict, Optional
from pathlib import Path
import yt_dlp
from models import DownloadStatus, DownloadStatusResponse
from validators import detect_platform
from config_manager import ConfigManager

SITES_NEEDING_COOKIES = (
    "instagram.com",
    "facebook.com",
    "fb.watch",
    "threads.net",
)

BROWSER_PATHS = {
    "chrome": ("LOCALAPPDATA", "Google/Chrome/User Data"),
    "edge": ("LOCALAPPDATA", "Microsoft/Edge/User Data"),
    "firefox": ("APPDATA", "Mozilla/Firefox"),
    "brave": ("LOCALAPPDATA", "BraveSoftware/Brave-Browser/User Data"),
    "opera": ("LOCALAPPDATA", "Opera Software/Opera Stable"),
}

COOKIE_RETRY_HINTS = (
    "login required",
    "rate-limit reached",
    "requested content is not available",
    "could not copy",
    "failed to decrypt",
    "unable to load cookies",
    "could not find",
    "no such browser",
)


def url_needs_cookies(url: str) -> bool:
    lowered = url.lower()
    return any(site in lowered for site in SITES_NEEDING_COOKIES)


def is_cookie_retryable(error: str) -> bool:
    lowered = error.lower()
    return any(hint in lowered for hint in COOKIE_RETRY_HINTS)


def installed_browsers() -> list[str]:
    found = []
    for name, (env_key, relative) in BROWSER_PATHS.items():
        root = os.environ.get(env_key)
        if root and (Path(root) / relative).exists():
            found.append(name)
    return found


def cookie_browsers_for(url: str, configured: Optional[str]) -> list[Optional[str]]:
    if configured:
        browsers: list[Optional[str]] = [configured]
    else:
        browsers = []

    if url_needs_cookies(url):
        for browser in installed_browsers():
            if browser not in browsers:
                browsers.append(browser)

    return browsers or [None]


def login_required_message(url: str, original: str) -> str:
    if "instagram.com" not in url.lower():
        return original
    return (
        "Instagram exige login. Entre no Instagram no Chrome, Edge ou Firefox "
        "e selecione esse browser em Configurações > Browser para cookies."
    )


class DownloadManager:
    """Gerencia downloads de vídeos usando yt-dlp"""
    
    def __init__(self, config_manager: ConfigManager):
        self.config_manager = config_manager
        self.downloads: Dict[str, DownloadStatusResponse] = {}
        self.active_downloads: Dict[str, asyncio.Task] = {}
        print(f"🎬 yt-dlp versão: {yt_dlp.version.__version__}", flush=True)
    
    def _progress_hook(self, download_id: str, d: dict):
        """Callback de progresso do yt-dlp"""
        if download_id not in self.downloads:
            return
        
        status = self.downloads[download_id]
        
        if d['status'] == 'downloading':
            status.status = DownloadStatus.DOWNLOADING
            
            # Calcula progresso
            if 'total_bytes' in d and d['total_bytes'] > 0:
                downloaded = d.get('downloaded_bytes', 0)
                status.progress = (downloaded / d['total_bytes']) * 100
            elif 'total_bytes_estimate' in d and d['total_bytes_estimate'] > 0:
                downloaded = d.get('downloaded_bytes', 0)
                status.progress = (downloaded / d['total_bytes_estimate']) * 100
            
            # Velocidade
            speed = d.get('speed')
            if speed:
                status.speed = f"{speed / 1024 / 1024:.1f} MB/s"
            
            # ETA
            eta = d.get('eta')
            if eta:
                minutes, seconds = divmod(int(eta), 60)
                status.eta = f"{minutes:02d}:{seconds:02d}"
            
            # Nome do arquivo
            filename = d.get('filename')
            if filename:
                path_obj = Path(filename)
                status.filename = path_obj.name
                status.relative_path = self.config_manager.get_relative_path(str(path_obj))
        
        elif d['status'] == 'finished':
            status.status = DownloadStatus.PROCESSING
            status.progress = 100.0
            filename = d.get('filename')
            if filename:
                path_obj = Path(filename)
                status.filename = path_obj.name
                status.relative_path = self.config_manager.get_relative_path(str(path_obj))
    
    async def download_video(self, url: str, custom_path: Optional[str] = None) -> str:
        """
        Inicia download de vídeo
        
        Args:
            url: URL do vídeo
            custom_path: Caminho customizado (opcional)
            
        Returns:
            ID do download
        """
        download_id = str(uuid.uuid4())
        
        # Detecta plataforma
        platform_info = detect_platform(url)
        if not platform_info:
            raise ValueError("URL inválida ou não suportada")
        
        # Determina caminho de download
        if custom_path:
            download_path = custom_path
        else:
            download_path = self.config_manager.get_download_path(
                platform_info.folder_name
            )
        
        # Cria status inicial
        self.downloads[download_id] = DownloadStatusResponse(
            id=download_id,
            status=DownloadStatus.QUEUED,
            platform=platform_info.name,
            progress=0.0
        )
        
        # Inicia download em background
        task = asyncio.create_task(
            self._download_task(download_id, url, download_path)
        )
        self.active_downloads[download_id] = task
        
        return download_id
    
    async def _download_task(self, download_id: str, url: str, download_path: str):
        """Tarefa assíncrona de download"""
        try:
            # Configurações do yt-dlp
            ydl_opts = {
                'format': 'best',
                'outtmpl': f'{download_path}/%(title)s.%(ext)s',
                'progress_hooks': [lambda d: self._progress_hook(download_id, d)],
                'quiet': False,
                'no_warnings': False,
            }

            loop = asyncio.get_event_loop()
            await loop.run_in_executor(
                None,
                self._run_download_attempts,
                url,
                ydl_opts
            )
            
            # Marca como completo
            if download_id in self.downloads:
                self.downloads[download_id].status = DownloadStatus.COMPLETED
                self.downloads[download_id].progress = 100.0
        
        except Exception as e:
            # Marca como falho
            error_msg = str(e)
            if download_id in self.downloads:
                self.downloads[download_id].status = DownloadStatus.FAILED
                self.downloads[download_id].error = error_msg
            
            # Observa o log por erros onde a lib pede para ser atualizada
            keywords = ["yt-dlp -U"]
            if any(kw.lower() in error_msg.lower() for kw in keywords):
                print("⚠️ Erro do yt-dlp sugere versão desatualizada.")
                print("🔄 Iniciando auto-update...")
                from updater import update_yt_dlp
                # Executa update em background
                asyncio.create_task(asyncio.to_thread(update_yt_dlp))
        
        finally:
            # Remove da lista de downloads ativos
            if download_id in self.active_downloads:
                del self.active_downloads[download_id]
    
    def _run_download_attempts(self, url: str, base_opts: dict):
        configured = self.config_manager.get_config().cookies_from_browser
        last_error: Optional[Exception] = None

        for browser in cookie_browsers_for(url, configured):
            opts = dict(base_opts)
            if browser:
                opts["cookiesfrombrowser"] = (browser,)
                print(f"🍪 Tentando cookies do {browser}...", flush=True)
            try:
                self._run_yt_dlp(url, opts)
                return
            except Exception as error:
                last_error = error
                print(f"⚠️ Falha com cookies={browser}: {error}", flush=True)
                if not is_cookie_retryable(str(error)):
                    raise

        if last_error:
            raise RuntimeError(login_required_message(url, str(last_error))) from last_error

    def _run_yt_dlp(self, url: str, opts: dict):
        """Executa yt-dlp (blocking)"""
        with yt_dlp.YoutubeDL(opts) as ydl:
            ydl.download([url])
    
    def get_status(self, download_id: str) -> Optional[DownloadStatusResponse]:
        """Retorna status de um download"""
        return self.downloads.get(download_id)
    
    def get_all_downloads(self) -> Dict[str, DownloadStatusResponse]:
        """Retorna todos os downloads"""
        return self.downloads
