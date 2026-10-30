use blocks_iterator::bitcoin::consensus::serialize;
use blocks_iterator::BlockExtra;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

/// Writes the blocks whose height is a multiple of `every` to a file, in the same format read from
/// stdin, so that a small but representative dataset can be replayed locally.
///
/// Blocks are written to a temporary file renamed at the end, so an interrupted run doesn't
/// leave a truncated sample.
pub struct Sampler {
    path: PathBuf,
    tmp_path: PathBuf,
    writer: BufWriter<File>,
    every: u64,
}

impl Sampler {
    pub fn new(path: PathBuf, every: u64) -> io::Result<Self> {
        let mut tmp_path = path.clone().into_os_string();
        tmp_path.push(".tmp");
        let tmp_path = PathBuf::from(tmp_path);
        let writer = BufWriter::new(File::create(&tmp_path)?);
        Ok(Sampler {
            path,
            tmp_path,
            writer,
            every,
        })
    }

    pub fn maybe_write(&mut self, block_extra: &BlockExtra) -> io::Result<()> {
        if u64::from(block_extra.height()) % self.every == 0 {
            self.writer.write_all(&serialize(block_extra))?;
        }
        Ok(())
    }

    pub fn finish(self) -> io::Result<()> {
        self.writer.into_inner()?.sync_all()?;
        fs::rename(&self.tmp_path, &self.path)
    }
}
