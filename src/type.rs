#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Text(String),
    Null,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
      pub primary_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub values: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub column: String,
    pub value: Value,
}

pub enum Statement {
    CreateTable { table_name: String, columns: Vec<ColumnDef>, primary_key: Option<String> },
    Insert      { table_name: String, values: Vec<Value> },
    Select      { table_name: String, where_clause: Option<WhereClause> },
    Delete       { table_name: String, where_clause: WhereClause },
    Update       { table_name: String, column: String, value: Value, where_clause: WhereClause },
      CreateIndex { table_name: String, column: String },
  DropIndex   { table_name: String, column: String }
}

pub enum QueryResult {
    Created,
    Inserted,
    Rows { columns: Vec<String>, rows: Vec<Row> },
    Deleted(usize),
    Updated(usize),
    Error(String),
      IndexCreated,
  IndexDropped,
}
