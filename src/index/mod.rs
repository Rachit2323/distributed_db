use crate::r#type::{DataType, Row, TableSchema, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};

const DATA_DIR: &str = "./data";

pub struct Index {
    pub table_name: String,
    pub column: String,
    pub map: HashMap<Value, Vec<usize>>,
}

pub fn build(table_name: &str, column: &str, scehma: &TableSchema, rows: Vec<Row>) -> Index {
    let col_index = scehma
        .columns
        .iter()
        .position(|c| c.name == column)
        .unwrap_or(0);

    let mut map: HashMap<Value, Vec<usize>> = HashMap::new();

    for (pos, row) in rows.iter().enumerate() {
        let val = row.values[col_index].clone();
        map.entry(val).or_insert_with(Vec::new).push(pos);
    }
    /* `index::Index` value */

    Index {
        table_name: table_name.to_string(),
        column: column.to_string(),
        map,
    }
}

pub fn save(index: &Index) -> Result<(), String> {
    let file_path = format!("{}/{}_{}.index", DATA_DIR, index.table_name, index.column);

    let mut file =
        File::create(&file_path).map_err(|e| format!("Cannot create index file: {}", e))?;

    for (value, pos) in &index.map {
        let value_str = match value {
            Value::Integer(n) => n.to_string(),
            Value::Text(s) => s.clone(),
            Value::Null => "".to_string(),
        };

        // convert positions list to string: [0,2] → "0,2"
        let pos_str = pos
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // write "1:0" or "paid:0,2"
        writeln!(file, "{}:{}", value_str, pos_str)
            .map_err(|e| format!("Cannot write index: {}", e))?;
    }

    Ok(())
}


pub fn lookup<'a>(index: &'a Index, value: &Value) -> Option<&'a Vec<usize>> {
    index.map.get(value)
}

pub fn update_on_insert(index: &mut Index, schema: &TableSchema, row: &Row, position: usize) {
    let col_index = schema.columns
        .iter()
        .position(|c| c.name == index.column)
        .unwrap_or(0);

    let value = row.values[col_index].clone();
    index.map.entry(value).or_insert_with(Vec::new).push(position);
}

pub fn rebuild(index: &mut Index, schema: &TableSchema, rows: Vec<Row>) {
    index.map.clear();

    let col_index = schema.columns
        .iter()
        .position(|c| c.name == index.column)
        .unwrap_or(0);

    for (pos, row) in rows.iter().enumerate() {
        let val = row.values[col_index].clone();
        index.map.entry(val).or_insert_with(Vec::new).push(pos);
    }
}

pub fn load(table_name: &str, column: &str, schema: &TableSchema) -> Result<Index, String> {
    let file_path = format!("{}/{}_{}.index", DATA_DIR, table_name, column);
    let file = File::open(&file_path)
        .map_err(|e| format!("Cannot open index file: {}", e))?;
    let reader = BufReader::new(file);

    // find column type from schema so we know how to parse values
    let col_type = schema.columns
        .iter()
        .find(|c| c.name == column)
        .map(|c| &c.data_type);

    let mut map: HashMap<Value, Vec<usize>> = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        if line.is_empty() { continue; }

        // "Rachit:0" → ["Rachit", "0"]
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 { continue; }

        let value_str    = parts[0];
        let position_str = parts[1];

        // parse value_str into Value using column type
        let value = match col_type {
            Some(DataType::Integer) => value_str.parse::<i64>()
                .map(Value::Integer)
                .unwrap_or(Value::Null),
            Some(DataType::Text) => Value::Text(value_str.to_string()),
            _ => Value::Null,
        };

        // parse "0,2,5" → [0, 2, 5]
        let positions: Vec<usize> = position_str
            .split(',')
            .filter_map(|p| p.parse::<usize>().ok())
            .collect();

        map.insert(value, positions);
    }

    Ok(Index {
        table_name: table_name.to_string(),
        column: column.to_string(),
        map,
    })
}
