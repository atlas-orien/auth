use std::sync::{OnceLock, RwLock};

#[derive(Debug, Clone)]
pub struct AvatarProvider {
    pub public_base_url: String,
}

static AVATAR_PROVIDER: OnceLock<RwLock<Option<AvatarProvider>>> = OnceLock::new();

fn provider_store() -> &'static RwLock<Option<AvatarProvider>> {
    AVATAR_PROVIDER.get_or_init(|| RwLock::new(None))
}

pub fn register_avatar_provider(public_base_url: String) {
    *provider_store()
        .write()
        .expect("avatar provider lock poisoned") = Some(AvatarProvider { public_base_url });
}

pub fn get_avatar_provider() -> Option<AvatarProvider> {
    provider_store()
        .read()
        .expect("avatar provider lock poisoned")
        .clone()
}

#[cfg(test)]
mod tests {
    use super::{get_avatar_provider, register_avatar_provider};

    #[test]
    fn stores_latest_avatar_provider() {
        register_avatar_provider("https://files.example.com".to_string());
        assert_eq!(
            get_avatar_provider().unwrap().public_base_url,
            "https://files.example.com"
        );
    }
}
