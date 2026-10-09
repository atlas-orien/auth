use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct JwtVerifyConfigResponse {
    pub public_key_pem: String,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayUserIdToUuidResponse {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserByIdentifierQuery {
    pub identifier: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterAvatarProviderRequest {
    pub public_base_url: String,
}
