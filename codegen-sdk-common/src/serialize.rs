use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use std::path::PathBuf;

pub fn get_serialize_path(path: &PathBuf) -> anyhow::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("codegen")?;
    let path = path.as_os_str().to_str().unwrap();
    let version = buildid::build_id().unwrap();
    let encoded = format!("{}/{}", URL_SAFE.encode(version), URL_SAFE.encode(path));
    Ok(xdg_dirs.place_cache_file(encoded)?)
}
