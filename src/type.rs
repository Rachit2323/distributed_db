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
    CreateTable { table_name: String, columns: Vec<ColumnDef> },
    Insert      { table_name: String, values: Vec<Value> },
    Select      { table_name: String, where_clause: Option<WhereClause> },
}

pub enum QueryResult {
    Created,
    Inserted,
    Rows { columns: Vec<String>, rows: Vec<Row> },
    Error(String),
}
