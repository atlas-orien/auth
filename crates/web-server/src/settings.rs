use std::{fs, path::Path};

use axum::http::header::HeaderName;
use db_core::DatabaseConfig;
use serde::Deserialize;
use toolcraft_config::load_settings;
use toolcraft_jwt::JwtCfg;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub http: HttpCfg,
    pub jwt: JwtCfg,
    pub db: Vec<DatabaseConfig>,
    #[serde(default)]
    pub internal: InternalCfg,
}

#[derive(Debug, Deserialize)]
pub struct HttpCfg {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct InternalCfg {
    #[serde(default = "default_internal_auth_header")]
    pub auth_header: String,
    #[serde(default)]
    pub auth_token: String,
}

#[derive(Debug, Clone)]
pub struct JwtVerifyConfig {
    pub public_key_pem: String,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone)]
pub struct InternalAuthConfig {
    pub header_name: HeaderName,
    pub token: String,
}

fn default_internal_auth_header() -> String {
    "x-internal-token".to_string()
}

impl Settings {
    pub fn load(config_path: &str) -> Result<Self> {
        let r = load_settings(config_path)?;
        Ok(r)
    }

    pub fn build_jwt_verify_config(&self) -> Result<JwtVerifyConfig> {
        let public_key_pem = if let Some(key_dir) = &self.jwt.key_dir {
            let path = Path::new(key_dir).join("access_public_key.pem");
            fs::read_to_string(&path).map_err(|e| {
                crate::error::Error::Custom(format!(
                    "failed to read access public key from {}: {}",
                    path.display(),
                    e
                ))
            })?
        } else {
            self.jwt.access_public_key_pem.clone().ok_or_else(|| {
                crate::error::Error::Custom(
                    "missing jwt.access_public_key_pem when jwt.key_dir is not set".to_owned(),
                )
            })?
        };

        Ok(JwtVerifyConfig {
            public_key_pem,
            issuer: self.jwt.issuer.clone(),
            audience: self.jwt.audience.clone(),
        })
    }

    pub fn build_internal_auth_config(&self) -> Result<InternalAuthConfig> {
        if self.internal.auth_token.is_empty() {
            return Err(crate::error::Error::Custom(
                "missing internal.auth_token in config/services.toml".to_string(),
            ));
        }

        let header_name = HeaderName::from_bytes(self.internal.auth_header.trim().as_bytes())
            .map_err(|e| {
                crate::error::Error::Custom(format!(
                    "invalid internal.auth_header '{}': {}",
                    self.internal.auth_header, e
                ))
            })?;

        Ok(InternalAuthConfig {
            header_name,
            token: self.internal.auth_token.clone(),
        })
    }
}
