use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use bytes::Bytes;
use rkyv::ser::writer::IoWriter;
use sha2::{Digest, Sha256};
use zstd::stream::AutoFinishEncoder;

use crate::ParseError;
pub struct Cache {
    base_dir: PathBuf,
    build_id: String,
}
impl Cache {
    pub fn new() -> anyhow::Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("codegen")?;
        let build_id = buildid::build_id().unwrap();
        let encoded_build_id = URL_SAFE.encode(build_id);
        xdg_dirs.create_cache_directory(&encoded_build_id)?;
        Ok(Self {
            base_dir: xdg_dirs.get_cache_home(),
            build_id: encoded_build_id,
        })
    }
    pub fn get_path(&self, path: &PathBuf) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(path.as_os_str().to_str().unwrap().as_bytes());
        let path_hash = hasher.finalize();
        self.base_dir
            .join(format!("{}/{}", self.build_id, URL_SAFE.encode(path_hash)))
    }
    pub fn read_entry(&self, path: &PathBuf) -> Result<Bytes, ParseError> {
        let file = File::open(path)?;
        let mut buf = Vec::new();
        let mut reader = zstd::Decoder::new(BufReader::new(file))?;
        reader.read_to_end(&mut buf)?;
        Ok(Bytes::from(buf))
    }
    pub fn get_writer(
        &self,
        path: &PathBuf,
    ) -> Result<
        IoWriter<
            AutoFinishEncoder<
                BufWriter<File>,
                Box<dyn FnMut(Result<BufWriter<File>, std::io::Error>) + Send>,
            >,
        >,
        ParseError,
    > {
        let file = File::create(path)?;
        let writer = zstd::Encoder::new(BufWriter::new(file), 1)?.auto_finish();
        Ok(IoWriter::new(writer))
    }
}
