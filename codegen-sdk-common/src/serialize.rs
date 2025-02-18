use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};
use crate::ParseError;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use bytes::Bytes;
use rkyv::ser::writer::IoWriter;
use sha2::{Digest, Sha256};
use zstd::stream::AutoFinishEncoder;
pub fn get_serialize_path(path: &PathBuf) -> anyhow::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("codegen")?;
    let path = path.as_os_str().to_str().unwrap();
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let path_hash = hasher.finalize();
    let version = buildid::build_id().unwrap();
    let encoded = format!(
        "{}/{}",
        URL_SAFE.encode(version),
        URL_SAFE.encode(path_hash)
    );
    Ok(xdg_dirs.place_cache_file(encoded)?)
}
pub fn read_bytes(path: &PathBuf) -> Result<Bytes, ParseError> {
    let file = File::open(path)?;
    let mut buf = Vec::new();
    let mut reader = zstd::Decoder::new(BufReader::new(file))?;
    reader.read_to_end(&mut buf)?;
    Ok(Bytes::from(buf))
}
pub fn get_writer(
    path: &PathBuf,
) -> Result<
    IoWriter<
        AutoFinishEncoder<
            '_,
            BufWriter<File>,
            Box<dyn FnMut(Result<BufWriter<File>, std::io::Error>) + Send>,
        >,
    >,
    ParseError
> {
    let file = File::create(path)?;
    let writer = IoWriter::new(zstd::Encoder::new(BufWriter::new(file), 1)?.auto_finish());
    Ok(writer)
}
