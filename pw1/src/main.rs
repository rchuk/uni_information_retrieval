mod tests;
mod lexer;
mod storage;
mod dictionary;
mod document;
mod common;

use std::env;
use anyhow::Result;
use threadpool::ThreadPool;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use crate::common::add_file_to_dict;
use crate::storage::{DictionaryStorage, JsonDictionaryStorage, KeyValDictionaryStorage};

fn get_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .map(|entry| entry.ok())
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let base_path = args.get(1).map(AsRef::as_ref).unwrap_or("data/shakespeare");

    let paths = match get_files(base_path) {
        Ok(paths) => paths,
        Err(err) => {
            println!("Error occured: {}", err);

            return Ok(());
        }
    };
    if paths.is_empty() {
        println!("There are no files in the given folder!");

        return Ok(());
    }
    let job_count = paths.len();
    println!("Processing {job_count} documents in folder \"{base_path}\"");
    println!("Files: ");
    paths.iter()
        .map(|path| path.display())
        .enumerate()
        .for_each(|(i, path)| println!("\t{i}. {path}"));

    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    for path in paths {
        let tx = tx.clone();
        pool.execute(move || {
            tx.send(add_file_to_dict(path).unwrap()).unwrap();
        });
    }

    let result = rx.iter()
        .take(job_count)
        .flatten()
        .reduce(|mut a, b| {
            a.0.merge(b.0);
            a.1.merge(b.1);

            a
        });

    if let Some((dictionary, stats)) = result {
        println!("Unique word count: {}. Total word count: {}", dictionary.unique_word_count(), dictionary.total_word_count());
        println!("Lines read: {}. Characters read: {}. Characters ignored: {}", stats.lines, stats.characters_read, stats.characters_ignored);

        println!("Writing dictionary to file...");
        JsonDictionaryStorage::write(Path::new("data/dictionary.json"), &dictionary)?;
        KeyValDictionaryStorage::write(Path::new("data/dictionary.txt"), &dictionary)?;

        println!("Reading dictionary from a file");
        let dict1 = JsonDictionaryStorage::read(Path::new("data/dictionary.json"))?;
        let dict2 = KeyValDictionaryStorage::read(Path::new("data/dictionary.txt"))?;
        println!("Dictionary[1] (json) Unique word count: {}. Total word count: {}", dict1.unique_word_count(), dict1.total_word_count());
        println!("Dictionary[2] (txt) Unique word count: {}. Total word count: {}", dict2.unique_word_count(), dict2.total_word_count());
    } else {
        println!("No files were processed.");
    }

    Ok(())
}
