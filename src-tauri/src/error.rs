use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML serialize: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialize: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Config: {0}")]
    Config(String),

    #[error("Proxy: {0}")]
    Proxy(String),

    #[error("DB: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Launchctl: {0}")]
    Launchctl(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn other(msg: impl Into<String>) -> Self {
        AppError::Other(anyhow::anyhow!("{}", msg.into()))
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
