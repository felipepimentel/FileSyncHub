use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

/// Configuração de credenciais para o Google Drive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub token: Option<String>,
}

/// Configuração de credenciais para o OneDrive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub token: Option<String>,
}

/// Tipos de provedores suportados
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderCredentials {
    GoogleDrive(GoogleDriveCredentials),
    OneDrive(OneDriveCredentials),
}

/// Mapeamento de pastas locais para remotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMapping {
    pub local_path: PathBuf,
    pub remote_path: String,
}

/// Configuração de um provedor de sincronização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub credentials: ProviderCredentials,
    pub mappings: Vec<FolderMapping>,
    pub enabled: bool,
}

/// Configuração principal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // 1. Primeiro tenta carregar das variáveis de ambiente
        if let Ok(config) = Self::from_env() {
            return Ok(config);
        }

        // 2. Se não encontrar, tenta carregar do arquivo de configuração do usuário
        Self::from_file()
    }

    fn from_env() -> Result<Self> {
        let mut providers = Vec::new();

        // Google Drive from env
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GOOGLE_DRIVE_CLIENT_ID"),
            std::env::var("GOOGLE_DRIVE_CLIENT_SECRET"),
        ) {
            let credentials = GoogleDriveCredentials {
                client_id,
                client_secret,
                token: std::env::var("GOOGLE_DRIVE_TOKEN").ok(),
            };

            // Exemplo de mapeamento via env (pode ser expandido conforme necessário)
            let mappings = if let Ok(mappings_str) = std::env::var("GOOGLE_DRIVE_MAPPINGS") {
                Self::parse_mappings_from_env(&mappings_str)?
            } else {
                Vec::new()
            };

            providers.push(ProviderConfig {
                name: "googledrive-main".to_string(),
                credentials: ProviderCredentials::GoogleDrive(credentials),
                mappings,
                enabled: true,
            });
        }

        // OneDrive from env (similar ao Google Drive)
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("ONEDRIVE_CLIENT_ID"),
            std::env::var("ONEDRIVE_CLIENT_SECRET"),
        ) {
            let credentials = OneDriveCredentials {
                client_id,
                client_secret,
                token: std::env::var("ONEDRIVE_TOKEN").ok(),
            };

            let mappings = if let Ok(mappings_str) = std::env::var("ONEDRIVE_MAPPINGS") {
                Self::parse_mappings_from_env(&mappings_str)?
            } else {
                Vec::new()
            };

            providers.push(ProviderConfig {
                name: "onedrive-main".to_string(),
                credentials: ProviderCredentials::OneDrive(credentials),
                mappings,
                enabled: true,
            });
        }

        Ok(Config { providers })
    }

    fn parse_mappings_from_env(mappings_str: &str) -> Result<Vec<FolderMapping>> {
        // Formato esperado: "local1->remote1;local2->remote2"
        let mut mappings = Vec::new();
        
        for mapping_str in mappings_str.split(';') {
            if let Some((local, remote)) = mapping_str.split_once("->") {
                mappings.push(FolderMapping {
                    local_path: PathBuf::from(local.trim()),
                    remote_path: remote.trim().to_string(),
                });
            }
        }

        Ok(mappings)
    }

    fn from_file() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Config { providers: Vec::new() });
        }

        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))
    }

    fn get_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        
        let config_dir = PathBuf::from(home).join(".config").join("filesync");
        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))
    }

    /// Encontra um provedor pelo nome
    pub fn find_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }

    /// Encontra um provedor pelo nome e retorna mutável
    pub fn find_provider_mut(&mut self, name: &str) -> Option<&mut ProviderConfig> {
        self.providers.iter_mut().find(|p| p.name == name)
    }
}
