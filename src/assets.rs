use anyhow::anyhow;
use gpui_kit::{AssetSource, Result, SharedString};
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
        gpui_kit::assets::Assets
            .load(path)
            .map_err(|error| anyhow!(error))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut assets = Self::iter()
            .filter_map(|asset| asset.starts_with(path).then(|| asset.into()))
            .collect::<Vec<_>>();
        assets.extend(gpui_kit::assets::Assets.list(path)?);
        Ok(assets)
    }
}
