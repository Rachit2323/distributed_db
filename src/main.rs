pub mod r#type;
mod parser;
mod storage;
mod executor;
mod wal;
mod index;

use std::io::{self, BufRead, Write};
use crate::r#type::{QueryResult, Value};

fn main() {
    // create ./data/ folder if it doesn't exist
    storage::ensure_data_dir().expect("Cannot create data dir");

    // load all table schemas from disk, build executor
    let mut executor = executor::Executor::new().expect("Cannot load schemas");

    let stdin = io::stdin();

    loop {
        // print the prompt — no newline, flush so it appears before user types
        print!("db> ");
        io::stdout().flush().unwrap();

        // read one line from keyboard
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,          // EOF (Ctrl+D)
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line == "exit" || line == "quit" {
            break;
        }

        // parse the SQL string into a Statement
        let stmt = match parser::parse(line) {
            Err(e) => {
                println!("Parse error: {}", e);
                continue;
            }
            Ok(s) => s,
        };

        // execute the statement and print the result
        match executor.execute(stmt) {
            QueryResult::Created => println!("Table created."),
            QueryResult::Inserted => println!("1 row inserted."),
            QueryResult::Deleted(n) => println!("{} row(s) deleted.", n),
            QueryResult::Updated(n) => println!("{} row(s) updated.", n),
            QueryResult::Error(e) => println!("Error: {}", e),
            QueryResult::Rows { columns, rows } => print_rows(columns, rows),
            QueryResult::IndexCreated => todo!(),
            QueryResult::IndexDropped => todo!(),
        }
    }

    println!("Bye.");
}

fn print_rows(columns: Vec<String>, rows: Vec<crate::r#type::Row>) {
    // print header
    println!("{}", columns.join(" | "));

    // print separator
    let sep_len = columns.join(" | ").len();
    println!("{}", "-".repeat(sep_len));

    // print each row
    let row_count = rows.len();
    for row in rows {
        let vals: Vec<String> = row.values.iter().map(|v| match v {
            Value::Integer(n) => n.to_string(),
            Value::Text(s)    => s.clone(),
            Value::Null       => "NULL".to_string(),
        }).collect();
        println!("{}", vals.join(" | "));
    }

    // print row count
    println!("({} row{})", row_count, if row_count == 1 { "" } else { "s" });
}
