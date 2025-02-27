use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use bytes::Bytes;
use rkyv::{
    Archive, Deserialize,
    bytecheck::CheckBytes,
    de::Pool,
    from_bytes,
    rancor::{Error, Strategy},
    ser::writer::IoWriter,
    validation::{Validator, archive::ArchiveValidator, shared::SharedValidator},
};
use sha2::{Digest, Sha256};
use zstd::stream::AutoFinishEncoder;
type Writer<'a> = IoWriter<
    AutoFinishEncoder<
        'a,
        BufWriter<File>,
        Box<dyn FnMut(Result<BufWriter<File>, std::io::Error>) + Send>,
    >,
>;
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
    fn read_entry_raw(&self, path: &PathBuf) -> Result<Bytes, ParseError> {
        let file = File::open(path)?;
        let mut buf = Vec::new();
        let mut reader = zstd::Decoder::new(BufReader::new(file))?;
        reader.read_to_end(&mut buf)?;
        Ok(Bytes::from(buf))
    }
    pub fn read_entry<T: Archive>(&self, path: &PathBuf) -> Result<T, ParseError>
    where
        T::Archived:
            for<'a> CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, Error>>,
        T::Archived: Deserialize<T, Strategy<Pool, Error>>,
    {
        let bytes = self.read_entry_raw(path)?;
        let value = from_bytes::<T, rkyv::rancor::Error>(&bytes)?;
        Ok(value)
    }
    pub fn get_writer<'a>(&self, path: &PathBuf) -> Result<Writer<'a>, ParseError> {
        let file = File::create(path)?;
        let writer = zstd::Encoder::new(BufWriter::new(file), 1)?.auto_finish();
        Ok(IoWriter::new(writer))
    }
}
