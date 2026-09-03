#[derive(Clone, Debug)]
pub(crate) struct Settings {
    pub(crate) api_base_url: String,
}

impl Settings {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_base_url: std::env::var("SUNSYNK_API_URL")
                .unwrap_or_else(|_| "https://api.sunsynk.net".into()),
        })
    }
}
