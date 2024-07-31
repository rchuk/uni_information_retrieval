mod lexer;
mod term_index;
mod file;
mod common;
mod position;
mod document;
mod logic_op;

use std::collections::HashSet;
use std::{env, io};
use std::fs::File;
use std::io::BufWriter;
use std::ops::{BitAnd, BitOr, Not, Sub};
use anyhow::{Context, Result};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use bitvec::vec::BitVec;
use itertools::Itertools;
use crate::common::add_file_to_index;
use crate::document::DocumentRegistry;
use crate::logic_op::LogicNode;
use crate::position::DocumentId;
use crate::term_index::{InvertedIndex, TermIndex, TermMatrix};

fn query_matrix_build(index: &TermMatrix, query_ast: &LogicNode) -> BitVec {
    match query_ast {
        LogicNode::False => BitVec::new(),
        LogicNode::Term(term) => index.get_term_query(term),
        LogicNode::And(lhs, rhs) => {
            query_matrix_build(index, lhs) & query_matrix_build(index, rhs)
        },
        LogicNode::Or(lhs, rhs) => {
            query_matrix_build(index, lhs) | query_matrix_build(index, rhs)
        },
        LogicNode::Not(operand) => {
            !query_matrix_build(index, operand)
        }
    }
}

fn query_matrix(matrix: &TermMatrix, query_ast: &LogicNode) -> HashSet<DocumentId> {
    let query = query_matrix_build(matrix, query_ast);

    matrix.get_term_documents(&query)
}

fn query_index(index: &InvertedIndex, query_ast: &LogicNode) -> HashSet<DocumentId> {
    match query_ast {
        LogicNode::False => HashSet::new(),
        LogicNode::Term(term) => index.get_term_documents(term),
        LogicNode::And(lhs, rhs) => {
            &query_index(index, lhs) & &query_index(index, rhs)
        },
        LogicNode::Or(lhs, rhs) => {
            &query_index(index, lhs) | &query_index(index, rhs)
        },
        LogicNode::Not(operand) => {
            &index.get_documents() - &query_index(index, &operand)
        }
    }
}

fn time_call<FnT, ResT>(func: FnT) -> (ResT, Duration)
where FnT: FnOnce() -> ResT
{
    let start = Instant::now();
    let result = func();
    let time = start.elapsed();

    (result, time)
}

fn query(document_registry: &DocumentRegistry, index: &InvertedIndex, matrix: &TermMatrix, query_text: &str) -> Result<()> {
    let ast = logic_op::parse_logic_expr(query_text).context("Invalid query")?;

    let (index_result, index_time) = time_call(|| query_index(index, &ast));
    let (matrix_result, matrix_time) = time_call(|| query_matrix(matrix, &ast));

    println!("Results match: {}", index_result == matrix_result);
    println!("Inverted index time {:?}. Matrix index time: {:?}", index_time, matrix_time);
    if !index_result.is_empty() {
        let result_str = index_result.iter()
            .sorted()
            .map(|&id| document_registry.get_document(id))
            .flatten()
            .enumerate()
            .map(|(i, document)| format!("\t{}. [{}] {}", i, document.id().0, document.name()))
            .join("\n");
        println!("Result: {result_str}");
    } else {
        println!("No matches found");
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let base_path = args.get(1).map(AsRef::as_ref).unwrap_or("data/shakespeare");

    let document_registry = DocumentRegistry::new(base_path)?;
    let job_count = document_registry.documents_count();
    println!("Processing {job_count} documents in folder \"{base_path}\"");
    println!("Files: ");

    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    for i in 0..job_count {
        let tx = tx.clone();
        let registry = document_registry.clone();

        println!("\t{}. {}", i, document_registry.get_document(DocumentId(i)).unwrap().name());

        pool.execute(move || {
            tx.send(add_file_to_index(registry, DocumentId(i)).unwrap()).unwrap()
        });
    }

    let result = rx.iter()
        .take(job_count)
        .flatten()
        .reduce(|mut a, b| {
            a.0.merge(b.0);
            a.1.merge(b.1);
            a.2.merge(b.2);

            a
        });

    if let Some((index, matrix, stats)) = result {
        println!("Unique word count: {}. Total word count: {}", index.unique_word_count(), index.total_word_count());
        println!("Lines read: {}. Characters read: {}. Characters ignored: {}", stats.lines, stats.characters_read, stats.characters_ignored);

        println!("Writing index to a file...");
        serde_json::to_writer_pretty(BufWriter::new(File::create("data/index.json")?), &index)?;

        let mut buffer = String::new();
        loop {
            println!("Please input your query or 'q' to exit: ");
            io::stdin().read_line(&mut buffer)?;
            if buffer.trim() == "q" {
                break;
            }

            if let Err(err) = query(&document_registry, &index, &matrix, &buffer) {
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
