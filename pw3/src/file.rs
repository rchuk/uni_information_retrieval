use std::fmt::{Display, Formatter};
use anyhow::{Context, Result};
use memmap::Mmap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct FileId(usize);

impl Display for FileId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "File({})", self.0)
    }
}

pub struct FilePool {
    files: Vec<File>
}

impl FilePool {
    pub fn new() -> Self {
        FilePool {
            files: Vec::new()
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn file(&self, file_id: FileId) -> Option<&File> {
        self.files.get(file_id.0)
    }

    pub fn add_file(&mut self, path: &PathBuf) -> Result<FileId> {
        let file = File::new(path)?;
        let id = self.files.len();
        self.files.push(file);

        Ok(FileId(id))
    }
}

pub struct File {
    mmap: Option<Mmap>
}

impl File {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let file = fs::File::open(path)?;
        if file.metadata()?.len() == 0 {
            return Ok(File { mmap: None });
        }
        let mmap = unsafe { Mmap::map(&file)? };

        std::str::from_utf8(&mmap).context("File contains non UTF-8 data")?;

        Ok(File { mmap: Some(mmap) })
    }

    pub fn str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(self.bytes())
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match &self.mmap {
            Some(mmap) => &mmap,
            None => &[]
        }
    }
}
