use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use crate::r#type::{DataType, Row, TableSchema, Value};
use crate::storage;

const DATA_DIR: &str = "./data";

pub fn write_entry(table_name: &str, row: &Row) -> Result<(), String> {
    let wal_path = format!("{}/{}.wal", DATA_DIR, table_name);

    let mut file =
        File::create(wal_path).map_err(|e| format!("Error while creating the file {}", e))?;

    let string: Vec<String> = row
        .values
        .iter()
        .map(|val| match val {
            Value::Integer(n) => n.to_string(),
            Value::Text(s) => s.clone(),
            Value::Null => "".to_string(),
        })
        .collect();

    writeln!(file, "INSERT|{}", string.join(","))
        .map_err(|e| format!("caanot write into WAL {}", e))?;

    Ok(())
}

pub fn clear_entry(table_name: &str) -> Result<(), String> {
    let file = format!("{}/{}.wal", DATA_DIR, table_name);

    std::fs::remove_file(file).map_err(|e| format!("Issue while clearning the file {}", e))?;
    Ok(())
}


pub fn recover(schemas: &HashMap<String, TableSchema>) -> Result<(), String> {
    // step 1: scan ./data/ for .wal files
    let entries = match fs::read_dir(DATA_DIR) {
        Err(_)       => return Ok(()),  // data dir doesn't exist yet
        Ok(entries)  => entries,
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("Dir read error: {}", e))?;
        let path = entry.path();

        // step 2: skip files that are not .wal
        if path.extension().and_then(|s| s.to_str()) != Some("wal") {
            continue;
        }

        // step 3: get table name from filename "users.wal" → "users"
        let table_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // step 4: read the WAL file content
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read WAL: {}", e))?;

        let line = content.trim();
        if line.is_empty() { continue; }

        // step 5: split "INSERT|1,Rachit" → ["INSERT", "1,Rachit"]
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 { continue; }

        let operation = parts[0];
        let row_data  = parts[1];

        if operation != "INSERT" { continue; }

        // step 6: find schema for this table
        let schema = match schemas.get(&table_name) {
            None    => continue,  // table doesn't exist, skip
            Some(s) => s,
        };

        // step 7: parse "1,Rachit" back into a Row
        let raw_parts: Vec<&str> = row_data.splitn(schema.columns.len(), ',').collect();
        let values: Vec<Value> = raw_parts.iter().zip(schema.columns.iter())
            .map(|(part, col)| {
                if part.is_empty() { return Value::Null; }
                match col.data_type {
                    DataType::Integer => part.parse::<i64>()
                        .map(Value::Integer)
                        .unwrap_or(Value::Null),
                    DataType::Text => Value::Text(part.to_string()),
                }
            })
            .collect();

        let row = Row { values };

        // step 8: replay the write
        storage::append_row(&table_name, &row)?;

        // step 9: delete the WAL file
        clear_entry(&table_name)?;
    }

    Ok(())
}