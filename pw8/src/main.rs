mod lexer;
mod term_index;
mod file;
mod common;
mod document;
mod inf_context;
mod term;

use std::{env, io};
use std::fs::File;
use std::io::BufWriter;
use std::str::FromStr;
use anyhow::{anyhow, Context, Result};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use human_bytes::human_bytes;
use itertools::Itertools;
use crate::common::add_file_to_index;
use crate::inf_context::InfContext;
use crate::term_index::{InvertedIndex, TermIndex};
use rayon::prelude::*;
use crate::document::DocumentId;
use crate::lexer::{Lexer, LexerStats};

const PREPROCESS_LEADER_COUNT: usize = 2;
const QUERY_LEADER_COUNT: usize = 2;

fn time_call<FnT, ResT>(func: FnT) -> (ResT, Duration)
where FnT: FnOnce() -> ResT
{
    let start = Instant::now();
    let result = func();
    let time = start.elapsed();

    (result, time)
}

fn query(query_text: &str, index: &dyn TermIndex, ctx: &InfContext) -> Result<()> {
    if query_text.is_empty() {
        return Err(anyhow!("Query can't be empty"));
    }

    let mut lexer = Lexer::new(DocumentId(0), query_text, ctx)?;
    let mut query_index = InvertedIndex::new();
    lexer.lex(&mut query_index);

    let (result, time) = time_call(|| index.query(&query_index.terms(), QUERY_LEADER_COUNT));
    let result = result?;

    println!("Query time: {time:?}.");
    if !result.is_empty() {
        let result_str = result.iter()
            .filter_map(|&(id, weight)| ctx.document(id).map(|doc| (id, doc, weight)))
            .enumerate()
            .map(|(i, (id, doc, weight))| format!("\t{}. [{}][W: {:.4}] {}", i, id, weight, doc.name()))
            .join("\n");
        println!("Result:\n{result_str}");
    } else {
        println!("No matches found.");
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let base_path = args.get(1).map(AsRef::as_ref).unwrap_or("data/shakespeare");
    let file_limit = args.get(2).map(|str| usize::from_str(str).ok()).unwrap_or(None);

    println!("Processing...");
    let (ctx, opening_files_time) = time_call(|| InfContext::new(base_path, file_limit).unwrap());
    println!("Opening files took: {opening_files_time:?}");
    let mut document_ids = ctx.document_ids().collect::<Vec<_>>();
    let document_count = document_ids.len();
    println!("Processing {document_count} documents in folder \"{base_path}\"");

    let pool = ThreadPool::new((num_cpus::get() - 1).max(1));
    let (tx, rx) = channel();
    for document_id in document_ids.drain(..) {
        let tx = tx.clone();
        let ctx1 = ctx.clone();

        pool.execute(move || {
            tx.send(add_file_to_index(document_id, ctx1).unwrap()).unwrap()
        });
    }

    let ((mut index, stats), index_time) = time_call(|| {
        rx.into_iter()
            .take(document_count)
            .flatten()
            .par_bridge()
            .reduce(|| (InvertedIndex::new(), LexerStats::default()), |mut a, b| {
                a.0.merge(b.0);
                a.1.merge(b.1);

                a
            })
    });

    println!("Indexing took: {index_time:?}");
    let total_time = opening_files_time + index_time;
    println!("Total time: {total_time:?}");
    let data_size: usize = ctx.files().files()
        .map(|file| file.bytes().len())
        .sum();
    println!("Amount of data indexed: {}", human_bytes(data_size as f64));
    println!("Speed is: {}/s", human_bytes(data_size as f64 / total_time.as_secs_f64()));

    println!("Unique word count: {}.", index.term_count());
    println!("Lines read: {}. Characters read: {}. Characters ignored: {}", stats.lines, stats.characters_read, stats.characters_ignored);

    println!("Writing index to a file...");
    index.save(BufWriter::new(File::create("data/index.txt")?))?;
    let index_size = File::open("data/index.txt")?.metadata()?.len();
    println!("Index size: {}", human_bytes(index_size as f64));

    index.preprocess(PREPROCESS_LEADER_COUNT);

    let mut buffer = String::new();
    loop {
        println!("Please input your query or 'q' to exit: ");
        io::stdin().read_line(&mut buffer)?;
        if buffer.trim() == "q" {
            break;
        }

        if let Err(err) = query(&buffer, &index, &ctx) {
            println!("Error: {}. Caused by: {}", err, err.root_cause());
        }
        println!();

        buffer.clear();
    }

    Ok(())
}
