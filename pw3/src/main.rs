mod lexer;
mod term_index;
mod file;
mod common;
mod position;
mod document;
mod query_lang;
mod inf_context;
mod two_word_index;

use std::{env, io};
use std::fs::File;
use std::io::BufWriter;
use anyhow::{Context, Result};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use itertools::Itertools;
use crate::common::add_file_to_index;
use crate::inf_context::InfContext;
use crate::term_index::TermIndex;

fn time_call<FnT, ResT>(func: FnT) -> (ResT, Duration)
where FnT: FnOnce() -> ResT
{
    let start = Instant::now();
    let result = func();
    let time = start.elapsed();

    (result, time)
}

fn query(query_text: &str, index: &dyn TermIndex, ctx: &InfContext) -> Result<()> {
    let ast = query_lang::parse_logic_expr(query_text).context("Invalid query")?;
    // println!("Ast: {ast:?}");

    let (result, time) = time_call(|| index.query(&ast));
    let result = result?;

    println!("Query time: {:?}.", time);
    if !result.is_empty() {
        let result_str = result.iter()
            .sorted()
            .filter_map(|&id| ctx.document(id).map(|doc| (id, doc)))
            .enumerate()
            .map(|(i, (id, doc))| format!("\t{}. [{}] {}", i, id, doc.name()))
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

    let ctx = InfContext::new(base_path)?;
    let mut document_ids = ctx.document_ids().collect::<Vec<_>>();
    let document_count = document_ids.len();
    println!("Processing {document_count} documents in folder \"{base_path}\"");
    println!("Files: ");

    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    for (i, document_id) in document_ids.drain(..).enumerate() {
        let tx = tx.clone();
        let ctx1 = ctx.clone();

        println!("\t{}. {}", i, ctx1.document(document_id).unwrap().name());

        pool.execute(move || {
            tx.send(add_file_to_index(document_id, ctx1).unwrap()).unwrap()
        });
    }

    let result = rx.iter()
        .take(document_count)
        .flatten()
        .reduce(|mut a, b| {
            a.0.merge(b.0);
            a.1.merge(b.1);
            a.2.merge(b.2);

            a
        });

    if let Some((inverted_index, two_word_index, stats)) = result {
        println!("Unique word count: {}. Total word count: {}", inverted_index.unique_word_count(), inverted_index.total_word_count());
        println!("Lines read: {}. Characters read: {}. Characters ignored: {}", stats.lines, stats.characters_read, stats.characters_ignored);

        println!("Writing index to a file...");
        serde_json::to_writer_pretty(BufWriter::new(File::create("data/index.json")?), &inverted_index)?;
        serde_json::to_writer_pretty(BufWriter::new(File::create("data/two_word_index.json")?), &two_word_index)?;

        let mut buffer = String::new();
        let mut use_inverted_index = true;
        loop {
            println!("Please input your query or 'q' to exit: ");
            io::stdin().read_line(&mut buffer)?;
            if buffer.trim() == "q" {
                break;
            }
            if buffer.trim() == "s" {
                use_inverted_index = !use_inverted_index;
                let index_name = if use_inverted_index { "inverted coordinate index" } else { "two word index" };
                println!("Switched index to {index_name}. Input 's' to return back.");
                buffer.clear();
                continue;
            }

            let index: &dyn TermIndex = if use_inverted_index { &inverted_index } else { &two_word_index };

            if let Err(err) = query(&buffer, index, &ctx) {
                println!("Error: {}. Caused by: {}", err, err.root_cause());
            }
            println!();

            buffer.clear();
        }
    } else {
        println!("No files were processed.");
    }

    Ok(())
}
