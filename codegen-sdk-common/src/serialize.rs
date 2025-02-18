use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use std::path::PathBuf;

pub fn get_serialize_path(path: &PathBuf) -> anyhow::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("codegen")?;
    let path = path.as_os_str().to_str().unwrap();
    let version = "1";
    let encoded = URL_SAFE.encode(path) + "_" + version;
    Ok(xdg_dirs.place_cache_file(encoded)?)
}
