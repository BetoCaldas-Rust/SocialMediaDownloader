FROM python:3.11-slim

# Instala ffmpeg (necessário para yt-dlp processar vídeos)
RUN apt-get update && \
    apt-get install -y ffmpeg && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Define diretório de trabalho
WORKDIR /app

# Instala UV - gerenciador de pacotes Python ultrarrápido
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

# Copia requirements e instala dependências com UV
COPY requirements.txt .
RUN uv pip install --system --no-cache -r requirements.txt

# Copia código do backend
COPY . .

# Expõe porta da API
EXPOSE 8000

# Comando para iniciar o servidor
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
