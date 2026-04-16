  use std::collections::HashMap;
  use std::fs::{self, File, OpenOptions};                                                                  
  use std::io::{BufRead, BufReader, Write};
  use std::path::Path;                                                                                     
  use crate::r#type::{ColumnDef, DataType, Row, TableSchema, Value};

const DATA_DIR: &str = "./data";


pub fn ensure_data_dir() -> Result<(), String> {
    std::fs::create_dir_all(DATA_DIR)
        .map_err(|e| format!("Could not create data directory: {}", e))
}

pub fn create_table(schema: &TableSchema) -> Result<(), String> {
    // build file paths
    let csv_path    = format!("{}/{}.csv",    DATA_DIR, schema.name);
    let schema_path = format!("{}/{}.schema", DATA_DIR, schema.name);

    // check table does not already exist
    if Path::new(&csv_path).exists() {
        return Err(format!("Table '{}' already exists", schema.name));
    }

    // write schema file — one "name:Type" line per column
    let mut schema_file = File::create(&schema_path)
        .map_err(|e| format!("Cannot create schema file: {}", e))?;

    for col in &schema.columns {
        let type_str = match col.data_type {
            DataType::Integer => "Integer",
            DataType::Text    => "Text",
        };

        let is_pk = schema.primary_key.as_deref() == Some(col.name.as_str());
        if is_pk {
            writeln!(schema_file, "{}:{}:PK", col.name, type_str)
                .map_err(|e| format!("Cannot write schema: {}", e))?;
        } else {
            writeln!(schema_file, "{}:{}", col.name, type_str)
                .map_err(|e| format!("Cannot write schema: {}", e))?;
        }
    }

    // write csv file — just the header line (column names joined by comma)
    let mut csv_file = File::create(&csv_path)
        .map_err(|e| format!("Cannot create csv file: {}", e))?;

    let header: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
    writeln!(csv_file, "{}", header.join(","))
        .map_err(|e| format!("Cannot write csv header: {}", e))?;

    Ok(())
}


pub fn append_row (table_name :&str, row :&Row) -> Result<(),String> 
{
    let csv_path =  format!("{}/{}.csv",    DATA_DIR, table_name);
    let mut file = OpenOptions::new().append(true).open(&csv_path).map_err(|e| "Error in opening the file ".to_string())?;

    let strings:Vec<String> = row.values.iter().map(|val| {
        match val {
             Value::Integer(n) => n.to_string(),
             Value::Text(s)=> s.clone(),
             Value::Null=> "".to_string(),

        }
    }).collect();

    println!("String {:?}",strings);
    writeln!(file ,"{}",strings.join(",")).map_err(|e| "Error in writing the file".to_string())?;
    Ok(())
} 

pub fn read_rows(table_name: &str, schema: &TableSchema) -> Result<Vec<Row>, String> {
    let csv_path = format!("{}/{}.csv", DATA_DIR, table_name);
    let file = File::open(&csv_path)
        .map_err(|e| format!("Cannot open table '{}': {}", table_name, e))?;
    let reader = BufReader::new(file);

    let mut rows: Vec<Row> = Vec::new();
    let mut first_line = true;

    for line in reader.lines() {
        // unwrap the Result<String> that .lines() gives us
        let line = line.map_err(|e| format!("Read error: {}", e))?;

        // skip the header line ("id,name")
        if first_line {
            first_line = false;
            continue;
        }

        // skip empty lines
        if line.is_empty() {
            continue;
        }

        // split "1,Rachit" → ["1", "Rachit"]
        let parts: Vec<&str> = line.splitn(schema.columns.len(), ',').collect();

        // zip parts with columns to know the type of each value
        // ("1", id:Integer) → Value::Integer(1)
        // ("Rachit", name:Text) → Value::Text("Rachit")
        let values: Vec<Value> = parts
            .iter()
            .zip(schema.columns.iter())
            .map(|(part, col)| {
                if part.is_empty() {
                    return Value::Null;
                }
                match col.data_type {
                    DataType::Integer => part
                        .parse::<i64>()
                        .map(Value::Integer)
                        .unwrap_or(Value::Null),
                    DataType::Text => Value::Text(part.to_string()),
                }
            })
            .collect();

        rows.push(Row { values });
    }

    Ok(rows)
}




  pub fn load_schemas() -> Result<HashMap<String, TableSchema>, String> {

      let mut schemas = HashMap::new();

      // if data folder doesn't exist yet → no tables → return empty
      if !Path::new(DATA_DIR).exists() {
          return Ok(schemas);
      }

      // loop through every file in ./data/
      let entries = fs::read_dir(DATA_DIR)
          .map_err(|e| format!("Cannot read data dir: {}", e))?;

      for entry in entries {
          let entry = entry.map_err(|e| format!("Dir read error: {}", e))?;
          let path = entry.path();

          // skip files that are not .schema
          if path.extension().and_then(|s| s.to_str()) != Some("schema") {
              continue;
          }

          // get table name from filename  "users.schema" → "users"
          let table_name = path
              .file_stem()
              .and_then(|s| s.to_str())
              .unwrap_or("")
              .to_string();

          // read the file line by line
          let content = fs::read_to_string(&path)
              .map_err(|e| format!("Cannot read schema file: {}", e))?;

          let mut columns = Vec::new();
          let mut primary_key: Option<String> = None;

          for line in content.lines() {
              if line.is_empty() { continue; }

              // split "id:Integer:PK" → ["id", "Integer", "PK"]
              // or    "name:Text"     → ["name", "Text"]
              let parts: Vec<&str> = line.splitn(3, ':').collect();
              if parts.len() < 2 { continue; }

              let data_type = match parts[1] {
                  "Integer" => DataType::Integer,
                  "Text"    => DataType::Text,
                  _         => continue,
              };

              // check for :PK suffix
              if parts.len() == 3 && parts[2] == "PK" {
                  primary_key = Some(parts[0].to_string());
              }

              columns.push(ColumnDef {
                  name: parts[0].to_string(),
                  data_type,
              });
          }

          schemas.insert(table_name.clone(), TableSchema {
              name: table_name,
              columns,
              primary_key,
          });
      }

      Ok(schemas)
  }



  pub fn rewrite_rows(table_name :&str, schema: &TableSchema, rows:Vec<Row>) -> Result<(),String>
  {
     let csv_path =format!("{}/{}.csv",DATA_DIR,table_name);
     let mut file = File::create(csv_path).map_err(|e| format!("Issue while opening the file {:?}",e))?;
     let header : Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
     writeln!(file ,"{}", header.join(",")).map_err(|e| format!("Cannot write csv header {}",e))?;

     for row in rows {
     
     let string : Vec<String>=  row.values.iter().map(|val| {
        match val {
            Value::Integer(n) => n.to_string(),
            Value::Text(s)=> s.clone() ,
            Value::Null => "".to_string()
        }
         
     } ).collect();

     writeln!(file ,"{}",string.join(",")).map_err(|e| "Error in writing this file".to_string())?;

     }

      Ok(())
}


