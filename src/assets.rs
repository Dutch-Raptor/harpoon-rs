use anyhow::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/assets"]
pub struct Asset;

pub fn get_app_icon_filepath() -> Result<String> {
    let mut path = std::env::temp_dir();
    path.push("harpoon_rs");
    path.push("harpoon.ico");

    if path.exists() {
        match path.to_str() {
            Some(s) => return Ok(s.to_string()),
            None => return Err(anyhow::anyhow!("Failed to convert path to string.")),
        }
    }

    let asset = match Asset::get("harpoon.ico") {
        Some(a) => a,
        None => return Err(anyhow::anyhow!("Failed to get asset.")),
    };

    // check if %temp%/harpoon_rs exists
    if !path.parent().unwrap().exists() {
        match std::fs::create_dir_all(path.parent().unwrap()) {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        }
    }

    match std::fs::write(&path, &asset.data) {
        Ok(_) => Ok(match path.to_str() {
            Some(s) => s.to_string(),
            None => return Err(anyhow::anyhow!("Failed to convert path to string.")),
        }),
        Err(e) => Err(e.into()),
    }
}
