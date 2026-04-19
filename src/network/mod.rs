
use crate::executor::Executor;
use crate::parser;
use crate::r#type::QueryResult;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_server(address: &str, executor: Arc<Mutex<Executor>>) {
    let listner = TcpListener::bind(address).expect("Cannot bind to the address");

    println!("Listening on {}", address);

    for stream in listner.incoming() {
        match stream {
            Err(e) => println!("Connection error {}", e),
            Ok(stream) => {
                let executor = Arc::clone(&executor);
                thread::spawn(move || {
                    handle_client(stream, executor);
                });
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, executor: Arc<Mutex<Executor>>) {
    let reader = BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let line = match line {
            Err(_) => break,
            Ok(l) => l,
        };

        let line = line.trim().to_string();

        if line.is_empty() {
            continue;
        }
        if line == "exit" || line == "quit" {
            break;
        }

        let stmt = match parser::parse(&line) {
            Err(e) => {
                let _ = writeln!(stream, "Parse error : {}", e);
                continue;
            }
            Ok(s) => s,
        };

        let result = executor.lock().unwrap().execute(stmt);

        let response = match result {
            QueryResult::Created => "Table created.\n".to_string(),
            QueryResult::Inserted => "1 row inserted.\n".to_string(),
            QueryResult::Deleted(n) => format!("{} row(s) deleted.\n", n),
            QueryResult::Updated(n) => format!("{} row(s) updated.\n", n),
            QueryResult::IndexCreated => "Index created.\n".to_string(),
            QueryResult::IndexDropped => "Index dropped.\n".to_string(),
            QueryResult::Error(e) => format!("Error: {}\n", e),
            QueryResult::Rows { columns, rows } => format_rows(columns, rows),
        };

        let _ = write!(stream, "{}", response);
        let _ = stream.flush();
    }
}

fn format_rows(columns: Vec<String>, rows: Vec<crate::r#type::Row>) -> String {
    use crate::r#type::Value;
    let mut out = String::new();

    out.push_str(&columns.join(" | "));
    out.push('\n');
    out.push_str(&"-".repeat(columns.join(" | ").len()));
    out.push('\n');

    let count = rows.len();
    for row in rows {
        let vals: Vec<String> = row.values.iter().map(|v| match v {
            Value::Integer(n) => n.to_string(),
            Value::Text(s)    => s.clone(),
            Value::Null       => "NULL".to_string(),
        }).collect();
        out.push_str(&vals.join(" | "));
        out.push('\n');
    }

    out.push_str(&format!("({} row{})\n", count, if count == 1 { "" } else { "s" }));
    out
}
