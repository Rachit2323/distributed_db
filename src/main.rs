pub mod r#type;
mod parser;
mod storage;
mod executor;
mod wal;
mod index;
mod network;

use std::sync::{Arc, Mutex};
  fn main() {
      storage::ensure_data_dir().expect("Cannot create data dir");
      let executor = executor::Executor::new().expect("Cannot load schemas");
      let executor = Arc::new(Mutex::new(executor));
      network::start_server("127.0.0.1:7878", executor);
  }


