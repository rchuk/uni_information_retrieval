use anyhow::{anyhow, Result};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use crate::dictionary::Dictionary;
use crate::storage::DictionaryStorage;

pub struct KeyValDictionaryStorage;

impl KeyValDictionaryStorage {
    const SEPARATOR: &'static str = "=";

    fn parse_line(line: String) -> Result<(String, usize)> {
        let mut split = line.split(Self::SEPARATOR);
        if let Some(first) = split.next() {
            let word = first.to_owned();
            if let Some(second) = split.next() {
                let count = usize::from_str(second)?;
                if let Some(extra) = split.next() {
                    return Err(anyhow!("Line must have word and size separated by \"{}\". Encountered extra: \"{}\"", Self::SEPARATOR, extra));
                }

                return Ok((word, count));
            }
        }

        return Err(anyhow!("Line must have word and size separated by \"{}\"", Self::SEPARATOR));
    }
}

impl DictionaryStorage for KeyValDictionaryStorage {
    fn read(path: &Path) -> Result<Dictionary> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut dictionary = Dictionary::new();
        let entries = reader.lines()
            .map(|line| line.ok())
            .flatten()
            .map(Self::parse_line);

        for entry in entries {
            let (word, count) = entry?;
            dictionary.add_word_with_count(word, count);
        }

        Ok(dictionary)
    }

    fn write(path: &Path, dictionary: &Dictionary) -> Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);

        for (word, count) in dictionary.word_counts().iter() {
            writeln!(writer, "{}{}{}", word, Self::SEPARATOR, count)?;
        }

        Ok(())
    }
}
