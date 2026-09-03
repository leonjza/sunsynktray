use anyhow::anyhow;
use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(asset) = Self::get(path) {
            return Ok(Some(asset.data));
        }
        gpui_component_assets::Assets
            .load(path)
            .map_err(|error| anyhow!(error))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut assets = Self::iter()
            .filter_map(|asset| asset.starts_with(path).then(|| asset.into()))
            .collect::<Vec<_>>();
        assets.extend(gpui_component_assets::Assets.list(path)?);
        Ok(assets)
    }
}
