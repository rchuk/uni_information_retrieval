use anyhow::Result;
use memmap::Mmap;
use std::fs::File;
use std::path::Path;
use std::str::Utf8Error;

pub struct Document {
    file: Mmap
}

impl Document {
    pub fn new(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let file = File::open(path)?;
        if file.metadata()?.len() == 0 {
            return Ok(None);
        }
        let file = unsafe { Mmap::map(&file)? };

        Ok(Some(Document { file }))
    }

    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.bytes())
    }

    pub unsafe fn to_str_unchecked(&self) -> &str {
        std::str::from_utf8_unchecked(self.bytes())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.file
    }
}
