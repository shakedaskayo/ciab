use ciab_core::types::config::OAuthProviderConfig;

use crate::oauth2_flow::OAuth2Flow;

pub struct GcpOAuth {
    #[allow(dead_code)]
    flow: OAuth2Flow,
}

impl GcpOAuth {
    pub fn new(config: &OAuthProviderConfig, client_secret: String) -> Self {
        Self {
            flow: OAuth2Flow::new(config, client_secret),
        }
    }
}
