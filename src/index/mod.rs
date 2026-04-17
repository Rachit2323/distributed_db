  use std::collections::HashMap;
  use std::fs::{self, File};
  use std::io::{BufRead, BufReader, Write};
  use crate::r#type::{Value, Row, TableSchema, DataType};


    pub struct Index {
      pub table_name: String,
      pub column:     String,
      pub map:        HashMap<Value, Vec<usize>>,
  }
  