use std::fs::File;
use std::io::Write;

use crate::r#type::{Row, Value};
const DATA_DIR: &str = "./data";

pub fn write_entry(table_name: &str, row : &Row) -> Result<(),String >
{
     let wal_path = format!("{}/{}.wal", DATA_DIR,table_name);

     let mut file = File::create(wal_path).map_err(|e| format!("Error while creating the file {}",e ))?;

     let string : Vec<String> = row.values.iter().map (|val|{
        match val {
            Value::Integer(n)=> { n.to_string() },
            Value::Text(s)=>{s.clone()},
            Value::Null => "".to_string()
        }
     }).collect();

     writeln!(file,"INSERT|{}",string.join(",")).map_err(|e| format!("caanot write into WAL {}",e))?;

     Ok(())

}

pub fn clear_entry(table_name: &str)-> Result<(),String>
{

     let file = format!("{}/{}.wal",DATA_DIR,table_name);
      
      std::fs::remove_file(file).map_err(|e| format!("Issue while clearning the file {}",e))?;
      Ok(())
}

