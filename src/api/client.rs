use reqwest::Client;

use super::models::{
    Config, ConfigUpdateRequest, DownloadRequest, DownloadResponse, DownloadStatus, PlatformInfo,
};

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn health_check(&self) -> Result<bool, String> {
        let url = format!("{}/", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => Err(format!("Erro ao conectar: {}", e)),
        }
    }

    pub async fn start_download(
        &self,
        url: String,
        custom_path: Option<String>,
    ) -> Result<DownloadResponse, String> {
        let api_url = format!("{}/download", self.base_url);
        let request = DownloadRequest { url, custom_path };

        match self.client.post(&api_url).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .json::<DownloadResponse>()
                        .await
                        .map_err(|e| format!("Erro ao parsear resposta: {}", e))
                } else {
                    Err(format!("Erro HTTP: {}", response.status()))
                }
            }
            Err(e) => Err(format!("Erro ao enviar request: {}", e)),
        }
    }

    pub async fn get_download_status(&self, download_id: &str) -> Result<DownloadStatus, String> {
        let url = format!("{}/status/{}", self.base_url, download_id);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .json::<DownloadStatus>()
                        .await
                        .map_err(|e| format!("Erro ao parsear status: {}", e))
                } else {
                    Err(format!("Download não encontrado"))
                }
            }
            Err(e) => Err(format!("Erro ao consultar status: {}", e)),
        }
    }

    pub async fn get_config(&self) -> Result<Config, String> {
        let url = format!("{}/config", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .json::<Config>()
                        .await
                        .map_err(|e| format!("Erro ao parsear config: {}", e))
                } else {
                    Err(format!("Erro ao buscar config"))
                }
            }
            Err(e) => Err(format!("Erro ao consultar config: {}", e)),
        }
    }

    pub async fn update_config(&self, request: ConfigUpdateRequest) -> Result<Config, String> {
        let url = format!("{}/config", self.base_url);

        match self.client.post(&url).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .json::<Config>()
                        .await
                        .map_err(|e| format!("Erro ao parsear config: {}", e))
                } else {
                    Err(format!("Erro ao atualizar config"))
                }
            }
            Err(e) => Err(format!("Erro ao atualizar config: {}", e)),
        }
    }

    pub async fn detect_platform(&self, url: &str) -> Result<PlatformInfo, String> {
        let api_url = format!("{}/platform?url={}", self.base_url, url);

        match self.client.get(&api_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .json::<PlatformInfo>()
                        .await
                        .map_err(|e| format!("Erro ao detectar plataforma: {}", e))
                } else {
                    Err(format!("URL inválida"))
                }
            }
            Err(e) => Err(format!("Erro ao detectar plataforma: {}", e)),
        }
    }
}
