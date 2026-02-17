import os
import sys
import shutil
import urllib.request
import tarfile
import json
from pathlib import Path

def get_update_dir():
    """Retorna o diretório de updates no APPDATA no Windows"""
    if os.name == 'nt':
        appdata = os.environ.get('APPDATA')
        if appdata:
            path = Path(appdata) / "SMD" / "updates"
            path.mkdir(parents=True, exist_ok=True)
            return path
    return Path("updates")

def update_yt_dlp():
    """Baixa e extrai a versão mais recente do yt-dlp"""
    update_dir = get_update_dir()
    url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.tar.gz"
    tar_path = update_dir / "yt-dlp.tar.gz"
    extract_path = update_dir / "yt-dlp_source"

    try:
        print(f"📥 Baixando yt-dlp de {url}...")
        with urllib.request.urlopen(url) as response, open(tar_path, 'wb') as out_file:
            shutil.copyfileobj(response, out_file)
        
        print(f"📦 Extraindo {tar_path}...")
        with tarfile.open(tar_path, "r:gz") as tar:
            tar.extractall(path=extract_path)
        
        # O tarball do GitHub geralmente tem uma subpasta como yt-dlp-202X.XX.XX
        # Precisamos encontrar a pasta que contém o pacote 'yt_dlp'
        for root, dirs, files in os.walk(extract_path):
            if 'yt_dlp' in dirs:
                # Encontramos o diretório pai do pacote
                final_path = Path(root)
                # Salva o caminho atualizado para o main.py usar
                with open(update_dir / "current_path.txt", "w") as f:
                    f.write(str(final_path.absolute()))
                
                print(f"✅ yt-dlp atualizado em: {final_path}")
                
                # Sendo um update crítico, solicitamos restart
                restart_smd()
                return True
                
        return False
    except Exception as e:
        print(f"❌ Erro ao atualizar yt-dlp: {e}")
        return False
    finally:
        if tar_path.exists():
            os.remove(tar_path)

def restart_smd():
    """Solicita restart do usuário e tenta relançar o frontend Rust"""
    print("\n" + "="*40)
    print("🔄 ATUALIZAÇÃO CONCLUÍDA!")
    print("O yt-dlp foi atualizado para a versão mais recente.")
    print("⌨️ Pressione qualquer tecla para REINICIAR o SMD...")
    print("="*40)
    
    # Aguarda input no console
    os.system("pause > nul")
    
    try:
        # Tenta encontrar o executável do Rust (social-media-downloader.exe)
        # sys.executable é o smd-backend.exe quando congelado
        exe_path = Path(sys.executable)
        root_dir = exe_path.parent
        
        # O nome do frontend pode variar se estiver rodando via cargo ou instalado
        smd_exe = root_dir / "social-media-downloader.exe"
        
        if smd_exe.exists():
            print(f"🚀 Reiniciando {smd_exe.name}...")
            os.startfile(smd_exe)
        else:
            print("ℹ️ Não foi possível reiniciar automaticamente. Por favor, abra o SMD manualmente.")
            
    except Exception as e:
        print(f"⚠️ Erro ao tentar reiniciar: {e}")
    
    # Mata o processo atual (backend)
    os._exit(0)

def apply_update_path():
    """Injeta o caminho do update no sys.path se existir"""
    update_dir = get_update_dir()
    path_file = update_dir / "current_path.txt"
    if path_file.exists():
        try:
            with open(path_file, "r") as f:
                update_path = f.read().strip()
                if os.path.exists(update_path):
                    # Injeta no início do path para ter prioridade sobre a versão congelada
                    sys.path.insert(0, update_path)
                    import yt_dlp
                    print(f"✨ Usando yt-dlp atualizado (v{yt_dlp.version.__version__}) de {update_path}")
                    return True
        except Exception as e:
            print(f"⚠️ Falha ao carregar yt-dlp atualizado: {e}")
    return False
