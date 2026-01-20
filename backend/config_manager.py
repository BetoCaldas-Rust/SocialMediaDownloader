import json
import os
from pathlib import Path
from typing import Dict, Optional
from models import Config


class ConfigManager:
    """Gerencia configurações do aplicativo"""
    
    def __init__(self, config_file: str = "config.json"):
        self.config_file = Path(config_file)
        self.config: Config = self._load_config()
    
    def _get_default_download_path(self) -> str:
        """Retorna o caminho padrão de downloads do sistema"""
        # Verifica variável de ambiente (Docker)
        env_path = os.environ.get("DOWNLOAD_PATH")
        if env_path:
            downloads = Path(env_path)
        # Windows
        elif os.name == 'nt':
            downloads = Path.home() / "Downloads" / "SocialMediaDownloader"
        # Linux/Mac
        else:
            downloads = Path.home() / "Downloads" / "SocialMediaDownloader"
        
        # Cria a pasta se não existir
        downloads.mkdir(parents=True, exist_ok=True)
        return str(downloads)
    
    def _load_config(self) -> Config:
        """Carrega configurações do arquivo"""
        if self.config_file.exists():
            try:
                with open(self.config_file, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                    return Config(**data)
            except Exception as e:
                print(f"Erro ao carregar config: {e}")
        
        # Configuração padrão
        return Config(
            default_path=self._get_default_download_path(),
            platform_paths={},
            temporary=False
        )
    
    def save_config(self, config: Config):
        """Salva configurações no arquivo"""
        self.config = config
        try:
            with open(self.config_file, 'w', encoding='utf-8') as f:
                json.dump(config.model_dump(), f, indent=2, ensure_ascii=False)
        except Exception as e:
            print(f"Erro ao salvar config: {e}")
    
    def get_download_path(self, platform: str) -> str:
        """
        Retorna o caminho de download para uma plataforma específica
        
        Args:
            platform: Nome da plataforma (ex: 'youtube', 'instagram')
            
        Returns:
            Caminho completo para salvar downloads
        """
        # Verifica se há path customizado para a plataforma
        if platform in self.config.platform_paths:
            custom_path = Path(self.config.platform_paths[platform])
        else:
            # Usa o path padrão + subpasta da plataforma
            custom_path = Path(self.config.default_path) / platform
        
        # Cria a pasta se não existir
        custom_path.mkdir(parents=True, exist_ok=True)
        
        return str(custom_path)
    
    def get_relative_path(self, full_path: str) -> str:
        """Retorna o caminho relativo à base de downloads"""
        try:
            full = Path(full_path)
            base = Path(self.config.default_path)
            return str(full.relative_to(base))
        except ValueError:
            # Se não for subcaminho, retorna o nome do arquivo
            return Path(full_path).name
    
    def update_config(self, default_path: Optional[str] = None, 
                     platform_paths: Optional[Dict[str, str]] = None,
                     temporary: bool = False) -> Config:
        """
        Atualiza configurações
        
        Args:
            default_path: Novo caminho padrão
            platform_paths: Novos caminhos por plataforma
            temporary: Se True, não salva no arquivo
            
        Returns:
            Configuração atualizada
        """
        if default_path:
            self.config.default_path = default_path
        
        if platform_paths:
            self.config.platform_paths.update(platform_paths)
        
        if not temporary:
            self.save_config(self.config)
        
        return self.config
    
    def get_config(self) -> Config:
        """Retorna configuração atual"""
        return self.config
