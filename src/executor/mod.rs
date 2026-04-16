use crate::r#type::*;
use crate::storage;
use crate::storage::create_table;
use std::collections::HashMap;

pub struct Executor {
    schemas: HashMap<String, TableSchema>,
}

impl Executor {
    pub fn new() -> Result<Self, String> {
        let schema = storage::load_schemas()?;
        return Ok(Executor { schemas: schema });
    }

    fn handle_create_table(&mut self, table_name: String, columns: Vec<ColumnDef>) -> QueryResult {
        if self.schemas.contains_key(&table_name) {
            return QueryResult::Error("Schema already exist".to_string());
        } else {
            let schemha = create_table(&TableSchema {
                name: table_name.clone(),
                columns: columns.clone(),
            });
            match schemha {
                Ok(()) => {
                    self.schemas.insert(
                        table_name.clone(),
                        TableSchema {
                            name: table_name.clone(),
                            columns: columns,
                        },
                    );
                    QueryResult::Created
                }
                Err(e) => QueryResult::Error("Error in creating the schema".to_string()),
            }
        }
    }

    fn handle_insert(&mut self, table_name: String, values: Vec<Value>) -> QueryResult {
        // step 1: find the schema
        let schema = match self.schemas.get(&table_name) {
            None    => return QueryResult::Error(format!("Table '{}' does not exist", table_name)),
            Some(s) => s,
        };

        // step 2: check value count matches column count
        if values.len() != schema.columns.len() {
            return QueryResult::Error(format!(
                "Table '{}' has {} column(s) but {} value(s) given",
                table_name,
                schema.columns.len(),
                values.len()
            ));
        }

        // step 3: check each value type matches column type
        for (val, col) in values.iter().zip(schema.columns.iter()) {
            if !type_matches(val, &col.data_type) {
                return QueryResult::Error(format!(
                    "Column '{}' expects {:?} but got a different type",
                    col.name, col.data_type
                ));
            }
        }

        // step 4: write to disk
        let row = Row { values };
        if let Err(e) = storage::append_row(&table_name, &row) {
            return QueryResult::Error(e);
        }

        // TODO Phase 2: WAL
        QueryResult::Inserted
    }

    fn handle_select(&mut self, table_name: String, where_clause: Option<WhereClause>) -> QueryResult {
        // step 1: find the schema
        let schema = match self.schemas.get(&table_name) {
            None    => return QueryResult::Error(format!("Table '{}' does not exist", table_name)),
            Some(s) => s,
        };

        // step 2: read ALL rows from disk
        let all_rows = match storage::read_rows(&table_name, schema) {
            Err(e)   => return QueryResult::Error(e),
            Ok(rows) => rows,
        };

        // TODO Phase 3: check index before full scan

        // step 3: filter by WHERE clause if one exists
        let rows = match where_clause {
            // no WHERE → return everything
            None => all_rows,

            // WHERE exists → filter
            Some(wc) => {
                // find which column position to check
                let col_index = schema.columns
                    .iter()
                    .position(|c| c.name == wc.column);

                match col_index {
                    None => return QueryResult::Error(format!(
                        "Column '{}' does not exist in table '{}'",
                        wc.column, table_name
                    )),
                    Some(idx) => all_rows
                        .into_iter()
                        .filter(|row| row.values.get(idx) == Some(&wc.value))
                        .collect(),
                }
            }
        };

        // step 4: collect column names from schema
        let columns: Vec<String> = schema.columns
            .iter()
            .map(|c| c.name.clone())
            .collect();

        QueryResult::Rows { columns, rows }
    }

    pub fn execute(&mut self, stmt: Statement) -> QueryResult {
        match stmt {
            Statement::CreateTable { table_name, columns } => {
                self.handle_create_table(table_name, columns)
            }
            Statement::Insert { table_name, values } => {
                self.handle_insert(table_name, values)
            }
            Statement::Select { table_name, where_clause } => {
                self.handle_select(table_name, where_clause)
            }
        }
    }
}


fn type_matches(value: &Value, expected: &DataType) -> bool {
    match (value, expected) {
        (Value::Integer(_), DataType::Integer) => true,
        (Value::Text(_),    DataType::Text)    => true,
        (Value::Null,       _)                 => true,
        _                                      => false,
    }
}
