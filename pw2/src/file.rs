use anyhow::{Context, Result};
use memmap::Mmap;
use std::fs;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Serialize)]
#[derive(Debug)]
pub struct File {
    path: PathBuf,
    #[serde(skip)]
    file: Mmap
}

impl File {
    pub fn new(path: PathBuf) -> Result<Option<Self>> {
        let file = fs::File::open(&path)?;
        if file.metadata()?.len() == 0 {
            return Ok(None);
        }
        let file = unsafe { Mmap::map(&file)? };

        std::str::from_utf8(&file).context("File contains non UTF-8 data")?;

        Ok(Some(File { path, file }))
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(self.bytes())
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.file
    }
}
