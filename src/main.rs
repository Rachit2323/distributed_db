pub mod r#type;
mod parser;
mod storage;
mod executor;
mod wal;
mod index;
mod network;
mod raft;

use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let id: u64 = if args.len() > 1 { args[1].parse().expect("Invalid node id") } else { 1 };
    let port: String = if args.len() > 2 { args[2].clone() } else { "7878".to_string() };
    let peers: Vec<String> = if args.len() > 3 {
        args[3].split(',').map(|s| s.to_string()).collect()
    } else {
        vec![]
    };

    println!("Node {} starting on port {}", id, port);

    storage::ensure_data_dir().expect("Cannot create data dir");

    let raft_node = raft::RaftNode::new(id, peers.clone());
    let raft_node = Arc::new(Mutex::new(raft_node));

    let raft_port = port.parse::<u16>().expect("Invalid port") + 1000;
    raft::RaftNode::start_raft_listener(raft_port, Arc::clone(&raft_node));
    raft::RaftNode::start_heartbeat(Arc::clone(&raft_node));
    raft::election::start_election_timer(Arc::clone(&raft_node));

    let executor = executor::Executor::new(id, peers, Arc::clone(&raft_node)).expect("Cannot load schemas");
    let executor = Arc::new(Mutex::new(executor));
    let address = format!("127.0.0.1:{}", port);
    network::start_server(&address, executor);
}


